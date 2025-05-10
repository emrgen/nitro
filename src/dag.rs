use crate::change::Change;
use crate::persist::WeakStoreDataRef;
use crate::store::{DocStore, WeakStoreRef};
use crate::Id;
use hashbrown::HashMap;

// Dag stores the directed acyclic graph of item dependencies.
// it is used to determine the order of item integration into the document.
// Dag can be used to rollback the document to a previous state.
// The nodes in the dag are the Change object
#[derive(Default, Clone, Debug)]
pub(crate) struct ChangeDag {
    graph: HashMap<Change, Vec<Change>>,
}

impl ChangeDag {
    fn new() -> Self {
        Self::default()
    }

    // connect the new change to the existing changes
    fn add_change(&mut self, change: &Change, previous: Vec<Change>) {
        if self.graph.contains_key(change) {
            return;
        }

        // add the change to the graph
        self.graph.insert(change.clone(), vec![]);
        // add the previous changes to the graph
        for prev in previous {
            if let Some(deps) = self.graph.get_mut(&prev) {
                deps.push(change.clone());
            } else {
                self.graph.insert(prev.clone(), vec![change.clone()]);
            }
        }
    }

    // changes causally after given changes
    // pub(crate) fn after(&self, before: Vec<Change>) -> Vec<Change> {
    //     let mut visited = BTreeSet::new();
    //     let mut result = Vec::new();
    //     let mut stack = Vec::new();
    //
    //     for change in before {
    //         if visited.insert(change.clone()) {
    //             stack.push(change);
    //         }
    //     }
    //
    //     while let Some(change) = stack.pop() {
    //         if let Some(deps) = self.graph.get(&change) {
    //             for dep in deps {
    //                 if visited.insert(dep.clone()) {
    //                     stack.push(dep.clone());
    //                 }
    //             }
    //         }
    //     }
    //
    //     result
    // }
}

impl PartialEq for ChangeDag {
    fn eq(&self, other: &Self) -> bool {
        self.graph == other.graph
    }
}

impl Eq for ChangeDag {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::Change;
    use crate::Id;

    #[test]
    fn test_change_dag() {
        let mut dag = ChangeDag::new();
        dag.add_change(&Change::new(1, 0, 0), vec![]);
    }
}
