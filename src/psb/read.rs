use std::io::{self, BufRead, Read, Seek, SeekFrom, Take};

use byteorder::{LittleEndian, ReadBytesExt};
use scopeguard::guard;
use serde::de::DeserializeOwned;

use crate::{
    PSB_SIGNATURE,
    psb::{btree::PsbBtree, error::PsbOpenError, table::StringTable},
    value::{
        de::{self, Deserializer},
        util::read_uint_array,
    },
};

#[derive(Debug, Clone)]
pub struct PsbFile<T> {
    pub encrypted: bool,
    pub version: u16,

    pub names: StringTable,
    pub strings: StringTable,
    resources: Vec<PsbResourceItem>,

    /// Offset to root object
    entrypoint: u64,

    pub checksum: Option<u32>,
    extra: Vec<PsbResourceItem>,
    stream: T,
}

impl<T: BufRead + Seek> PsbFile<T> {
    /// Open Psb file from stream
    pub fn open(mut stream: T) -> Result<Self, PsbOpenError> {
        let start = stream.stream_position()?;
        let signature = stream.read_u32::<LittleEndian>()?;
        if signature != PSB_SIGNATURE {
            return Err(PsbOpenError::InvalidSignature);
        }

        let version = stream.read_u16::<LittleEndian>()?;
        let encrypted = stream.read_u16::<LittleEndian>()? != 0;

        let _ = stream.read_u32::<LittleEndian>()?;

        let name_offset = stream.read_u32::<LittleEndian>()?;

        let string_offset = stream.read_u32::<LittleEndian>()?;
        let string_data_start = stream.read_u32::<LittleEndian>()?;

        let resource_offset = stream.read_u32::<LittleEndian>()?;
        let resource_lengths = stream.read_u32::<LittleEndian>()?;
        let resource_data_start = stream.read_u32::<LittleEndian>()?;

        let entrypoint = stream.read_u32::<LittleEndian>()?;

        let checksum = if version > 2 {
            Some(stream.read_u32::<LittleEndian>()?)
        } else {
            None
        };

        let mut buf: Vec<u64> = vec![];

        let extra = if version > 3 {
            let extra_resource_offset = stream.read_u32::<LittleEndian>()?;
            let extra_resource_lengths = stream.read_u32::<LittleEndian>()?;
            let extra_resource_data_start = stream.read_u32::<LittleEndian>()?;

            stream.seek(std::io::SeekFrom::Start(
                start + extra_resource_offset as u64,
            ))?;
            Self::read_resources(
                &mut stream,
                &mut buf,
                start + extra_resource_lengths as u64,
                start + extra_resource_data_start as u64,
            )
            .map_err(PsbOpenError::Resources)?
        } else {
            vec![]
        };

        stream.seek(std::io::SeekFrom::Start(start + name_offset as u64))?;
        let names = PsbBtree::read(&mut stream, &mut buf)
            .map_err(PsbOpenError::Names)?
            .0;

        stream.seek(SeekFrom::Start(start + string_offset as u64))?;
        let strings = Self::read_strings(&mut stream, &mut buf, start + string_data_start as u64)
            .map_err(PsbOpenError::Strings)?;

        stream.seek(std::io::SeekFrom::Start(start + resource_offset as u64))?;
        let resources = Self::read_resources(
            &mut stream,
            &mut buf,
            start + resource_lengths as u64,
            start + resource_data_start as u64,
        )
        .map_err(PsbOpenError::Resources)?;

        Ok(Self {
            encrypted,
            version,
            names,
            strings,
            resources,
            entrypoint: start + entrypoint as u64,
            checksum,
            extra,
            stream,
        })
    }

    fn read_strings(
        stream: &mut T,
        buf: &mut Vec<u64>,
        data_pos: u64,
    ) -> Result<StringTable, de::Error> {
        let offset_start = buf.len();
        read_uint_array(stream, buf)?;

        let mut table = StringTable::new();
        let mut string_buf = vec![];
        for offset in buf.drain(offset_start..) {
            stream.seek(SeekFrom::Start(data_pos + offset))?;
            stream.read_until(0x00, &mut string_buf)?;
            string_buf.pop();
            table.push_str(str::from_utf8(&string_buf).map_err(|_| de::Error::InvalidValue)?);
            string_buf.clear();
        }

        Ok(table)
    }

    fn read_resources(
        stream: &mut T,
        buf: &mut Vec<u64>,
        lengths_pos: u64,
        data_pos: u64,
    ) -> Result<Vec<PsbResourceItem>, de::Error> {
        // offsets
        let offset_start = buf.len();
        let mut buf = guard(buf, |buf| {
            buf.drain(offset_start..);
        });
        read_uint_array(stream, *buf)?;

        // lengths
        let length_start = buf.len();
        stream.seek(SeekFrom::Start(lengths_pos))?;
        read_uint_array(stream, *buf)?;

        let mut list = vec![];
        for i in 0..length_start {
            let position = data_pos + buf[i];
            let size = buf[length_start + i];
            list.push(PsbResourceItem { position, size });
        }
        Ok(list)
    }

    #[inline]
    pub const fn resources(&self) -> usize {
        self.resources.len()
    }

    #[inline]
    pub const fn extra_resources(&self) -> usize {
        self.extra.len()
    }

    pub fn root_deserializer<'a>(&'a mut self) -> io::Result<Deserializer<'a, &'a mut T>> {
        self.stream.seek(SeekFrom::Start(self.entrypoint))?;
        Ok(Deserializer::new(&self.names, &self.strings, &mut self.stream))
    }

    pub fn deserialize_root<V: DeserializeOwned>(&mut self) -> Result<V, de::Error> {
        V::deserialize(&mut self.root_deserializer()?)
    }

    pub fn open_resource<'a>(
        &'a mut self,
        index: usize,
    ) -> io::Result<Option<PsbResourceStream<'a, T>>> {
        let Some(&res) = self.resources.get(index) else {
            return Ok(None);
        };

        self.stream.seek(SeekFrom::Start(res.position))?;
        Ok(Some(PsbResourceStream(Read::take(
            &mut self.stream,
            res.size,
        ))))
    }

    pub fn open_extra_resource<'a>(
        &'a mut self,
        index: usize,
    ) -> io::Result<Option<PsbResourceStream<'a, T>>> {
        let Some(&res) = self.extra.get(index) else {
            return Ok(None);
        };

        self.stream.seek(SeekFrom::Start(res.position))?;
        Ok(Some(PsbResourceStream(Read::take(
            &mut self.stream,
            res.size,
        ))))
    }

    #[inline]
    pub fn into_inner(self) -> T {
        self.stream
    }
}

#[repr(transparent)]
pub struct PsbResourceStream<'a, T>(Take<&'a mut T>);

impl<T: Read> Read for PsbResourceStream<'_, T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.0.read(buf)
    }
}

impl<T: Seek> Seek for PsbResourceStream<'_, T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.0.seek(pos)
    }
}

impl<T: BufRead> BufRead for PsbResourceStream<'_, T> {
    fn fill_buf(&mut self) -> io::Result<&[u8]> {
        self.0.fill_buf()
    }

    fn consume(&mut self, amount: usize) {
        self.0.consume(amount);
    }
}

#[derive(Debug, Clone, Copy)]
struct PsbResourceItem {
    position: u64,
    size: u64,
}
