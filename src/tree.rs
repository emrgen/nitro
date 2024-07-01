use fractional_index::FractionalIndex;

use crate::item::WithIndex;
use crate::Type;

#[derive(Clone, Debug, Default)]
pub(crate) struct IndexTree {
    root: Option<Box<TreeNode>>,
}

#[derive(Debug, Clone)]
pub(crate) struct TreeNode {
    item: Type,
    left: Option<Box<TreeNode>>,
    right: Option<Box<TreeNode>>,
    color: Color,
    deleted: bool,
    left_count: usize,
}

impl TreeNode {
    pub(crate) fn new(item: Type) -> Self {
        Self {
            item,
            left: None,
            right: None,
            color: Color::Red,
            deleted: false,
            left_count: 0,
        }
    }

    fn index(&self) -> FractionalIndex {
        self.item.index()
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub(crate) enum Color {
    Red,
    Black,
}

impl IndexTree {
    pub(crate) fn new() -> Self {
        Self { root: None }
    }
    pub fn insert(&mut self, value: Type) {
        self.root = Some(Self::insert_node(self.root.take(), value));
        if let Some(ref mut node) = self.root {
            node.color = Color::Black;
        }
    }

    pub fn search(&self, value: &Type) -> bool {
        Self::search_node(&self.root, value)
    }

    fn search_node(node: &Option<Box<TreeNode>>, value: &Type) -> bool {
        match node {
            Some(n) => {
                if value.index() == n.index() {
                    true
                } else if value.index() < n.index() {
                    Self::search_node(&n.left, value)
                } else {
                    Self::search_node(&n.right, value)
                }
            }
            None => false,
        }
    }

    fn delete(&mut self, value: Type) {
        Self::delete_node(self.root.take(), value);
    }

    pub fn left_count(&self, value: &Type) -> usize {
        Self::find_left_count(&self.root, value)
    }

    fn find_left_count(node: &Option<Box<TreeNode>>, value: &Type) -> usize {
        match node {
            Some(n) => {
                return if value.index() == n.index() {
                    n.left_count
                } else if value.index() < n.index() {
                    Self::find_left_count(&n.left, value)
                } else {
                    n.left_count + 1 + Self::find_left_count(&n.right, value)
                }
            }
            None => 0,
        }
    }

    fn delete_node(mut node: Option<Box<TreeNode>>, value: Type) -> bool {
        match node {
            Some(mut n) => {
                return if value.index() < n.index() {
                    if Self::delete_node(n.left.take(), value) {
                        n.left_count -= 1;
                        true
                    } else {
                        false
                    }
                } else if value.index() > n.index() {
                    Self::delete_node(n.right.take(), value)
                } else {
                    n.deleted = true;
                    true
                }
            }
            None => false,
        }
    }

    fn insert_node(mut node: Option<Box<TreeNode>>, value: Type) -> Box<TreeNode> {
        match node {
            Some(mut n) => {
                if value.index() < n.index() {
                    n.left = Some(Self::insert_node(n.left.take(), value));
                    n.left_count += 1;
                } else {
                    n.right = Some(Self::insert_node(n.right.take(), value));
                }
                // Self::fix_violations(n)
                n
            }
            None => Box::new(TreeNode::new(value)),
        }
    }

    fn fix_violations(mut node: Box<TreeNode>) -> Box<TreeNode> {
        if Self::is_red(&node.right) && !Self::is_red(&node.left) {
            node = Self::rotate_left(&mut node);
        }

        if Self::is_red(&node.left) && Self::is_red(&node.left.as_ref().unwrap().left) {
            node = Self::rotate_right(node);
        }

        if Self::is_red(&node.left) && Self::is_red(&node.right) {
            Self::flip_colors(&mut node);
        }

        node
    }

    fn is_red(node: &Option<Box<TreeNode>>) -> bool {
        match node {
            Some(n) => n.color == Color::Red,
            None => false,
        }
    }

    fn rotate_left(mut node: &mut Box<TreeNode>) -> Box<TreeNode> {
        let mut new_node = node.right.take().unwrap();
        node.right = new_node.left.take();
        new_node.left = Some(node.clone());
        new_node.color = new_node.left.as_ref().unwrap().color;
        new_node.left.as_mut().unwrap().color = Color::Red;
        new_node
    }

    fn rotate_right(mut node: Box<TreeNode>) -> Box<TreeNode> {
        let mut new_node = node.left.take().unwrap();
        node.left = new_node.right.take();
        new_node.right = Some(node.clone());
        new_node.color = new_node.right.as_ref().unwrap().color;
        new_node.right.as_mut().unwrap().color = Color::Red;
        new_node
    }

    fn flip_colors(node: &mut Box<TreeNode>) {
        node.color = match node.color {
            Color::Red => Color::Black,
            Color::Black => Color::Red,
        };
        if let Some(ref mut left) = node.left {
            left.color = match left.color {
                Color::Red => Color::Black,
                Color::Black => Color::Red,
            };
        }
        if let Some(ref mut right) = node.right {
            right.color = match right.color {
                Color::Red => Color::Black,
                Color::Black => Color::Red,
            };
        }
    }

    pub fn iter(&self) -> TreeIter {
        TreeIter::new(&self.root)
    }

    // print tree with dashes to show the tree structure
    pub fn print_tree(&self) {
        self.print_tree_node(&self.root, 0);
    }

    fn print_tree_node(&self, node: &Option<Box<TreeNode>>, level: usize) {
        match node {
            Some(n) => {
                self.print_tree_node(&n.right, level + 1);
                for _ in 0..level {
                    print!("--");
                }
                println!("{:?}", n.item.content());
                self.print_tree_node(&n.left, level + 1);
            }
            None => {}
        }
    }
}

pub(crate) struct TreeIter<'a> {
    stack: Vec<&'a Box<TreeNode>>,
}

impl<'a> TreeIter<'a> {
    pub(crate) fn new(root: &'a Option<Box<TreeNode>>) -> Self {
        let mut stack = Vec::new();
        let mut current = root;
        while let Some(node) = current {
            stack.push(node);
            current = &node.left;
        }
        Self { stack }
    }
}

impl<'a> Iterator for TreeIter<'a> {
    type Item = &'a TreeNode;

