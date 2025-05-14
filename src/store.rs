use crate::bimapid::{ClientId, Field, FieldId, FieldMap};
use crate::change::{ChangeId, ChangeStore};
use crate::dag::ChangeDag;
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::delete::DeleteItem;
use crate::diff::Diff;
use crate::doc::DocId;
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::frontier::Frontier;
use crate::id::{ClockTick, Id, IdRange, Split, WithId, WithIdRange};
use crate::id_store::ClientIdStore;
use crate::item::{ItemData, ItemKind, ItemRef};
use crate::state::ClientState;
use crate::types::Type;
use crate::{print_yaml, Client};
use bimap::BiMap;
use hashbrown::HashMap;
use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};
use std::cell::RefCell;
use std::collections::btree_map::IterMut;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt::Debug;
use std::ops::Add;
use std::rc::{Rc, Weak};

pub(crate) type StoreRef = Rc<RefCell<DocStore>>;
pub(crate) type WeakStoreRef = Weak<RefCell<DocStore>>;

/// DocStore is a store for the document CRDT items and metadata.
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub(crate) struct DocStore {
    pub(crate) doc_id: DocId,
    pub(crate) created_by: Client,

    pub(crate) client: ClientId,
    pub(crate) clock: ClockTick,

    pub(crate) fields: FieldMap,
    pub(crate) id_map: IdRangeMap,
    pub(crate) state: ClientState,

    pub(crate) moves: HashMap<Id, Vec<Type>>,
    pub(crate) proxies: HashMap<Id, Vec<Type>>,

    pub(crate) items: TypeStore,
    pub(crate) deleted_items: DeleteItemStore,
    pub(crate) pending: PendingStore,
    // ready store is used during time travel to past
    pub(crate) ready: ReadyStore,

    pub(crate) changes: ChangeStore,
    pub(crate) dag: ChangeDag,
}

impl DocStore {
    pub(crate) fn add_mover(&mut self, target_id: Id, mover: Type) {
        let entry = self.moves.entry(target_id).or_default();
        // mark the last mover as moved so that it will be treated as an invisible item
        if entry.len() > 0 {
            entry.last().unwrap().item_ref().mark_moved();
        }
        entry.push(mover);
    }

    /// remove the last mover for the given target id
    /// the last mover after remove is marked as unmoved
    pub(crate) fn remove_mover(&mut self, target_id: Id, mover: &Type) {
        let mover_id = mover.id();
        self.moves.entry(target_id).and_modify(|v| {
            v.retain(|x| x.id() != mover_id);
            if v.len() > 0 {
                v.last().unwrap().item_ref().unmark_moved();
            }
            // TODO: empty vectors should be removed
        });

        if self.moves.get(&target_id).map_or(false, |v| v.is_empty()) {
            self.find(&target_id).map(|target| {
                target.item_ref().unmark_moved();
            });
        }
    }

    pub(crate) fn get_move(&mut self, id: &Id) -> Option<Type> {
        self.moves.get(id).and_then(|v| v.last()).cloned()
    }

    pub(crate) fn add_proxy(&mut self, target_id: Id, proxy: Type) {
        self.proxies.entry(target_id).or_default().push(proxy);
    }

    pub(crate) fn remove_proxy(&mut self, id: &Id, proxy: &Type) {
        let proxy_id = proxy.id();
        self.proxies.entry(*id).and_modify(|v| {
            v.retain(|x| x.id() != proxy_id);
            // TODO: empty vectors should be removed
        });
    }

    pub(crate) fn get_proxies(&mut self, id: &Id) -> Option<Vec<Type>> {
        self.proxies.get(id).cloned()
    }

    pub(crate) fn get_field_id(&mut self, field: &Field) -> u32 {
        self.fields.get_or_insert(field)
    }
    pub(crate) fn get_field(&self, field_id: &FieldId) -> Option<&Field> {
        self.fields.get_field(field_id)
    }

    pub(crate) fn get_client(&mut self, client_id: &Client) -> ClientId {
        self.state.clients.get_or_insert(client_id)
    }

    pub(crate) fn update_client(&mut self, client: &Client, clock: ClockTick) -> ClientId {
        self.client = self.state.clients.get_or_insert(client);
        self.clock = clock.max(1);

        self.client
    }

