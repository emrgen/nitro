use crate::bimapid::ClientId;
use crate::change::ChangeId;
use crate::change_store::ClientStackStore;
use crate::decoder::{Decode, Decoder};
use crate::encoder::{Encode, Encoder};
use crate::id::{IdComp, WithId};
use crate::Id;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::{Debug, Formatter};

//     Default + WithId + Clone + Encode + Decode + Eq + PartialEq + Serialize
struct ChangeNode {
    skip: bool,
    change: ChangeId,
    parents: Vec<ChangeId>,
}

impl ChangeNode {
    pub(crate) fn root(change: ChangeId) -> Self {
        Self {
            change,
            parents: Vec::new(),
            skip: false,
        }
    }

    #[inline]
    pub fn new(change: ChangeId, parents: Vec<ChangeId>) -> Self {
        Self {
            change,
            parents,
            skip: false,
        }
    }

    #[inline]
    pub(crate) fn skipped(mut self) -> Self {
        self.skip = true;
        self
    }

    #[inline]
    pub(crate) fn client(&self) -> &ClientId {
        &self.change.client
    }
}

impl Default for ChangeNode {
    fn default() -> Self {
        Self {
            change: ChangeId::default(),
            parents: vec![],
            skip: false,
        }
    }
}

impl Debug for ChangeNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl IdComp for ChangeNode {
    fn comp_id(&self, other: &Id) -> Ordering {
        self.change.comp_id(&other)
    }
}

impl Clone for ChangeNode {
    fn clone(&self) -> Self {
        todo!()
    }
}

impl Ord for ChangeNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.change.cmp(&other.change)
    }
}

impl Eq for ChangeNode {}

impl PartialOrd for ChangeNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
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

#[derive(Debug, Clone, Default)]
struct ChangeLinks {
    children: HashMap<Id, (u32, u32)>, // (total_count, current_count)
    dirty: HashSet<Id>,                // dirty children that need to be updated
}

impl ChangeLinks {
    fn add(&mut self, id: Id) {
        let entry = self.children.entry(id).or_insert((0, 0));
        entry.0 += 1; // Increment total count
        entry.1 += 1; // Increment current count
    }

    // when child is undone the parent current count is decremented by 1
    fn unlink_parent(&mut self, parent_id: Id) -> bool {
        if let Some(entry) = self.children.get_mut(&parent_id) {
            self.dirty.insert(parent_id);
            if entry.1 > 0 {
                entry.1 -= 1; // Decrement total count
            }
            entry.1 == 0
        } else {
            false // Parent not found, nothing to undo
        }
    }

    fn reset(&mut self) {
        self.dirty.iter().for_each(|id| {
            if let Some(entry) = self.children.get_mut(id) {
                entry.1 = entry.0;
            }
        });
        self.dirty.clear();
    }
}

#[derive(Clone, Default)]
pub(crate) struct ChangeDag {
    // store of changes, indexed by (client, clock)
    store: ClientStackStore<ChangeNode>,
    // links between changes and their children
    parents: ChangeLinks,
    // ready to be undone
    queue: BTreeSet<ChangeId>,
    // ends tracks the ends of the DAG for each client which is in the queue
    ends: HashMap<ClientId, ChangeId>,
    // dirty clients that need to be reset one the undo-do-redo is done
    dirty: HashSet<ClientId>,
}

impl ChangeDag {
    // Insert a new change into the DAG.
    pub(crate) fn insert(&mut self, node: ChangeNode) -> Result<(), String> {
        node.parents
            .iter()
            .for_each(|change_id| self.parents.add(change_id.id()));

        if let Some(last) = self.store.last(&node.client()) {
            self.queue.remove(&last.change);
        }

        self.queue.insert(node.change.clone());
        // insert into ends
        self.ends.insert(node.change.client, node.change.clone());

        self.store.insert(node.change.client, node);

        Ok(())
    }

    // pop the last change from the store in topological order
    fn undo(&mut self) -> Option<ChangeId> {
        // pop the last change from the queue
        let last_id = self.queue.pop_last();
        if let Some(change_id) = last_id {
            let cursor = self.store.cursor(change_id.client);

            // move the cursor to the previous change
            // TODO: keep doing prev if current change is shippable
            while self
                .store
                .prev(change_id.client)
                .map_or(false, |node| node.skip)
            {
                // if the change is can be skipped for undo, we continue to the previous one
            }

            self.dirty.insert(change_id.client);

            if let Some(cursor) = cursor {
                self.store
                    .at_cursor(change_id.client, cursor)
                    .map(|node| &node.parents)
                    .map(|parents| {
                        parents.iter().for_each(|id| {
                            if self.parents.unlink_parent(id.id()) {
                                if let Some(last) = self.store.current(id.client) {
                                    // if the last change has become ready to be undone,
                                    if last.id() == id.id() {
                                        self.queue.insert(last.change);
                                        self.ends.insert(last.change.client, last.change.clone());
                                    }
                                }
                            }
                        });
                    });
            }
        }

        last_id
    }

    // Reset the state of the DAG, clearing the queue and resetting the store
    pub(crate) fn done(&mut self) {
        self.dirty.iter().for_each(|client_id| {
            self.store.reset_cursor(&client_id);
            if let Some(end) = self.ends.get(client_id) {
                self.queue.remove(end);
                if let Some(last) = self.store.last(client_id) {
                    self.queue.insert(last.change.clone());
                }
            }
        });
    }

    // this is for testing purposes, to sort the changes in the order they were undone
    fn sort_changes(&mut self) -> Vec<ChangeId> {
        let mut sorted_changes = Vec::new();
        while let Some(change_id) = self.undo() {
            sorted_changes.push(change_id);
        }

        sorted_changes.reverse();

        sorted_changes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_dag_insert_and_undo() {
        let mut dag = ChangeDag::default();
        let c1 = ChangeId::new(1, 0, 0);
        let c2 = ChangeId::new(1, 1, 1);
        let c3 = ChangeId::new(1, 2, 2);

        dag.insert(ChangeNode::root(c1));
        dag.insert(ChangeNode::new(c2, vec![c1]));
        dag.insert(ChangeNode::new(c3, vec![c2]));

        assert_eq!(dag.queue.len(), 1);
        let item = dag.undo();
        assert_eq!(item.unwrap(), c3);

        assert_eq!(dag.queue.len(), 1);
        let item = dag.undo();
        assert_eq!(item.unwrap(), c2);

        assert_eq!(dag.queue.len(), 1);
        let item = dag.undo();
        assert_eq!(item.unwrap(), c1);

        dag.done();

        let items = dag.sort_changes();
        assert_eq!(items.len(), 3);
        assert_eq!(items, vec![c1, c2, c3]);
    }
}
