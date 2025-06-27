// keep the changes in sorted order for the document

use crate::index_map::{IndexMapper, IndexRef};
use crate::Id;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::io::Write;
use std::mem;

// ChangeTree is a BTree like append-only structure
// It has a simplified vector like API for efficient insertion by index.
// Support index look up and slice api
struct ChangeTree<V: Hash + Eq> {
    root: Node<V>,
    order: usize,
    dirty: HashSet<V>,
    refs: HashMap<V, IndexRef>,
    mapper: IndexMapper,
}

impl<V: Hash + Eq> ChangeTree<V> {
    pub(crate) fn new() -> Self {
        Self::with_order(10)
    }

    fn with_order(k: usize) -> Self {
        ChangeTree {
            root: Node::leaf(k),
            order: k,
            dirty: Default::default(),
            refs: Default::default(),
            mapper: Default::default(),
        }
    }

    // compress the index mappers
    fn compress(&mut self) {
        for id in &self.dirty {
            // update the index_ref with current index
            if let Some(idx_ref) = self.refs.get_mut(&id) {
                idx_ref.index = self.mapper.map_ref(idx_ref);
                idx_ref.mapper = 0;
            }
        }

        self.dirty.clear()
    }

    pub(crate) fn insert(&mut self, index: usize, value: V) {
        if let Some(node) = self.root.insert(index, value) {
            let mut new_root = Node::Branch(Branch::new(self.order));
            let old_root = mem::replace(&mut self.root, new_root);
            match self.root {
                Node::Branch(ref mut branch) => {
                    branch.children.push(old_root);
                    branch.children.push(node);
                    branch.update_count();
                }
                _ => unreachable!(),
            }
        }
    }

    pub(crate) fn index_of(&self, value: &V) -> Option<usize> {
        if let Some(index_ref) = self.refs.get(value) {
            Some(self.mapper.map_ref(index_ref))
        } else {
            None
        }
    }

    pub(crate) fn at_index(&self, index: usize) -> Option<&V> {
        if index >= self.root.size() {
            return None;
        }
        self.root.at_index(index)
    }

    pub(crate) fn at_index_mut(&mut self, index: usize) -> Option<&mut V> {
        if index >= self.root.size() {
            return None;
        }
        self.root.at_index_mut(index)
    }

    pub(crate) fn contains(&self, value: &V) -> bool {
        self.refs.contains_key(value)
    }

    pub(crate) fn size(&self) -> usize {
        self.root.size()
    }
}

enum Node<V> {
    Branch(Branch<V>),
    Leaf(Leaf<V>),
}

impl<V> Node<V> {
    fn branch(order: usize) -> Self {
        Node::Branch(Branch::new(order))
    }

    fn leaf(order: usize) -> Self {
        Node::Leaf(Leaf::new(order))
    }

    pub(crate) fn insert(&mut self, index: usize, value: V) -> Option<Node<V>> {
        match self {
            Node::Branch(branch) => branch.insert(index, value),
            Node::Leaf(leaf) => leaf.insert(index, value),
        }
    }

    pub(crate) fn at_index(&self, index: usize) -> Option<&V> {
        match self {
            Node::Branch(branch) => branch.at_index(index),
            Node::Leaf(leaf) => leaf.at_index(index),
        }
    }

    pub(crate) fn at_index_mut(&mut self, index: usize) -> Option<&mut V> {
        match self {
            Node::Branch(branch) => branch.at_index_mut(index),
            Node::Leaf(leaf) => leaf.at_index_mut(index),
        }
    }

    fn size(&self) -> usize {
        match self {
            Node::Branch(branch) => branch.size(),
            Node::Leaf(leaf) => leaf.size(),
        }
    }
}

struct Branch<V> {
    children: Vec<Node<V>>,
    counts: Vec<usize>,
}

impl<V> Branch<V> {
    fn new(order: usize) -> Self {
        Branch {
            children: Vec::with_capacity(order),
            counts: Vec::with_capacity(order),
        }
    }

