mod util;

use std::{
    collections::HashMap,
    io::{self, Read, Write},
};

use scopeguard::guard;
use slab::Slab;

use crate::{
    psb::{btree::util::SparseVec, table::StringTable},
    value::{
        de,
        util::{read_uint_array, write_uint_array},
    },
};

pub fn read_btree(stream: &mut impl Read, buf: &mut Vec<u64>) -> Result<StringTable, de::Error> {
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
        table.push_str(str::from_utf8(&name).map_err(|_| de::Error::InvalidValue)?);
        name.clear();
    }

    Ok(table)
}

pub struct PsbBtree(pub StringTable);

impl PsbBtree {
    pub fn write_tree(&self, stream: &mut impl Write) -> io::Result<()> {
        let mut arena = self.build_tree();

        let mut offsets = SparseVec::new();
        let mut tree = SparseVec::new();
        let mut indexes = SparseVec::new();

        offsets.push(1);
        let root = arena.root;
        self.make_sub_tree(&mut arena, root, Vec::new(), &mut offsets, &mut tree, &mut indexes);

        write_uint_array(stream, &offsets.into_inner())?;
        write_uint_array(stream, &tree.into_inner())?;
        write_uint_array(stream, &indexes.into_inner())?;
        Ok(())
    }

    fn build_tree(&self) -> Tree {
        let mut arena = Tree::new();

        for data in self.0.iter() {
            let mut current = arena.root;

            for &byte in data.as_bytes() {
                current = arena.get_or_insert(current, byte);
            }
            arena.get_or_insert(current, 0);
        }

        arena
    }

    fn make_sub_tree(
        &self,
        arena: &mut Tree,
        current_idx: usize,
        value: Vec<u8>,
        offsets: &mut SparseVec<u64>,
        tree: &mut SparseVec<u64>,
        indexes: &mut SparseVec<u64>,
    ) {
        let min_value = arena.nodes[current_idx].min_value().unwrap_or(0);
        let begin_pos = arena.nodes[current_idx].begin_pos;
        let current_id = arena.nodes[current_idx].id;

        // Collect children to avoid holding references into the slab while mutating it
        let children: Vec<(u8, usize)> = arena.nodes[current_idx]
            .children
            .iter()
            .map(|(&v, &idx)| (v, idx))
            .collect();

        // First pass: assign IDs to children
        for &(child_value, child_idx) in &children {
            let id = if current_id == 0 || min_value < 1 {
                child_value as u64 + offsets.get(current_id as usize).unwrap()
            } else {
                (child_value - min_value) as u64 + begin_pos
            };

            tree.set(id as usize, current_id);
            arena.nodes[child_idx].id = id;
        }

        // Second pass: compute positions and offsets
        for &(child_value, child_idx) in &children {
            let child_id = arena.nodes[child_idx].id;
            let child_max = arena.nodes[child_idx].max_value().unwrap_or(0) as usize;
            let child_min = arena.nodes[child_idx].min_value().unwrap_or(0) as usize;

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

            if child_value == 0 {
                let index = self
                    .0
                    .iter()
                    .position(|val| val.as_bytes().eq(&value))
                    .unwrap() as u64;
                offsets.set(child_id as usize, index);
                indexes.set(index as usize, child_id);
            } else {
                let offset = (pos - child_min) as u64;
                offsets.set(child_id as usize, offset);
                arena.nodes[child_idx].begin_pos = pos as u64;
            }
        }

        // Third pass: recurse into children
        for (child_value, child_idx) in children {
            let mut child_path = value.clone();
            child_path.push(child_value);
            self.make_sub_tree(arena, child_idx, child_path, offsets, tree, indexes);
        }
    }
}

/// Flat arena for tree nodes, keyed by slab indices.
struct Tree {
    nodes: Slab<TreeNode>,
    root: usize,
}

impl Tree {
    fn new() -> Self {
        let mut nodes = Slab::new();
        let root = nodes.insert(TreeNode::new());
        Self { nodes, root }
    }

    /// Returns the slab index of the child with the given byte value, inserting a new node if
    /// absent.
    fn get_or_insert(&mut self, parent: usize, value: u8) -> usize {
        if let Some(&idx) = self.nodes[parent].children.get(&value) {
            return idx;
        }
        let new_idx = self.nodes.insert(TreeNode::new());
        self.nodes[parent].children.insert(value, new_idx);
        new_idx
    }
}

#[derive(Debug)]
struct TreeNode {
    /// Maps a byte value to the slab index of the corresponding child node.
    children: HashMap<u8, usize>,

    begin_pos: u64,
    id: u64,
}

impl TreeNode {
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            id: 0,
            begin_pos: 0,
        }
    }

    fn min_value(&self) -> Option<u8> {
        self.children.keys().min().copied()
    }

    fn max_value(&self) -> Option<u8> {
        self.children.keys().max().copied()
    }
}
