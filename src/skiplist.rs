pub(crate) struct SkipList {
    head: Node,
    tail: Node,
    len: usize,
}

impl SkipList {
    pub(crate) fn new() -> Self {
        Self {
            head: Node::default(),
            tail: Node::default(),
            len: 0,
        }
    }

    pub(crate) fn insert(&mut self, key: i32) {
        let mut level = self.head.next.len();
        let mut current = &self.head;
        let mut update = Vec::with_capacity(level);

        while level > 0 {
            level -= 1;
            while let Some(ref mut next) = current.get_next(level) {
                if next.key < key {
                    current = next;
                } else {
                    break;
                }
            }
            update[level] = Some(current.clone());
        }

        let mut new_level = 1;
        while new_level < 32 && rand::random() {
            new_level += 1;
        }

        let mut new_node = Node {
            next: vec![None; new_level],
            key,
        };

        for i in 0..new_level {
            if let Some(ref mut next) = update[i].as_mut().unwrap().next[i] {
                new_node.next[i] = Some(next.clone());
            }
            update[i].as_mut().unwrap().next[i] = Some(Box::new(new_node.clone()));
        }

        self.len += 1;
    }

    pub(crate) fn find(&self, key: i32) -> bool {
        let mut level = self.head.next.len();
        let mut current = &self.head;

        while level > 0 {
            level -= 1;
            while let Some(ref next) = current.get_next(level) {
                if next.key < key {
                    current = next;
                } else {
                    break;
                }
            }
        }

        if let Some(next) = current.get_next(0) {
            next.key == key
        } else {
            false
        }
    }

    pub(crate) fn remove(&mut self, key: i32) -> bool {
        let mut level = self.head.next.len();
        let mut current = &self.head;
        let mut update = Vec::with_capacity(level);

        while level > 0 {
            level -= 1;
            while let Some(ref mut next) = current.get_next(level) {
                if next.key < key {
                    current = next;
                } else {
                    break;
                }
            }
            update[level] = Some(current.clone());
        }

        if let Some(ref mut next) = current.get_next(0) {
            if next.key == key {
                for i in 0..next.next.len() {
                    // update[i].as_mut().unwrap().next[i] = next.get_next(i);
                }
                self.len -= 1;
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct Node {
    next: Vec<Option<Box<Node>>>,
    key: i32,
}

impl Node {
    pub(crate) fn new(key: i32, level: usize) -> Self {
        Self {
            next: vec![None; level],
            key,
        }
    }

    pub(crate) fn new_with_next(key: i32, next: Vec<Option<Box<Node>>>) -> Self {
        Self { next, key }
    }

    pub(crate) fn get_next(&self, level: usize) -> Option<&Node> {
        self.next[level].as_ref().map(|node| &**node)
    }
}
