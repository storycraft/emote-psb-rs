use std::io::{self, ErrorKind, Write};

use indexmap::{IndexSet, set::Slice};
use smol_str::SmolStr;

use crate::value::ser::Error;

#[derive(Debug, Clone)]
/// Intermediate psb value serialization buffer
pub struct Buffer {
    names: IndexSet<SmolStr>,
    strings: IndexSet<SmolStr>,
    pub(crate) bytes: Vec<u8>,
    pub(crate) values: Vec<BufferValue>,
    pub(crate) objects: Vec<BufferObject>,

    // temporary buffers for list, map header
    pub(crate) keys: Vec<u32>,
    pub(crate) offsets: Vec<u64>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            names: IndexSet::new(),
            strings: IndexSet::new(),
            bytes: vec![],
            values: vec![],
            objects: vec![],
            keys: vec![],
            offsets: vec![],
        }
    }

    pub fn names(&self) -> &Slice<SmolStr> {
        self.names.as_slice()
    }

    pub fn strings(&self) -> &Slice<SmolStr> {
        self.strings.as_slice()
    }

    /// Clear buffer for reuse
    pub fn clear(&mut self) {
        self.bytes.clear();
        self.values.clear();
        self.objects.clear();
        self.names.clear();
        self.strings.clear();
    }

    #[inline]
    pub fn write(&self, stream: &mut impl Write) -> io::Result<()> {
        self.write_inner(0, 0, stream)?;
        Ok(())
    }

    fn write_inner(
        &self,
        value_index: usize,
        data_start: usize,
        stream: &mut impl Write,
    ) -> io::Result<(usize, usize)> {
        let Some(&current) = self.values.get(value_index) else {
            return Ok((0, 0));
        };

        match current {
            BufferValue::Invalid => Err(ErrorKind::InvalidData.into()),
            BufferValue::Value(size) => {
                stream.write_all(&self.bytes[data_start..][..size])?;
                Ok((size, 1))
            }
            BufferValue::Object { index } => {
                let object = self.objects[index];
                stream.write_all(&self.bytes[object.header_start..][..object.header_size])?;

                let mut value_offset = 1;
                let mut data_offset = 0;
                for _ in 0..object.len {
                    let item_index = value_index + value_offset;
                    let item_offset = data_start + data_offset;
                    let (written, values_read) =
                        self.write_inner(item_index, item_offset, stream)?;

                    data_offset += written;
                    value_offset += values_read;
                }
                Ok((data_offset + object.header_size, value_offset))
            }
        }
    }

    pub(crate) fn alloc_name(&mut self, string: &str) -> Result<u32, Error> {
        let (index, _) = self.names.insert_full(string.into());
        index.try_into().map_err(|_| Error::IndexOverflow)
    }

    pub(crate) fn alloc_string(&mut self, string: &str) -> Result<u32, Error> {
        let (index, _) = self.strings.insert_full(string.into());
        index.try_into().map_err(|_| Error::IndexOverflow)
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum BufferValue {
    Invalid,
    Value(usize),
    Object { index: usize },
}

impl BufferValue {
    pub fn size(self, buf: &Buffer) -> usize {
        match self {
            BufferValue::Invalid => 0,
            BufferValue::Value(size) => size,
            BufferValue::Object { index } => {
                let obj = buf.objects[index];
                obj.header_start + obj.header_size
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct BufferObject {
    pub len: usize,
    pub header_start: usize,
    pub header_size: usize,
}
