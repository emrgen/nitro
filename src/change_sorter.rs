// keep the changes in sorted order for the document

// ChangeTree is a BTree like append-only structure
// It has a simplified vector like API for efficient insertion by index.
// Support index look up and slice api
struct ChangeTree<V> {
  root: Node<V>
}

enum Node<V> {
  Branch(Branch<V>),
  Leaf(Leaf<V>),
}

struct Branch<V> {
  children: Vec<Node<V>>,
  total: usize,
}
