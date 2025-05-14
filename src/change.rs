use crate::bimapid::ClientId;
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::delete::DeleteItem;
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::frontier::ChangeFrontier;
use crate::id::{IdRange, WithId};
use crate::store::{ClientStore, DeleteItemStore, ItemDataStore, ItemStore};
use crate::{ClockTick, Content, Id, ItemData};
use btree_slab::BTreeMap;
use hashbrown::hash_map::Iter;
use hashbrown::{HashMap, HashSet};
use serde::ser::SerializeStruct;
use serde::Serialize;
use std::collections::{BTreeSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hasher;
use std::ops::Range;

/// ChangeData represents a set of changes made to a document by one client in a single transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct Change {
    pub(crate) id: ChangeId,
    pub(crate) items: ItemDataStore,
    pub(crate) delete: DeleteItemStore,
    pub(crate) deps: Vec<IdRange>,
}

impl Change {
    pub(crate) fn new(id: ChangeId, items: ItemDataStore, delete: DeleteItemStore) -> Change {
        let mut deps = Vec::new();
        for (_, store) in items.iter() {
            for (_, item) in store.iter() {
                deps.extend(item.deps());
            }
        }

        for (_, store) in delete.iter() {
            for (_, item) in store.iter() {
                deps.extend(item.deps());
            }
        }

        Change {
            id,
            items,
            delete,
            deps,
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct PendingChangeStore {
    pub(crate) pending: HashMap<ClientId, VecDeque<Change>>,
    pub(crate) heads: HashMap<ClientId, Change>,
}

impl PendingChangeStore {
    pub(crate) fn insert(&mut self, change: Change) {
        self.pending
            .entry(change.id.client)
            .or_insert_with(VecDeque::new)
            .push_back(change);
    }

    pub(crate) fn iter(&self) -> Iter<'_, ClientId, Change> {
        self.heads.iter()
    }

    /// remove the change from the head and insert the first change from pending to the heads
    pub(crate) fn progress(&mut self, client_id: ClientId) {
        let change = self
            .pending
            .get_mut(&client_id)
            .and_then(|queue| queue.pop_front());
        if let Some(change) = change {
            self.heads.insert(client_id, change.clone());
        } else {
            self.heads.remove(&client_id);
        }
    }
}

/// Change represents a set of consecutive items inserted (insert, delete, move etc.) into the document by a client.
/// One change includes a range of clock ticks associated with the items within a change.
/// In context of an editor like carbon, a change is equivalent to a single editor transaction.
/// The change clock ticks are inclusive, meaning that the start clock tick is included in the change and the end clock tick is not.
/// Change{ client, [start, end] }
#[derive(Debug, Clone, Default, Eq, PartialEq, Hash)]
pub(crate) struct ChangeId {
    pub(crate) client: ClientId,
    pub(crate) start: ClockTick,
    pub(crate) end: ClockTick,
}

impl ChangeId {
    pub fn new(client: ClientId, start: ClockTick, end: ClockTick) -> Self {
        ChangeId { client, start, end }
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

impl From<Id> for ChangeId {
    fn from(id: Id) -> Self {
        ChangeId::new(id.client, id.clock, id.clock)
    }
}

impl From<IdRange> for ChangeId {
    fn from(id: IdRange) -> Self {
        ChangeId::new(id.client, id.start, id.end)
    }
}

impl Ord for ChangeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.compare(other)
    }
}

impl PartialOrd for ChangeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.compare(other))
    }
}

impl WithId for ChangeId {
    fn id(&self) -> Id {
        Id::new(self.client, self.start)
    }
}

impl Encode for ChangeId {
    #[inline]
    fn encode<T: Encoder>(&self, e: &mut T, ctx: &mut EncodeContext) {
        e.u32(self.client);
        e.u32(self.start);
        e.u32(self.end);
    }
}

impl Decode for ChangeId {
    fn decode<D: Decoder>(d: &mut D, ctx: &DecodeContext) -> Result<ChangeId, String> {
        let client = d.u32()?;
        let start = d.u32()?;
        let end = d.u32()?;

        Ok(ChangeId::new(client, start, end))
    }
}

impl Serialize for ChangeId {
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
pub(crate) type ChangeStore = ClientStore<ChangeId>;

impl ChangeStore {
    /// find all previous changes for a given dependencies
    pub(crate) fn previous(&self, change: &Vec<Id>) -> HashSet<ChangeId> {
        let mut result = HashSet::new();
        for id in change {
            if let Some(c) = self.find(id) {
                result.insert(c);
            }
        }

        result
    }

    /// The most recent change for all clients
    pub(crate) fn change_frontier(&self) {
        let mut frontier = ChangeFrontier::default();
        for (_, store) in self.iter() {
            if let Some((_, change)) = store.iter().last() {
                frontier.insert(change.clone());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::delete::DeleteItem;
    use crate::id::Id;
    use crate::store::{DeleteItemStore, ItemStore};
    use crate::Type::Atom;

    #[test]
    fn test_find_change_by_item_id() {
        let mut cs = ChangeStore::default();
        cs.insert(ChangeId::new(1, 0, 1)); // [0,1]
        cs.insert(ChangeId::new(1, 2, 3)); // [1,2]
        cs.insert(ChangeId::new(1, 4, 4)); // [1,2]

        // if the change is in the store, it should return the change
        assert_eq!(cs.find(&Id::new(1, 0)), Some(ChangeId::new(1, 0, 1)),);
        assert_eq!(cs.find(&Id::new(1, 2)), Some(ChangeId::new(1, 2, 3)),);
        assert_eq!(cs.find(&Id::new(1, 4)), Some(ChangeId::new(1, 4, 4)),);
        assert_eq!(cs.find(&Id::new(1, 5)), None);
    }

    #[test]
    fn test_find_previous_changes_by_item_ids() {
        let mut cs = ChangeStore::default();
        cs.insert(ChangeId::new(1, 0, 1)); // [0,1]
        cs.insert(ChangeId::new(1, 2, 3)); // [1,2]
        cs.insert(ChangeId::new(1, 4, 4)); // [1,2]

        let changes = cs.previous(&vec![Id::new(1, 0), Id::new(1, 2), Id::new(1, 4)]);
        assert_eq!(changes.len(), 3);
        assert!(changes.contains(&ChangeId::new(1, 0, 1)));
        assert!(changes.contains(&ChangeId::new(1, 2, 3)));
        assert!(changes.contains(&ChangeId::new(1, 4, 4)));
    }

    #[test]
    fn test_find_items_by_change() {
        let mut items = DeleteItemStore::default();
        items.insert(DeleteItem::new(Id::new(1, 1), IdRange::new(1, 0, 0)));
        items.insert(DeleteItem::new(Id::new(1, 3), IdRange::new(1, 2, 2)));
        items.insert(DeleteItem::new(Id::new(1, 5), IdRange::new(1, 4, 4)));
        items.insert(DeleteItem::new(Id::new(1, 7), IdRange::new(2, 6, 6)));

        let found = items.find_by_range(ChangeId::new(1, 0, 5));
        assert_eq!(found.len(), 3);

        let found = items.find_by_range(ChangeId::new(1, 6, 7));
        assert_eq!(found.len(), 1);
    }
}
