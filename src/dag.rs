use crate::change::Change;
use crate::frontier::{ChangeFrontier, Frontier};
use crate::persist::WeakStoreDataRef;
use crate::store::{DocStore, WeakStoreRef};
use crate::Id;
use hashbrown::{HashMap, HashSet};
use std::collections::VecDeque;

// Dag stores the directed acyclic graph of item dependencies.
// it is used to determine the order of item integration into the document.
// Dag can be used to roll back the document to a previous state.
// The nodes in the dag are the Change object
#[derive(Default, Clone, Debug)]
pub(crate) struct ChangeDag {
    changes: HashMap<Change, u64>,
    forward: HashMap<Change, Vec<Change>>,
    backward: HashMap<Change, Vec<Change>>,
    tick: u64,
}

impl ChangeDag {
    fn new() -> Self {
        Self::default()
    }

    // connect the new change to the existing changes
    fn add_change(&mut self, change: &Change, previous: Vec<Change>) {
        if self.forward.contains_key(change) {
            return;
        }

        // if self.tick reaches u64::MAX, recreate the dag
        if self.tick == u64::MAX {
            let sorted = self.topological_sort();
            self.changes.clear();
            self.tick = 0;

            // insert all changes in the sorted order
            for change in sorted {
                self.changes.insert(change.clone(), self.tick);
                self.tick += 1;
            }
        }

        // add the change to the change map
        self.changes.insert(change.clone(), self.tick);
        self.tick += 1;

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
    }

    // use khan's algorithm to sort the changes in topological order
    fn topological_sort(&self) -> Vec<Change> {
        let mut result = Vec::new();
        let mut queue: VecDeque<Change> = VecDeque::new();
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

    /// find all changes that are after the given changes in application order
    /// the changes are sorted in the order they were added to the dag
    /// to restore the document to the frontier, the changes must be rolled back in the reverse order
    pub(crate) fn after(&self, frontier: ChangeFrontier) -> Vec<Change> {
        let mut result = Vec::new();

        // sort the changes by their index in the change list, lower index first
        let mut change_list = frontier.changes.clone();
        change_list.sort_by_key(|c| self.changes.get(c).unwrap());

        // use stack based dfs for finding topological order
        let mut stack = Vec::new();
        let mut visited: HashSet<Change> = HashSet::new();

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
                        result.push(dep.clone());
                        stack.push(dep.clone());
                    }
                }
            }
        }

        // TODO: optimize later, for now extra overhead is not a problem
        result.sort_by_key(|c| self.changes.get(c).unwrap());

        result
    }

    /// rollback removes the given changes from the dag
    pub(crate) fn rollback(&mut self, changes: &Vec<Change>) {}
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
    use crate::change::Change;
    use crate::Id;

    #[test]
    fn test_change_dag() {
        let mut dag = ChangeDag::new();
        dag.add_change(&Change::new(1, 0, 0), vec![]);
    }
}
