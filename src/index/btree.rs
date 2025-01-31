use ptree::{print_tree, TreeBuilder};
use std::cmp::Ord;
use std::fmt::{Debug, Display};

#[derive(Debug)]
struct BTree<K: Ord + Clone + Display + Debug, V: Debug> {
    root: Node<K, V>,
    degree: usize,
}

impl<K: Ord + Clone + Display + Debug, V: Debug> BTree<K, V> {
    fn new(degree: usize) -> Self {
        BTree {
            root: Node::leaf(degree),
            degree,
        }
    }

    fn ptree(&self) {
        let mut tree = TreeBuilder::new("tree".to_string());
        self.root.ptree(&mut tree);

        print_tree(&tree.build()).unwrap();
    }

    fn at_index(&self, index: usize) -> Option<&V> {
        self.root.at_index(index)
    }

    fn index_of(&self, key: &K) -> Option<usize> {
        self.root.index_of(key)
    }

    fn insert(&mut self, key: K, value: V) {
        let right = self.root.insert(key, value);
        if let Some((key, right)) = right {
            let mut new_root = InternalNode::new(self.degree);
            let old_root = std::mem::replace(&mut self.root, Node::internal(self.degree));
            new_root.keys.push(key);
            new_root.children.push(old_root);
            new_root.children.push(right);

            new_root.total = new_root.children.iter().map(|child| child.size()).sum();

            self.root = Node::Internal(new_root);
        }
    }

    fn max(&self) -> Option<&K> {
        self.root.max()
    }

    fn min(&self) -> Option<&K> {
        self.root.min()
    }

    fn size(&self) -> usize {
        self.root.size()
    }
}

/// BTree node that can be either internal or leaf
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Node<K: Ord + Clone + Display + Debug, V: Debug> {
    Internal(InternalNode<K, V>),
    Leaf(LeafNode<K, V>),
}

impl<K: Ord + Clone + Display + Debug, V: Debug> Node<K, V> {
    fn leaf(degree: usize) -> Self {
        Node::Leaf(LeafNode {
            keys: Vec::with_capacity(degree),
            values: Vec::with_capacity(degree),
        })
    }

    fn internal(degree: usize) -> Self {
        Node::Internal(InternalNode {
            keys: Vec::with_capacity(degree),
            children: Vec::with_capacity(degree + 1),
            total: 0,
        })
    }

    // return the value at index
    fn at_index(&self, index: usize) -> Option<&V> {
        match self {
            Node::Internal(node) => {
                let mut total = 0;
                for child in &node.children {
                    let size = child.size();
                    if total + size > index {
                        return child.at_index(index - total);
                    }
                    total += size;
                }
                None
            }
            Node::Leaf(node) => node.values.get(index),
        }
    }

    fn key_at_index(&self, index: usize) -> Option<&K> {
        match self {
            Node::Internal(node) => {
                let mut total = 0;
                for child in &node.children {
                    let size = child.size();
                    if total + size > index {
                        return child.key_at_index(index - total);
                    }
                    total += size;
                }
                None
            }
            Node::Leaf(node) => node.keys.get(index),
        }
    }

    // return the index of the key in the node
    fn index_of(&self, key: &K) -> Option<usize> {
        match self {
            Node::Internal(node) => {
                let mut total = 0;
                for (i, k) in node.keys.iter().enumerate() {
                    if key <= k {
                        return node.children[i]
                            .index_of(key)
                            .and_then(|index| Some(total + index));
                    }
                    total += node.children[i].size();
                }
                node.children.last().unwrap().index_of(key)
            }
            Node::Leaf(node) => node.keys.iter().position(|k| k == key),
        }
    }

    fn max(&self) -> Option<&K> {
        match self {
            Node::Internal(node) => node.max(),
            Node::Leaf(node) => node.max(),
        }
    }

    fn min(&self) -> Option<&K> {
        match self {
            Node::Internal(node) => node.children.first().unwrap().min(),
            Node::Leaf(node) => node.keys.first(),
        }
    }

