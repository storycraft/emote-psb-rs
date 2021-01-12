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

pub struct XorShiftStream<T> {

    stream: T,

    read_seeds: [u32; 4],
    write_seeds: [u32; 4]

}

impl<T> XorShiftStream<T> {

    pub fn new(stream: T, seeds: [u32; 4]) -> Self {
        Self {
            stream, read_seeds: seeds, write_seeds: seeds
        }
    }

    pub fn new_emote(stream: T, key: u32) -> Self {
        Self::new(stream, [123456789, 362436069, 521288629, key])
    }

    fn next_read(&mut self) -> u32 {
        Self::next(&mut self.read_seeds)
    }

    fn next_write(&mut self) -> u32 {
        Self::next(&mut self.write_seeds)
    }

    fn next(seeds: &mut [u32; 4]) -> u32 {
        let x = seeds[0] ^ (seeds[0] << 11);

        seeds[0] = seeds[1];
        seeds[1] = seeds[2];
        seeds[2] = seeds[3];

        seeds[3] = (seeds[3] ^ (seeds[3] >> 19)) ^ (x ^ (x >> 8));

        seeds[3]
    }

}

impl<T: Write + Seek> Write for XorShiftStream<T> {

    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let current = self.stream.seek(SeekFrom::Current(0)).unwrap() as usize;

        let arr = self.next_write().to_le_bytes();

        self.stream.write(
            &buf.iter().enumerate().map(|(i, &val)| val ^ arr[(current + i) % 4]).collect::<Vec<u8>>()
        )
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.stream.flush()
    }

}

impl<T: Read + Seek> Read for XorShiftStream<T> {

    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let current = self.stream.seek(SeekFrom::Current(0)).unwrap() as usize;
        
        let read = self.stream.read(buf)?;
        let arr = self.next_read().to_le_bytes();

        for i in 0..read {
            buf[i] ^= arr[(current + i) % 4];
        }

        Ok(read)
    }

}