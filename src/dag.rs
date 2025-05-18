use crate::bimapid::{ClientId, ClientMap};
use crate::change::{Change, ChangeId};
use crate::frontier::{ChangeFrontier, Frontier};
use crate::id::WithId;
use crate::persist::WeakStoreDataRef;
use crate::store::{DocStore, WeakStoreRef};
use crate::{ClientFrontier, ClockTick, Id};
use hashbrown::{HashMap, HashSet};
use std::collections::VecDeque;

// Dag stores the directed acyclic graph of Change dependencies.
// Dag can be used to roll back the document to a previous state.
#[derive(Default, Clone, Debug)]
pub(crate) struct ChangeDag {
    root: Option<ChangeId>,
    pub(crate) changes: HashMap<ChangeId, u64>,
    forward: HashMap<ChangeId, Vec<ChangeId>>,
    backward: HashMap<ChangeId, Vec<ChangeId>>,
    // local_tick is used to assign a unique index to each change
    // used to sort the changes in topological order
    local_tick: u64,
}

impl ChangeDag {
    /// connect the new change to the existing changes
    pub(crate) fn insert(&mut self, change: &ChangeId, previous: Vec<ChangeId>) {
        if self.forward.contains_key(change) {
            return;
        }

        // initial change is the document create, so it can't be rolled back
        if self.changes.is_empty() {
            self.root = Some(change.clone());
        }

        // if self.tick reaches u64::MAX, recreate the dag
        if self.local_tick == u64::MAX {
            let sorted = self.topological_sort();
            self.changes.clear();
            self.local_tick = 0;

            // insert all changes in the sorted order
            for change in sorted {
                self.changes.insert(change.clone(), self.local_tick);
                self.local_tick += 1;
            }
        }

        // add the change to the change map
        self.changes.insert(change.clone(), self.local_tick);
        self.local_tick += 1;

        // add the change to the graph
        self.forward.insert(change.clone(), vec![]);
        self.backward.insert(change.clone(), vec![]);

        for prev in &previous {
            // add the change to the forward graph
            if let Some(deps) = self.forward.get_mut(&prev) {
                deps.push(change.clone());
            } else {
                self.forward.insert(prev.clone(), vec![change.clone()]);
            }

            // add the change to the backward graph
            if let Some(deps) = self.backward.get_mut(change) {
                deps.push(prev.clone());
            } else {
                self.backward.insert(change.clone(), vec![prev.clone()]);
            }
        }

        // keep the forward and backward graph sorted
        // so that all clients with same items will have same topological order with
        for prev in &previous {
            self.forward.get_mut(&prev).unwrap().sort();
            self.backward.get_mut(&change).unwrap().sort();
        }
    }

    /// Find all changes done in the document
    /// timeline excludes the first change (the document root create change)
    pub(crate) fn timeline(&self) -> Vec<Change> {
        self.after(ChangeFrontier::new(vec![self.root.clone().unwrap()]))
    }

    // use khan's algorithm to sort the changes in topological order
    fn topological_sort(&self) -> Vec<ChangeId> {
        let mut result = Vec::new();
        let mut queue: VecDeque<ChangeId> = VecDeque::new();
        let mut in_degree = HashMap::new();

        // calculate the in-degree of each change
        for (change, deps) in &self.forward {
            in_degree.insert(change.clone(), deps.len());
            if deps.is_empty() {
                queue.push_back(change.clone());
            }
        }

        // pop, update, and push the changes in the queue
        while !queue.is_empty() {
            let change = queue.pop_front().unwrap();

            if let Some(deps) = self.forward.get(&change) {
                for dep in deps {
                    if let Some(count) = in_degree.get_mut(dep) {
                        *count -= 1;
                        if *count == 0 {
                            queue.push_back(dep.clone());
                        }
                    }
                }
            }

            result.push(change);
        }

        result
    }

    /// Find all changes that are after the given changes in integration order.
    /// The changes are sorted in the order they were added to the dag
    /// to restore the document to the frontier, the changes must be rolled back in the reverse order of integration.
    pub(crate) fn after(&self, frontier: ChangeFrontier) -> Vec<Change> {
        let mut result = Vec::new();

        // sort the changes by their index in the change list, lower index first
        let mut change_list = frontier.changes.clone();
        change_list.sort_by_key(|c| self.changes.get(c).unwrap());

        // use stack based dfs for finding topological order
        let mut stack = Vec::new();
        let mut visited: HashSet<ChangeId> = HashSet::new();

        for change in change_list {
            if visited.contains(&change) {
                continue;
            }

            stack.push(change.clone());

            // dfs
            while !stack.is_empty() {
                let change = stack.pop().unwrap();
                if let Some(deps) = self.forward.get(&change) {
                    for dep in deps {
                        // if the dep is already visited, skip it
                        if visited.contains(dep) {
                            continue;
                        }
                        visited.insert(dep.clone());
                        result.push(Change::with_deps(
                            dep.clone(),
                            self.backward
                                .get(&change)
                                .map_or(vec![], |deps| deps.clone()),
                        ));
                        stack.push(dep.clone());
                    }
                }
            }
        }

        // TODO: optimize later, for now extra overhead is not a problem
        result.sort_by_key(|c| self.changes.get(&c.id).unwrap());

        result
    }

