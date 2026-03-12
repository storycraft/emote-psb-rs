use indexmap::{IndexSet, set::Slice};
use smol_str::SmolStr;

use crate::value::ser::Error;

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

#[derive(Debug, Clone)]
pub(crate) enum BufferValue {
    Invalid,
    Value(u64),
    List { len: usize },
    Map { key_start: usize, len: usize },
}