    pub(crate) fn next_id(&mut self) -> Id {
        let id = Id::new(self.client, self.clock);
        self.clock += 1;

        id
    }

    pub(crate) fn next_id_range(&mut self, size: ClockTick) -> IdRange {
        let id = IdRange::new(self.client, self.clock, self.clock + size - 1);
        self.clock += size;

        id
    }

    #[inline]
    pub(crate) fn contains(&self, id: &Id) -> bool {
        self.items.find(id).is_some()
    }

    #[inline]
    pub(crate) fn find(&self, id: &Id) -> Option<Type> {
        let key = self.id_map.find(id);
        self.items.find(&key)
    }

    pub(crate) fn insert(&mut self, item: impl Into<Type>) {
        let item = item.into();
        if item.kind() == ItemKind::String {
            self.id_map.insert(item.id().range(item.size()));
        }

        let id = item.id();
        self.items.insert(item);

        self.state.update(id.client, id.clock);
    }

    pub(crate) fn remove(&mut self, id: &Id) {
        self.items.remove(id);
    }

    pub(crate) fn insert_delete(&mut self, item: DeleteItem) -> &mut DocStore {
        self.deleted_items.insert(item);

        self
    }

    pub(crate) fn replace(&mut self, item: &Type, items: (Type, Type)) -> &mut DocStore {
        self.items.replace(item, items);

        self
    }

    pub(crate) fn client(&mut self, client_id: &Client) -> ClientId {
        self.state.get_or_insert(client_id).0
    }

