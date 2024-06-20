use std::cell::RefCell;
use std::rc::Rc;

type NodeRef = Rc<RefCell<Node>>;

#[derive(Debug, Clone)]
struct Node {
    value: i32,
    left: Option<NodeRef>,
    right: Option<NodeRef>,
}

impl Node {
    fn new(value: i32) -> Node {
        Node {
            value,
            left: None,
            right: None,
        }
    }

    fn into_ref(self) -> NodeRef {
        Rc::new(RefCell::new(self))
    }
}

impl From<Node> for NodeRef {
    fn from(node: Node) -> NodeRef {
        Rc::new(RefCell::new(node))
    }
}

struct List {
    root: NodeRef,
}

impl List {
    fn new() -> List {
        List {
            root: Node::new(-1).into(),
        }
    }

    fn append(&self, value: i32) {
        let node = Node::new(value).into_ref();
        let mut current = self.root.clone();

        loop {
            let next = current.borrow().right.clone();
            if let Some(next) = next {
                current = next;
            } else {
                let mut current_node = current.borrow_mut();
                current_node.right = Some(node.clone());
                break;
            }
        }
    }

    fn prepend(&self, value: i32) {
        let node = Node::new(value).into_ref();

        let root = self.root.clone();
        let next = root.borrow().right.clone();

        node.borrow_mut().right = next;
        root.borrow_mut().right = Some(node.clone());
        self.root.borrow_mut().right = Some(node.clone());
    }

    fn values(&self) -> Vec<i32> {
        let mut values = vec![];
        let mut current = self.root.clone();
        loop {
            let next = current.borrow().right.clone();
            if let Some(next) = next {
                values.push(next.borrow().value);
                current = next;
            } else {
                break;
            }
        }
        values
    }
}

#[cfg(test)]
mod test {
    use crate::z_list::List;

    #[test]
    fn test_rc_refcell_list() {
        let list = List::new();
        list.append(1);
        list.append(2);
        list.append(3);
        list.prepend(0);

        assert_eq!(list.values(), vec![0, 1, 2, 3]);
    }
}
