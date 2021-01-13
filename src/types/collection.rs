/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{collections::{HashMap, hash_map}, io::{Read, Seek, SeekFrom, Write}, ops::Index, slice::Iter};

use crate::{PsbError, PsbErrorKind, PsbRefs};

use byteorder::{ReadBytesExt, WriteBytesExt};
use itertools::Itertools;

use super::{PSB_TYPE_INTEGER_ARRAY_N, PsbValue, number::PsbNumber};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PsbUintArray {

    vec: Vec<u64>

}

impl PsbUintArray {

    pub fn new() -> Self {
        Self {
            vec: Vec::new()
        }
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn iter(&self) -> Iter<'_, u64> {
        self.vec.iter()
    }

    pub fn vec(&self) -> &Vec<u64> {
        &self.vec
    }

    pub fn vec_mut(&mut self) -> &mut Vec<u64> {
        &mut self.vec
    }

    pub fn unwrap(self) -> Vec<u64> {
        self.vec
    }

    /// Item byte size
    pub fn get_item_n(&self) -> u8 {
        PsbNumber::get_uint_n(self.vec.iter().max().unwrap().clone())
    }

    pub fn get_n(&self) -> u8 {
        PsbNumber::get_uint_n(self.vec.len() as u64).max(1)
    }

    pub fn from_bytes(n: u8, stream: &mut impl Read) -> Result<(u64, PsbUintArray), PsbError> {
        let (count_read, item_count) = PsbNumber::read_uint(n, stream)?;

        let item_byte_size = stream.read_u8()? - PSB_TYPE_INTEGER_ARRAY_N;

        let mut list = Vec::<u64>::new();

        let mut item_total_read = 0_u64;
        for _ in 0..item_count {
            let (item_read, item) = PsbNumber::read_uint(item_byte_size, stream)?;
            list.push(item as u64);

            item_total_read += item_read;
        }

        Ok((count_read + item_total_read + 1, PsbUintArray::from(list)))
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
        let len = self.vec.len() as u64;

        let count_written = PsbNumber::write_uint(self.get_n(), len, stream)? as u64;

        if len < 1 {
            stream.write_u8(PSB_TYPE_INTEGER_ARRAY_N + 1)?;
            Ok(1 + count_written)
        } else {
            let n = self.get_item_n();

            stream.write_u8(n + PSB_TYPE_INTEGER_ARRAY_N)?;

            for num in &self.vec {
                PsbNumber::write_uint(n, *num, stream)?;
            }

            Ok(1 + count_written + n as u64 * self.vec.len() as u64)
        }
    }

}

impl From<Vec<u64>> for PsbUintArray {

    fn from(vec: Vec<u64>) -> Self {
        Self {
            vec
        }
    }

}

impl Index<usize> for PsbUintArray {
    
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index]
    }
}

#[derive(Debug, PartialEq)]
pub struct PsbList {

    values: Vec<PsbValue>

}

impl PsbList {

    pub fn new() -> Self {
        Self {
            values: Vec::new()
        }
    }

    pub fn values(&self) -> &Vec<PsbValue> {
        &self.values
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn iter(&self) -> Iter<'_, PsbValue> {
        self.values.iter()
    }

    pub fn unwrap(self) -> Vec<PsbValue> {
        self.values
    }

    pub fn from_bytes<T: Read + Seek>(stream: &mut T, table: &PsbRefs) -> Result<(u64, PsbList), PsbError> {
        let (offsets_read, ref_offsets) = match PsbValue::from_bytes(stream)? {
    
            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))

        }?;

        if ref_offsets.len() < 1 {
            return Ok((offsets_read, Self::new()));
        }

        let max_offset = ref_offsets.iter().max().unwrap();

        let mut values = Vec::<PsbValue>::with_capacity(ref_offsets.len());

        let start = stream.seek(SeekFrom::Current(0)).unwrap();
        let mut total_read = 0_u64;

        for offset in ref_offsets.iter() {
            stream.seek(SeekFrom::Start(start + *offset as u64))?;
            let (read, val) = PsbValue::from_bytes_refs(stream, table)?;

            values.push(val);

            if max_offset == offset {
                total_read = read + *offset as u64;
            }
        }

        stream.seek(SeekFrom::Start(start + total_read))?;

        Ok((offsets_read + total_read, Self::from(values)))
    }

    pub fn write_bytes(&self, stream: &mut impl Write, table: &PsbRefs) -> Result<u64, PsbError> {
        let mut offsets = Vec::<u64>::new();
        let mut data_buffer = Vec::<u8>::new();

        let mut total_data_written = 0_i64;

        for value in &self.values {
            offsets.push(total_data_written as u64);

            total_data_written += value.write_bytes_refs(&mut data_buffer, table)? as i64;
        }

        let offset_written = PsbValue::IntArray(PsbUintArray::from(offsets)).write_bytes(stream)?;
        stream.write_all(&data_buffer)?;

        Ok(offset_written + total_data_written as u64)
    }

    pub fn collect_strings(&self, vec: &mut Vec<String>) {
        for child in self.values.iter() {
            match child {

                PsbValue::Object(child_obj) => {
                    child_obj.collect_strings(vec);
                }

                PsbValue::List(child_list) => {
                    child_list.collect_strings(vec);
                }

                PsbValue::String(string) => {
                    if !vec.contains(string.string()) {
                        vec.push(string.string().clone());
                    }
                }

                _ => {}
            }
            
        }
    }

    pub fn collect_names(&self, vec: &mut Vec<String>) {
        for child in self.values.iter() {
            match child {

                PsbValue::Object(child_obj) => {
                    child_obj.collect_names(vec);
                }

                PsbValue::List(child_list) => {
                    child_list.collect_names(vec);
                }

                _ => {}
            }
            
        }
    }

}

