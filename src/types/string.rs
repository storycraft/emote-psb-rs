/*
 * Created on Tue Jan 12 2021
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::io::{Read, Write};

use crate::{PsbError, PsbErrorKind, PsbRefs};

use super::reference::PsbReference;

#[derive(Debug, PartialEq, Eq)]
pub struct PsbString {

    string: String

}

impl PsbString {

    pub fn new() -> Self {
        Self {
            string: String::new()
        }
    }

    pub fn string(&self) -> &String {
        &self.string
    }

    pub fn string_mut(&mut self) -> &mut String {
        &mut self.string
    }

    pub fn set_string(&mut self, string: String) {
        self.string = string;
    }

    pub fn unwrap(self) -> String {
        self.string
    }

    pub fn from_bytes(n: u8, stream: &mut impl Read, table: &PsbRefs) -> Result<(u64, Self), PsbError> {
        let (read, reference) = PsbReference::from_bytes(n, stream)?;

        let string = table.get_string(reference.ref_index() as usize);

        if string.is_none() {
            return Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None));
        }
        
        Ok((read, Self::from(string.unwrap().clone())))
    }

    // Returns written, ref_index tuple
    pub fn write_bytes(&self, stream: &mut impl Write, ref_table: &PsbRefs) -> Result<u64, PsbError> {
        match ref_table.find_string_index(&self.string) {

            Some(ref_index) => {
                PsbReference::new(ref_index as i64).write_bytes(stream)
            },

            None => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))
        }
    }

}

impl From<String> for PsbString {

    fn from(string: String) -> Self {
        Self { string }
    }

}