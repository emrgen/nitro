use crate::change::Change;
use crate::id::WithId;
use crate::Id;
use std::collections::HashSet;

/// The Frontier struct represents the most recent operations in a document from all clients.
#[derive(Default, Clone, Debug)]
pub struct Frontier {
    pub(crate) changes: Vec<Id>,
}

impl Frontier {
    pub fn from(changes: Vec<Change>) -> Self {
        let changes = changes
            .into_iter()
            .map(|c| Id::new(c.client, c.end))
            .collect();
        Self { changes }
    }
}

/// The ChangeFrontier struct represents the most recent changes in a document from all clients.
#[derive(Default, Clone, Debug)]
pub(crate) struct ChangeFrontier {
    pub(crate) changes: Vec<Change>,
}

impl ChangeFrontier {
    pub(crate) fn from(changes: Vec<Change>) -> Self {
        Self { changes }
    }

    pub(crate) fn insert(&mut self, change: Change) {
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
