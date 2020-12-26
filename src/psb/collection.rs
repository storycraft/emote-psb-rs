/*
 * Created on Fri Dec 25 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{collections::HashMap, io::{Read, Seek, SeekFrom, Write}, ops::Index, slice::Iter};

use crate::{ScnError, ScnErrorKind, ScnRefTable};

use byteorder::{ReadBytesExt, WriteBytesExt};

use super::{PSB_TYPE_INTEGER_ARRAY_N, PsbValue, number::PsbNumber};

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

    pub fn iter(&self) -> Iter<'_, u64> {
        self.vec.iter()
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

    ref_offsets: PsbIntArray

}

impl PsbList {

    pub fn new(ref_offsets: PsbIntArray) -> Self {
        Self {
            ref_offsets
        }
    }

    pub fn len(&self) -> usize {
        self.ref_offsets.len()
    }

    pub fn unwrap(self) -> PsbIntArray {
        self.ref_offsets
    }

    pub fn from_bytes(stream: &mut impl Read) -> Result<(u64, PsbList), ScnError> {
        let (offsets_read, offsets) = PsbIntArray::from_bytes(stream.read_u8()? - PSB_TYPE_INTEGER_ARRAY_N, stream)?;
        
        Ok((offsets_read + 1, Self::new(offsets)))
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, ScnError> {
        stream.write_u8(self.ref_offsets.n())?;
        let written = self.ref_offsets.write_bytes(stream)?;

        Ok(1 + written)
    }

    pub fn load_from_stream<T: Read + Seek>(&self, stream: &mut T) -> Result<Vec<PsbValue>, ScnError> {
        let mut list = Vec::<PsbValue>::new();

        let start = stream.seek(SeekFrom::Current(0))?;

        for offset in self.ref_offsets.iter() {
            stream.seek(SeekFrom::Start(*offset))?;
            let (_, val) = PsbValue::from_bytes(stream)?;

            list.push(val);
        }

        stream.seek(SeekFrom::Start(start))?;

        Ok(list)
    }

}

#[derive(Debug)]
pub struct PsbMap {

    name_refs: PsbIntArray,
    ref_offsets: PsbIntArray

}

impl PsbMap {

    pub fn new(
        name_refs: PsbIntArray,
        ref_offsets: PsbIntArray
    ) -> Self {
        Self {
            name_refs,
            ref_offsets
        }
    }

    pub fn len(&self) -> usize {
        self.name_refs.len()
    }

    pub fn from_bytes(stream: &mut impl Read) -> Result<(u64, PsbMap), ScnError> {
        let (names_read, name_refs) = PsbIntArray::from_bytes(stream.read_u8()? - PSB_TYPE_INTEGER_ARRAY_N, stream)?;
        let (offsets_read, ref_offsets) = PsbIntArray::from_bytes(stream.read_u8()? - PSB_TYPE_INTEGER_ARRAY_N, stream)?;

        Ok((names_read + offsets_read + 2, Self::new(name_refs, ref_offsets)))
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, ScnError> {
        let mut written = 0_u64;

        stream.write_u8(self.name_refs.n())?;
        written += self.name_refs.write_bytes(stream)?;

        stream.write_u8(self.ref_offsets.n())?;
        written += self.ref_offsets.write_bytes(stream)?;

        Ok(written + 2)
    }

    pub fn load_from_stream<T: Read + Seek>(&self, ref_table: &ScnRefTable, stream: &mut T) -> Result<HashMap<String, PsbValue>, ScnError> {
        let mut map = HashMap::<String, PsbValue>::new();

        let start = stream.seek(SeekFrom::Current(0))?;

        for i in 0..self.name_refs.len() {
            stream.seek(SeekFrom::Start(self.ref_offsets[i]))?;
            let (_, val) = PsbValue::from_bytes(stream)?;

            let string = ref_table.get_string(self.name_refs[i] as usize);

            if string.is_none() {
                return Err(ScnError::new(ScnErrorKind::InvalidPSBValue, None));
            }

            map.insert(string.unwrap().clone(), val);
        }

        stream.seek(SeekFrom::Start(start))?;

        Ok(map)
    }

}