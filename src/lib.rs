

/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod psb;

pub mod header;

pub mod reader;
pub mod writer;

use io::Seek;
pub use reader::ScnReader;

use std::{error::Error, io::{self, Read}};

/// scn file signature
pub const SCN_SIGNATURE: u32 = 0x425350;

/// compressed scn file signature
pub const SCN_MDF_SIGNATURE: u32 = 0x66646D;

#[derive(Debug)]
pub struct ScnError {

    kind: ScnErrorKind,
    err: Option<Box<dyn Error>>

}

impl ScnError {

    pub fn new(kind: ScnErrorKind, err: Option<Box<dyn Error>>) -> Self {
        Self { kind, err }
    }

    pub fn kind(&self) -> &ScnErrorKind {
        &self.kind
    }

    pub fn source(&self) -> &Option<Box<dyn Error>> {
        &self.err
    }

}

#[derive(Debug)]
pub enum ScnErrorKind {

    Io(io::Error),
    InvalidFile,
    InvalidHeader,
    UnknownHeaderVersion,
    InvalidIndex,
    InvalidPSBValue,
    InvalidOffsetTable

}

impl From<io::Error> for ScnError {

    fn from(err: io::Error) -> Self {
        ScnError::new(ScnErrorKind::Io(err), None)
    }

}

#[derive(Debug)]
pub struct ScnRefTable {

    strings: Vec<String>,

    resources: Vec<Vec<u8>>,

    extra: Vec<Vec<u8>>

}

impl ScnRefTable {

    pub fn new(strings: Vec<String>, resources: Vec<Vec<u8>>, extra: Vec<Vec<u8>>) -> Self {
        Self {
            strings, resources, extra
        }
    }

    pub fn strings(&self) -> &Vec<String> {
        &self.strings
    }

    pub fn strings_mut(&mut self) -> &mut Vec<String> {
        &mut self.strings
    }

    pub fn strings_len(&self) -> usize {
        self.strings.len()
    }

    pub fn get_string(&self, index: usize) -> Option<&String> {
        self.strings.get(index)
    }

    pub fn get_string_mut(&mut self, index: usize) -> Option<&mut String> {
        self.strings.get_mut(index)
    }

    pub fn resources(&self) -> &Vec<Vec<u8>> {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut Vec<Vec<u8>> {
        &mut self.resources
    }

    pub fn resources_len(&self) -> usize {
        self.resources.len()
    }

    pub fn get_resource(&self, index: usize) -> Option<&Vec<u8>> {
        self.resources.get(index)
    }

    pub fn get_resource_mut(&mut self, index: usize) -> Option<&mut Vec<u8>> {
        self.resources.get_mut(index)
    }

    pub fn extra(&self) -> &Vec<Vec<u8>> {
        &self.extra
    }

    pub fn extra_mut(&mut self) -> &mut Vec<Vec<u8>> {
        &mut self.extra
    }

    pub fn extra_len(&self) -> usize {
        self.extra.len()
    }

    pub fn get_extra(&self, index: usize) -> Option<&Vec<u8>> {
        self.extra.get(index)
    }

    pub fn get_extra_mut(&mut self, index: usize) -> Option<&mut Vec<u8>> {
        self.extra.get_mut(index)
    }

}

#[derive(Debug)]
pub struct ScnFile<T: Read + Seek> {
    
    ref_table: ScnRefTable,

    entry_point: u64,

    stream: T

}

impl<T: Read + Seek> ScnFile<T> {

    pub fn new(ref_table: ScnRefTable, entry_point: u64, stream: T) -> Self {
        Self {
            ref_table,
            entry_point,
            stream
        }
    }

    pub fn ref_table(&self) -> &ScnRefTable {
        &self.ref_table
    }

    pub fn entry_point(&self) -> u64 {
        self.entry_point
    }

    pub fn unwrap(self) -> (ScnRefTable, u64, T) {
        (self.ref_table, self.entry_point, self.stream)
    }

}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::BufReader};

    use crate::reader::ScnReader;

    #[test]
    fn test() {
        let mut file = File::open("sample.ks.scn").unwrap();

        let file = ScnReader::open_scn_file(BufReader::new(&mut file)).unwrap();
        
        let ref_table = file.ref_table();

        for i in 0..ref_table.strings_len() {
            let string = &ref_table.strings()[i];

            println!("[{}]={}", i, string);
        }
    }
}