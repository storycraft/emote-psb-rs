mod util;

use std::{
    collections::{HashMap, hash_map},
    io::{Read, Seek, Write},
};

use scopeguard::guard;

use crate::{
    psb::{binary_tree::util::SparseVec, table::StringTable},
    value::{de, util::read_uint_array},
};

/// Binary tree
pub struct PsbBinaryTree(pub StringTable);

impl PsbBinaryTree {
    pub fn read_io(stream: &mut (impl Read + Seek), buf: &mut Vec<u64>) -> Result<Self, de::Error> {
        let offsets_start = buf.len();
        let mut buf = guard(buf, |buf| {
            buf.drain(offsets_start..);
        });
        read_uint_array(stream, *buf)?;
        let tree_start = buf.len();
        read_uint_array(stream, *buf)?;
        let indexes_start = buf.len();
        read_uint_array(stream, *buf)?;

        let offsets = &buf[offsets_start..tree_start];
        let tree = &buf[tree_start..indexes_start];
        let indexes = &buf[indexes_start..];
        let mut table = StringTable::with_capacity(buf.len() - indexes_start);
        let mut name = vec![];
        for &index in indexes {
            let mut id = tree[index as usize];

            while id != 0 {
                // travel to child tree
                let next = tree[id as usize];

                // get values from offsets
                let decoded = id - offsets[next as usize];

                id = next;

                name.push(decoded as u8);
            }
            name.reverse();
            table.push(str::from_utf8(&name).map_err(|_| de::Error::InvalidValue)?);
            name.clear();
        }

        Ok(Self(table))
    }

    // pub fn write_tree(
    //     &self,
    //     writer: &mut impl Write,
    // ) -> Result<(), PsbValueWriteError> {
    //     let mut root = self.build_tree();

    //     let mut offsets = SparseVec::new();
    //     let mut tree = SparseVec::new();
    //     let mut indexes = SparseVec::new();

    //     offsets.push(1);
    //     self.make_sub_tree(&mut root, Vec::new(), &mut offsets, &mut tree, &mut indexes);

    //     writer.write_uint_array(&offsets.into_inner())?;
    //     writer.write_uint_array(&tree.into_inner())?;
    //     writer.write_uint_array(&indexes.into_inner())?;
    //     Ok(())
    // }

    // fn build_tree(&self) -> TreeNode {
    //     let mut root = TreeNode::new();

    //     for data in self.0.iter() {
    //         let mut last_node = &mut root;

    //         for byte in data.as_bytes() {
    //             last_node = last_node.get_or_insert_mut(*byte);
    //         }
    //         last_node.get_or_insert(0);
    //     }

    //     root
    // }

    // Returns last node
    fn make_sub_tree(
        &self,
        current_node: &mut TreeNode,
        value: Vec<u8>,
        offsets: &mut SparseVec<u64>,
        tree: &mut SparseVec<u64>,
        indexes: &mut SparseVec<u64>,
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
                let index = self
                    .0
                    .iter()
                    .position(|val| val.as_bytes().eq(&value))
                    .unwrap() as u64;
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

#[derive(Debug)]
struct TreeNode {
    /// Children value, node
    children: HashMap<u8, TreeNode>,

    pub begin_pos: u64,
    pub id: u64,
}

impl Default for TreeNode {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeNode {
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            id: 0,
            begin_pos: 0,
        }
    }

    pub fn min_value(&self) -> Option<&u8> {
        self.children.keys().min()
    }

    pub fn max_value(&self) -> Option<&u8> {
        self.children.keys().max()
    }

    pub fn iter(&self) -> hash_map::Iter<'_, u8, Self> {
        self.children.iter()
    }

    pub fn iter_mut(&mut self) -> hash_map::IterMut<'_, u8, Self> {
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
        self.children.entry(value).or_default();

        self.children.get(&value).unwrap()
    }

    pub fn get_or_insert_mut(&mut self, value: u8) -> &mut Self {
        self.children.entry(value).or_default();

        self.children.get_mut(&value).unwrap()
    }
}
