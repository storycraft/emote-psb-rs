/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod types;

pub mod header;

pub mod reader;
pub mod writer;

pub mod safe_index_vec;

use header::PsbHeader;
use io::Seek;
use types::PsbValue;
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

    resources: Vec<Vec<u8>>,

    extra: Vec<Vec<u8>>

}

impl PsbRefTable {

    pub fn new(names: Vec<String>,strings: Vec<String>, resources: Vec<Vec<u8>>, extra: Vec<Vec<u8>>) -> Self {
        Self {
            names, strings, resources, extra
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
pub struct PsbFile<T: Read + Seek> {

    header: PsbHeader,
    
    ref_table: PsbRefTable,

    entry_point: u64,

    stream: T

}

impl<T: Read + Seek> PsbFile<T> {

    pub fn new(header: PsbHeader, ref_table: PsbRefTable, entry_point: u64, stream: T) -> Result<Self, PsbError> {
        Ok(Self {
            header,
            ref_table,
            entry_point,
            stream
        })
    }

    pub fn header(&self) -> PsbHeader {
        self.header
    }

    pub fn set_header(&mut self, header: PsbHeader) {
        self.header = header;
    }

    pub fn ref_table(&self) -> &PsbRefTable {
        &self.ref_table
    }

    pub fn entry_point(&self) -> u64 {
        self.entry_point
    }

    /// Read root tree.
    /// Returns read size, PsbValue tuple.
    pub fn read_root(&mut self) -> Result<(u64, PsbValue), PsbError> {
        self.stream.seek(SeekFrom::Start(self.entry_point as u64))?;
        PsbValue::from_bytes(&mut self.stream)
    }

    /// Unwrap stream
    pub fn unwrap(self) -> T {
        self.stream
    }

}