use crate::bimapid::ClientId;
use crate::dag::ChangeDag;
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::delete::DeleteItem;
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::frontier::ChangeFrontier;
use crate::id::{IdRange, WithId};
use crate::store::{
    ClientStore, DeleteItemStore, ItemDataStore, ItemStore, TypeStore, WeakStoreRef,
};
use crate::{ClientState, ClockTick, Content, Id, ItemData, Type};
use btree_slab::BTreeMap;
use hashbrown::hash_map::Iter;
use hashbrown::{HashMap, HashSet};
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde_columnar::Itertools;
use std::collections::{BTreeSet, VecDeque};
use std::default::Default;
use std::fmt::Debug;
use std::hash::Hasher;
use std::mem::swap;
use std::ops::Range;

/// Change represents a set of changes made to a document by one client in a single transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct Change {
    pub(crate) id: ChangeId,
    pub(crate) items: Vec<ItemData>,
    pub(crate) deletes: Vec<DeleteItem>,
    pub(crate) deps: Vec<ChangeId>,
}

impl Change {
    pub(crate) fn new(
        id: ChangeId,
        items: Vec<ItemData>,
        delete: Vec<DeleteItem>,
        deps: Vec<ChangeId>,
    ) -> Change {
        Change {
            id,
            items,
            deletes: delete,
            deps,
        }
    }

    pub(crate) fn from_id(id: ChangeId) -> Change {
        Change {
            id,
            items: Vec::new(),
            deletes: Vec::new(),
            deps: Vec::new(),
        }
    }

    pub(crate) fn with_deps(id: ChangeId, deps: Vec<ChangeId>) -> Change {
        Change {
            id,
            deps,
            ..Self::default()
        }
    }

    pub(crate) fn add_item(&mut self, item: ItemData) {
        self.items.push(item);
    }

    pub(crate) fn add_delete(&mut self, item: DeleteItem) {
        self.deletes.push(item);
    }

    // apply the changes to the document through the store
    pub(crate) fn try_apply(&mut self, store: WeakStoreRef) -> Result<(), String> {
        Ok(())
    }
}

/// ChangeData represents a set of changes made to a document by one client in a single transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct ChangeData {
    pub(crate) id: ChangeId,
    pub(crate) items: Vec<ItemData>,
    pub(crate) delete: Vec<DeleteItem>,
    pub(crate) deps: Vec<Id>,
}

