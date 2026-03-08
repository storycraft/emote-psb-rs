use std::io::{BufRead, Read, Seek, SeekFrom};

use byteorder::{LittleEndian, ReadBytesExt};
use scopeguard::guard;

use crate::{
    PSB_SIGNATURE,
    psb::{
        binary_tree::PsbBinaryTree, error::PsbOpenError, string::StringTable, util::read_uint_array,
    },
    value::io::{error::PsbValueReadError, read::PsbStreamValueReader},
};

#[derive(Debug, Clone)]
pub struct PsbFile {
    pub encrypted: bool,
    pub version: u16,

    names: StringTable,
    strings: StringTable,
    resources: Vec<PsbResourceItem>,

    /// Offset to root object
    entrypoint: u32,

    checksum: Option<u32>,
    extra: Option<Vec<PsbResourceItem>>,
}

impl PsbFile {
    /// Open Psb file from stream
    pub fn open<T: BufRead + Seek>(mut stream: T) -> Result<Self, PsbOpenError> {
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
            Some(
                Self::read_resources(
                    &mut stream,
                    &mut buf,
                    start + extra_resource_lengths as u64,
                    start + extra_resource_data_start as u64,
                )
                .map_err(PsbOpenError::Resources)?,
            )
        } else {
            None
        };

        stream.seek(std::io::SeekFrom::Start(start + name_offset as u64))?;
        let names = PsbBinaryTree::read_io(&mut stream, &mut buf)
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
            entrypoint,
            checksum,
            extra,
        })
    }

    fn read_strings(
        mut stream: &mut (impl BufRead + Seek),
        buf: &mut Vec<u64>,
        data_pos: u64,
    ) -> Result<StringTable, PsbValueReadError> {
        let offset_start = buf.len();
        read_uint_array(&mut PsbStreamValueReader::new(&mut stream), buf)?;

        let mut table = StringTable::new();
        let mut string_buf = vec![];
        for offset in buf.drain(offset_start..) {
            stream.seek(SeekFrom::Start(data_pos + offset))?;
            stream.read_until(0x00, &mut string_buf)?;
            string_buf.pop();
            table.push(str::from_utf8(&string_buf).map_err(|_| PsbValueReadError::InvalidValue)?);
            string_buf.clear();
        }

        Ok(table)
    }

    fn read_resources(
        mut stream: &mut (impl Read + Seek),
        buf: &mut Vec<u64>,
        lengths_pos: u64,
        data_pos: u64,
    ) -> Result<Vec<PsbResourceItem>, PsbValueReadError> {
        // offsets
        let offset_start = buf.len();
        let mut buf = guard(buf, |buf| {
            buf.drain(offset_start..);
        });
        read_uint_array(&mut PsbStreamValueReader::new(&mut stream), *buf)?;

        // lengths
        let length_start = buf.len();
        stream.seek(SeekFrom::Start(lengths_pos))?;
        read_uint_array(&mut PsbStreamValueReader::new(&mut stream), *buf)?;

        let mut list = vec![];
        for i in 0..length_start {
            let position = data_pos + buf[i];
            let size = buf[length_start + i];
            list.push(PsbResourceItem { position, size });
        }
        Ok(list)
    }

    #[inline]
    pub const fn names(&self) -> usize {
        self.names.len()
    }

    pub fn get_name(&self, id: usize) -> Option<&str> {
        self.names.get(id)
    }

    #[inline]
    pub const fn strings(&self) -> usize {
        self.strings.len()
    }

    #[inline]
    pub fn get_string(&self, id: usize) -> Option<&str> {
        self.strings.get(id)
    }
}

#[derive(Debug, Clone, Copy)]
struct PsbResourceItem {
    pub position: u64,
    pub size: u64,
}
