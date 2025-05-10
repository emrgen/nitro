use crate::bimapid::ClientId;
use crate::id::IdRange;
use crate::{ClockTick, Id};
use btree_slab::BTreeMap;
use std::ops::Range;
// Change represents a set of consecutive changes in the document by a client, which includes a range of clock ticks that are applied to the document.
// It is used to track the changes made by a client in an editor transaction.

#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub(crate) struct Change {
    client: ClientId,
    start: ClockTick,
    end: ClockTick,
}

impl Change {
    pub fn new(client: ClientId, start: ClockTick, end: ClockTick) -> Self {
        Change { client, start, end }
    }

    pub(crate) fn range(&self) -> Range<ClockTick> {
        self.start..self.end
    }

    #[inline]
    pub(crate) fn compare(&self, other: &Self) -> std::cmp::Ordering {
        if self.client == other.client {
            if self.end < other.start {
                std::cmp::Ordering::Less
            } else if self.start > other.end {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Equal
            }
        } else {
            self.client.cmp(&other.client)
        }
    }
}

impl From<Id> for Change {
    fn from(id: Id) -> Self {
        Change::new(id.client, id.clock, id.clock)
    }
}

impl Ord for Change {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.compare(other)
    }
}

impl PartialOrd for Change {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.compare(other))
    }
}

/// ChangeStore is a collection of changes made to a document.
/// It stores the serialized changes for sequential integration and rollback.
#[derive(Clone, Default)]
pub(crate) struct ChangeStore {
    map: BTreeMap<Change, Change>,
    changes: Vec<Change>,
}

impl ChangeStore {
    pub(crate) fn new() -> Self {
        ChangeStore {
            map: BTreeMap::new(),
            changes: Vec::new(),
        }
    }

    pub(crate) fn add_change(&mut self, change: Change) {
        self.changes.push(change.clone());
        self.map.insert(change.id(), change.clone());
    }
}