    pub(crate) fn diff(&self, id: DocId, created_by: Client, state: ClientState) -> Diff {
        let state = state.as_per(&self.state);

        let items = self.items.diff(&state);

        let deletes = self.deleted_items.diff(&state);

        let mut clients = self.state.clients.clone();

        for (_, client_id) in clients.iter() {
            if (items.client_size(client_id) + deletes.client_size(client_id)) == 0 {
                // clients.remove(client_id);
            }
        }

        let state = state.merge(&self.state);

        let changes = self.changes.clone();

        let mut moves = self
            .items
            .iter()
            .any(|(_, store)| store.iter().any(|(_, item)| item.kind().is_move()));

        Diff::from(
            id,
            created_by,
            self.fields.clone(),
            changes,
            state,
            items,
            deletes,
            moves,
        )
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub(crate) struct ReadyStore {
    pub(crate) id_range_map: IdRangeMap,
    pub(crate) queue: VecDeque<ItemData>,
    pub(crate) items: ItemDataStore,
    pub(crate) items_exists: ClientIdStore,
    pub(crate) delete_items: DeleteItemStore,
}

impl ReadyStore {
    pub(crate) fn insert(&mut self, item: ItemData) {
        self.items_exists.insert(item.id());
        self.queue.push_back(item.clone());
        self.items.insert(item);
    }

    pub(crate) fn contains(&self, id: &Id) -> bool {
        // self.items_exists.contains(id)
        self.find_item(id).is_some()
    }

    pub(crate) fn find_item(&self, id: &Id) -> Option<ItemData> {
        let id = self.id_range_map.find(id);
        self.items.find(&id)
    }

    pub(crate) fn insert_delete(&mut self, item: DeleteItem) {
        self.delete_items.insert(item);
    }

    pub(crate) fn remove(&mut self, id: &Id) {
        self.items.remove(id);
    }

    pub(crate) fn remove_delete(&mut self, id: &Id) {
        self.delete_items.remove(id);
    }

    pub(crate) fn iter_items(
        &self,
    ) -> std::collections::btree_map::Iter<ClientId, ItemStore<ItemData>> {
        self.items.iter()
    }

    pub(crate) fn iter_delete_items(
        &self,
    ) -> std::collections::btree_map::Iter<ClientId, ItemStore<DeleteItem>> {
        self.delete_items.iter()
    }
}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub(crate) struct PendingStore {
    pub(crate) items: ItemDataStore,
    pub(crate) delete_items: DeleteItemStore,
}

impl PendingStore {
    pub(crate) fn insert(&mut self, item: ItemData) {
        self.items.insert(item);
    }

    pub(crate) fn insert_delete(&mut self, item: DeleteItem) {
        self.delete_items.insert(item);
    }

    pub(crate) fn remove(&mut self, id: &Id) {
        self.items.remove(id);
    }

    /// pop the first pending item for the given client
    pub(crate) fn pop_first(&mut self, client_id: &ClientId) -> Option<ItemData> {
        let store = self.items.id_store_mut(client_id)?;
        store.pop_first()
    }

    pub(crate) fn remove_delete(&mut self, id: &Id) {
        self.delete_items.remove(id);
    }

    pub(crate) fn iter_items(
        &self,
    ) -> std::collections::btree_map::Iter<ClientId, ItemStore<ItemData>> {
        self.items.iter()
    }

    pub(crate) fn iter_mut_items(
        &mut self,
    ) -> std::collections::btree_map::IterMut<ClientId, ItemStore<ItemData>> {
        self.items.iter_mut()
    }

    pub(crate) fn iter_delete_items(
        &self,
    ) -> std::collections::btree_map::Iter<ClientId, ItemStore<DeleteItem>> {
        self.delete_items.iter()
    }

    pub(crate) fn extend(&mut self, other: &PendingStore) {
        for (_, store) in other.items.iter() {
            for (_, item) in store.iter() {
                self.items.insert(item.clone());
            }
        }

        for (_, store) in other.delete_items.iter() {
            for (_, item) in store.iter() {
                self.delete_items.insert(item.clone());
            }
        }
    }
}

impl Encode for PendingStore {
    fn encode<T: Encoder>(&self, e: &mut T, ctx: &mut EncodeContext) {
        self.items.encode(e, ctx);
        self.delete_items.encode(e, ctx);
    }
}

impl Decode for PendingStore {
    fn decode<T: Decoder>(d: &mut T, ctx: &DecodeContext) -> Result<Self, String>
    where
        Self: Sized,
    {
        let items = ItemDataStore::decode(d, ctx)?;
        let delete_items = DeleteItemStore::decode(d, ctx)?;

        Ok(PendingStore {
            items,
            delete_items,
        })
    }
}

impl Add<PendingStore> for PendingStore {
    type Output = PendingStore;

    fn add(self, rhs: PendingStore) -> Self::Output {
        let mut items = self.items.clone();
        let mut delete_items = self.delete_items.clone();

        for (_, store) in rhs.items.iter() {
            for (_, item) in store.iter() {
                items.insert(item.clone());
            }
        }

        for (_, store) in rhs.delete_items.iter() {
            for (_, item) in store.iter() {
                delete_items.insert(item.clone());
            }
        }

        PendingStore {
            items,
            delete_items,
        }
    }
}

pub type ItemDataStore = ClientStore<ItemData>;

impl From<TypeStore> for ItemDataStore {
    fn from(value: TypeStore) -> Self {
        let mut store = ItemDataStore::default();
        for (_, items) in value.items.iter() {
            for (_, item) in items.iter() {
                store.insert(item.item_ref().borrow().data.clone());
            }
        }

        store
    }
}

pub type DeleteItemStore = ClientStore<DeleteItem>;
pub type TypeStore = ClientStore<Type>;

impl TypeStore {
    pub(crate) fn frontier(&self) -> Frontier {
        let mut frontier = Frontier::default();
        for (_, store) in self.items.iter() {
            if let Some((_, item)) = store.iter().last() {
                frontier.add(item.range().end_id());
            }
        }

        frontier
    }
}

/// A map of client id to a set of id ranges
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub(crate) struct IdRangeMap {
    // TODO: check if the clientId is needed in IdRange
    // as it is already in the BTreeMap key may be we can remove it from the IdRange
    pub(crate) map: BTreeMap<ClientId, BTreeSet<IdRange>>,
}

impl IdRangeMap {
    pub(crate) fn find(&self, id: &Id) -> Id {
        let e = self
            .map
            .get(&id.client)
            .and_then(|set| set.get(&id.range(1)));
        if let Some(range) = e {
            range.id()
        } else {
            id.clone()
        }
    }
}

impl IdRangeMap {
    pub(crate) fn insert(&mut self, id: IdRange) {
        let set = self.map.entry(id.client).or_default();
        set.insert(id);
    }

    pub(crate) fn has(&self, id: &Id) -> bool {
        self.map
            .get(&id.client)
            .map(|set| set.contains(&id.range(1)))
            .unwrap_or(false)
    }

    pub(crate) fn remove(&mut self, id: &Id) {
        if let Some(set) = self.map.get_mut(&id.client) {
            set.remove(&id.range(1));
        }
    }

    pub(crate) fn get(&self, id: &Id) -> Option<&IdRange> {
        self.map
            .get(&id.client)
            .and_then(|set| set.get(&id.range(1)))
    }

    pub(crate) fn replace(&mut self, id: IdRange, with: (IdRange, IdRange)) {
        let set = self.map.entry(id.client).or_default();
        set.remove(&id);
        set.insert(with.0);
        set.insert(with.1);
    }
}

impl IdDiff for DeleteItemStore {
    type Target = DeleteItemStore;

    fn diff(&self, state: &ClientState) -> DeleteItemStore {
        let mut diff = DeleteItemStore::default();

        for (client, store) in self.items.iter() {
            let clock = state.get(client).unwrap_or(&0);
            let items = store.diff(*clock);
            if items.size() > 0 {
                diff.items.insert(*client, items);
            }
        }

        diff
    }
}

impl IdDiff for TypeStore {
    type Target = ItemDataStore;

    fn diff(&self, state: &ClientState) -> ItemDataStore {
        let mut diff = ItemDataStore::default();

        for (client, store) in self.items.iter() {
            let clock = state.get(client).unwrap_or(&0);
            let items = store.diff(*clock);
            if items.size() > 0 {
                diff.items.insert(*client, items);
            }
        }

        diff
    }
}
impl IdDiff for ItemDataStore {
    type Target = ItemDataStore;

    fn diff(&self, state: &ClientState) -> ItemDataStore {
        let mut diff = ItemDataStore::default();

        for (client, store) in self.items.iter() {
            let clock = state.get(client).unwrap_or(&0);
            let items = store.diff(*clock);
            if items.size() > 0 {
                diff.items.insert(*client, items);
            }
        }

        diff
    }
}

pub(crate) trait IdDiff {
    type Target;
    fn diff(&self, state: &ClientState) -> Self::Target;
}

pub(crate) trait ClientStoreEntry:
    Default + WithId + Clone + Encode + Decode + Eq + PartialEq + Serialize
{
}

// blanket implementation for all types that implement dependencies
impl<T: Default + WithId + Clone + Encode + Decode + Eq + PartialEq + Serialize> ClientStoreEntry
    for T
{
}

/// A map of items created by a client at local site
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct ClientStore<T: ClientStoreEntry> {
    pub(crate) items: BTreeMap<ClientId, ItemStore<T>>,
}

impl<T: ClientStoreEntry> ClientStore<T> {
    pub(crate) fn clients(&self) -> Vec<ClientId> {
        self.items.keys().cloned().collect()
    }

    pub(crate) fn id_store(&self, client: &ClientId) -> Option<&ItemStore<T>> {
        self.items.get(client)
    }

    pub(crate) fn id_store_mut(&mut self, client: &ClientId) -> Option<&mut ItemStore<T>> {
        self.items.get_mut(client)
    }

    pub(crate) fn size(&self) -> u32 {
        self.iter().map(|(_, store)| store.size() as u32).sum()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.items.is_empty() || self.items.iter().all(|(_, store)| store.is_empty())
    }
}

impl<T: ClientStoreEntry> ClientStore<T> {
    pub(crate) fn contains(&self, id: &Id) -> bool {
        self.items
            .get(&id.client)
            .map(|store| store.contains(id))
            .unwrap_or(false)
    }

    pub(crate) fn clear(&mut self) {
        self.items.clear();
    }
}

impl<T: ClientStoreEntry> ClientStore<T> {
    /// get the number of items for the given client
    pub(crate) fn client_size(&self, id: &ClientId) -> usize {
        self.items.get(id).map(|p1| p1.size()).unwrap_or(0)
    }

    pub(crate) fn insert(&mut self, item: T) {
        let id = item.id();
        let store = self.items.entry(id.client).or_default();
        store.insert(item);
    }

    /// get the item for the given id
    pub(crate) fn find(&self, id: &Id) -> Option<T> {
        self.items.get(&id.client).and_then(|store| store.get(&id))
    }

    /// get items in the inclusive clock range [start, end] for the given client
    pub(crate) fn find_by_range(&self, range: impl Into<IdRange>) -> Vec<T> {
        let range = range.into();
        self.items
            .get(&range.client)
            .map(|store| store.get_range(&range))
            .unwrap_or_default()
    }

    pub(crate) fn last(&self) -> Option<(ClientId, ItemStore<T>)> {
        self.iter().last().map(|(k, v)| (*k, v.clone()))
    }

    pub(crate) fn keys(&self) -> std::collections::btree_map::Keys<ClientId, ItemStore<T>> {
        self.items.keys()
    }

    pub(crate) fn iter(&self) -> std::collections::btree_map::Iter<ClientId, ItemStore<T>> {
        self.items.iter()
    }

    /// get the last item for the given client

    pub(crate) fn iter_mut(&mut self) -> IterMut<ClientId, ItemStore<T>> {
        self.items.iter_mut()
    }

    pub(crate) fn remove(&mut self, id: &Id) {
        if let Some(store) = self.items.get_mut(&id.client) {
            store.remove(id);
        }
    }

    pub(crate) fn replace(&mut self, item: &T, items: (T, T)) {
        let id = item.id();
        let store = self.items.get_mut(&id.client).unwrap();

        store.remove(&item.id());

        store.insert(items.0);
        store.insert(items.1);
    }

    pub(crate) fn merge(&self, other: &ClientStore<T>) -> ClientStore<T> {
        let mut store = self.clone();
        for (client, items) in other.items.iter() {
            let store = store.items.entry(*client).or_default();
            for (_, item) in items.iter() {
                store.insert(item.clone());
            }
        }

        store
    }
}

impl<T: ClientStoreEntry> std::iter::IntoIterator for ClientStore<T> {
    type Item = (ClientId, ItemStore<T>);
    type IntoIter = std::collections::btree_map::IntoIter<ClientId, ItemStore<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<T: ClientStoreEntry> Serialize for ClientStore<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.items.serialize(serializer)
    }
}

impl<T: ClientStoreEntry> Encode for ClientStore<T> {
    fn encode<E: Encoder>(&self, e: &mut E, cx: &mut EncodeContext) {
        e.u32(self.items.len() as u32);
        for (client, store) in self.items.iter() {
            e.u32(*client);
            store.encode(e, cx);
        }
    }
}

impl<T: ClientStoreEntry> Decode for ClientStore<T> {
    fn decode<D: Decoder>(d: &mut D, cx: &DecodeContext) -> Result<ClientStore<T>, String> {
        let len = d.u32()? as usize;
        let mut items = BTreeMap::new();
        for _ in 0..len {
            let client = d.u32()?;
            let store = ItemStore::decode(d, cx)?;
            items.insert(client, store);
        }

        Ok(ClientStore { items })
    }
}

pub(crate) trait ItemStoreEntry:
    WithId + Clone + Encode + Decode + Eq + PartialEq + Serialize
{
}

// blanket implementation for all types that implement dependencies
impl<T: WithId + Clone + Encode + Decode + Eq + PartialEq + Serialize> ItemStoreEntry for T {}

/// A map of ids to items by item id
#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct ItemStore<T: ItemStoreEntry> {
    map: BTreeMap<Id, T>,
}

