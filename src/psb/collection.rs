/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{collections::{HashMap, hash_map}, io::{Read, Seek, SeekFrom, Write}, iter::Zip, ops::Index, slice::Iter};

use crate::ScnError;

use byteorder::{ReadBytesExt, WriteBytesExt};

use super::{PSB_TYPE_INTEGER_ARRAY_N, PsbValue, number::PsbNumber};

#[derive(Debug, Clone)]
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
    pub fn n(&self) -> u8 {
        PsbNumber::get_n(self.vec.iter().max().unwrap().clone()).min(1)
    }

    pub fn from_bytes(n: u8, stream: &mut impl Read) -> Result<(u64, PsbIntArray), ScnError> {
        let (count_read, item_count) = PsbNumber::read_integer(n, stream)?;

        let item_byte_size = stream.read_u8()? - PSB_TYPE_INTEGER_ARRAY_N;

        let mut list = Vec::<u64>::new();

        let mut item_total_read = 0_u64;
        for _ in 0..item_count {
            let (item_read, item) = PsbNumber::read_integer(item_byte_size, stream)?;
            list.push(item);

            item_total_read += item_read;
        }

        Ok((count_read + item_total_read, PsbIntArray::new(list)))
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, ScnError> {
        if self.vec.len() < 1 {
            stream.write_u8(PSB_TYPE_INTEGER_ARRAY_N + 1)?;
            Ok(1)
        } else {
            let n = self.n().min(1);

            stream.write_u8(n + PSB_TYPE_INTEGER_ARRAY_N)?;

            for num in &self.vec {
                stream.write_all(&num.to_le_bytes()[..n as usize])?;
            }

            Ok(1 + n as u64 * self.vec.len() as u64)
        }
    }

}

impl Index<usize> for PsbIntArray {
    
    type Output = u64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.vec[index]
    }
}

#[derive(Debug)]
pub struct PsbList {

    values: Vec<PsbValue>

}

impl PsbList {

    pub fn new(values: Vec<PsbValue>) -> Self {
        Self {
            values
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

    pub fn from_bytes<T: Read + Seek>(stream: &mut T) -> Result<(u64, PsbList), ScnError> {
        let (offsets_read, ref_offsets) = PsbIntArray::from_bytes(stream.read_u8()? - PSB_TYPE_INTEGER_ARRAY_N, stream)?;

        if ref_offsets.len() < 1 {
            return Ok((offsets_read + 1, Self::new(Vec::new())));
        }

        let max_offset = ref_offsets.iter().max().unwrap();

        let mut values = Vec::<PsbValue>::with_capacity(ref_offsets.len());

        let start = stream.seek(SeekFrom::Current(0))?;
        let mut total_read = 0_u64;

        for offset in ref_offsets.iter() {
            stream.seek(SeekFrom::Start(start + *offset))?;
            let (read, val) = PsbValue::from_bytes(stream)?;

            values.push(val);

            if *max_offset == *offset {
                total_read = read + *offset;
            }
        }

        stream.seek(SeekFrom::Start(start + total_read))?;

        Ok((offsets_read + 1 + total_read, Self::new(values)))
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, ScnError> {

        todo!();

        Ok(1)
    }

}

#[derive(Debug)]
pub struct PsbObject {

    // String ref, PsbValue HashMap
    map: HashMap<u64, PsbValue>

}

impl PsbObject {

    pub fn new(
        map: HashMap<u64, PsbValue>
    ) -> Self {
        Self {
            map
        }
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn get_value(&self, string_ref: u64) -> Option<&PsbValue> {
        self.map.get(&string_ref)
    }

    pub fn map(&self) -> &HashMap<u64, PsbValue> {
        &self.map
    }

    pub fn iter(&self) -> hash_map::Iter<'_, u64, PsbValue>{
        self.map.iter()
    }

    pub fn from_bytes<T: Read + Seek>(stream: &mut T) -> Result<(u64, PsbObject), ScnError> {
        let (names_read, name_refs) = PsbIntArray::from_bytes(stream.read_u8()? - PSB_TYPE_INTEGER_ARRAY_N, stream)?;
        let (offsets_read, ref_offsets) = PsbIntArray::from_bytes(stream.read_u8()? - PSB_TYPE_INTEGER_ARRAY_N, stream)?;

        if name_refs.len() < 1 {
            return Ok((names_read + offsets_read + 2, Self::new(HashMap::new())));
        }

        let max_offset = ref_offsets.iter().max().unwrap();

        let mut map = HashMap::<u64, PsbValue>::with_capacity(name_refs.len());

        let start = stream.seek(SeekFrom::Current(0))?;
        let mut total_read = 0_u64;

        for (name_ref, offset) in name_refs.iter().zip(ref_offsets.iter()) {
            stream.seek(SeekFrom::Start(start + *offset))?;
            let (read, val) = PsbValue::from_bytes(stream)?;
           
            map.insert(*name_ref, val);

            if *max_offset == *offset {
                total_read = read + *offset;
            }
        }

        stream.seek(SeekFrom::Start(start + total_read))?;

        Ok((names_read + offsets_read + 2 + total_read, Self::new(map)))
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, ScnError> {
        let mut written = 0_u64;

        todo!();

        Ok(written + 2)
    }

}