    fn next(&mut self) -> Option<Self::Item> {
        let node = self.stack.pop()?;
        let mut current = &node.right;
        while let Some(n) = current {
            self.stack.push(n);
            current = &n.left;
        }

        Some(&node)
    }
}

#[cfg(test)]
mod test {
  use fractional_index::FractionalIndex;

  use crate::{Doc, Type};
  use crate::item::WithIndex;
  use crate::tree::IndexTree;

  #[test]
    fn test_insert_node() {
        let mut tree = IndexTree::new();
    }

    #[test]
    fn test_search_node() {
        let mut tree = IndexTree::new();
        let doc = Doc::default();

        let s1: Type = doc.string("a").into();
        let s2: Type = doc.string("b").into();
        let s3: Type = doc.string("c").into();
        let s4: Type = doc.string("d").into();

        s1.item_ref().borrow_mut().index = Some(FractionalIndex::default());
        s2.item_ref().borrow_mut().index = Some(FractionalIndex::new_after(&s1.index()));
        s3.item_ref().borrow_mut().index = Some(FractionalIndex::new_after(&s2.index()));
        s4.item_ref().borrow_mut().index =
            Some(FractionalIndex::new_between(&s2.index(), &s3.index()).unwrap());

        tree.insert(s1.clone());
        tree.insert(s2.clone());
        tree.insert(s3.clone());
        tree.insert(s4.clone());

        assert_eq!(tree.search(&s1), true);
        assert_eq!(tree.search(&s2), true);
        assert_eq!(tree.search(&s3), true);

        // for i in tree.iter() {
        //     println!(
        //         "item: {:?}, index: {:?}, left: {:?}",
        //         i.item.content(),
        //         i.index(),
        //         tree.left_count(&i.item)
        //     );
        // }
        //
        // tree.print_tree();

        assert_eq!(tree.left_count(&s1), 0);
        assert_eq!(tree.left_count(&s2), 1);
        assert_eq!(tree.left_count(&s4), 2);
        assert_eq!(tree.left_count(&s3), 3);
    }
}