impl<T: ItemStoreEntry> ItemStore<T> {
    pub(crate) fn insert(&mut self, value: T) {
        self.map.insert(value.id(), value);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub(crate) fn get(&self, value: &Id) -> Option<T> {
        self.map.get(value).cloned()
    }

    // get items in the inclusive clock range [start, end]
    pub(crate) fn get_range(&self, range: &IdRange) -> Vec<T> {
        let start = Id::new(range.client, range.start);
        let end = Id::new(range.client, range.end + 1);
        self.map
            .range(start..=end)
            .map(|(_, v)| v.clone())
            .collect()
    }

    pub(crate) fn remove(&mut self, value: &Id) -> Option<T> {
        self.map.remove(value)
    }

    pub(crate) fn contains(&self, value: &Id) -> bool {
        self.map.contains_key(value)
    }

    pub(crate) fn size(&self) -> usize {
        self.map.len()
    }

    pub(crate) fn pop_first(&mut self) -> Option<T> {
        self.map.pop_first().map(|(_, v)| v)
    }

    pub(crate) fn first(&self) -> Option<&T> {
        self.map.first_key_value().map(|(_, v)| v)
    }
}

impl<T: ItemStoreEntry> ItemStore<T> {
    pub(crate) fn iter_mut(&mut self) -> std::collections::btree_map::IterMut<Id, T> {
        self.map.iter_mut()
    }

