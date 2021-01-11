use std::io::{Read, Seek, SeekFrom, Write};

/*
 * Created on Tue Jan 12 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

#[derive(Debug)]
pub struct SafeIndexVec<T> {

    vec: Vec<T>

}

impl<T: Default + Clone> SafeIndexVec<T> {

    pub fn new() -> Self {
        Self {
            vec: Vec::new()
        }
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

pub struct XorStream<T> {

    stream: T,
    key: [u8; 4]

}

impl<T> XorStream<T> {

    pub fn new(stream: T, key: u32) -> Self {
        Self {
            stream, key: key.to_le_bytes()
        }
    }

    pub fn key(&self) -> u32 {
        u32::from_le_bytes(self.key)
    }

    pub fn unwrap(self) -> T {
        self.stream
    }

}

impl<T: Write + Seek> Write for XorStream<T> {

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let current = self.stream.seek(SeekFrom::Current(0)).unwrap() as usize;

        self.stream.write(
            &buf.iter().enumerate().map(|(i, &val)| val ^ self.key[(current + i) % 4]).collect::<Vec<u8>>()
        )
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }

}

impl<T: Read + Seek> Read for XorStream<T> {

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let current = self.stream.seek(SeekFrom::Current(0)).unwrap() as usize;
        
        let read = self.stream.read(buf)?;

        for i in 0..read {
            buf[i] ^= self.key[(current + i) % 4];
        }

        Ok(read)
    }

}