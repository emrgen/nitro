use crate::bimapid::{ClientId, ClientMap};
use crate::change::{Change, ChangeId};
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::frontier::{ChangeFrontier, Frontier};
use crate::id::WithId;
use crate::persist::WeakStoreDataRef;
use crate::store::{ClientStore, DocStore, WeakStoreRef};
use crate::{ClientFrontier, ClockTick, Id};
use btree_slab::{BTreeMap, BTreeSet};
use hashbrown::{HashMap, HashSet};
use priority_queue::PriorityQueue;
use serde::Serialize;
use std::collections::VecDeque;

//     Default + WithId + Clone + Encode + Decode + Eq + PartialEq + Serialize
#[derive(Debug, Clone, Default, Eq, Serialize, Hash)]
struct ChangeNode {
    change: ChangeId,
    parent: Vec<ChangeId>,
    children: u32,
}

impl PartialEq for ChangeNode {
    fn eq(&self, other: &Self) -> bool {
        self.change == other.change
    }
}

impl WithId for ChangeNode {
    fn id(&self) -> Id {
        self.change.id()
    }
}

impl Encode for ChangeNode {
    fn encode<T: Encoder>(&self, e: &mut T, cx: &mut EncodeContext) {
        todo!()
    }
}

impl Decode for ChangeNode {
    fn decode<T: Decoder>(d: &mut T, ctx: &DecodeContext) -> Result<Self, String>
    where
        Self: Sized,
    {
        todo!()
    }
}

#[derive(Clone)]
pub(crate) struct ChangeDag {
    store: ClientStore<ChangeNode>,
    ends: HashMap<ClientId, ChangeId>,
    queue: BTreeSet<ChangeId>,
}

impl ChangeDag {
    pub(crate) fn new() -> Self {
        Self {
            store: ClientStore::default(),
            ends: HashMap::new(),
            queue: BTreeSet::new(),
        }
    }

    /// Insert a new change into the DAG.
    pub(crate) fn insert(
        &mut self,
        change_id: ChangeId,
        parent_ids: Vec<ChangeId>,
    ) -> Result<(), String> {
        let id = change_id.id();
        if self.store.contains(&id) {
            return Ok(()); // Change already exists
        }

        // Update the parent nodes' children count
        for parent_id in &parent_ids {
            if let Some(parent_node) = self.store.get_mut(&parent_id.id()) {
                parent_node.children += 1;
            } else {
                // If the parent is not found, it means the change is not in the store
                // This should not happen in a well-formed DAG
                panic!("Parent change not found in the store: {:?}", parent_id);
            }
        }

        // If the change is the last change for this client, update the end
        if let Some(end) = self.ends.get_mut(&change_id.client) {
            if end.id() < id {
                self.queue.remove(end);
                self.queue.insert(change_id.clone());
                *end = change_id.clone();
            }
        }

        // Create a new ChangeNode
        let node = ChangeNode {
            change: change_id,
            parent: parent_ids,
            children: 0,
        };

        // Insert the new node into the store
        self.store.insert(node);

        Ok(())
    }

    // pop the last change from the store in topological order
    #[inline]
    fn undo(&mut self) -> Option<ChangeNode> {
        let last_node = self.pop_queue();

        if let Some((node)) = last_node {
            // Decrease the children count of the parent nodes
            for parent in &node.parent {
                if let Some(parent_node) = self.store.get_mut(&parent.id()) {
                    parent_node.children -= 1;
                    // If the parent has no more children, and is the last change for this client,
                    if parent_node.children == 0 {
                        let is_end = self
                            .ends
                            .get(&parent_node.change.client)
                            .map_or(false, |end| end.id() == parent_node.change.id());
                        if is_end {
                            self.queue.insert(parent_node.change);
                        }
                    }
                } else {
                    // If the parent is not found, it means the change is not in the store
                    // This should not happen in a well-formed DAG
                    panic!("Parent change not found in the store: {:?}", parent);
                }
            }

            // check if the last change for this client has no children
            self.store.get_last(&node.change.client).map(|end| {
                if end.children == 0 {
                    self.queue.insert(node.change);
                }
            });

            Some(node)
        } else {
            None
        }
    }