    /// rollback removes the given changes from the dag and returns the changes in the order they were applied
    pub(crate) fn rollback(&mut self, changes: &Vec<ChangeId>) {
        // reverse iterate over the changes to remove them in the reverse order
        // of integration
        for change in changes.iter().rev() {
            if let Some(deps) = self.forward.remove(change) {
                for dep in deps {
                    if let Some(backward_deps) = self.backward.get_mut(&dep) {
                        backward_deps.retain(|c| c != change);
                    }
                }
            }

            if let Some(backward_deps) = self.backward.remove(change) {
                for dep in backward_deps {
                    if let Some(forward_deps) = self.forward.get_mut(&dep) {
                        forward_deps.retain(|c| c != change);
                    }
                }
            }

            self.changes.remove(change);
        }
    }

    pub(crate) fn contains(&self, target_id: &Id) -> bool {
        self.changes.contains_key(&target_id.into())
    }

    /// find the client frontier for the given hash, if the hash is not found, return None
    pub(crate) fn find_client_frontier(
        &self,
        commit_hash: String,
        client_map: &ClientMap,
    ) -> Option<ClientFrontier> {
        let changes = self.timeline();
        let mut client_frontier = ClientFrontier::default();

        /// apply changes and check if the commit hash matches
        for change in &changes {
            if let Some(client) = client_map.get_client(&change.id.client) {
                client_frontier.add(client.clone(), change.id.end);
            }

            if commit_hash.len() == 8 && client_frontier.short_hash() == commit_hash {
                return Some(client_frontier.clone());
            } else if client_frontier.short_hash() == commit_hash {
                return Some(client_frontier.clone());
            }
        }

        None
    }
}

impl PartialEq for ChangeDag {
    fn eq(&self, other: &Self) -> bool {
        self.forward == other.forward
    }
}

impl Eq for ChangeDag {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::change::ChangeId;
    use crate::Id;

    macro_rules! change {
        ($c:expr) => {
            ChangeId::new($c, 0, 0)
        };
    }

    macro_rules! changes {
        ($($c:expr),*) => {
            vec![$(change!($c)),*]
        };
    }

    macro_rules! frontier {
        ($($c:expr),*) => {
            ChangeFrontier::new(vec![$(change!($c)),*])
        };
    }

    fn change_ids(changes: Vec<Change>) -> Vec<ChangeId> {
        changes.into_iter().map(|c| c.id).collect()
    }

    #[test]
    fn test_change_dag() {
        let mut dag = ChangeDag::default();
        dag.insert(&ChangeId::new(0, 0, 0), vec![]);
        dag.insert(&ChangeId::new(2, 0, 0), vec![ChangeId::new(1, 0, 0)]);
        dag.insert(&ChangeId::new(3, 0, 0), vec![ChangeId::new(1, 0, 0)]);
        dag.insert(
            &ChangeId::new(4, 0, 0),
            vec![ChangeId::new(2, 0, 0), ChangeId::new(3, 0, 0)],
        );

        let frontier = ChangeFrontier {
            changes: vec![ChangeId::new(1, 0, 0)],
        };
        let after = dag.after(frontier);
        assert_eq!(after.len(), 3);

        let frontier = frontier!(2, 4);
        let after = dag.after(frontier);
        assert_eq!(after.len(), 1);
    }

    fn create_dag() -> ChangeDag {
        let mut dag = ChangeDag::default();
        let change = |c| ChangeId::new(c, 0, 0);
        dag.insert(&change(1), vec![]);
        dag.insert(&change(2), changes!(1));
        dag.insert(&change(3), changes!(1));
        dag.insert(&change(4), changes!(2, 3));
        dag.insert(&change(5), changes!(3));
        dag.insert(&change(6), changes!(4));
        dag.insert(&change(7), changes!(3));
        dag.insert(&change(8), changes!(4, 7));
        dag.insert(&change(9), changes!(6, 8));
        dag.insert(&change(10), changes!(7));
        dag.insert(&change(11), changes!(8, 10));
        dag.insert(&change(12), changes!(8));
        dag.insert(&change(13), changes!(9, 11));

        dag
    }

    #[test]
    fn test_after_rollback() {
        let mut dag = create_dag();

        let after = dag.after(frontier!(8));
        assert_eq!(after.len(), 4);
        assert_eq!(change_ids(after), changes!(9, 11, 12, 13));

        let after = dag.after(frontier!(4, 8));
        assert_eq!(after.len(), 6);
        assert_eq!(change_ids(after), changes!(6, 8, 9, 11, 12, 13));

        let after = dag.after(frontier!(2, 5, 10));
        assert_eq!(change_ids(after), changes!(4, 6, 8, 9, 11, 12, 13));

        let after = dag.after(frontier!(8));
        dag.rollback(&change_ids(after));
        assert_eq!(dag.changes.len(), 9); // [1-8,10]

        let after = dag.after(frontier!(4));
        assert_eq!(change_ids(after), changes!(6, 8));
    }

    #[test]
    fn test_timeline() {
        let dag = create_dag();
        let timeline = dag
            .timeline()
            .iter()
            .map(|c| c.id.clone())
            .collect::<Vec<_>>();
        assert_eq!(timeline.len(), 12);
        assert_eq!(timeline, changes!(2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13));
    }
}
