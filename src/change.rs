use crate::bimapid::ClientId;
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::id::{IdRange, WithId};
use crate::store::ClientStore;
use crate::{ClockTick, Content, Id};
use hashbrown::HashSet;
use serde::ser::SerializeStruct;
use serde::Serialize;
use std::collections::BTreeSet;
use std::fmt::Debug;
use std::hash::Hasher;
use std::ops::Range;

/// Change represents a set of consecutive items inserted (insert, delete, move etc.) into the document by a client.
/// One change includes a range of clock ticks associated with the items within a change.
/// In context of an editor like carbon, a change is equivalent to a single editor transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub(crate) struct Change {
    pub(crate) client: ClientId,
    pub(crate) start: ClockTick,
    pub(crate) end: ClockTick,
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

impl From<IdRange> for Change {
    fn from(id: IdRange) -> Self {
        Change::new(id.client, id.start, id.end)
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

impl WithId for Change {
    fn id(&self) -> Id {
        Id::new(self.client, self.start)
    }
}

impl Encode for Change {
    #[inline]
    fn encode<T: Encoder>(&self, e: &mut T, ctx: &mut EncodeContext) {
        e.u32(self.client);
        e.u32(self.start);
        e.u32(self.end);
    }
}

impl Decode for Change {
    fn decode<D: Decoder>(d: &mut D, ctx: &DecodeContext) -> Result<Change, String> {
        let client = d.u32()?;
        let start = d.u32()?;
        let end = d.u32()?;

        Ok(Change::new(client, start, end))
    }
}

impl Serialize for Change {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut state = serializer.serialize_struct("Change", 3)?;
        state.serialize_field("client", &self.client)?;
        state.serialize_field("start", &self.start)?;
        state.serialize_field("end", &self.end)?;
        state.end()
    }
}

/// ChangeStore is a store for changes made to a document.
pub(crate) type ChangeStore = ClientStore<Change>;

impl ChangeStore {
    /// find all previous changes for a given dependencies
    pub(crate) fn previous(&self, change: &Vec<Id>) -> HashSet<Change> {
        let mut result = HashSet::new();
        for id in change {
            if let Some(c) = self.find(id) {
                result.insert(c);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::Id;

    #[test]
    fn test_find_change_by_item_id() {
        let mut cs = ChangeStore::default();
        cs.insert(Change::new(1, 0, 1)); // [0,1]
        cs.insert(Change::new(1, 2, 3)); // [1,2]
        cs.insert(Change::new(1, 4, 4)); // [1,2]

        // if the change is in the store, it should return the change
        assert_eq!(cs.find(&Id::new(1, 0)), Some(Change::new(1, 0, 1)),);
        assert_eq!(cs.find(&Id::new(1, 2)), Some(Change::new(1, 2, 3)),);
        assert_eq!(cs.find(&Id::new(1, 4)), Some(Change::new(1, 4, 4)),);
        assert_eq!(cs.find(&Id::new(1, 5)), None);
    }

    #[test]
    fn test_find_previous_changes_by_item_ids() {
        let mut cs = ChangeStore::default();
        cs.insert(Change::new(1, 0, 1)); // [0,1]
        cs.insert(Change::new(1, 2, 3)); // [1,2]
        cs.insert(Change::new(1, 4, 4)); // [1,2]

        let changes = cs.previous(&vec![Id::new(1, 0), Id::new(1, 2), Id::new(1, 4)]);
        assert_eq!(changes.len(), 3);
        assert!(changes.contains(&Change::new(1, 0, 1)));
        assert!(changes.contains(&Change::new(1, 2, 3)));
        assert!(changes.contains(&Change::new(1, 4, 4)));
    }
}
