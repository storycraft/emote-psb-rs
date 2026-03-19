//! PSB string table used to store names and string values.

use core::fmt::Debug;

/// A compact, append-only table of strings backed by a single contiguous buffer.
///
/// Strings are stored end-to-end without separators; their boundaries are tracked
/// by a separate index vector, so each string can be retrieved in O(1) time by
/// its integer identifier.
///
/// This structure is used internally to hold the PSB name table (object keys) and
/// the string value table.
#[derive(Clone)]
pub struct StringTable {
    data: String,
    indices: Vec<usize>,
}

impl Default for StringTable {
    fn default() -> Self {
        Self::new()
    }
}

impl StringTable {
    /// Creates a new, empty [`StringTable`].
    pub const fn new() -> Self {
        Self {
            data: String::new(),
            indices: vec![],
        }
    }

    /// Creates a new, empty [`StringTable`] with pre-allocated capacity for `size` entries.
    pub fn with_capacity(size: usize) -> Self {
        Self {
            data: String::new(),
            indices: Vec::with_capacity(size),
        }
    }

    /// Appends a string built from the given character iterator and returns its identifier.
    pub fn push(&mut self, data: impl IntoIterator<Item = char>) -> usize {
        let start = self.data.len();
        self.data.extend(data);
        let id = self.indices.len();
        self.indices.push(start);
        id
    }

    /// Appends `data` to the table and returns its identifier.
    pub fn push_str(&mut self, data: &str) -> usize {
        let start = self.data.len();
        self.data.push_str(data);
        let id = self.indices.len();
        self.indices.push(start);
        id
    }

    /// Returns the string with the given `id`, or `None` if `id` is out of range.
    pub fn get(&self, id: usize) -> Option<&str> {
        let start = *self.indices.get(id)?;
        let end = self.indices.get(id + 1).copied();
        Some(if let Some(end) = end {
            &self.data[start..end]
        } else {
            &self.data[start..]
        })
    }

    /// Returns `true` if the table contains no strings.
    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the total byte length of all stored strings (not the number of entries).
    #[inline]
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns an iterator over all strings in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        (0..self.indices.len()).flat_map(|i| self.get(i))
    }
}

impl Debug for StringTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
