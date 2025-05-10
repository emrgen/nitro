use crate::change::{Change, ChangeId};
use crate::persist::WeakStoreDataRef;
use crate::store::{DocStore, WeakStoreRef};
use crate::Id;
use btree_slab::{BTreeMap, BTreeSet};
use hashbrown::HashMap;

// Dag stores the directed acyclic graph of item dependencies.
// it is used to determine the order of item integration into the document.
// Dag can be used to rollback the document to a previous state.
// The nodes in the dag are the Change object
#[derive(Default, Clone)]
struct ChangeDag {
    changes: BTreeSet<Change>,
    graph: HashMap<Change, Vec<Change>>,
}

impl ChangeDag {
    fn new() -> Self {
        Self::default()
    }

    // connect the new change to the existing changes
    fn add_change(&mut self, change: Change, deps: Vec<Id>) {
        // check if the change is already in the graph
        if self.graph.contains_key(&change) {
            return;
        }

        // add the change to the graph
        self.graph.insert(change.clone(), Vec::new());

        // add the dependencies to the graph
        for dep in deps {
            let change = self.changes.get(&change).unwrap();

            if let Some(dep_change) = self.graph.get_mut(change) {
                dep_change.push(change.clone());
            } else {
                self.graph
                    .insert(Change::from(change), vec![change.clone()]);
            }
        }
    }

    // fn find_subgraph(&self, deps: &Vec<Id>) -> Vec<ChangeId> {
    //     let mut subgraph = Vec::new();
    // }
}
