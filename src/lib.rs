/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod types;

pub mod header;

pub mod reader;
pub mod writer;

mod internal;

use header::PsbHeader;
use io::Seek;
use types::{PsbValue, collection::PsbObject};
pub use reader::PsbReader;

use std::{error::Error, io::{self, Read, SeekFrom}};

/// psb file signature
pub const PSB_SIGNATURE: u32 = 0x425350;

/// compressed psb file signature
pub const PSB_MDF_SIGNATURE: u32 = 0x66646D;

#[derive(Debug)]
pub struct PsbError {

    kind: PsbErrorKind,
    err: Option<Box<dyn Error>>

}

impl PsbError {

    pub fn new(kind: PsbErrorKind, err: Option<Box<dyn Error>>) -> Self {
        Self { kind, err }
    }

    pub fn kind(&self) -> &PsbErrorKind {
        &self.kind
    }

    pub fn source(&self) -> &Option<Box<dyn Error>> {
        &self.err
    }

}

#[derive(Debug)]
pub enum PsbErrorKind {

    Io(io::Error),
    InvalidFile,
    InvalidHeader,
    UnknownHeaderVersion,
    InvalidIndex,
    InvalidPSBValue,
    InvalidPSBRoot,
    InvalidOffsetTable,
    Custom

}

impl From<io::Error> for PsbError {

    fn from(err: io::Error) -> Self {
        PsbError::new(PsbErrorKind::Io(err), None)
    }

}

#[derive(Debug, Clone)]
pub struct PsbRefTable {

    names: Vec<String>,

    strings: Vec<String>,

}

impl PsbRefTable {

    pub fn new(names: Vec<String>,strings: Vec<String>) -> Self {
        Self {
            names, strings
        }
    }

    pub fn names(&self) -> &Vec<String> {
        &self.names
    }

    pub fn names_mut(&mut self) -> &mut Vec<String> {
        &mut self.names
    }

    pub fn names_len(&self) -> usize {
        self.names.len()
    }

    pub fn get_name(&self, index: usize) -> Option<&String> {
        self.names.get(index)
    }

    pub fn get_name_mut(&mut self, index: usize) -> Option<&mut String> {
        self.names.get_mut(index)
    }

