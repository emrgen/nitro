use crate::bimapid::{ClientId, ClientMap, ClientMapper};
use crate::dag::{ChangeDag, ChangeNodeFlags};
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::delete::DeleteItem;
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::hash::calculate_hash;
use crate::id::{IdComp, IdRange, WithId};
use crate::item::ItemKind;
use crate::store::{
    ClientStore, DeleteItemStore, ItemDataStore, ItemStore, TypeStore, WeakStoreRef,
};
use crate::{Client, ClientState, ClockTick, Content, Id, ItemData, Type};
use btree_slab::BTreeMap;
use hashbrown::hash_map::Iter;
use hashbrown::{HashMap, HashSet};
use queues::Queue;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde_columnar::Itertools;
use std::cmp::Ordering;
use std::collections::{BTreeSet, VecDeque};
use std::default::Default;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::mem::swap;
use std::ops::Range;

pub(crate) fn sort_changes(parents: HashMap<ChangeId, Vec<ChangeId>>) -> VecDeque<ChangeId> {
    let mut ready = VecDeque::new();

    ready
}

/// Change represents a set of changes made to a document by one client in a single transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct Change {
    pub(crate) id: ChangeId,
    pub(crate) flag: u8,
    // if the change contains any move operations
    pub(crate) items: Vec<ItemData>,
    pub(crate) deletes: Vec<DeleteItem>,
    pub(crate) deps: Vec<ChangeId>,
}

impl Change {
    pub(crate) fn new(
        id: ChangeId,
        items: Vec<ItemData>,
        deletes: Vec<DeleteItem>,
        deps: Vec<ChangeId>,
    ) -> Change {
        Change {
            id,
            flag: 0,
            items,
            deletes,
            deps,
        }
    }

    pub(crate) fn from_id(id: ChangeId) -> Change {
        Change {
            id,
            ..Self::default()
        }
    }

    pub(crate) fn with_deps(id: ChangeId, deps: Vec<ChangeId>) -> Change {
        Change {
            id,
            deps,
            ..Self::default()
        }
    }

    pub(crate) fn moves(&self) -> Vec<ItemData> {
        self.items
            .iter()
            .filter(|item| item.kind == ItemKind::Move)
            .cloned()
            .collect()
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
#[derive(Debug, Clone, Default)]
pub(crate) struct ChangeData {
    pub(crate) id: ChangeId,
    pub(crate) flag: u8,
    pub(crate) items: Vec<ItemData>,
    pub(crate) delete: Vec<DeleteItem>,
    pub(crate) deps: Vec<Id>,
}

impl ChangeData {
    // / Create an empty ChangeData with the given id.
    pub(crate) fn empty(id: ChangeId) -> ChangeData {
        ChangeData {
            id,
            flag: 0,
            items: Vec::new(),
            delete: Vec::new(),
            deps: Vec::new(),
        }
    }

    /// Create a new ChangeData with the given id, items, and delete items.
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
            flag: 0,
            items,
            delete,
            deps,
        }
    }

    pub(crate) fn has_mover(&self) -> bool {
        self.flag & ChangeNodeFlags::MOVE.bits() != 0
    }

    pub(crate) fn with_mover(mut self, moves: bool) -> Self {
        self.flag |= ChangeNodeFlags::MOVE.bits();

        self
    }
}

impl Eq for ChangeData {}

impl PartialEq for ChangeData {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Hash for ChangeData {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
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

    pub(crate) fn to_client_change_id<T: ClientMapper>(
        &self,
        client_map: &T,
    ) -> Option<ClientChangeId> {
        client_map
            .get_client(&self.client)
            .cloned()
            .map(|client| ClientChangeId::new(client, self.start, self.end))
    }

    pub(crate) fn range(&self) -> Range<ClockTick> {
        self.start..self.end
    }

    #[inline]
    pub(crate) fn comp(&self, other: &Self) -> Ordering {
        if self.client == other.client {
            if self.end < other.start {
                Ordering::Less
            } else if self.start > other.end {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        } else {
            self.client.cmp(&other.client)
        }
    }

    pub(crate) fn contains(&self, id: &Id) -> bool {
        self.client == id.client && self.start <= id.clock && id.clock <= self.end
    }
}

#[derive(Debug, Clone, Default, Hash)]
pub(crate) struct ClientChangeId {
    pub(crate) client: Client,
    pub(crate) start: ClockTick,
    pub(crate) end: ClockTick,
}

impl ClientChangeId {
    pub(crate) fn new(client: Client, start: ClockTick, end: ClockTick) -> Self {
        ClientChangeId { client, start, end }
    }

    pub(crate) fn range(&self) -> Range<ClockTick> {
        self.start..self.end
    }
}

impl PartialEq<Self> for ClientChangeId {
    fn eq(&self, other: &Self) -> bool {
        self.client == other.client && self.start == other.start
    }
}
impl Eq for ClientChangeId {}

impl PartialOrd<Self> for ClientChangeId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ClientChangeId {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.client == other.client {
            if self.end < other.start {
                (Ordering::Less)
            } else if self.start > other.end {
                (Ordering::Greater)
            } else {
                (Ordering::Equal)
            }
        } else {
            let left = calculate_hash(&format!("{}{}", self.client, self.start));
            let right = calculate_hash(&format!("{}{}", other.client, other.start));
            left.cmp(&right)
        }
    }
}

impl ClientChangeId {
    pub(crate) fn to_change_id<T: ClientMapper>(&self, client_map: &T) -> Option<ChangeId> {
        client_map
            .get_client_id(&self.client)
            .cloned()
            .map(|client_id| ChangeId::new(client_id, self.start, self.end))
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

impl IdComp for ChangeId {
    fn comp_id(&self, other: &Id) -> std::cmp::Ordering {
        if self.client == other.client {
            if other.clock < self.start {
                std::cmp::Ordering::Greater
            } else if self.end < other.clock {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        } else {
            self.client.cmp(&other.client)
        }
    }
}

impl From<IdRange> for ChangeId {
    fn from(id: IdRange) -> Self {
        ChangeId::new(id.client, id.start, id.end)
    }
}

impl Ord for ChangeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.comp(other)
    }
}

impl PartialOrd for ChangeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.comp(other))
    }
}

impl WithId for ChangeId {
    #[inline]
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
/// ChangeStoreX is a store for changes made to a document.
pub(crate) type ChangeStore = ClientStore<ChangeId>;

impl ChangeStore {
    /// find all previous changes for a given dependencies
    pub(crate) fn deps(&self, change: &Vec<Id>) -> HashSet<ChangeId> {
        let mut deps = HashSet::new();
        for id in change {
            if let Some(c) = self.get(id) {
                deps.insert(c.clone());
            }
        }

        deps
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
        assert_eq!(cs.get(&Id::new(1, 0)), Some(ChangeId::new(1, 0, 1)),);
        assert_eq!(cs.get(&Id::new(1, 2)), Some(ChangeId::new(1, 2, 3)),);
        assert_eq!(cs.get(&Id::new(1, 4)), Some(ChangeId::new(1, 4, 4)),);
        assert_eq!(cs.get(&Id::new(1, 5)), None);
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
