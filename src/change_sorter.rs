// keep the changes in sorted order for the document

use crate::index_map::{IndexMapper, IndexRef};
use crate::Id;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

// ChangeTree is a BTree like append-only structure
// It has a simplified vector like API for efficient insertion by index.
// Support index look up and slice api
struct ChangeTree<V: Hash + Eq> {
    root: Node<V>,
    order: usize,
    dirty: HashSet<IndexRef>,
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

    pub(crate) fn insert(&mut self, index: usize, value: V) {
        // self.root.insert(index, value);
    }

    pub(crate) fn index_of(&self, value: &V) -> Option<usize> {
        if let Some(index_ref) = self.refs.get(value) {
            Some(self.mapper.map_ref(index_ref))
        } else {
            None
        }
    }

    pub(crate) fn at_index(&self, index: usize) -> Option<&V> {
        self.root.at_index(index)
    }

    pub(crate) fn at_index_mut(&mut self, index: usize) -> Option<&mut V> {
        self.root.at_index_mut(index)
    }

    pub(crate) fn contains(&self, value: &V) -> bool {
        self.refs.contains_key(value)
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
    total: usize,
}

impl<V> Branch<V> {
    fn new(order: usize) -> Self {
        Branch {
            children: Vec::with_capacity(order),
            total: 0,
        }
    }

    pub(crate) fn insert(&mut self, index: usize, value: V) -> Option<Node<V>> {
        None
    }

    pub(crate) fn at_index(&self, index: usize) -> Option<&V> {
        if index >= self.size() {
            return None;
        }

        let mut current_index = index;
        let sizes = self.children.iter().map(|c| c.size()).collect::<Vec<_>>();
        for (i, child) in self.children.iter().enumerate() {
            if i > 0 {
                current_index -= sizes[i - 1];
            }

            if current_index < child.size() {
                return child.at_index(current_index);
            }
        }

        None
    }

    pub(crate) fn at_index_mut(&mut self, index: usize) -> Option<&mut V> {
        if index >= self.size() {
            return None;
        }

        let mut current_index = index;
        let sizes = self.children.iter().map(|c| c.size()).collect::<Vec<_>>();
        for (i, child) in self.children.iter_mut().enumerate() {
            if i > 0 {
                current_index -= sizes[i - 1];
            }

            if current_index < child.size() {
                return child.at_index_mut(current_index);
            }
        }

        None
    }

    fn is_full(&self) -> bool {
        self.children.len() == self.children.capacity()
    }

    fn size(&self) -> usize {
        self.total
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