impl From<Vec<PsbValue>> for PsbList {

    fn from(values: Vec<PsbValue>) -> Self {
        Self {
            values
        }
    }

}

#[derive(Debug, PartialEq)]
pub struct PsbObject {

    // key, PsbValue Map
    map: HashMap<String, PsbValue>

}

impl PsbObject {

    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn get_value(&self, key: String) -> Option<&PsbValue> {
        self.map.get(&key)
    }

    pub fn map(&self) -> &HashMap<String, PsbValue> {
        &self.map
    }

    pub fn iter(&self) -> hash_map::Iter<'_, String, PsbValue>{
        self.map.iter()
    }

    pub fn unwrap(self) -> HashMap<String, PsbValue> {
        self.map
    }

    pub fn from_bytes<T: Read + Seek>(stream: &mut T, table: &PsbRefs) -> Result<(u64, PsbObject), PsbError> {
        let (names_read, name_refs) = match PsbValue::from_bytes(stream)? {
    
            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))

        }?;

        let (offsets_read, ref_offsets) = match PsbValue::from_bytes(stream)? {
    
            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))

        }?;

        if name_refs.len() < 1 {
            return Ok((names_read + offsets_read, Self::new()));
        }

        let max_offset = ref_offsets.iter().max().unwrap();

        let mut map = HashMap::<String, PsbValue>::new();

        let start = stream.seek(SeekFrom::Current(0)).unwrap();
        let mut total_read = 0_u64;

        for (name_ref, offset) in name_refs.iter().zip(ref_offsets.iter()) {
            stream.seek(SeekFrom::Start(start + *offset as u64))?;
            let (read, val) = PsbValue::from_bytes_refs(stream, table)?;

            let key = table.names().get(*name_ref as usize);
           
            if key.is_none() {
                return Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None));
            }

            map.insert(key.unwrap().clone(), val);

            if *max_offset == *offset {
                total_read = read + *offset as u64;
            }
        }

        stream.seek(SeekFrom::Start(start + total_read))?;

        Ok((names_read + offsets_read + total_read, Self::from(map)))
    }

    pub fn write_bytes(&self, stream: &mut impl Write, ref_table: &PsbRefs) -> Result<u64, PsbError> {
        let mut ref_cache = HashMap::<&String, u64>::new();

        let mut name_refs = Vec::<u64>::new();
        let mut offsets = Vec::<u64>::new();
        let mut data_buffer = Vec::<u8>::new();

        let mut total_data_written = 0_u64;

        for name in self.map.keys().into_iter().sorted() {
            let value = self.map.get(name).unwrap();

            let name_ref = if ref_cache.contains_key(name) {
                *ref_cache.get(name).unwrap()
            } else {
                match ref_table.find_name_index(name) {
                    Some(index) => {
                        ref_cache.insert(name, index);

                        Ok(index)
                    },

                    None => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))
                }?
            };

            name_refs.push(name_ref);
            offsets.push(total_data_written);

            total_data_written += value.write_bytes_refs(&mut data_buffer, ref_table)?;
        }

        let names_written = PsbValue::IntArray(PsbUintArray::from(name_refs)).write_bytes(stream)?;
        let offset_written = PsbValue::IntArray(PsbUintArray::from(offsets)).write_bytes(stream)?;

        stream.write_all(&data_buffer)?;

        Ok(names_written + offset_written + total_data_written as u64)
    }

    pub fn collect_names(&self, vec: &mut Vec<String>) {
        for (name, child) in self.map.iter() {
            match child {

                PsbValue::Object(child_obj) => {
                    child_obj.collect_names(vec);
                }

                PsbValue::List(child_list) => {
                    child_list.collect_names(vec);
                }

                _ => {}
            }

            if !vec.contains(&name) {
                vec.push(name.clone());
            }
        }
    }

    pub fn collect_strings(&self, vec: &mut Vec<String>) {
        for (_, child) in self.map.iter() {
            match child {

                PsbValue::Object(child_obj) => {
                    child_obj.collect_strings(vec);
                }

                PsbValue::List(child_list) => {
                    child_list.collect_strings(vec);
                }

                PsbValue::String(string) => {
                    if !vec.contains(string.string()) {
                        vec.push(string.string().clone());
                    }
                }

                _ => {}
            }
            
        }
    }

}

impl From<HashMap<String, PsbValue>> for PsbObject {

    fn from(map: HashMap<String, PsbValue>) -> Self {
        Self {
            map
        }
    }

}