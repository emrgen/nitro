use std::fmt::Debug;

#[derive(Debug, Clone)]
struct Leaf<K, V> {
    values: Vec<V>,
    keys: Vec<K>,
}

impl<K: Ord + Clone, V: Clone> Leaf<K, V> {
    fn new(order: usize) -> Self {
        Leaf {
            values: Vec::with_capacity(order),
            keys: Vec::with_capacity(order),
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<Node<K, V>> {
        let pos = self
            .keys
            .binary_search_by_key(&key, |k| k.clone())
            .unwrap_or_else(|e| e);

        // If leaf is full, we need to handle splitting
        if self.keys.len() == self.keys.capacity() {
            Some(Node::new_leaf(self.insert_and_split(pos, key, value)))
        } else {
            self.keys.insert(pos, key);
            self.values.insert(pos, value);
            None
        }
    }

    fn insert_and_split(&mut self, pos: usize, key: K, value: V) -> (Leaf<K, V>) {
        let mid = self.keys.len() / 2;

        let mut new_leaf = Leaf::new(self.keys.capacity());
        new_leaf.keys.extend(self.keys.drain(mid..));
        new_leaf.values.extend(self.values.drain(mid..));

        if pos < mid {
            self.keys.insert(pos, key);
            self.values.insert(pos, value);
        } else {
            new_leaf.keys.insert(pos - mid, key);
            new_leaf.values.insert(pos - mid, value);
        }

        new_leaf
    }

    fn min_key(&self) -> Option<&K> {
        self.keys.first()
    }

    fn max_key(&self) -> Option<&K> {
        self.keys.last()
    }

    fn size(&self) -> usize {
        self.values.len()
    }
}

#[derive(Debug, Clone)]
struct Branch<K, V> {
    keys: Vec<K>,
    children: Vec<Node<K, V>>,
    count: usize, // pre-computed size for performance
}

impl<K: Ord + Clone, V: Clone> Branch<K, V> {
    fn new(order: usize) -> Self {
        Branch {
            keys: Vec::with_capacity(order),
            children: Vec::with_capacity(order + 1),
            count: 0,
        }
    }

    fn insert(&mut self, key: K, value: V) -> Option<Node<K, V>> {
        self.count += 1;

        None
    }

    fn min_key(&self) -> Option<&K> {
        self.children.first().map(|child| child.min_key()).flatten()
    }

    fn max_key(&self) -> Option<&K> {
        self.children.last().map(|child| child.max_key()).flatten()
    }

    fn size(&self) -> usize {
        self.count
    }
}

#[derive(Debug, Clone)]
enum Node<K, T> {
    Leaf(Leaf<K, T>),
    Branch(Branch<K, T>),
}

impl<K: Ord + Clone, V: Clone> Node<K, V> {
    fn new_leaf(leaf: Leaf<K, V>) -> Self {
        Node::Leaf(leaf)
    }

    fn new_branch(branch: Branch<K, V>) -> Self {
        Node::Branch(branch)
    }

    fn insert(&mut self, key: K, value: V) -> Option<Node<K, V>> {
        match self {
            Node::Leaf(leaf) => leaf.insert(key, value),
            Node::Branch(branch) => branch.insert(key, value),
        }
    }

    fn min_key(&self) -> Option<&K> {
        match self {
            Node::Leaf(leaf) => leaf.min_key(),
            Node::Branch(branch) => branch.min_key(),
        }
    }

    fn max_key(&self) -> Option<&K> {
        match self {
            Node::Leaf(leaf) => leaf.max_key(),
            Node::Branch(branch) => branch.max_key(),
        }
    }

    fn size(&self) -> usize {
        match self {
            Node::Leaf(leaf) => leaf.size(),
            Node::Branch(branch) => branch.size(),
        }
    }

    fn is_leaf(&self) -> bool {
        matches!(self, Node::Leaf(_))
    }

    fn is_branch(&self) -> bool {
        matches!(self, Node::Branch(_))
    }

    fn is_empty(&self) -> bool {
        match self {
            Node::Leaf(leaf) => leaf.keys.is_empty(),
            Node::Branch(branch) => branch.keys.is_empty(),
        }
    }
}

#[derive(Debug, Clone)]
struct BTree<K, V> {
    root: Node<K, V>,
    order: usize,
}

impl<K: Debug + Ord + Clone, V: Clone + Debug> BTree<K, V> {
    fn default() -> Self {
        BTree::new(10) // Default order is 2
    }

    fn new(order: usize) -> Self {
        BTree {
            root: Node::Leaf(Leaf::new(order)),
            order,
        }
    }

    // Insert a key-value pair into the B-Tree
    fn insert(&mut self, key: K, value: V) {
        if let Some(node) = self.root.insert(key, value) {
            let mut new_root = Branch::new(self.order);
            new_root.children.push(self.root.clone());
            new_root.children.push(node);
            self.root = Node::new_branch(new_root);
        }
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        // Removal logic will go here
        // This is a placeholder for the actual implementation
        None
    }

    fn find(&self, key: &K) -> Option<&V> {
        // Find logic will go here
        // This is a placeholder for the actual implementation
        None
    }

    fn is_empty(&self) -> bool {
        self.root.is_empty()
    }

    fn size(&self) -> usize {
        self.root.size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_btree_insert() {
        let mut btree = BTree::new(4);
        btree.insert(10, "A");
        btree.insert(20, "B");
        btree.insert(5, "C");

        assert_eq!(btree.size(), 3);
        assert!(!btree.is_empty());
    }

    #[test]
    fn test_leaf_split() {
        let mut leaf = Leaf::new(2);
        leaf.insert(10, "A");
        leaf.insert(20, "B");
        let new_leaf = leaf.insert(5, "C");
        assert!(new_leaf.is_some());
        assert_eq!(new_leaf.unwrap().size(), 1);
        assert_eq!(leaf.size(), 2);
    }

    //     #[test]
    //     fn test_btree_find() {
    //         let mut btree = BTree::new(3);
    //         btree.insert(10, "A");
    //         btree.insert(20, "B");
    //
    //         assert_eq!(btree.find(&10), Some(&"A"));
    //         assert_eq!(btree.find(&20), Some(&"B"));
    //         assert_eq!(btree.find(&30), None);
    //     }
    // }
    //
    //     #[test]
    //     fn test_btree_remove() {
    //         let mut btree = BTree::new(3);
    //         btree.insert(10, "A");
    //         btree.insert(20, "B");
    //         btree.insert(5, "C");
    //
    //         assert_eq!(btree.remove(&10), None); // Placeholder for actual removal logic
    //         assert_eq!(btree.size(), 3); // Size should remain the same until remove is implemented
    //     }
}
