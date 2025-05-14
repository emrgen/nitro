use crate::bimapid::ClientMap;
use crate::change::{ChangeId, ChangeStore};
use crate::id::WithId;
use crate::{ClientFrontier, Id};
use std::collections::HashSet;

/// The Frontier struct represents the most recent operations in a document from all clients.
#[derive(Default, Clone, Debug)]
pub struct Frontier {
    pub(crate) changes: Vec<Id>,
}

impl Frontier {
    pub fn from(changes: Vec<ChangeId>) -> Self {
        let changes = changes
            .into_iter()
            .map(|c| Id::new(c.client, c.end))
            .collect();
        Self { changes }
    }

    pub(crate) fn add(&mut self, id: Id) {
        /// FIXME: may contain duplicates
        self.changes.push(id);
    }

    /// Turn the frontier into a ChangeFrontier
    pub(crate) fn change_frontier(&self, store: &ChangeStore) -> ChangeFrontier {
        let mut change_frontier = ChangeFrontier::default();
        for id in &self.changes {
            if let Some(change) = store.find(id) {
                change_frontier.insert(change.clone());
            }
        }

        change_frontier
    }
}

/// The ChangeFrontier struct represents the most recent changes in a document from all clients.
#[derive(Default, Clone, Debug)]
pub(crate) struct ChangeFrontier {
    pub(crate) changes: Vec<ChangeId>,
}

impl ChangeFrontier {
    pub(crate) fn from(changes: Vec<ChangeId>) -> Self {
        Self { changes }
    }

    pub(crate) fn insert(&mut self, change: ChangeId) {
        self.changes.push(change);
    }

    pub(crate) fn frontier(&self) -> Frontier {
        let changes = self
            .changes
            .iter()
            .map(|c| Id::new(c.client, c.end))
            .collect();

        Frontier { changes }
    }
}
