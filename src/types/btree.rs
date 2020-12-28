/*
 * Created on Sun Dec 27 2020
 *
 * Copyright (c) storycraft. Licensed under the MIT Licence.
 */

/// String b-tree strcture
pub struct StringBTree {

    root: BTreeItem

}

impl StringBTree {

    pub fn new() -> Self {
        Self {
            root: BTreeItem::new(0)
        }
    }

    pub fn insert(&mut self, string: String) {
        let last = &mut self.root;
    }

}

struct BTreeItem {

    pub value: u8,
    pub children: Vec<BTreeItem>

}

impl BTreeItem {

    pub fn new(value: u8) -> Self {
        Self {
            value,
            children: Vec::new()
        }
    }

}