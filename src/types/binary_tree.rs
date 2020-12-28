/*
 * Created on Sun Dec 27 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{io::{Read, Seek, Write}, slice::Iter};

use crate::{PsbError, PsbErrorKind};

use super::PsbValue;

/// Binary tree
pub struct BinaryTree {

    pub list: Vec<Vec<u8>>

}

impl BinaryTree {

    pub fn new() -> Self {
        Self {
            list: Vec::new()
        }
    }

    pub fn list(&self) -> &Vec<Vec<u8>> {
        &self.list
    }

    pub fn list_mut(&mut self) -> &mut Vec<Vec<u8>> {
        &mut self.list
    }

    pub fn len(&self) -> usize {
        self.list.len()
    }

    pub fn iter(&self) -> Iter<'_, Vec<u8>> {
        self.list.iter()
    }

    pub fn unwrap(self) -> Vec<Vec<u8>> {
        self.list
    }

    pub fn from_bytes<T: Read + Seek>(stream: &mut T) -> Result<(u64, Self), PsbError> {
        let mut list = Vec::<Vec<u8>>::new();

        let (sets_read, sets) = match PsbValue::from_bytes(stream)? {
    
            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))

        }?;
        let (tree_read, tree) = match PsbValue::from_bytes(stream)? {

            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))

        }?;
        let (indexes_read, indexes) = match PsbValue::from_bytes(stream)? {

            (read, PsbValue::IntArray(array)) => Ok((read, array)),

            _ => Err(PsbError::new(PsbErrorKind::InvalidOffsetTable, None))

        }?;

        // Unwrap all to vec
        let sets = sets.unwrap();
        let tree = tree.unwrap();
        let indexes = indexes.unwrap();

        for index in indexes {
            let mut buffer = Vec::<u8>::new();
            
            let mut byte = tree[index as usize];

            while byte != 0 {
                let code = tree[byte as usize];

                let decoded = byte - sets[code as usize];
                
                byte = code;

                buffer.push(decoded as u8);
            }

            buffer.reverse();
            list.push(buffer);
        }

        Ok((sets_read + tree_read + indexes_read, Self::from(list)))
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
        todo!();
    }

}

impl From<Vec<Vec<u8>>> for BinaryTree {

    fn from(list: Vec<Vec<u8>>) -> Self {
        Self {
            list
        }
    }

}