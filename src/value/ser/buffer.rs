use std::io::{self, ErrorKind, Write};

use indexmap::{IndexSet, set::Slice};
use smol_str::SmolStr;

use crate::value::ser::Error;

#[derive(Debug, Clone)]
/// Intermediate psb value serialization buffer
pub struct Buffer {
    pub(crate) names: IndexSet<SmolStr>,
    pub(crate) strings: IndexSet<SmolStr>,
    pub(crate) bytes: Vec<u8>,
    pub(crate) values: Vec<BufferValue>,
    pub(crate) objects: Vec<BufferObject>,
    pub(crate) indexes: Vec<usize>,

    // temporary buffers for list, map header
    pub(crate) keys: Vec<u32>,
    pub(crate) offsets: Vec<u64>,
    pub(crate) map_indexes: Vec<usize>,
    pub(crate) permutations: Vec<usize>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            names: IndexSet::new(),
            strings: IndexSet::new(),
            bytes: vec![],
            values: vec![],
            objects: vec![],
            indexes: vec![],

            keys: vec![],
            offsets: vec![],
            map_indexes: vec![],
            permutations: vec![],
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
        self.write_inner(0, stream)?;
        Ok(())
    }

    fn write_inner(&self, value_index: usize, stream: &mut impl Write) -> io::Result<()> {
        let Some(&current) = self.values.get(value_index) else {
            return Ok(());
        };

        match current {
            BufferValue::Invalid => Err(ErrorKind::InvalidData.into()),
            BufferValue::Value { data_start, size } => {
                stream.write_all(&self.bytes[data_start..][..size as usize])?;
                Ok(())
            }
            BufferValue::Object { index } => {
                let object = self.objects[index];
                stream.write_all(&self.bytes[object.header_start..object.header_end])?;

                for i in 0..object.len {
                    let value_index = self.indexes[object.index_start + i];
                    self.write_inner(value_index, stream)?;
                }

                Ok(())
            }
        }
    }

    pub(crate) fn write_value(
        &mut self,
        f: impl FnOnce(&mut Vec<u8>) -> Result<(), Error>,
    ) -> Result<(), Error> {
        let data_start = self.bytes.len();
        f(&mut self.bytes)?;
        self.values.push(BufferValue::Value {
            data_start,
            size: (self.bytes.len() - data_start) as u32,
        });

        Ok(())
    }
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum BufferValue {
    Invalid,
    Value { data_start: usize, size: u32 },
    Object { index: usize },
}

impl BufferValue {
    pub fn size(self, buf: &Buffer) -> usize {
        match self {
            BufferValue::Invalid => 0,
            BufferValue::Value { size, .. } => size as _,
            BufferValue::Object { index } => {
                let obj = buf.objects[index];
                obj.header_end - obj.data_start
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BufferObject {
    pub len: usize,
    pub data_start: usize,
    pub header_start: usize,
    pub header_end: usize,
    pub index_start: usize,
}
