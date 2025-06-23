use crate::change::ChangeId;
use btree_slab::BTreeMap;
use fractional_index::FractionalIndex;
use hashbrown::HashMap;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

type NodeRef = Rc<RefCell<Node>>;
type WeakNodeRef = Weak<RefCell<Node>>;

#[derive(Debug, Clone, Default)]
struct Node {
    // skip rollback for this change
    // skip: bool,
    change_id: ChangeId,
    children: Option<Vec<NodeRef>>,
    // next node visited in order traversal
    next: Option<WeakNodeRef>,
    // previous node visited in order traversal
    prev: Option<WeakNodeRef>,
}

#[derive(Default, Clone)]
pub(crate) struct ChangeList {
    nodes: HashMap<ChangeId, NodeRef>,
    tree: NodeRef,
}

impl ChangeList {
    pub(crate) fn from(root: ChangeId) -> Self {
        let mut change_list = ChangeList::default();
        change_list.tree = Rc::new(RefCell::new(Node {
            change_id: root,
            children: None,
            prev: None,
            next: None,
        }));

        change_list.nodes.insert(root, change_list.tree.clone());

        change_list
    }

    pub(crate) fn insert(&mut self, change_id: ChangeId, parent_id: ChangeId) {
        let parent_node = self.nodes.get(&parent_id).expect("Parent node not found");
        let new_node = Rc::new(RefCell::new(Node {
            change_id,
            children: None,
            prev: None,
            next: None,
        }));

        let index = if let Some(children) = &mut parent_node.borrow_mut().children {
            let index = children
                .binary_search_by_key(&change_id, |node| node.borrow().change_id)
                .unwrap_or_else(|e| e);
            children.insert(index, new_node.clone());
            index
        } else {
            parent_node.borrow_mut().children = Some(vec![new_node.clone()]);
            0
        };

        // Update the new node's prev and next pointers
        if let Some(children) = &mut parent_node.borrow_mut().children {
            if index > 0 {
                let prev_node = &children[index - 1];
                new_node.borrow_mut().prev = prev_node.borrow().prev.clone()
            }
            if index < children.len() - 1 {
                new_node.borrow_mut().next = Some(Rc::downgrade(&children[index + 1]));
            }
        }

        self.nodes.insert(change_id, new_node.clone());
    }

    pub(crate) fn get(&self, change_id: ChangeId) -> Option<NodeRef> {
        self.nodes.get(&change_id).cloned()
    }

    // return a double iterator over the change list
}
