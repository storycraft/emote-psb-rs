//! PSB file writing support.

use core::fmt::{self, Debug};
use std::io::{self, Read, Seek, SeekFrom, Write};

use adler2::Adler32;
use byteorder::{LittleEndian, WriteBytesExt};
use serde::Serialize;
use smol_str::SmolStr;

use crate::{
    PSB_SIGNATURE,
    psb::{btree::PsbBtree, error::PsbWriteError, table::StringTable},
    value::{
        ser::{Buffer, serialize},
        util::write_uint_array,
    },
};

#[derive(Debug)]
/// A PSB file writer that serializes a root value and optional binary resources.
///
/// Create with [`PsbWriter::new`] (or [`PsbWriter::new_with_buffer`] for a pre-built
/// [`Buffer`]), then optionally attach binary resources via [`add_resource`] /
/// [`add_extra`], and finally call [`finish`] to flush the complete file.
///
/// [`add_resource`]: PsbWriter::add_resource
/// [`add_extra`]: PsbWriter::add_extra
/// [`finish`]: PsbWriter::finish
pub struct PsbWriter<T> {
    version: u16,
    offset_start: u64,
    header_length: u32,

    offsets: Offsets,

    resources: Resources,
    extra: Resources,

    stream: PsbStream<T>,
}

impl<T> PsbWriter<T>
where
    T: Write + Seek,
{
    /// Creates a new [`PsbWriter`], serializing `root` and writing the PSB header to `stream`.
    ///
    /// # Parameters
    ///
    /// - `version` — PSB format version (2, 3, or 4).
    /// - `encrypted` — whether the PSB encryption flag should be set.
    /// - `root` — the root value to serialize.
    /// - `stream` — writable, seekable output stream.
    ///
    /// # Errors
    ///
    /// Returns [`PsbWriteError`] if serialization or writing the header fails.
    pub fn new(
        version: u16,
        encrypted: bool,
        root: &impl Serialize,
        stream: T,
    ) -> Result<Self, PsbWriteError> {
        let mut buf = Buffer::new();
        serialize(&root, &mut buf)?;
        Self::new_with_buffer(version, encrypted, &mut buf, stream)
    }

    /// Creates a new [`PsbWriter`] from a pre-populated serialization [`Buffer`].
    ///
    /// This is useful when the same buffer is reused across multiple writes.
    /// The name and string tables in `buf` must already be sorted before calling
    /// this function (i.e. after a call to [`serialize`]).
    ///
    /// # Errors
    ///
    /// Returns [`PsbWriteError`] if writing the header fails.
    ///
    /// [`serialize`]: crate::value::ser::serialize
    pub fn new_with_buffer(
        version: u16,
        encrypted: bool,
        buf: &mut Buffer,
        stream: T,
    ) -> Result<Self, PsbWriteError> {
        let mut stream = PsbStream::new(stream)?;
        stream.write_u32::<LittleEndian>(PSB_SIGNATURE)?;
        stream.write_u16::<LittleEndian>(version)?;
        stream.write_u16::<LittleEndian>(encrypted as _)?;

        let header_length = header_length(version);
        stream.write_u32::<LittleEndian>(header_length)?;

        let offset_start = stream.stream_position()?;
        for _ in 0..header_length - 12 {
            stream.write_u8(0)?;
        }

        let name_offset = stream.psb_position()?;
        write_names(&mut stream, buf.names().iter())?;

        let entrypoint = stream.psb_position()?;
        buf.write(&mut stream)?;

        let mut offsets = Vec::<u64>::with_capacity(buf.strings().len());
        let mut offset = 0;
        for string in buf.strings() {
            offsets.push(offset);
            offset += string.len() as u64 + 1;
        }
        let string_offsets_offset = stream.psb_position()?;
        write_uint_array(&mut stream, &offsets)?;

        let string_data_offset = stream.psb_position()?;
        for string in buf.strings() {
            stream.write_all(string.as_bytes())?;
            stream.write_u8(0)?;
        }

        Ok(Self {
            version,
            offset_start,
            header_length,
            offsets: Offsets {
                name: name_offset,
                entrypoint,
                string_offsets: string_offsets_offset,
                string_data: string_data_offset,
            },
            resources: Resources::new(),
            extra: Resources::new(),
            stream,
        })
    }

    #[inline]
    /// Attaches a binary resource stream and returns its zero-based resource index.
    ///
    /// The resource will be appended to the PSB resource section when [`finish`] is called.
    ///
    /// [`finish`]: PsbWriter::finish
    pub fn add_resource(&mut self, res: impl Read + Seek + 'static) -> io::Result<usize> {
        self.resources.add(res)
    }

    #[inline]
    /// Attaches an extra (version 4+) binary resource stream and returns its index.
    ///
    /// Extra resources are written to the extra resource section introduced in PSB version 4.
    pub fn add_extra(&mut self, res: impl Read + Seek + 'static) -> io::Result<usize> {
        self.extra.add(res)
    }

    /// Finalizes the PSB file by writing all resource data and updating the header offsets.
    ///
    /// Consumes the writer. The underlying stream is flushed but not closed.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if any write or seek operation fails.
    pub fn finish(mut self) -> io::Result<()> {
        let extra_offsets = if self.version > 3 {
            let extra_offset = self.stream.psb_position()?;
            self.extra.write_offsets(&mut self.stream)?;
            let extra_length = self.stream.psb_position()?;
            self.extra.write_lengths(&mut self.stream)?;
            let extra_data = self.stream.psb_position()?;
            self.extra.write_data(&mut self.stream)?;

            Some((extra_offset, extra_length, extra_data))
        } else {
            None
        };

        let resource_offset = self.stream.psb_position()?;
        self.resources.write_offsets(&mut self.stream)?;
        let resource_length = self.stream.psb_position()?;
        self.resources.write_lengths(&mut self.stream)?;
        let resource_data = self.stream.psb_position()?;
        self.resources.write_data(&mut self.stream)?;

        self.stream.seek(SeekFrom::Start(self.offset_start))?;
        self.write_offsets(
            resource_offset,
            resource_length,
            resource_data,
            extra_offsets,
        )?;
        self.stream.seek(SeekFrom::End(0))?;
        self.stream.flush()?;
        Ok(())
    }

    fn write_offsets(
        &mut self,
        resource_offset: u32,
        resource_length: u32,
        resource_data: u32,
        extra_offsets: Option<(u32, u32, u32)>,
    ) -> io::Result<()> {
        self.stream.write_u32::<LittleEndian>(self.offsets.name)?;

        self.stream
            .write_u32::<LittleEndian>(self.offsets.string_offsets)?;
        self.stream
            .write_u32::<LittleEndian>(self.offsets.string_data)?;

        self.stream.write_u32::<LittleEndian>(resource_offset)?;
        self.stream.write_u32::<LittleEndian>(resource_length)?;
        self.stream.write_u32::<LittleEndian>(resource_data)?;

        self.stream
            .write_u32::<LittleEndian>(self.offsets.entrypoint)?;
        if self.version > 2 {
            self.stream
                .write_u32::<LittleEndian>(self.calculate_checksum(
                    resource_offset,
                    resource_length,
                    resource_data,
                    extra_offsets,
                ))?;
        }

        if let Some(extra) = extra_offsets {
            self.stream.write_u32::<LittleEndian>(extra.0)?;
            self.stream.write_u32::<LittleEndian>(extra.1)?;
            self.stream.write_u32::<LittleEndian>(extra.2)?;
        }

        Ok(())
    }

    fn calculate_checksum(
        &self,
        resource_offset: u32,
        resource_length: u32,
        resource_data: u32,
        extra_offsets: Option<(u32, u32, u32)>,
    ) -> u32 {
        let mut adler = Adler32::new();
        adler.write_slice(&self.header_length.to_le_bytes());
        adler.write_slice(&self.offsets.name.to_le_bytes());
        adler.write_slice(&self.offsets.string_offsets.to_le_bytes());
        adler.write_slice(&self.offsets.string_data.to_le_bytes());
        adler.write_slice(&resource_offset.to_le_bytes());
        adler.write_slice(&resource_length.to_le_bytes());
        adler.write_slice(&resource_data.to_le_bytes());
        adler.write_slice(&self.offsets.entrypoint.to_le_bytes());
        if let Some(extra) = extra_offsets {
            adler.write_slice(&extra.0.to_le_bytes());
            adler.write_slice(&extra.1.to_le_bytes());
            adler.write_slice(&extra.2.to_le_bytes());
        }
        adler.checksum()
    }
}