    pub(crate) fn insert(&mut self, index: usize, value: V) -> Option<Node<V>> {
        if let Some((pos, index)) = self.get_child_with_index(index) {
            if let Some(child) = self.children.get_mut(pos) {
                if let Some(new_node) = child.insert(index, value) {
                    if self.is_full() {
                        return Some(self.insert_and_split(pos, new_node));
                    } else {
                        self.children.insert(pos + 1, new_node);
                        self.update_count();
                    }
                }
            }
        }

        None
    }

    fn insert_and_split(&mut self, pos: usize, new_child: Node<V>) -> Node<V> {
        let order = self.children.capacity();
        let mid = order / 2;

        let mut new_branch = Branch::new(self.children.capacity());

        self.children.insert(pos + 1, new_child);

        // TODO: potential BUG might be here
        new_branch.children.extend(self.children.drain(mid..));

        self.children.truncate(mid);

        new_branch.children.shrink_to(order);
        self.children.shrink_to(order);

        new_branch.update_count();
        self.update_count();

        Node::Branch(new_branch)
    }

    pub(crate) fn at_index(&self, index: usize) -> Option<&V> {
        if let Some((pos, index)) = self.get_child_with_index(index) {
            self.children
                .get(pos)
                .map(|node| node.at_index(index))
                .flatten()
        } else {
            None
        }
    }

    pub(crate) fn at_index_mut(&mut self, index: usize) -> Option<&mut V> {
        if let Some((pos, index)) = self.get_child_with_index(index) {
            self.children
                .get_mut(pos)
                .map(|node| node.at_index_mut(index))
                .flatten()
        } else {
            None
        }
    }

    #[inline]
    fn get_child_with_index(&self, index: usize) -> Option<(usize, usize)> {
        let pos = self
            .counts
            .partition_point(|&n| n <= index)
            .max(self.children.len() - 1);

        let index = if pos == 0 {
            index
        } else {
            index - self.counts[pos - 1]
        };

        Some((pos, index))
    }

    #[inline]
    fn update_count(&mut self) {
        let mut count = 0;
        for (index, child) in self.children.iter().enumerate() {
            count += child.size();
            self.counts[index] += count;
        }
    }

    fn is_full(&self) -> bool {
        self.children.len() == self.children.capacity()
    }

    fn size(&self) -> usize {
        self.counts.last().copied().unwrap_or(0)
    }
}

struct Leaf<V> {
    values: Vec<V>,
}

impl<V> Leaf<V> {
    fn new(order: usize) -> Self {
        Leaf {
            values: Vec::with_capacity(order),
        }
    }

    // Insert the value at index, splitting the node if full
    pub(crate) fn insert(&mut self, index: usize, value: V) -> Option<Node<V>> {
        if self.values.len() == self.values.capacity() {
            let mid = self.values.len() / 2;
            let mut new_leaf = Leaf::new(self.values.capacity());
            new_leaf.values.extend(self.values.drain(mid..));
            if index < mid {
                self.values.insert(index, value);
            } else {
                new_leaf.values.insert(index - mid, value);
            }
            Some(Node::Leaf(new_leaf))
        } else {
            self.values.insert(index, value);
            None
        }
    }

    pub(crate) fn at_index(&self, index: usize) -> Option<&V> {
        self.values.get(index)
    }

    pub(crate) fn at_index_mut(&mut self, index: usize) -> Option<&mut V> {
        self.values.get_mut(index)
    }

    fn size(&self) -> usize {
        self.values.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_partition() {
        let p = vec![10, 20, 40];
        println!("{}", p.partition_point(|&n| n <= 9));
        println!("{}", p.partition_point(|&n| n < 11));
    }

    #[test]
    fn test_btree_insert() {
        let mut tree = ChangeTree::new();
        tree.insert(0, 0);
        tree.insert(0, 1);
        tree.insert(0, 2);
        tree.insert(0, 3);

        assert_eq!(tree.size(), 4)
    }
}