    pub(crate) fn iter(&self) -> std::collections::btree_map::Iter<Id, T> {
        self.map.iter()
    }
}

impl<T: ItemStoreEntry> IntoIterator for ItemStore<T> {
    type Item = (Id, T);
    type IntoIter = std::collections::btree_map::IntoIter<Id, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<T: ItemStoreEntry> Serialize for ItemStore<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.map.serialize(serializer)
    }
}

impl<T: ItemStoreEntry> Encode for ItemStore<T> {
    fn encode<E: Encoder>(&self, e: &mut E, cx: &mut EncodeContext) {
        e.u32(self.map.len() as u32);
        for (_, value) in self.map.iter() {
            value.encode(e, cx);
        }
    }
}

impl<T: ItemStoreEntry> Decode for ItemStore<T> {
    fn decode<D: Decoder>(d: &mut D, cx: &DecodeContext) -> Result<ItemStore<T>, String> {
        let len = d.u32()? as usize;
        let mut data = BTreeMap::new();
        for _ in 0..len {
            let value = T::decode(d, cx)?;
            data.insert(value.id(), value);
        }
        Ok(ItemStore { map: data })
    }
}

pub(crate) trait IdClockDiff {
    type Target;
    fn diff(&self, clock: ClockTick) -> Self::Target;
}

