use std::io::{self, Seek, SeekFrom, Write};

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
pub struct PsbWriter<T> {
    version: u16,
    start: u64,
    offset_start: u64,
    header_length: u32,
    name_offset: u32,
    entrypoint: u32,
    string_offsets_offset: u32,
    string_data_offset: u32,
    resources: Vec<()>,
    stream: T,
}

impl<T> PsbWriter<T>
where
    T: Write + Seek,
{
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

    pub fn new_with_buffer(
        version: u16,
        encrypted: bool,
        buf: &mut Buffer,
        mut stream: T,
    ) -> Result<Self, PsbWriteError> {
        let start = stream.stream_position()?;
        stream.write_u32::<LittleEndian>(PSB_SIGNATURE)?;
        stream.write_u16::<LittleEndian>(version)?;
        stream.write_u16::<LittleEndian>(encrypted as _)?;

        let header_length = header_length(version);
        stream.write_u32::<LittleEndian>(header_length)?;

        let offset_start = stream.stream_position()?;
        for _ in 0..header_length - 12 {
            stream.write_u8(0)?;
        }

        let name_offset = (stream.stream_position()? - start) as u32;
        write_names(&mut stream, buf.names().iter())?;

        let entrypoint = (stream.stream_position()? - start) as u32;
        buf.write(&mut stream)?;

        let mut offsets = Vec::<u64>::with_capacity(buf.strings().len());
        let mut offset = 0;
        for string in buf.strings() {
            offsets.push(offset);
            offset += string.len() as u64 + 1;
        }
        let string_offsets_offset = (stream.stream_position()? - start) as u32;
        write_uint_array(&mut stream, &offsets)?;

        let string_data_offset = (stream.stream_position()? - start) as u32;
        for string in buf.strings() {
            stream.write_all(string.as_bytes())?;
            stream.write_u8(0)?;
        }

        Ok(Self {
            version,
            start,
            offset_start,
            header_length,
            name_offset,
            entrypoint,
            string_offsets_offset,
            string_data_offset,
            resources: vec![],
            stream,
        })
    }

    pub fn finish(mut self) -> io::Result<()> {
        let resource_offsets_offset = (self.stream.stream_position()? - self.start) as u32;
        write_uint_array(&mut self.stream, &[] as &[u64])?;
        let resource_length_offset = (self.stream.stream_position()? - self.start) as u32;
        write_uint_array(&mut self.stream, &[] as &[u64])?;

        let resource_data_offset = (self.stream.stream_position()? - self.start) as u32;

        self.stream.seek(SeekFrom::Start(self.offset_start))?;
        self.stream.write_u32::<LittleEndian>(self.name_offset)?;

        self.stream
            .write_u32::<LittleEndian>(self.string_offsets_offset)?;
        self.stream
            .write_u32::<LittleEndian>(self.string_data_offset)?;

        self.stream
            .write_u32::<LittleEndian>(resource_offsets_offset)?;
        self.stream
            .write_u32::<LittleEndian>(resource_length_offset)?;
        self.stream
            .write_u32::<LittleEndian>(resource_data_offset)?;

        self.stream.write_u32::<LittleEndian>(self.entrypoint)?;
        if self.version > 2 {
            let mut adler = Adler32::new();
            adler.write_slice(&self.header_length.to_le_bytes());
            adler.write_slice(&self.name_offset.to_le_bytes());
            adler.write_slice(&self.string_offsets_offset.to_le_bytes());
            adler.write_slice(&self.string_data_offset.to_le_bytes());
            adler.write_slice(&resource_offsets_offset.to_le_bytes());
            adler.write_slice(&resource_length_offset.to_le_bytes());
            adler.write_slice(&resource_data_offset.to_le_bytes());
            adler.write_slice(&self.entrypoint.to_le_bytes());

            self.stream.write_u32::<LittleEndian>(adler.checksum())?;
        }

        if self.version > 3 {
            // TODO
            self.stream
                .write_u32::<LittleEndian>(resource_offsets_offset)?;
            self.stream
                .write_u32::<LittleEndian>(resource_length_offset)?;
            self.stream
                .write_u32::<LittleEndian>(resource_data_offset)?;
        }

        self.stream.seek(SeekFrom::End(0))?;
        self.stream.flush()?;
        Ok(())
    }
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