    fn ptree(&self, ptree: &mut TreeBuilder) {
        match self {
            Node::Internal(node) => {
                for (i, child) in node.children.iter().enumerate() {
                    if i != 0 {
                        let mut tree = ptree.begin_child(format!("{}", node.keys[i - 1]));
                        child.ptree(tree);
                        tree.end_child();
                    } else {
                        let mut tree = ptree.begin_child("".to_string());
                        child.ptree(tree);
                        tree.end_child();
                    }
                }
            }
            Node::Leaf(node) => {
                for (i, key) in node.keys.iter().enumerate() {
                    ptree.begin_child(format!("key-{}", key)).end_child();
                }
            }
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<(K, Node<K, V>)> {
        match self {
            Node::Internal(node) => node.insert(key, value),
            Node::Leaf(node) => node.insert(key, value),
        }
    }

    fn size(&self) -> usize {
        match self {
            Node::Internal(node) => node.size(),
            Node::Leaf(node) => node.keys.len(),
        }
    }
}

struct ValueIter<'a, K: Ord + Clone + Display + Debug, V: std::fmt::Debug> {
    internals: Vec<(&'a Node<K, V>, usize)>,
    leaf: &'a Node<K, V>,
    index: usize,
}

impl<'a, K: Ord + Clone + Display + Debug, V: Debug> ValueIter<'a, K, V> {
    fn new(node: &'a Node<K, V>) -> Self {
        let mut internals = vec![(node, 0)];

        while let (Node::Internal(internal), _) = internals.last().unwrap() {
            let child = &internal.children[0];
            internals.push((child, 0));
        }

        for (node, index) in internals.iter() {
            if let Node::Leaf(leaf) = node {
                println!("leaf keys: {:#?}", leaf.keys);
            }

            if let Node::Internal(internal) = node {
                println!("internal keys: {:#?}", internal.keys);
            }
        }

        let (leaf, _) = internals.pop().unwrap();

        ValueIter {
            internals,
            leaf,
            index: 0,
        }
    }
}

impl<'a, K: Ord + Clone + Display + Debug, V: Debug> Iterator for ValueIter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        // for (node, index) in self.internals.iter() {
        //     println!("node keys: {}, index: {}", node.size(), index);
        // }

        if self.index < self.leaf.size() {
            let value = self.leaf.at_index(self.index)?;
            let key = self.leaf.key_at_index(self.index)?;
            self.index += 1;

            Some((key, value))
        } else {
            while self.internals.len() > 0 {
                let (node, index) = self.internals.pop().unwrap();
                if let Node::Internal(internal) = node {
                    if index + 1 < internal.children.len() {
                        let child = &internal.children[index + 1];
                        self.internals.push((node, index + 1));
                        self.internals.push((child, 0));
                    } else {
                        continue;
                    }

                    while let (Node::Internal(internal), index) = self.internals.last().unwrap() {
                        if index < &internal.children.len() {
                            let child = &internal.children[index + 1];
                            self.internals.push((child, 0));
                            break;
                        } else {
                            self.internals.pop();
                        }
                    }

                    let (leaf, _) = self.internals.pop().unwrap();

                    self.leaf = leaf;
                    self.index = 0;

                    return self.next();
                }
            }

            None
        }
    }
}

impl<'a, K: Ord + Clone + Display + Debug, V: Debug> IntoIterator for &'a BTree<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = ValueIter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        println!("index: {}", 0);
        ValueIter::new(&self.root)
    }
}

impl<K: Ord + Clone + Display + Debug, V: Debug> From<LeafNode<K, V>> for Node<K, V> {
    fn from(node: LeafNode<K, V>) -> Self {
        Node::Leaf(node)
    }
}