impl IdClockDiff for ItemStore<ItemRef> {
    type Target = ItemStore<ItemData>;

    fn diff(&self, clock: ClockTick) -> Self::Target {
        let mut items = ItemStore::default();
        for (id, item) in self.map.iter() {
            let data = item.borrow().data.clone();
            let ticks = data.ticks();
            // collect items that are newer than the given clock
            if id.clock > clock {
                items.insert(data);
            } else if id.clock < clock && clock < id.clock + ticks {
                if let Ok((_, r)) = data.split(clock) {
                    items.insert(r);
                }
            }
        }

        items
    }
}

impl IdClockDiff for ItemStore<Type> {
    type Target = ItemStore<ItemData>;

    fn diff(&self, clock: ClockTick) -> Self::Target {
        let mut items = ItemStore::default();
        for (id, item) in self.map.iter() {
            let data = item.item_ref().borrow().data.clone();
            let ticks = data.ticks();
            // collect items that are newer than the given clock
            if id.clock > clock {
                items.insert(data);
            } else if id.clock < clock && clock < id.clock + ticks {
                if let Ok((_, r)) = data.split(clock) {
                    items.insert(r);
                }
            }
        }

        items
    }
}

impl IdClockDiff for ItemStore<ItemData> {
    type Target = ItemStore<ItemData>;

    fn diff(&self, clock: ClockTick) -> Self::Target {
        let mut items = ItemStore::default();
        for (id, data) in self.map.iter() {
            let ticks = data.ticks();
            // collect items that are newer than the given clock
            if id.clock > clock {
                items.insert(data.clone());
            } else if id.clock < clock && clock < id.clock + ticks {
                if let Ok((_, r)) = data.split(clock) {
                    items.insert(r);
                }
            }
        }

        items
    }
}

impl IdClockDiff for ItemStore<DeleteItem> {
    type Target = ItemStore<DeleteItem>;

