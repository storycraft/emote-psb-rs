use std::io::{self, ErrorKind, Write};

use indexmap::{IndexSet, set::Slice};
use smol_str::SmolStr;

use crate::value::ser::Error;

/// Intermediate buffer that accumulates a serialized PSB value tree before it is
/// written to an output stream.
///
/// A `Buffer` is populated by [`serialize`] and then passed to
/// [`PsbWriter::new_with_buffer`] to produce a complete PSB file. It can be
/// [`cleared`](Buffer::clear) and reused across multiple serialize/write cycles.
///
/// [`serialize`]: crate::value::ser::serialize
/// [`PsbWriter::new_with_buffer`]: crate::psb::write::PsbWriter::new_with_buffer
#[derive(Debug, Clone)]
pub struct Buffer {
    pub(crate) names: IndexSet<SmolStr>,
    pub(crate) strings: IndexSet<SmolStr>,
    pub(crate) bytes: Vec<u8>,
    pub(crate) values: Vec<BufferValue>,
    pub(crate) objects: Vec<BufferObject>,
    pub(crate) indexes: Vec<usize>,
}

impl Buffer {
    /// Creates a new, empty [`Buffer`].
    pub fn new() -> Self {
        Self {
            names: IndexSet::new(),
            strings: IndexSet::new(),
            bytes: vec![],
            values: vec![],
            objects: vec![],
            indexes: vec![],
        }
    }

    /// Returns a slice of all collected object-key names in their serialized (sorted) order.
    pub fn names(&self) -> &Slice<SmolStr> {
        self.names.as_slice()
    }

    /// Returns a slice of all collected string values in their serialized (sorted) order.
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

    /// Writes the serialized PSB value tree to `stream`, starting from the root value.
    ///
    /// # Errors
    ///
    /// Returns an [`io::Error`] if writing to `stream` fails.
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

/// Temporary buffers for list, map serialization
pub(crate) struct SerializerBuffer {
    pub keys: Vec<u32>,
    pub offsets: Vec<u64>,
    pub map_indexes: Vec<usize>,
    pub permutations: Vec<usize>,
}

impl SerializerBuffer {
    #[inline]
    pub const fn new() -> Self {
        Self {
            keys: vec![],
            offsets: vec![],
            map_indexes: vec![],
            permutations: vec![],
        }
    }
}

/// A single node in the serialized PSB value tree held by a [`Buffer`].
#[derive(Debug, Clone, Copy)]
pub enum BufferValue {
    /// A placeholder for a value that has not been fully written yet.
    Invalid,
    /// A leaf value stored as a raw byte slice at `data_start` with `size` bytes.
    Value { data_start: usize, size: u32 },
    /// A composite value (list or object) whose children are tracked in the object table.
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

/// Metadata for a list or object node stored in a [`Buffer`]'s object table.
#[derive(Debug, Clone, Copy)]
pub struct BufferObject {
    /// Number of child values.
    pub len: usize,
    /// Byte offset in `Buffer::bytes` where this node's data begins.
    pub data_start: usize,
    /// Byte offset of the serialized header (type tag + offset/key arrays).
    pub header_start: usize,
    /// Byte offset immediately after the header (first byte of child data).
    pub header_end: usize,
    /// Starting index in `Buffer::indexes` for this node's children.
    pub index_start: usize,
}