impl<K: Ord + Clone + Display + Debug, V: Debug> From<InternalNode<K, V>> for Node<K, V> {
    fn from(node: InternalNode<K, V>) -> Self {
        Node::Internal(node)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub(crate) struct LeafNode<K: Ord, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K: Ord + Clone + Display + Debug, V: Debug> LeafNode<K, V> {
    // insert key-value pair into leaf node, return right node if split
    fn insert(&mut self, key: K, value: V) -> Option<(K, Node<K, V>)> {
        let index = self.keys.binary_search(&key).unwrap_or_else(|x| x);
        if self.keys.len() + 1 == self.keys.capacity() {
            let mut right = self.split();

            if index < self.keys.len() {
                self.keys.insert(index, key);
                self.values.insert(index, value);
            } else {
                right.keys.insert(index - self.keys.len(), key);
                right.values.insert(index - self.keys.len(), value);
            }

            return Some((self.keys.last().unwrap().clone(), right.into()));
        }

        self.keys.insert(index, key);
        self.values.insert(index, value);

        None
    }

    // split leaf node into two, return left node
    fn split(&mut self) -> LeafNode<K, V> {
        let mid = self.keys.capacity() / 2;

        let mut left = LeafNode {
            keys: Vec::with_capacity(self.keys.capacity()),
            values: Vec::with_capacity(self.keys.capacity()),
        };

        self.keys.split_off(mid).into_iter().for_each(|key| {
            left.keys.push(key);
        });

        self.values.split_off(mid).into_iter().for_each(|value| {
            left.values.push(value);
        });

        left
    }

    fn min(&self) -> Option<&K> {
        self.keys.first()
    }

    fn max(&self) -> Option<&K> {
        self.keys.last()
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct InternalNode<K: Ord + Clone + Display + Debug, V: Debug> {
    keys: Vec<K>,
    children: Vec<Node<K, V>>,
    total: usize,
}

impl<K: Ord + Clone + Display + Debug, V: std::fmt::Debug> InternalNode<K, V> {
    fn new(degree: usize) -> Self {
        InternalNode {
            keys: Vec::with_capacity(degree),
            children: Vec::with_capacity(degree + 1),
            total: 0,
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<(K, Node<K, V>)> {
        self.total += 1;

        let index = self.keys.binary_search(&key).unwrap_or_else(|x| x);

        // println!("xxxxxxxxxxxxxx keys: {:?}", self.keys);
        // println!("xxxxxxxxxxxxxx key: {}", key);
        // println!("xxxxxxxxxxxxxx index: {}", index);
        // println!("xxxxxxxxxxxxxx children: {}", self.children.len());

        let child = &mut self.children[index];

        // println!("keys: {:#?}", self.keys);
        // println!("key: {:#?}, index: {:#?}", key, index);
        // insert key-value pair into child node
        let right = child.insert(key, value);

        // child node is split
        if let Some((key, child_right)) = right {
            if self.keys.len() + 1 == self.keys.capacity() {
                let mut self_right = self.split();
                // println!("key: {:#?}, index: {:#?}", key, index);

                if index < self.keys.len() {
                    self.keys.insert(index, key);
                    self.children.insert(index + 1, child_right);
                } else {
                    // println!("right: {:#?}, keys: {:?}", child_right, self.keys);
                    self_right.keys.insert(index - self.keys.len(), key);
                    self_right
                        .children
                        .insert(index - self.keys.len() + 1, child_right);
                }

                Some((self.keys.last().unwrap().clone(), self_right.into()))
            } else {
                // println!("key: {:#?}, index: {:#?}", key, index);
                if index < self.keys.len() {
                    self.keys.insert(index, key);
                    self.children.insert(index + 1, child_right);
                } else {
                    self.keys.push(key);
                    self.children.push(child_right);
                }

                None
            }
        } else {
            None
        }
    }

    // split child node at index into two, return right node
    fn split(&mut self) -> InternalNode<K, V> {
        assert_eq!(self.keys.len() + 1, self.keys.capacity());

        let mid = self.keys.capacity() / 2;

        let mut right = InternalNode {
            keys: Vec::with_capacity(self.keys.capacity()),
            children: Vec::with_capacity(self.keys.capacity() + 1),
            total: 0,
        };

        self.keys.split_off(mid).into_iter().for_each(|key| {
            right.keys.push(key);
        });

        self.children.split_off(mid).into_iter().for_each(|child| {
            right.children.push(child);
        });

        right.total = right.children.iter().map(|child| child.size()).sum();

        self.total -= right.total;

        right.into()
    }

    fn min(&self) -> Option<&K> {
        self.children.first().unwrap().min()
    }

    fn max(&self) -> Option<&K> {
        self.children.last().unwrap().max()
    }

    fn size(&self) -> usize {
        self.total
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn insert_into_leaf() {
        let mut left = super::LeafNode {
            keys: Vec::with_capacity(5),
            values: Vec::with_capacity(5),
        };

        left.insert(1, 1);
        left.insert(3, 3);
        left.insert(5, 5);
        left.insert(7, 7);

        assert_eq!(left.keys, vec![1, 3, 5, 7]);
    }

    #[test]
    fn insert_into_leaf_with_split() {
        let mut left = super::LeafNode {
            keys: Vec::with_capacity(4),
            values: Vec::with_capacity(4),
        };

        left.insert(1, 1);
        left.insert(3, 3);
        left.insert(5, 5);
        let right = left.insert(7, 7);

        assert_eq!(left.keys, vec![1, 3]);

        let right = right.unwrap().1;
        if let super::Node::Leaf(right) = right {
            assert_eq!(right.keys, vec![5, 7]);
        } else {
            panic!("right node is not leaf");
        }
    }

    #[test]
    fn insert_into_btree() {
        let mut tree = super::BTree::new(4);

        tree.insert(1, 1);
        tree.insert(3, 3);
        tree.insert(5, 5);
        tree.insert(7, 7);

        assert_eq!(tree.size(), 4);
    }

    #[test]
    fn insert_into_btree_with_split() {
        let mut tree = super::BTree::new(4);
        // random unia 40 digit numbers
        let mut vec: Vec<u32> = (0..100).collect();
        // shuffle the keys to test the split
        // vec.shuffle(&mut thread_rng());

        // println!("{:?}", vec);

        // shuffle the keys to test the split
        for i in vec {
            // println!("inserting {}", i);
            tree.insert(i, i);
        }

        // assert_eq!(tree.size(), 10);
        tree.ptree();

        // println!("{:#?}", tree);

        tree.into_iter().for_each(|(k, v)| {
            println!("{}: {}", k, v);
        });
    }
}
