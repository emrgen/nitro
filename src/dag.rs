use crate::bimapid::ClientId;
use crate::change::{ChangeId, ChangeStore};
use crate::change_store::ClientStackStore;
use crate::decoder::{Decode, Decoder};
use crate::encoder::{Encode, Encoder};
use crate::id::{IdComp, WithId};
use crate::Id;
use rand::prelude::{SliceRandom, StdRng};
use rand::{Rng, SeedableRng};
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt::{Debug, Formatter};

//     Default + WithId + Clone + Encode + Decode + Eq + PartialEq + Serialize
pub(crate) struct ChangeNode {
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

    // check if the parent is ready to be undone
    fn is_ready(&self, parent_id: Id) -> bool {
        if let Some(entry) = self.children.get(&parent_id) {
            entry.1 == 0 // If current count is 0, it's ready to be undone
        } else {
            true // if no entry exists, consider it ready, in other words the parent has no children
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

            if let Some(curr) = self.store.current(change_id.client) {
                if self.parents.is_ready(curr.change.id()) {
                    self.queue.insert(curr.change.clone());
                    self.ends.insert(curr.change.client, curr.change.clone());
                }
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

struct RandomDag {
    clients: Vec<ClientId>,
    ends: HashMap<ClientId, u32>,
    changes: Vec<ChangeId>,
    children: HashMap<ChangeId, Vec<ChangeId>>,
    parents: HashMap<ChangeId, Vec<ChangeId>>,
    rng: StdRng,
}

impl RandomDag {
    fn default() -> Self {
        Self::with_clients(1, 0)
    }

    fn with_clients(count: u32, rand: u64) -> Self {
        let clients = (0..count).map(|i| i).collect::<Vec<ClientId>>();
        let change = ChangeId::new(0, 1, 1);
        let mut ends = HashMap::new();
        for client in &clients {
            ends.insert(client.clone(), 1);
        }
        // first client has already taken a clock of 1
        ends.insert(0, 2);

        Self {
            clients,
            ends,
            changes: vec![change],
            children: HashMap::new(),
            parents: HashMap::new(),
            rng: StdRng::seed_from_u64(rand),
        }
    }

    // randomly generate a DAG with the given number of nodes
    fn generate(&mut self, nodes: u32) {
        for _ in 0..nodes {
            let client = self.clients[self.rng.gen_range(0..self.clients.len())];
            // randomly choose parents
            let parent_count = self.rng.gen_range(1..=4);
            let mut parents = HashSet::new();
            for _ in 0..parent_count {
                if let Some(parent) = self.changes.get(self.rng.gen_range(0..self.changes.len())) {
                    parents.insert(parent.clone());
                }
            }

            let start = self.ends.get(&client).cloned().unwrap_or(1);
            let end = start + self.rng.gen_range(1..4);
            self.ends.insert(client.clone(), end + 1);

            // create a new change
            let change = ChangeId::new(client, start, end);
            self.changes.push(change.clone());

            // add the change to the links
            parents.iter().for_each(|parent| {
                self.children
                    .entry(parent.clone())
                    .or_insert_with(Vec::new)
                    .push(change.clone());
            });

            let parents = parents.iter().cloned().collect::<Vec<ChangeId>>();
            self.parents.insert(change.clone(), parents);
        }
    }

    // random topological sort of the changes
    fn sort(&mut self) -> Vec<ChangeId> {
        let mut done = HashSet::new();
        let mut sorted = Vec::new();
        let mut store = ChangeStore::default();
        let deps = self.parents.clone();
        for change_id in self.changes.iter() {
            store.insert(change_id.clone());
        }
        let mut size = store.size();
        let clients = self.clients.clone();
        while size > 0 {
            // choose random client
            if let Some(client) = clients.choose(&mut self.rng).cloned() {
                if let Some(store) = store.id_store_mut(&client) {
                    if let Some(first) = store.first().cloned() {
                        let ok = deps.get(&first).map_or(true, |parents| {
                            parents.iter().all(|parent| done.contains(parent))
                        });
                        if ok {
                            sorted.push(first.clone());
                            done.insert(first.clone());
                            store.pop_first();
                            size -= 1;
                        }
                    }
                }
            }
        }

        sorted
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha1::digest::HashMarker;
    use std::fmt::format;

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

    #[test]
    fn test_change_dag_insert_and_undo_1() {
        let mut dag = ChangeDag::default();
        let c1 = ChangeId::new(1, 1, 1);
        let c2 = ChangeId::new(1, 2, 3);
        let c3 = ChangeId::new(1, 4, 7);

        dag.insert(ChangeNode::root(c1));
        dag.insert(ChangeNode::new(c2, vec![c1]));
        dag.insert(ChangeNode::new(c3, vec![c1]));

        let items = dag.sort_changes();
        assert_eq!(items.len(), 3);
        assert_eq!(items, vec![c1, c2, c3]);
    }

    #[test]
    fn test_change_dag_insert_and_undo_2() {
        for i in 0..1000 {
            let mut dag = ChangeDag::default();
            let c1 = ChangeId::new(0, 1, 1);
            let c2 = ChangeId::new(0, 2, 5);
            let c3 = ChangeId::new(0, 6, 9);
            let c4 = ChangeId::new(1, 1, 2);
            let c5 = ChangeId::new(1, 3, 5);

            dag.insert(ChangeNode::root(c1));
            dag.insert(ChangeNode::new(c2, vec![c1]));
            dag.insert(ChangeNode::new(c3, vec![c1]));
            dag.insert(ChangeNode::new(c4, vec![c3]));
            dag.insert(ChangeNode::new(c5, vec![c3, c4]));

            let items = dag.sort_changes();
            assert_eq!(items.len(), 5);
            assert_eq!(items, vec![c1, c2, c3, c4, c5]);
        }
    }

    #[test]
    fn generate_random_dag() {
        let mut rng = rand::thread_rng();
        let mut dag = RandomDag::with_clients(10, 2);

        dag.generate(1000);

        // println!("{:?}", dag.children);
        // println!("{:?}", dag.sort());

        let mut ch_dag1 = ChangeDag::default();
        let sort1 = dag.sort();
        for change in &sort1 {
            let parents = dag.parents.get(&change).cloned().unwrap_or_default();
            ch_dag1.insert(ChangeNode::new(change.clone(), parents));
        }
        let sorted_changes1 = ch_dag1.sort_changes();

        // fuzz test, for different topological sort must converge
        for i in 0..500 {
            let mut ch_dag2 = ChangeDag::default();
            let sort2 = dag.sort();
            for change in &sort2 {
                let parents = dag.parents.get(&change).cloned().unwrap_or_default();
                ch_dag2.insert(ChangeNode::new(change.clone(), parents));
            }
            let sorted_changes2 = ch_dag2.sort_changes();

            assert_ne!(sort1, sort2);
            assert_eq!(sorted_changes1, sorted_changes2)
        }
    }
}