    fn diff(&self, clock: ClockTick) -> Self::Target {
        let mut items = ItemStore::default();
        for (id, item) in self.map.iter() {
            if id.clock > clock {
                items.insert(item.clone());
            }
        }

        items
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Sub;

    use uuid::Uuid;

    use crate::codec_v1::EncoderV1;

    use super::*;

    #[test]
    fn test_id_store() {
        let mut store = ItemStore::default();
        assert!(!store.contains(&Id::new(1, 1,)));
        store.insert(Id::new(1, 1));
        assert!(store.contains(&Id::new(1, 1)));

        store.insert(Id::new(1, 5));
        assert!(store.contains(&Id::new(1, 5,)));
    }

    #[test]
    fn test_is_range_map() {
        let mut map = IdRangeMap::default();

        map.insert(Id::new(1, 1).into());
        map.insert(Id::new(1, 2).into());
        map.insert(Id::new(1, 3).range(5));
        map.insert(Id::new(1, 8).range(2));

        assert_eq!(map.get(&Id::new(1, 1)).unwrap(), &Id::new(1, 1).into());
        assert_eq!(map.get(&Id::new(1, 2)).unwrap(), &Id::new(1, 2).into());

        assert_eq!(map.get(&Id::new(1, 3)).unwrap(), &Id::new(1, 3).into());
        assert_eq!(map.get(&Id::new(1, 4)).unwrap(), &Id::new(1, 3).into());
        assert_eq!(map.get(&Id::new(1, 6)).unwrap(), &Id::new(1, 3).into());

        assert_eq!(map.get(&Id::new(1, 8)).unwrap(), &Id::new(1, 8).into());
        assert_eq!(map.get(&Id::new(1, 9)).unwrap(), &Id::new(1, 8).into());
    }

    #[test]
    fn test_encode_decode_client_id_store() {
        let mut store = ItemStore::default();
        let id1 = Id::new(1, 1);
        let id2 = Id::new(1, 2);
        let id3 = Id::new(1, 3);

        store.insert(id1);
        store.insert(id2);
        store.insert(id3);

        let mut e = EncoderV1::new();
        store.encode(&mut e, &mut EncodeContext::default());

        let mut d = e.decoder();
        let dd = ItemStore::<Id>::decode(&mut d, &DecodeContext::default()).unwrap();

        assert_eq!(store, dd);
    }

    #[test]
    fn test_encode_decode_client_store() {
        let mut store = ClientStore::default();
        let id1 = Id::new(1, 1);
        let id2 = Id::new(2, 2);
        let id3 = Id::new(3, 3);

        store.insert(id1);
        store.insert(id2);
        store.insert(id3);

        let mut e = EncoderV1::new();
        store.encode(&mut e, &mut EncodeContext::default());

        let mut d = e.decoder();
        let dd = ClientStore::<Id>::decode(&mut d, &DecodeContext::default()).unwrap();

        assert_eq!(store, dd);
    }

    impl Sub for &ClientState {
        type Output = ClientState;

        fn sub(self, rhs: Self) -> Self::Output {
            let mut clone = self.clone();

            for (client, clock) in rhs.state.iter() {
                let c = clone.state.get(client);
                if let Some(c) = c {
                    if clock > c {
                        clone.state.update_max(*client, *c);
                    } else {
                        clone.state.update_max(*client, *c - clock);
                    }
                } else {
                    clone.state.remove(client);
                }
            }

            for (client, clock) in rhs.clients.iter() {
                if !self.clients.contains_client(client) {
                    clone.clients.remove_client(client);
                }
            }

            clone
        }
    }

    impl Sub for ClientState {
        type Output = ClientState;

        fn sub(self, rhs: Self) -> Self::Output {
            &self - &rhs
        }
    }

    #[test]
    fn test_adjust_client_state() {
        let mut s1 = ClientState::default();
        let mut s2 = ClientState::default();
        let u1 = Uuid::new_v4().into();
        let u2 = Uuid::new_v4().into();
        let u3 = Uuid::new_v4().into();
        let u4 = Uuid::new_v4().into();

        let uid1 = s1.clients.get_or_insert(&u1);
        let uid2 = s1.clients.get_or_insert(&u2);
        s1.state.update_max(uid1, 1);
        s1.state.update_max(uid1, 2);
        s1.state.update_max(uid2, 1);
        s1.state.update_max(uid2, 2);

        let uid3 = s2.clients.get_or_insert(&u3);
        let uid4 = s2.clients.get_or_insert(&u4);

        s2.state.update_max(uid3, 1);
        s2.state.update_max(uid3, 2);
        s2.state.update_max(uid4, 1);

        let d1 = s1.adjust_max(&s2);
        let s12 = &s1 + &s2;

        // print_yaml(&s1);
        // print_yaml(&s2);

        // print_yaml(&s12);
        // print_yaml(&d1);

        assert_eq!(s12, d1);
    }
}
