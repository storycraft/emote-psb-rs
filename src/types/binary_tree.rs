/*
 * Created on Sun Dec 27 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

use std::{collections::{BTreeMap, btree_map}, io::{Read, Seek, Write}, slice::Iter};

use crate::{PsbError, PsbErrorKind, safe_index_vec::SafeIndexVec};

use super::{PsbValue, collection::PsbIntArray};

/// Binary tree
pub struct PsbBinaryTree {

    pub list: Vec<Vec<u8>>

}

impl PsbBinaryTree {

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
        let (offsets_read, offsets) = match PsbValue::from_bytes(stream)? {
    
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
        let offsets = offsets.unwrap();
        let tree = tree.unwrap();
        let indexes = indexes.unwrap();

        /* println!("Original offsets: {:?}", offsets);
        println!("Original tree: {:?}", tree);
        println!("Original indexes: {:?}", indexes);*/

        let mut list = Vec::<Vec<u8>>::with_capacity(indexes.len());

        for index in indexes {
            let mut buffer = Vec::<u8>::new();
            
            let mut id = tree[index as usize];

            while id != 0 {
                // travel to child tree
                let next = tree[id as usize];

                // get values from offsets
                let decoded = id - offsets[next as usize];
                
                id = next;

                buffer.push(decoded as u8);
            }

            buffer.reverse();
            list.push(buffer);
        }

        Ok((offsets_read + tree_read + indexes_read, Self::from(list)))
    }

    pub fn build_tree(&self) -> TreeNode {
        let mut root = TreeNode::new();
        
        for data in &self.list {
            let mut last_node = &mut root;

            for byte in data {
                last_node = last_node.get_or_insert_mut(*byte);
            }

            last_node.get_or_insert(0);
        }
        
        root
    }

    pub fn write_bytes(&self, stream: &mut impl Write) -> Result<u64, PsbError> {
        let mut root = self.build_tree();

        let mut offsets = SafeIndexVec::new();
        let mut tree = SafeIndexVec::new();
        let mut indexes = SafeIndexVec::new();

        offsets.push(1);
        self.make_sub_tree(&mut root, Vec::new(), &mut offsets, &mut tree, &mut indexes);

        println!("Original tree: {:?}", tree);

        let offsets_written = PsbValue::IntArray(PsbIntArray::from(offsets.into_inner())).write_bytes(stream)?;
        let tree_written = PsbValue::IntArray(PsbIntArray::from(tree.into_inner())).write_bytes(stream)?;
        let indexes_written = PsbValue::IntArray(PsbIntArray::from(indexes.into_inner())).write_bytes(stream)?;

        Ok(offsets_written + tree_written + indexes_written)
    }

    // Returns last node 
    fn make_sub_tree(
        &self,
        current_node: &mut TreeNode,
        value: Vec<u8>,
        offsets: &mut SafeIndexVec<u64>,
        tree: &mut SafeIndexVec<u64>,
        indexes: &mut SafeIndexVec<u64>
    ) {
        let min_value = *current_node.min_value().unwrap_or(&0);
        let begin_pos = current_node.begin_pos;
        let current_id = current_node.id;

        // make_tree
        for (child_value, child) in current_node.iter_mut() {
            let id = if current_id == 0 || min_value < 1 {
                *child_value as u64 + offsets.get(current_id as usize).unwrap()
            } else {
                (*child_value - min_value) as u64 + begin_pos
            };

            tree.set(id as usize, current_id);
            child.id = id;
        }

        for (child_value, child) in current_node.iter_mut() {
            let child_max = *child.max_value().unwrap_or(&0) as usize;
            let child_min = *child.min_value().unwrap_or(&0) as usize;

            let pos = {
                let len = tree.len();
                if len > child_max {
                    len
                } else {
                    tree.set(child_max, 0);

                    tree.len()
                }
            };

            let count = child_max - child_min;
            let end = pos + count;
            
            tree.set(end, 0);

            if *child_value == 0 {
                let index = self.list.iter().position(|val| val.eq(&value)).unwrap() as u64;
                offsets.set(child.id as usize, index);
                indexes.set(index as usize, child.id);
            } else {
                let offset = (pos - child_min) as u64;
                offsets.set(child.id as usize, offset);
                child.begin_pos = pos as u64;
            }
        }

        for (child_value, child) in current_node.iter_mut() {
            let mut value = value.clone();
            value.push(*child_value);
            self.make_sub_tree(child, value, offsets, tree, indexes);
        }
    }

}

impl From<Vec<Vec<u8>>> for PsbBinaryTree {

    fn from(list: Vec<Vec<u8>>) -> Self {
        Self {
            list
        }
    }

}

#[derive(Debug)]
pub struct TreeNode {

    /// Children value, node
    children: BTreeMap<u8, TreeNode>,
    
    pub begin_pos: u64,
    pub id: u64

}

impl TreeNode {

    pub fn new() -> Self {
        Self {
            children: BTreeMap::new(),
            id: 0,
            begin_pos: 0
        }
    }

    pub fn min_value(&self) -> Option<&u8> {
        self.children.keys().min()
    }

    pub fn max_value(&self) -> Option<&u8> {
        self.children.keys().max()
    }

    pub fn iter(&self) -> btree_map::Iter<u8, Self> {
        self.children.iter()
    }

    pub fn iter_mut(&mut self) -> btree_map::IterMut<u8, Self> {
        self.children.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.children.len()
    }

    pub fn get(&self, value: u8) -> Option<&Self> {
        self.children.get(&value)
    }

    pub fn get_mut(&mut self, value: u8) -> Option<&mut Self> {
        self.children.get_mut(&value)
    }

    pub fn get_or_insert(&mut self, value: u8) -> &Self {
        if !self.children.contains_key(&value) {
            let new_node = Self::new();

            self.children.insert(value, new_node);
        }

        self.children.get(&value).unwrap()
    }

    pub fn get_or_insert_mut(&mut self, value: u8) -> &mut Self {
        if !self.children.contains_key(&value) {
            let new_node = Self::new();

            self.children.insert(value, new_node);
        }

        self.children.get_mut(&value).unwrap()
    }

}