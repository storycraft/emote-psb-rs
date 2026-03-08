use core::fmt::Debug;

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
    pub const fn new() -> Self {
        Self {
            data: String::new(),
            indices: vec![],
        }
    }

    pub fn with_capacity(size: usize) -> Self {
        Self {
            data: String::new(),
            indices: Vec::with_capacity(size),
        }
    }

    pub fn push(&mut self, str: &str) -> usize {
        let start = self.data.len();
        self.data.push_str(str);
        let id = self.indices.len();
        self.indices.push(start);
        id
    }

    pub fn get(&self, id: usize) -> Option<&str> {
        let start = *self.indices.get(id)?;
        let end = self.indices.get(id + 1).copied();
        Some(if let Some(end) = end {
            &self.data[start..end]
        } else {
            &self.data[start..]
        })
    }

    #[inline]
    pub const fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    pub const fn len(&self) -> usize {
        self.data.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        (0..self.indices.len()).flat_map(|i| self.get(i))
    }
}

impl Debug for StringTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