#[derive(Debug)]
struct PsbStream<T> {
    start: u64,
    inner: T,
}

impl<T> PsbStream<T>
where
    T: Write + Seek,
{
    pub fn new(mut stream: T) -> io::Result<Self> {
        let start = stream.stream_position()?;
        Ok(Self {
            start,
            inner: stream,
        })
    }

    pub fn psb_position(&mut self) -> io::Result<u32> {
        Ok((self.inner.stream_position()? - self.start) as u32)
    }
}

impl<T: Write> Write for PsbStream<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Seek> Seek for PsbStream<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}

#[derive(Debug)]
struct Resources {
    offsets: Vec<u64>,
    lengths: Vec<u64>,
    streams: Vec<Resource>,
}

impl Resources {
    #[inline]
    pub const fn new() -> Self {
        Self {
            offsets: vec![],
            lengths: vec![],
            streams: vec![],
        }
    }

    pub fn add(&mut self, mut res: impl Read + Seek + 'static) -> io::Result<usize> {
        let cur = res.stream_position()?;
        let end = res.seek(SeekFrom::End(0))?;
        res.seek(SeekFrom::Start(cur))?;
        let size = end - cur;

        let id = self.offsets.len();
        self.offsets.push({
            let last_offset = self.offsets.last().copied().unwrap_or_default();
            let last_size = self.lengths.last().copied().unwrap_or_default();

            last_offset + last_size
        });
        self.lengths.push(size);
        self.streams.push(Resource(Box::new(res)));

        Ok(id)
    }

    pub fn write_offsets(&self, stream: &mut impl Write) -> io::Result<()> {
        write_uint_array(stream, &self.offsets)?;
        Ok(())
    }

    pub fn write_lengths(&self, stream: &mut impl Write) -> io::Result<()> {
        write_uint_array(stream, &self.lengths)?;
        Ok(())
    }

    pub fn write_data(&mut self, stream: &mut impl Write) -> io::Result<()> {
        for Resource(data) in &mut self.streams {
            io::copy(data, stream)?;
        }
        Ok(())
    }
}

#[repr(transparent)]
struct Resource(Box<dyn Read>);

impl Debug for Resource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Resource").finish_non_exhaustive()
    }
}

#[derive(Debug)]
struct Offsets {
    name: u32,
    entrypoint: u32,
    string_offsets: u32,
    string_data: u32,
}

const fn header_length(version: u16) -> u32 {
    match version {
        ..3 => 40,
        3 => 44,
        _ => 56,
    }
}

fn write_names<'a>(
    stream: &mut impl Write,
    names: impl Iterator<Item = &'a SmolStr>,
) -> io::Result<()> {
    let mut table = StringTable::new();
    for name in names {
        table.push_str(name);
    }
    let btree = PsbBtree(table);
    btree.write_tree(stream)?;
    Ok(())
}
