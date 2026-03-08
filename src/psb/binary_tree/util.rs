#[derive(Debug)]
pub struct SparseVec<T> {
    vec: Vec<T>,
}

impl<T: Default + Clone> SparseVec<T> {
    pub fn new() -> Self {
        Self { vec: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn set(&mut self, index: usize, value: T) {
        if self.vec.len() <= index {
            self.vec.resize_with(index + 1, T::default);
        }

        self.vec[index] = value;
    }

    pub fn push(&mut self, value: T) {
        self.vec.push(value);
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        self.vec.get(index)
    }

    pub fn into_inner(self) -> Vec<T> {
        self.vec
    }
}