    pub fn find_name_index(&self, name: &String) -> Option<u64> {
        for (i, nm) in self.names.iter().enumerate() {
            if nm == name {
                return Some(i as u64)
            }
        }

        None
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

    pub fn find_string_index(&self, string: &String) -> Option<u64> {
        for (i, st) in self.strings.iter().enumerate() {
            if st == string {
                return Some(i as u64)
            }
        }

        None
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PsbResourcesTable {

    offset_pos: u32,
    lengths_pos: u32,
    data_pos: u32

}

impl PsbResourcesTable {

    pub fn new(offset_pos: u32, lengths_pos: u32, data_pos: u32) -> Self {
        Self {
            offset_pos,
            lengths_pos,
            data_pos
        }
    }

    pub fn offset_pos(&self) -> u32 {
        self.offset_pos
    }

    pub fn lengths_pos(&self) -> u32 {
        self.lengths_pos
    }

    pub fn data_pos(&self) -> u32 {
        self.data_pos
    }

}

#[derive(Debug)]
pub struct VirtualPsb {

    header: PsbHeader,

    resources: Vec<Vec<u8>>,
    extra: Vec<Vec<u8>>,

    root: PsbObject

}

impl VirtualPsb {

    pub fn new(
        header: PsbHeader,
        resources: Vec<Vec<u8>>,
        extra: Vec<Vec<u8>>,
        root: PsbObject
    ) -> Self {
        Self {
            header,
            resources,
            extra,
            root
        }
    }

    pub fn header(&self) -> PsbHeader {
        self.header
    }

    pub fn resources(&self) -> &Vec<Vec<u8>> {
        &self.resources
    }

    pub fn resources_mut(&mut self) -> &mut Vec<Vec<u8>> {
        &mut self.resources
    }

    pub fn extra(&self) -> &Vec<Vec<u8>> {
        &self.extra
    }

    pub fn extra_mut(&mut self) -> &mut Vec<Vec<u8>> {
        &mut self.extra
    }

    pub fn root(&self) -> &PsbObject {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut PsbObject {
        &mut self.root
    }

    pub fn set_root(&mut self, root: PsbObject) {
        self.root = root;
    }

    pub fn unwrap(self) -> (PsbHeader, Vec<Vec<u8>>, Vec<Vec<u8>>, PsbObject) {
        (
            self.header,
            self.resources,
            self.extra,
            self.root
        )
    }

}

#[derive(Debug)]
pub struct PsbFile<T: Read + Seek> {

    header: PsbHeader,
    
    ref_table: PsbRefTable,
    res_table: PsbResourcesTable,
    extra_table: Option<PsbResourcesTable>,

    entry_point: u64,

    stream: T

}

impl<T: Read + Seek> PsbFile<T> {

    pub fn new(header: PsbHeader, ref_table: PsbRefTable, res_table: PsbResourcesTable, extra_table: Option<PsbResourcesTable>, entry_point: u64, stream: T) -> Self {
        Self {
            header,
            ref_table,
            res_table,
            extra_table,
            entry_point,
            stream
        }
    }

    pub fn header(&self) -> PsbHeader {
        self.header
    }

    pub fn ref_table(&self) -> &PsbRefTable {
        &self.ref_table
    }

    pub fn res_table(&self) -> PsbResourcesTable {
        self.res_table
    }

    pub fn extra_table(&self) -> Option<PsbResourcesTable> {
        self.extra_table
    }

    pub fn entry_point(&self) -> u64 {
        self.entry_point
    }

    pub fn load_root(&mut self) -> Result<PsbObject, PsbError> {
        self.stream.seek(SeekFrom::Start(self.entry_point as u64))?;
        let (_, root) = PsbValue::from_bytes_table(&mut self.stream, &self.ref_table)?;

        if let PsbValue::Object(root_obj) = root {
            Ok(root_obj)
        } else {
            Err(PsbError::new(PsbErrorKind::InvalidPSBRoot, None))
        }
    }

    pub fn load_resources(&mut self) -> Result<Vec<Vec<u8>>, PsbError> {
        Self::load_from_table(&mut self.stream, self.res_table)
    }

    pub fn load_extra(&mut self) -> Result<Vec<Vec<u8>>, PsbError> {
        if self.extra_table.is_none() {
            Ok(Vec::new())
        } else {
            Self::load_from_table(&mut self.stream, self.extra_table.unwrap())
        }
    }

    fn load_from_table<R: Read + Seek>(stream: &mut R, table: PsbResourcesTable) -> Result<Vec<Vec<u8>>, PsbError> {
        stream.seek(SeekFrom::Start(table.offset_pos() as u64))?;
        let (_, resource_offsets) = match PsbValue::from_bytes(stream)? {
    
            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))

        }?;
        
        stream.seek(SeekFrom::Start(table.lengths_pos() as u64))?;
        let (_, resource_lengths) = match PsbValue::from_bytes(stream)? {

            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))

        }?;

        if resource_offsets.len() < resource_lengths.len() {
            return Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None));
        }

        let mut resources = Vec::new();

        let resource_offsets = resource_offsets.unwrap();
        let resource_lengths = resource_lengths.unwrap();

        for i in 0..resource_offsets.len() {
            let mut buffer = Vec::new();

            stream.seek(SeekFrom::Start(table.data_pos() as u64 + resource_offsets[i]))?;
            stream.take(resource_lengths[i] as u64).read_to_end(&mut buffer)?;

            resources.push(buffer);
        }

        Ok(resources)
    }

    /// Load Psb file to memory.
    /// Returns VirtualPsb.
    pub fn load(&mut self) -> Result<VirtualPsb, PsbError> {
        let root = self.load_root()?;
        let res = self.load_resources()?;
        let extra = self.load_extra()?;

        Ok(VirtualPsb::new(self.header, res, extra, root))
        
    }

    /// Unwrap stream
    pub fn unwrap(self) -> T {
        self.stream
    }

}

#[cfg(test)]
#[test]
fn test() {
    let input = std::fs::File::open("01_com_001_01.ks.scn").unwrap();

    let mut psb_file = PsbReader::open_psb(input).unwrap();
    let psb = psb_file.load().unwrap();

    println!("Loaded: {:?}", psb)
}