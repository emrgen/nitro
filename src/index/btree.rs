use std::cmp::Ord;

struct BTree<K: Ord + Clone, V> {
    root: Node<K, V>,
    degree: usize,
}

impl<K: Ord + Clone, V> BTree<K, V>
where
    K: Ord,
{
    fn new(degree: usize) -> Self {
        BTree {
            root: Node::leaf(degree),
            degree,
        }
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

        println!("size: {}", self.size());
    }

    fn size(&self) -> usize {
        self.root.size()
    }
}

/// BTree node that can be either internal or leaf
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Node<K: Ord + Clone, V> {
    Internal(InternalNode<K, V>),
    Leaf(LeafNode<K, V>),
}

impl<K: Ord + Clone, V> Node<K, V> {
    fn leaf(degree: usize) -> Self {
        Node::Leaf(LeafNode {
            keys: Vec::with_capacity(degree),
            values: Vec::with_capacity(degree),
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

    fn internal(degree: usize) -> Self {
        Node::Internal(InternalNode {
            keys: Vec::with_capacity(degree),
            children: Vec::with_capacity(degree + 1),
            total: 0,
        })
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

impl<K: Ord + Clone, V> From<LeafNode<K, V>> for Node<K, V> {
    fn from(node: LeafNode<K, V>) -> Self {
        Node::Leaf(node)
    }
}

impl<K: Ord + Clone, V> From<InternalNode<K, V>> for Node<K, V> {
    fn from(node: InternalNode<K, V>) -> Self {
        Node::Internal(node)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct LeafNode<K: Ord, V> {
    keys: Vec<K>,
    values: Vec<V>,
}

impl<K: Ord + Clone, V> LeafNode<K, V> {
    // insert key-value pair into leaf node, return right node if split
    fn insert(&mut self, key: K, value: V) -> Option<(K, Node<K, V>)> {
        let index = self.keys.binary_search(&key).unwrap_or_else(|x| x);
        if self.keys.len() + 1 == self.keys.capacity() {
            let mut right = self.split();
            right.insert(key, value);

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
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct InternalNode<K: Ord + Clone, V> {
    keys: Vec<K>,
    children: Vec<Node<K, V>>,
    total: usize,
}

impl<K: Ord + Clone, V> InternalNode<K, V> {
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
        let child = &mut self.children[index];

        // insert key-value pair into child node
        let right = child.insert(key, value);

        // child node is split
        if let Some((key, child_right)) = right {
            if self.keys.len() == self.keys.capacity() {
                let mut self_right = self.split();

                if index < self.keys.len() {
                    self.keys.insert(index, key);
                    self.children.insert(index, child_right);
                } else {
                    self_right.keys.insert(index - self.keys.len(), key);
                    self_right
                        .children
                        .insert(index - self.keys.len(), child_right);
                }

                Some((self.keys.last().unwrap().clone(), self_right.into()))
            } else {
                self.keys.insert(index + 1, key);
                self.children.insert(index + 1, child_right);
                None
            }
        } else {
            None
        }
    }

    // split child node at index into two, return right node
    fn split(&mut self) -> InternalNode<K, V> {
        assert_eq!(self.keys.len(), self.keys.capacity());

        let mid = self.keys.len() / 2;

        let mut right = InternalNode {
            keys: Vec::with_capacity(self.keys.capacity()),
            children: Vec::with_capacity(self.keys.capacity() + 1),
            total: 0,
        };

        self.keys.split_off(mid).into_iter().for_each(|key| {
            right.keys.push(key);
        });

        self.children
            .split_off(mid + 1)
            .into_iter()
            .for_each(|child| {
                right.children.push(child);
            });

        right.total = right.children.iter().map(|child| child.size()).sum();

        self.total -= right.total;

        right.into()
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
}