impl ChangeData {
    pub(crate) fn new(id: ChangeId, items: Vec<ItemData>, delete: Vec<DeleteItem>) -> ChangeData {
        let mut deps = Vec::new();
        for item in items.iter() {
            deps.extend(item.deps());
        }

        for item in delete.iter() {
            deps.push(item.target());
        }

        ChangeData {
            id,
            items,
            delete,
            deps,
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct ChangeDeps {
    pub(crate) id: ChangeId,
    pub(crate) deps: Vec<Id>,
}

impl From<ChangeData> for ChangeDeps {
    fn from(change: ChangeData) -> Self {
        ChangeDeps {
            id: change.id,
            deps: change.deps,
        }
    }
}
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct PendingChangeStore {
    // the pending changes for each client
    pub(crate) pending: HashMap<ClientId, VecDeque<ChangeData>>,
    // the first change for each client
    pub(crate) heads: HashMap<ClientId, ChangeData>,
}

impl PendingChangeStore {
    /// find the first ready change for a client
    pub(crate) fn find_ready(&mut self, dag: &ChangeDag) -> Option<ChangeData> {
        let found = self
            .heads
            .iter()
            .find(|(_, change)| change.deps.iter().all(|id| dag.contains(id)));

        // println!("dag changes {:?}", dag.changes);
        // println!(
        //     "deps {:?}",
        //     self.heads
        //         .iter()
        //         .map(|(_, change)| change.deps.clone())
        //         .collect::<Vec<_>>()
        // );

        found.map(|(_, change)| change.clone())
    }
}

impl PendingChangeStore {
    pub(crate) fn add(&mut self, change: ChangeData) {
        // if the head is empty for this client, insert the change
        if self.heads.get(&change.id.client).is_none() {
            self.heads.insert(change.id.client, change.clone());
        }

        self.pending
            .entry(change.id.client)
            .or_insert_with(VecDeque::new)
            .push_back(change);
    }

    // return all header change ids
    pub(crate) fn change_heads(&mut self) -> &mut HashMap<ClientId, ChangeData> {
        &mut self.heads
    }

    pub(crate) fn iter(&self) -> Iter<'_, ClientId, ChangeData> {
        self.heads.iter()
    }

    /// remove the change from the head and insert the first change from pending to the heads
    pub(crate) fn progress(&mut self, client_id: &ClientId) -> Option<ChangeData> {
        let change = self
            .pending
            .get_mut(&client_id)
            .and_then(|queue| queue.pop_front());

        let head = self
            .pending
            .get(&client_id)
            .and_then(|queue| queue.front())
            .cloned();
        if let Some(head) = head {
            self.heads.insert(client_id.clone(), head.clone());
        } else {
            self.heads.remove(&client_id);
        }

        change
    }

    pub(crate) fn is_empty(&self) -> bool {
        // get sum all the pending changes
        let mut sum = 0;
        for queue in self.pending.values() {
            sum += queue.len();
        }

        sum == 0
    }
}

/// Change represents a set of consecutive items inserted (insert, delete, move etc.) into the document by a client.
/// One change includes a range of clock ticks associated with the items within a change.
/// In context of an editor like carbon, a change is equivalent to a single editor transaction.
/// The change clock ticks are inclusive, meaning that the start clock tick is included in the change and the end clock tick is not.
/// Change{ client, [start, end] }
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Hash)]
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

impl From<&Id> for ChangeId {
    fn from(id: &Id) -> Self {
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
        serializer.serialize_str(&format!(
            "ChangeId({}, {}, {})",
            self.client, self.start, self.end
        ))
    }
}

// TODO: use bitmap based change id store for smaller memory footprint in disk
/// ChangeStore is a store for changes made to a document.
pub(crate) type ChangeStore = ClientStore<ChangeId>;

impl ChangeStore {
    /// find all previous changes for a given dependencies
    pub(crate) fn deps(&self, change: &Vec<Id>) -> HashSet<ChangeId> {
        let mut deps = HashSet::new();
        for id in change {
            if let Some(c) = self.find(id) {
                deps.insert(c.clone());
            }
        }

        deps
    }

    /// The most recent change for all clients
    pub(crate) fn change_frontier(&self) -> ChangeFrontier {
        let mut frontier = ChangeFrontier::default();
        for (_, store) in self.iter() {
            if let Some((_, change)) = store.iter().last() {
                frontier.insert(change.clone());
            }
        }

        frontier
    }

    pub(crate) fn hash_set(&self) -> HashSet<ChangeId> {
        let mut set = HashSet::new();
        for (_, store) in self.iter() {
            for (_, change_id) in store.iter() {
                set.insert(change_id.clone());
            }
        }
        set
    }

    pub(crate) fn diff(&self, state: &ClientState) -> ChangeStore {
        let mut diff = ChangeStore::default();

        for (client, store) in self.items.iter() {
            let client_tick = state.get(client).unwrap_or_else(|| &0);
            let change_store = diff.store(client);
            store.iter().for_each(|(id, change_id)| {
                if change_id.start > *client_tick {
                    change_store.insert(change_id.clone());
                }
            })
        }

        diff
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
    fn test_find_dependency_changes_by_item_ids() {
        let mut cs = ChangeStore::default();
        cs.insert(ChangeId::new(1, 0, 1)); // [0,1]
        cs.insert(ChangeId::new(1, 2, 3)); // [1,2]
        cs.insert(ChangeId::new(1, 4, 4)); // [1,2]

        let changes = cs.deps(&vec![Id::new(1, 0), Id::new(1, 2), Id::new(1, 4)]);
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
