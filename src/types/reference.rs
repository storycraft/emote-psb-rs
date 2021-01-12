/*
 * Created on Sat Dec 26 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Read, Write};

use crate::PsbError;

use super::number::PsbNumber;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PsbReference {

    ref_index: u64

}

impl PsbReference {

    pub fn new(ref_index: u64) -> Self {
        Self {
            ref_index
        }
    }

    pub fn ref_index(&self) -> u64 {
        self.ref_index
    }

    pub fn set_index(&mut self, ref_index: u64) {
        self.ref_index = ref_index;
    }

    pub fn get_n(&self) -> u8 {
        PsbNumber::get_n(self.ref_index).max(1)
    }

    pub fn from_bytes(n: u8, stream: &mut impl Read) -> Result<(u64, Self), PsbError> {
        let (ref_index_read, ref_index) = PsbNumber::read_integer(n, stream)?;

        Ok((ref_index_read, Self::new(ref_index as u64)))
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
        Ok(PsbNumber::write_integer(self.get_n(), self.ref_index as i64, stream)? as u64)
    }

}