    // Pop the last change from the queue and update the store
    fn pop_queue(&mut self) -> Option<ChangeNode> {
        let change = self.queue.pop_last();
        if let Some(change_id) = change {
            let old_last_node = self.store.pop_last(&change_id.client);
            let new_last_node = self.store.get_last(&change_id.client);

            if let Some(node) = new_last_node {
                self.ends.insert(node.change.client, node.change);
            }

            old_last_node
        } else {
            None
        }
    }

    #[inline]
    fn redo(&mut self, change_id: ChangeId, parents: Vec<ChangeId>) {
        for parent in &parents {
            if let Some(node) = self.store.get_mut(&parent.id()) {
                node.children += 1;
            } else {
                // If the parent is not found, it means the change is not in the store
                // This should not happen in a well-formed DAG
                panic!("Parent change not found in the store: {:?}", parent);
            }
        }

        self.store.insert(ChangeNode {
            change: change_id,
            parent: parents,
            children: 0,
        });
    }
}

// // Dag stores the directed acyclic graph of Change dependencies.
// // Dag can be used to roll back the document to a previous state.
// #[derive(Default, Clone, Debug)]
// pub(crate) struct ChangeDag {
//     root: Option<ChangeId>,
//     pub(crate) changes: HashMap<ChangeId, u64>,
//     forward: HashMap<ChangeId, Vec<ChangeId>>,
//     backward: HashMap<ChangeId, Vec<ChangeId>>,
//     // local_tick is used to assign a unique index to each change
//     // used to sort the changes in topological order
//     local_tick: u64,
// }
//
// impl ChangeDag {
//     /// connect the new change to the existing changes
//     pub(crate) fn insert(&mut self, change: &ChangeId, deps: Vec<ChangeId>) {
//         if self.changes.contains_key(change) {
//             return;
//         }
//
//         // initial change is the document create, so it can't be rolled back
//         if self.changes.is_empty() {
//             self.root = Some(change.clone());
//         }
//
//         // if self.tick reaches u64::MAX, recreate the dag
//         if self.local_tick == u64::MAX {
//             let sorted = self.topological_sort();
//             self.changes.clear();
//             self.local_tick = 0;
//
//             // insert all changes in the sorted order
//             for change in sorted {
//                 self.changes.insert(change.clone(), self.local_tick);
//                 self.local_tick += 1;
//             }
//         }
//
//         // add the change to the change map
//         self.changes.insert(change.clone(), self.local_tick);
//         self.local_tick += 1;
//
//         // add the change to the graph
//         self.forward.insert(change.clone(), vec![]);
//         self.backward.insert(change.clone(), vec![]);
//
//         for dep in &deps {
//             // add the change to the forward graph
//             if let Some(deps) = self.forward.get_mut(&dep) {
//                 deps.push(change.clone());
//             } else {
//                 self.forward.insert(dep.clone(), vec![change.clone()]);
//             }
//
//             // add the change to the backward graph
//             if let Some(deps) = self.backward.get_mut(change) {
//                 deps.push(dep.clone());
//             } else {
//                 self.backward.insert(change.clone(), vec![dep.clone()]);
//             }
//         }
//
//         // keep the forward and backward graph sorted
//         // so that all clients with same items will have same topological order with
//         for prev in &deps {
//             self.forward.get_mut(&prev).unwrap().sort();
//             self.backward.get_mut(&change).unwrap().sort();
//         }
//     }
//
//     /// Find all changes done in the document
//     /// timeline excludes the first change (the document root create change)
//     pub(crate) fn timeline(&self) -> Vec<Change> {
//         self.after(ChangeFrontier::new(vec![self.root.clone().unwrap()]))
//     }
//
//     // use khan's algorithm to sort the changes in topological order
//     fn topological_sort(&self) -> Vec<ChangeId> {
//         let mut result = Vec::new();
//         let mut queue: VecDeque<ChangeId> = VecDeque::new();
//         let mut in_degree = HashMap::new();
//
//         // calculate the in-degree of each change
//         for (change, deps) in &self.forward {
//             in_degree.insert(change.clone(), deps.len());
//             if deps.is_empty() {
//                 queue.push_back(change.clone());
//             }
//         }
//
//         // pop, update, and push the changes in the queue
//         while !queue.is_empty() {
//             let change = queue.pop_front().unwrap();
//
//             if let Some(deps) = self.forward.get(&change) {
//                 for dep in deps {
//                     if let Some(count) = in_degree.get_mut(dep) {
//                         *count -= 1;
//                         if *count == 0 {
//                             queue.push_back(dep.clone());
//                         }
//                     }
//                 }
//             }
//
//             result.push(change);
//         }
//
//         result
//     }
//
//     /// Find all changes that are after the given changes in integration order.
//     /// The changes are sorted in the order they were added to the dag
//     /// to restore the document to the frontier, the changes must be rolled back in the reverse order of integration.
//     pub(crate) fn after(&self, frontier: ChangeFrontier) -> Vec<Change> {
//         let mut result = Vec::new();
//
//         // println!("after: {:?}", frontier);
//
//         // sort the changes by their index in the change list, lower index first
//         let mut change_list = frontier.changes.clone();
//         // println!("change_list: {:?}", change_list);
//         // println!("change_list: {:?}", self.changes.len());
//
//         change_list.sort_by_key(|c| self.changes.get(c).unwrap());
//
//         // use stack based dfs for finding topological order
//         let mut stack = Vec::new();
//         let mut visited: HashSet<ChangeId> = HashSet::new();
//
//         for change in change_list {
//             if visited.contains(&change) {
//                 continue;
//             }
//
//             stack.push(change.clone());
//
//             // dfs
//             while !stack.is_empty() {
//                 let change = stack.pop().unwrap();
//                 if let Some(deps) = self.forward.get(&change) {
//                     for dep in deps {
//                         // if the dep is already visited, skip it
//                         if visited.contains(dep) {
//                             continue;
//                         }
//                         visited.insert(dep.clone());
//                         result.push(Change::with_deps(
//                             dep.clone(),
//                             self.backward
//                                 .get(&change)
//                                 .map_or(vec![], |deps| deps.clone()),
//                         ));
//                         stack.push(dep.clone());
//                     }
//                 }
//             }
//         }
//
//         // TODO: optimize later, for now extra overhead is not a problem
//         result.sort_by_key(|c| self.changes.get(&c.id).unwrap());
//
//         result
//     }
//
//     /// rollback removes the given changes from the dag and returns the changes in the order they were applied
//     pub(crate) fn rollback(&mut self, changes: &[ChangeId]) -> Result<(), &'static str> {
//         // reverse iterate over the changes to remove them in the reverse order
//         // of integration
//         for change in changes.iter().rev() {
//             let dependents = self.forward.remove(change).map_or(0, |deps| deps.len());
//             if dependents != 0 {
//                 return Err("Cannot rollback changes that have dependents");
//             }
//
//             if let Some(deps) = self.backward.remove(change) {
//                 for dep in deps {
//                     if let Some(forward_deps) = self.forward.get_mut(&dep) {
//                         forward_deps.retain(|c| c != change);
//                     }
//                 }
//             }
//
//             self.changes.remove(change);
//         }
//
//         Ok(())
//     }
//
//     pub(crate) fn contains(&self, target_id: &Id) -> bool {
//         self.changes.contains_key(&target_id.into())
//     }
//
//     /// find the client frontier for the given hash, if the hash is not found, return None
//     pub(crate) fn find_client_frontier(
//         &self,
//         commit_hash: String,
//         client_map: &ClientMap,
//     ) -> Option<ClientFrontier> {
//         let changes = self.timeline();
//         let mut client_frontier = ClientFrontier::default();
//
//         /// apply changes and check if the commit hash matches
//         for change in &changes {
//             if let Some(client) = client_map.get_client(&change.id.client) {
//                 client_frontier.add(client.clone(), change.id.end);
//             }
//
//             if commit_hash.len() == 8 && client_frontier.short_hash() == commit_hash {
//                 return Some(client_frontier.clone());
//             } else if client_frontier.short_hash() == commit_hash {
//                 return Some(client_frontier.clone());
//             }
//         }
//
//         None
//     }
// }
//
// impl PartialEq for ChangeDag {
//     fn eq(&self, other: &Self) -> bool {
//         self.forward == other.forward
//     }
// }
//
// impl Eq for ChangeDag {}
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::change::ChangeId;
//     use crate::Id;
//
//     macro_rules! change {
//         ($c:expr) => {
//             ChangeId::new($c, 0, 0)
//         };
//     }
//
//     macro_rules! changes {
//         ($($c:expr),*) => {
//             vec![$(change!($c)),*]
//         };
//     }
//
//     macro_rules! frontier {
//         ($($c:expr),*) => {
//             ChangeFrontier::new(vec![$(change!($c)),*])
//         };
//     }
//
//     fn change_ids(changes: Vec<Change>) -> Vec<ChangeId> {
//         changes.into_iter().map(|c| c.id).collect()
//     }
//
//     #[test]
//     fn test_change_dag() {
//         let mut dag = ChangeDag::default();
//         dag.insert(&ChangeId::new(0, 0, 0), vec![]);
//         dag.insert(&ChangeId::new(2, 0, 0), vec![ChangeId::new(1, 0, 0)]);
//         dag.insert(&ChangeId::new(3, 0, 0), vec![ChangeId::new(1, 0, 0)]);
//         dag.insert(
//             &ChangeId::new(4, 0, 0),
//             vec![ChangeId::new(2, 0, 0), ChangeId::new(3, 0, 0)],
//         );
//
//         let frontier = ChangeFrontier {
//             changes: vec![ChangeId::new(1, 0, 0)],
//         };
//         let after = dag.after(frontier);
//         assert_eq!(after.len(), 3);
//
//         let frontier = frontier!(2, 4);
//         let after = dag.after(frontier);
//         assert_eq!(after.len(), 1);
//     }
//
//     fn create_dag() -> ChangeDag {
//         let mut dag = ChangeDag::default();
//         let change = |c| ChangeId::new(c, 0, 0);
//         dag.insert(&change(1), vec![]);
//         dag.insert(&change(2), changes!(1));
//         dag.insert(&change(3), changes!(1));
//         dag.insert(&change(4), changes!(2, 3));
//         dag.insert(&change(5), changes!(3));
//         dag.insert(&change(6), changes!(4));
//         dag.insert(&change(7), changes!(3));
//         dag.insert(&change(8), changes!(4, 7));
//         dag.insert(&change(9), changes!(6, 8));
//         dag.insert(&change(10), changes!(7));
//         dag.insert(&change(11), changes!(8, 10));
//         dag.insert(&change(12), changes!(8));
//         dag.insert(&change(13), changes!(9, 11));
//
//         dag
//     }
//
//     #[test]
//     fn test_after_rollback() {
//         let mut dag = create_dag();
//
//         let after = dag.after(frontier!(8));
//         assert_eq!(after.len(), 4);
//         assert_eq!(change_ids(after), changes!(9, 11, 12, 13));
//
//         let after = dag.after(frontier!(4, 8));
//         assert_eq!(after.len(), 6);
//         assert_eq!(change_ids(after), changes!(6, 8, 9, 11, 12, 13));
//
//         let after = dag.after(frontier!(2, 5, 10));
//         assert_eq!(change_ids(after), changes!(4, 6, 8, 9, 11, 12, 13));
//
//         let after = dag.after(frontier!(8));
//         dag.rollback(&change_ids(after));
//         assert_eq!(dag.changes.len(), 9); // [1-8,10]
//
//         let after = dag.after(frontier!(4));
//         assert_eq!(change_ids(after), changes!(6, 8));
//     }
//
//     #[test]
//     fn test_timeline() {
//         let dag = create_dag();
//         let timeline = dag
//             .timeline()
//             .iter()
//             .map(|c| c.id.clone())
//             .collect::<Vec<_>>();
//         assert_eq!(timeline.len(), 12);
//         assert_eq!(timeline, changes!(2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13));
//     }
// }
