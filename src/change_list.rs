use crate::change::ChangeId;
use crate::change_btree::BTree;
use crate::id::WithId;
use crate::Id;
use btree_slab::{BTreeMap, BTreeSet};
use fractional_index::FractionalIndex;
use hashbrown::{HashMap, HashSet};
use std::cell::RefCell;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

struct ChangeListNode {
    index: FractionalIndex,
    parent_id: ChangeId,
    children: Vec<Id>,
    last_decedent: Id,
    child_count: u32,
    flag: u8,
}

struct ChangeList {
    index_map: BTree<FractionalIndex, Id>,
    moves: BTree<FractionalIndex, ()>,
    changes: HashMap<Id, ChangeListNode>,
}

impl ChangeList {
    pub fn new() -> Self {
        ChangeList {
            index_map: BTree::new(10),
            moves: BTree::new(10),
            changes: HashMap::new(),
        }
    }

    pub(crate) fn insert_root(&mut self, change_id: &ChangeId) {
        let node = ChangeListNode {
            index: FractionalIndex::default(),
            parent_id: change_id.clone(),
            last_decedent: change_id.id(),
            children: Vec::new(),
            child_count: 0,
            flag: 0,
        };

        self.index_map.insert(node.index.clone(), change_id.id());
        self.changes.insert(change_id.id(), node);
    }

    pub(crate) fn insert(&mut self, change_id: &ChangeId, parent_id: &ChangeId, flags: u8) {
        if !self.changes.contains_key(&parent_id.id()) {
            panic!("Parent change ID does not exist in the change list.");
        }

        let id = change_id.id();

        let parent_node = self.changes.get_mut(&parent_id.id()).unwrap();
        let pos = parent_node
            .children
            .binary_search(&id)
            .unwrap_or_else(|e| e);
        // Insert the new change ID into the parent's children vector
        parent_node.children.insert(pos, id);

        let prev_item = if pos == 0 {
            parent_id.id()
        } else {
            let sibling = parent_node
                .children
                .get(pos - 1)
                .cloned()
                .map(|id| self.changes.get(&id))
                .flatten();

            sibling.unwrap().last_decedent
        };

        // calculate the fractional index for the new change
        let prev_frac_index = self
            .changes
            .get(&prev_item)
            .map(|n| &n.index)
            .unwrap()
            .clone();
        let prev_index = self.index_map.index_of(&prev_frac_index).unwrap_or(0);
        let next_frac_index = self
            .index_map
            .at_index(prev_index + 1)
            .map(|n| self.changes.get(n))
            .map(|n| n.map(|n| n.index.clone()))
            .flatten();

        let frac_index = match (next_frac_index) {
            Some(next_index) => FractionalIndex::new_between(&prev_frac_index, &next_index),
            _ => Option::from({ FractionalIndex::new_after(&prev_frac_index) }),
        }
        .unwrap_or_else(|| FractionalIndex::new_after(&prev_frac_index));

        // If the flags indicate a move, insert it into the moves map
        if flags > 0 {
            self.moves.insert(frac_index.clone(), ());
        }

        // Insert the change into the index map and changes map
        let node = ChangeListNode {
            index: frac_index,
            parent_id: parent_id.clone(),
            children: Vec::new(),
            last_decedent: id.clone(),
            child_count: 0,
            flag: flags,
        };

        self.index_map.insert(node.index.clone(), change_id.id());
        self.changes.insert(id, node);
        let mut parent_id = parent_id.clone();

        while let Some(parent_node) = self.changes.get_mut(&parent_id.id()) {
            if parent_node.parent_id == parent_id {
                break;
            }

            parent_node.child_count += 1;
        }
    }

    pub(crate) fn index_of(&self, change_id: &ChangeId) -> Option<usize> {
        self.changes
            .get(&change_id.id())
            .map(|node| self.index_map.index_of(&node.index))
            .flatten()
    }

    pub(crate) fn contains(&self, change_id: &ChangeId) -> bool {
        self.changes.contains_key(&change_id.id())
    }
}
