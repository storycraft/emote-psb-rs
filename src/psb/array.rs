/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::Read;

use crate::ScnError;

use byteorder::ReadBytesExt;

use super::{PSB_TYPE_INTEGER_ARRAY, number::PsbNumber};

#[derive(Debug)]
pub struct PsbIntArray {

    vec: Vec<u64>

}

impl PsbIntArray {

    pub fn new(vec: Vec<u64>) -> Self {
        Self {
            vec
        }
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn unwrap(self) -> Vec<u64> {
        self.vec
    }

    pub fn from_bytes(stream: &mut impl Read) -> Result<(u64, PsbIntArray), ScnError> {
        let (count_read, item_count) = PsbNumber::read_integer(stream.read_u8()? - PSB_TYPE_INTEGER_ARRAY, stream)?;

        let item_byte_size = stream.read_u8()? - PSB_TYPE_INTEGER_ARRAY;

        let mut list = Vec::<u64>::new();

        let mut item_total_read = 0_u64;
        for _ in 0..item_count {
            let (item_read, item) = PsbNumber::read_integer(item_byte_size, stream)?;
            list.push(item);

            item_total_read += item_read;
        }

        Ok((count_read + 1 + item_total_read, PsbIntArray::new(list)))
    }

}