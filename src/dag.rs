use crate::change::Change;

// Dag stores the directed acyclic graph of item dependencies.
// it is used to determine the order of item integration into the document.
// Dag can be used to rollback the document to a previous state.
// The nodes in the dag are the Change objects.
struct Dag {
    changes: Vec<Change>,
    graph: Vec<Vec<usize>>,
}
