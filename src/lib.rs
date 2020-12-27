

/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

pub mod psb;

pub mod header;

pub mod reader;
pub mod writer;

use header::ScnHeader;
use io::Seek;
use psb::PsbValue;
pub use reader::ScnReader;

use std::{error::Error, io::{self, Read, SeekFrom}};

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
    InvalidOffsetTable,
    Custom

}

impl From<io::Error> for ScnError {

    fn from(err: io::Error) -> Self {
        ScnError::new(ScnErrorKind::Io(err), None)
    }

}

#[derive(Debug)]
pub struct ScnRefTable {

    names: Vec<String>,

    strings: Vec<String>,

    resources: Vec<Vec<u8>>,

    extra: Vec<Vec<u8>>

}

impl ScnRefTable {

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
pub struct ScnFile<T: Read + Seek> {

    header: ScnHeader,
    
    ref_table: ScnRefTable,

    entry_point: u64,

    stream: T

}

impl<T: Read + Seek> ScnFile<T> {

    pub fn new(header: ScnHeader, ref_table: ScnRefTable, entry_point: u64, mut stream: T) -> Result<Self, ScnError> {
        Ok(Self {
            header,
            ref_table,
            entry_point,
            stream
        })
    }

    pub fn header(&self) -> &ScnHeader {
        &self.header
    }

    pub fn ref_table(&self) -> &ScnRefTable {
        &self.ref_table
    }

    pub fn entry_point(&self) -> u64 {
        self.entry_point
    }

    /// Read root tree.
    /// Returns read size, PsbValue tuple.
    pub fn read_root(&mut self) -> Result<(u64, PsbValue), ScnError> {
        self.stream.seek(SeekFrom::Start(self.entry_point as u64))?;
        PsbValue::from_bytes(&mut self.stream)
    }

    /// Unwrap as ScnHeader, ScnRefTable, entry point, stream tuple
    pub fn unwrap(self) -> (ScnHeader, ScnRefTable, u64, T) {
        (self.header, self.ref_table, self.entry_point, self.stream)
    }

}

#[cfg(test)]
mod tests {
    use std::{fs::File, io::{BufReader, Cursor, Read}};

    use crate::{ScnRefTable, psb::{PsbValue, collection::PsbList, number::PsbNumber}, reader::ScnReader};

    #[test]
    fn test() {
        let mut file = File::open("sample3.txt.scn").unwrap();

        let mut mem_buf = Vec::new();

        BufReader::new(&mut file).read_to_end(&mut mem_buf).unwrap();

        let mut file = ScnReader::open_scn_file(Cursor::new(mem_buf)).unwrap();
        
        let (_, root) = file.read_root().unwrap();

        match root {

            PsbValue::Object(root) => {
                let scn_root: &PsbList = root.iter().find_map(|item| match item.1 {
                        PsbValue::List(list) if list.len() > 0 => {
                            Some(list)
                        },
                        
                        _ => {
                            None
                        }
                }).expect("Cannot find root scn!!");
                
                let scn = scn_root.iter().last().unwrap();

                match scn {

                    PsbValue::Object(scn) => {
                        let list = scn.iter().find_map(|item| match item.1 {
                            PsbValue::List(list) if list.len() > 0 => {
                                Some(list)
                            },
                            
                            _ => {
                                None
                            }
                        }).expect("Cannot find scn!!");

                        // println!("entry: {:?}", entry);

                        for item in list.iter() {
                            match item {

                                PsbValue::List(obj) => {
                                    let (character, text) = (&obj.values()[0], &obj.values()[2]);

                                    let character = match character {

                                        PsbValue::String(string_ref) => {
                                            file.ref_table().get_string(string_ref.ref_index() as usize).unwrap()
                                        },

                                        _ => "None"
                                    };

                                    let text = match text {

                                        PsbValue::String(string_ref) => {
                                            file.ref_table().get_string(string_ref.ref_index() as usize).unwrap()
                                        },

                                        _ => "None"
                                    };

                                    println!("{:?}: {:?}", character, text);
                                },

                                found => {
                                    println!("{:?}", found)
                                }
                            }
                        }

                    },

                    _ => {
                        panic!("This cannot be happen!3")
                    }
                }
            }

            _ => {
                panic!("This cannot be happen!1")
            }
        }

        // println!("root: {:?}", root);
        
        // display(0, &root, file.ref_table());
    }

    fn display(depth: u16, value: &PsbValue, ref_table: &ScnRefTable) {
        match value {

            PsbValue::None => print!("None"),

            PsbValue::Null => print!("null"),

            PsbValue::Bool(flag) => print!("{}", flag),

            PsbValue::Number(number) => {
                match number {
                    PsbNumber::Integer(number) => {
                        print!("{}", number)
                    },

                    PsbNumber::Double(number) => {
                        print!("{}", number)
                    },

                    PsbNumber::Float(number) => {
                        print!("{}", number)
                    }
                }
            },

            PsbValue::IntArray(array) => {
                print!("[ ");
                for value in array.iter() {
                    print!("{}, ", value);
                }
                print!(" ]");
            },

            PsbValue::String(res) => print!("\"{}\"", ref_table.get_string(res.ref_index() as usize).unwrap()),

            PsbValue::List(list) => {
                println!("[",);
                for value in list.iter() {
                    print!("{}", " ".repeat(depth as usize * 2));
                    display(depth + 1, value, ref_table);
                    println!(", ");
                }
                print!("{}]", " ".repeat(depth as usize * 2));
            }

            PsbValue::Object(map) => {
                println!("{{");
                for (name_ref, value) in map.iter() {
                    let key = ref_table.get_name(*name_ref as usize).unwrap();
                    
                    print!("{}\"{}\": ", " ".repeat(depth as usize * 2), key);
                    display(depth + 1, value, ref_table);
                    println!(",");
                }

                print!("{}}}", " ".repeat(depth as usize * 2));
            }

            PsbValue::Resource(res) => {}

            PsbValue::ExtraResource(res) => {}

            PsbValue::CompilerNumber => {}

            PsbValue::CompilerString => {}
            PsbValue::CompilerResource => {}
            PsbValue::CompilerDecimal => {}
            PsbValue::CompilerArray => {}
            PsbValue::CompilerBool => {}
            PsbValue::CompilerBinaryTree => {}
        }
    }
}