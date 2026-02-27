use std::io;

use tokio::io::{AsyncRead, AsyncReadExt};

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

#[extend::ext(name = PsbValueReadExt)]
pub impl<T: AsyncRead + Unpin> T {
    async fn read_partial_uint(&mut self, size: u8) -> io::Result<u64> {
        match size {
            0 => Ok(0),
            1..=8 => {
                let mut buf = [0_u8; 8];
                self.read_exact(&mut buf[..size as usize]).await?;

                Ok(u64::from_le_bytes(buf))
            }

            _ => Err(io::Error::from(io::ErrorKind::InvalidInput)),
        }
    }

    async fn read_partial_int(&mut self, size: u8) -> io::Result<i64> {
        Ok(i64::from_ne_bytes(
            self.read_partial_uint(size).await?.to_ne_bytes(),
        ))
    }
}

pub fn get_n(mut number: i64) -> u8 {
    if number < 0 {
        number = -number;
    }

    if number <= 0x7f {
        1
    } else if number <= 0x7fff {
        2
    } else if number <= 0x7fffff {
        3
    } else if number <= 0x7fffffff {
        4
    } else if number <= 0x7fffffffff {
        5
    } else if number <= 0x7fffffffffff {
        6
    } else if number <= 0x7fffffffffffff {
        7
    } else {
        8
    }
}

pub fn get_uint_n(number: u64) -> u8 {
    if number <= 0xff {
        1
    } else if number <= 0xffff {
        2
    } else if number <= 0xffffff {
        3
    } else if number <= 0xffffffff {
        4
    } else if number <= 0xffffffffff {
        5
    } else if number <= 0xffffffffffff {
        6
    } else if number <= 0xffffffffffffff {
        7
    } else {
        8
    }
}
