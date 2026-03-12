use std::io::{self, ErrorKind, Write};

use byteorder::WriteBytesExt;
use indexmap::{IndexSet, set::Slice};
use smol_str::SmolStr;

use crate::value::{PSB_TYPE_LIST, PSB_TYPE_OBJECT, ser::Error};

#[derive(Debug, Clone)]
/// Intermediate psb value serialization buffer
pub struct Buffer {
    names: IndexSet<SmolStr>,
    strings: IndexSet<SmolStr>,
    pub(crate) bytes: Vec<u8>,
    pub(crate) keys: Vec<u32>,
    pub(crate) values: Vec<BufferValue>,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            names: IndexSet::new(),
            strings: IndexSet::new(),
            bytes: vec![],
            keys: vec![],
            values: vec![],
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
        self.keys.clear();
        self.values.clear();
        self.names.clear();
        self.strings.clear();
    }

    #[inline]
    pub fn write(&self, io: &mut impl Write) -> io::Result<()> {
        self.write_inner(0, 0, io)?;
        Ok(())
    }

    fn write_inner(&self, index: usize, data_start: usize, io: &mut impl Write) -> io::Result<u64> {
        let Some(&current) = self.values.get(index) else {
            return Ok(0);
        };

        match current {
            BufferValue::Invalid => Err(ErrorKind::InvalidData.into()),
            BufferValue::Value(size) => {
                io.write_all(&self.bytes[data_start..][..size as usize])?;
                Ok(size)
            }
            BufferValue::List { len } => {
                io.write_u8(PSB_TYPE_LIST)?;

                todo!()
            }
            BufferValue::Map { key_start, len } => {
                io.write_u8(PSB_TYPE_OBJECT)?;

                todo!()
            },
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
    List { size: usize, len: usize },
    Map { size: usize, key_start: usize, len: usize },
}
