use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::collections::btree_map::IterMut;
use std::fmt::Debug;
use std::ops::Add;
use std::rc::{Rc, Weak};

use crate::bimapid::{Client, ClientId, ClientMap, Field, FieldId, FieldMap};
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::delete::DeleteItem;
use crate::diff::Diff;
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::id::{Clock, Id, IdRange, Split, WithId};
use crate::item::{ItemData, ItemKind, ItemRef};
use crate::state::ClientState;
use crate::types::Type;

pub(crate) type StoreRef = Rc<RefCell<DocStore>>;
pub(crate) type WeakStoreRef = Weak<RefCell<DocStore>>;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub(crate) struct DocStore {
    pub(crate) client: ClientId,
    pub(crate) clock: Clock,

    pub(crate) state: ClientState,

    pub(crate) fields: FieldMap,
    pub(crate) id_map: IdRangeMap,
    pub(crate) clients: ClientMap,

    pub(crate) items: ItemStore,
    pub(crate) deleted_items: DeleteItemStore,
    pub(crate) pending: PendingStore,
}

impl DocStore {
    pub(crate) fn get_field_id(&mut self, field: &Field) -> u32 {
        self.fields.get_or_insert(field)
    }
    pub(crate) fn get_field(&self, field_id: &FieldId) -> Option<&Field> {
        self.fields.get_field(field_id)
    }

    pub(crate) fn get_client(&mut self, client_id: &Client) -> ClientId {
        self.clients.get_or_insert(client_id)
    }

    pub(crate) fn update_client(&mut self, client: &Client, clock: Clock) -> ClientId {
        self.client = self.clients.get_or_insert(client);
        self.clock = clock.max(1);

        self.client
    }

    pub(crate) fn next_id(&mut self) -> Id {
        let id = Id::new(self.client, self.clock);
        self.clock += 1;

        id
    }

    pub(crate) fn next_id_range(&mut self, size: Clock) -> IdRange {
        let id = IdRange::new(self.client, self.clock, self.clock + size - 1);
        self.clock += size;

        id
    }

    pub(crate) fn contains(&self, id: &Id) -> bool {
        self.items.find(id).is_some()
    }

    pub(crate) fn find(&self, id: Id) -> Option<Type> {
        let key = self.id_map.find(&id);
        self.items.find(&key)
    }

    pub(crate) fn insert(&mut self, item: impl Into<Type>) {
        let item = item.into();
        if item.kind() == ItemKind::String {
            self.id_map.insert(item.id().range(item.size()));
        }

        self.items.insert(item);
    }

    pub(crate) fn remove(&mut self, id: &Id) {
        self.items.remove(id);
    }

    pub(crate) fn insert_delete(&mut self, item: DeleteItem) {
        self.deleted_items.insert(item);
    }

    pub(crate) fn replace(&mut self, item: Type, items: (Type, Type)) {
        self.items.replace(item, items);
    }

    pub(crate) fn client(&mut self, client_id: &Client) -> ClientId {
        self.clients.get_or_insert(client_id)
    }

    pub(crate) fn diff(&self, guid: String, state: ClientState) -> Diff {
        let items = self.items.diff(state.clone(), &self.id_map);
        // println!("items: {:?}", items);
        let deletes = self.deleted_items.diff(state.clone(), &self.id_map);

        let mut clients = self.clients.clone();

        for (_, client_id) in clients.clone().iter() {
            if (items.client_size(client_id) + deletes.client_size(client_id)) == 0 {
                clients.remove(client_id);
            }
        }

        Diff::from(
            guid,
            clients,
            self.fields.clone(),
            self.state.clone(),
            items,
            deletes,
        )
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct ReadyStore {
    pub(crate) id_range_map: IdRangeMap,
    pub(crate) queue: Vec<ItemData>,
    pub(crate) items: ItemDataStore,
    pub(crate) delete_items: DeleteItemStore,
}

impl ReadyStore {
    pub(crate) fn insert(&mut self, item: ItemData) {
        self.queue.push(item.clone());
        self.items.insert(item);
    }

    pub(crate) fn contains(&self, id: &Id) -> bool {
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
    ) -> std::collections::btree_map::Iter<ClientId, IdStore<ItemData>> {
        self.items.iter()
    }

    pub(crate) fn iter_delete_items(
        &self,
    ) -> std::collections::btree_map::Iter<ClientId, IdStore<DeleteItem>> {
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

    pub(crate) fn take_first(&mut self, client_id: &ClientId) -> Option<ItemData> {
        let store = self.items.items.get_mut(client_id)?;
        let (_, item) = store.iter_mut().next()?;
        let item = item.clone();
        store.remove(&item.id());
        Some(item)
    }

    pub(crate) fn remove_delete(&mut self, id: &Id) {
        self.delete_items.remove(id);
    }

    pub(crate) fn iter_items(
        &self,
    ) -> std::collections::btree_map::Iter<ClientId, IdStore<ItemData>> {
        self.items.iter()
    }

    pub(crate) fn iter_mut_items(
        &mut self,
    ) -> std::collections::btree_map::IterMut<ClientId, IdStore<ItemData>> {
        self.items.iter_mut()
    }

    pub(crate) fn iter_delete_items(
        &self,
    ) -> std::collections::btree_map::Iter<ClientId, IdStore<DeleteItem>> {
        self.delete_items.iter()
    }

    pub(crate) fn extend(&mut self, other: PendingStore) {
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

pub(crate) type ItemDataStore = ClientStore<ItemData>;
pub(crate) type DeleteItemStore = ClientStore<DeleteItem>;
pub(crate) type ItemStore = ClientStore<Type>;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub(crate) struct IdRangeMap {
    pub(crate) map: BTreeSet<IdRange>,
}

impl IdRangeMap {
    pub(crate) fn find(&self, id: &Id) -> Id {
        let e = self.map.get(&id.range(1));
        if let Some(range) = e {
            range.id()
        } else {
            id.clone()
        }
    }
}

impl IdRangeMap {
    pub(crate) fn insert(&mut self, id: IdRange) {
        self.map.insert(id);
    }

    pub(crate) fn has(&self, id: &Id) -> bool {
        self.map.contains(&id.range(1))
    }

    pub(crate) fn remove(&mut self, id: &Id) {
        self.map.remove(&id.range(1));
    }

    pub(crate) fn get(&self, id: &Id) -> Option<&IdRange> {
        self.map.get(&id.range(1))
    }

    pub(crate) fn replace(&mut self, id: IdRange, with: (IdRange, IdRange)) {
        self.map.remove(&id);
        self.insert(with.0);
        self.insert(with.1);
    }
}

impl IdDiff for DeleteItemStore {
    type Target = DeleteItemStore;

    fn diff(&self, state: ClientState, id_map: &IdRangeMap) -> DeleteItemStore {
        let mut diff = DeleteItemStore::default();

        for (client, store) in self.items.iter() {
            let clock = state.get(client).unwrap_or(&0);
            let items = store.diff(*clock, id_map);
            if items.size() > 0 {
                diff.items.insert(*client, items);
            }
        }

        diff
    }
}

impl IdDiff for ItemStore {
    type Target = ItemDataStore;

    fn diff(&self, state: ClientState, id_map: &IdRangeMap) -> ItemDataStore {
        let mut diff = ItemDataStore::default();

        for (client, store) in self.items.iter() {
            let clock = state.get(client).unwrap_or(&0);
            let items = store.diff(*clock, id_map);
            if items.size() > 0 {
                diff.items.insert(*client, items);
            }
        }

        diff
    }
}

trait IdDiff {
    type Target;
    fn diff(&self, state: ClientState, id_map: &IdRangeMap) -> Self::Target;
}

pub(crate) trait ClientStoreEntry:
    Default + WithId + Clone + Encode + Decode + Eq + PartialEq
{
}

// blanket implementation for all types that implement dependencies
impl<T: Default + WithId + Clone + Encode + Decode + Eq + PartialEq> ClientStoreEntry for T {}

#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct ClientStore<T: ClientStoreEntry> {
    pub(crate) items: BTreeMap<ClientId, IdStore<T>>,
}

impl<T: ClientStoreEntry> ClientStore<T> {
    pub(crate) fn client_size(&self, id: &ClientId) -> usize {
        self.items.get(id).map(|p1| p1.size()).unwrap_or(0)
    }
}

impl<T: ClientStoreEntry> ClientStore<T> {
    pub(crate) fn find(&self, id: &Id) -> Option<T> {
        self.items.get(&id.client).and_then(|store| store.get(&id))
    }

    pub(crate) fn insert(&mut self, item: T) {
        let id = item.id();
        let store = self.items.entry(id.client).or_default();
        store.insert(item);
    }

    pub(crate) fn keys(&self) -> std::collections::btree_map::Keys<ClientId, IdStore<T>> {
        self.items.keys()
    }

    pub(crate) fn iter(&self) -> std::collections::btree_map::Iter<ClientId, IdStore<T>> {
        self.items.iter()
    }

    pub(crate) fn iter_mut(&mut self) -> IterMut<ClientId, IdStore<T>> {
        self.items.iter_mut()
    }

    pub(crate) fn remove(&mut self, id: &Id) {
        if let Some(store) = self.items.get_mut(&id.client) {
            store.remove(id);
        }
    }

    pub(crate) fn replace(&mut self, item: T, items: (T, T)) {
        let id = item.id();
        let store = self.items.get_mut(&id.client).unwrap();
        store.remove(&item.id());

        store.insert(items.0);
        store.insert(items.1);
    }
}

impl<T: ClientStoreEntry> std::iter::IntoIterator for ClientStore<T> {
    type Item = (ClientId, IdStore<T>);
    type IntoIter = std::collections::btree_map::IntoIter<ClientId, IdStore<T>>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl<T: ClientStoreEntry> Encode for ClientStore<T> {
    fn encode<E: Encoder>(&self, e: &mut E, ctx: &EncodeContext) {
        e.u32(self.items.len() as u32);
        for (client, store) in self.items.iter() {
            e.u32(*client);
            store.encode(e, ctx);
        }
    }
}

impl<T: WithId + Clone + Default + Encode + Decode + Eq + PartialEq> Decode for ClientStore<T> {
    fn decode<D: Decoder>(d: &mut D, ctx: &DecodeContext) -> Result<ClientStore<T>, String> {
        let len = d.u32()? as usize;
        let mut items = BTreeMap::new();
        for _ in 0..len {
            let client = d.u32()?;
            let store = IdStore::decode(d, ctx)?;
            items.insert(client, store);
        }

        Ok(ClientStore { items })
    }
}

pub(crate) trait IdStoreEntry: WithId + Clone + Encode + Decode + Eq + PartialEq {}

// blanket implementation for all types that implement dependencies
impl<T: WithId + Clone + Encode + Decode + Eq + PartialEq> IdStoreEntry for T {}

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct IdStore<T: IdStoreEntry> {
    map: BTreeMap<Id, T>,
}

impl<T: IdStoreEntry> IdStore<T> {
    pub(crate) fn iter_mut(&mut self) -> std::collections::btree_map::IterMut<Id, T> {
        self.map.iter_mut()
    }

    pub(crate) fn iter(&self) -> std::collections::btree_map::Iter<Id, T> {
        self.map.iter()
    }
}

impl<T: IdStoreEntry> IdStore<T> {
    pub(crate) fn insert(&mut self, value: T) {
        self.map.insert(value.id(), value);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub(crate) fn get(&self, value: &Id) -> Option<T> {
        self.map.get(value).cloned()
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
}

impl<T: IdStoreEntry> IntoIterator for IdStore<T> {
    type Item = (Id, T);
    type IntoIter = std::collections::btree_map::IntoIter<Id, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.map.into_iter()
    }
}

impl<T: IdStoreEntry> Encode for IdStore<T> {
    fn encode<E: Encoder>(&self, e: &mut E, ctx: &EncodeContext) {
        e.u32(self.map.len() as u32);
        for (_, value) in self.map.iter() {
            value.encode(e, ctx);
        }
    }
}

impl<T: IdStoreEntry> Decode for IdStore<T> {
    fn decode<D: Decoder>(d: &mut D, ctx: &DecodeContext) -> Result<IdStore<T>, String> {
        let len = d.u32()? as usize;
        let mut data = BTreeMap::new();
        for _ in 0..len {
            let value = T::decode(d, ctx)?;
            data.insert(value.id(), value);
        }
        Ok(IdStore { map: data })
    }
}

pub(crate) trait IdClockDiff {
    type Target;
    fn diff(&self, clock: Clock, id_map: &IdRangeMap) -> Self::Target;
}

impl IdClockDiff for IdStore<ItemRef> {
    type Target = IdStore<ItemData>;

    fn diff(&self, clock: Clock, id_map: &IdRangeMap) -> Self::Target {
        let mut items = IdStore::default();
        for (id, item) in self.map.iter() {
            let data = item.borrow().data.clone();
            // collect items that are newer than the given clock
            if id.clock > clock {
                items.insert(data);
            } else if let Some(range) = id_map.get(id) {
                // if id falls within a range split the item and collect the right side
                if id.clock > clock {
                    items.insert(data);
                } else if range.start < clock && clock <= range.end {
                    if let Ok((_, r)) = data.split(clock) {
                        items.insert(r);
                    }
                }
            }
        }

        items
    }
}

impl IdClockDiff for IdStore<Type> {
    type Target = IdStore<ItemData>;

    fn diff(&self, clock: Clock, id_map: &IdRangeMap) -> Self::Target {
        let mut items = IdStore::default();
        for (id, item) in self.map.iter() {
            let data = item.item_ref().borrow().data.clone();
            // collect items that are newer than the given clock
            if id.clock > clock {
                items.insert(data);
            } else if let Some(range) = id_map.get(id) {
                // if id falls within a range split the item and collect the right side
                if id.clock > clock {
                    items.insert(data);
                } else if range.start < clock && clock <= range.end {
                    if let Ok((_, r)) = data.split(clock) {
                        items.insert(r);
                    }
                }
            }
        }

        items
    }
}

impl IdClockDiff for IdStore<DeleteItem> {
    type Target = IdStore<DeleteItem>;

    fn diff(&self, clock: Clock, _id_map: &IdRangeMap) -> Self::Target {
        let mut items = IdStore::default();
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
    use crate::codec_v1::EncoderV1;

    use super::*;

    #[test]
    fn test_id_store() {
        let mut store = IdStore::default();
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
        let mut store = IdStore::default();
        let id1 = Id::new(1, 1);
        let id2 = Id::new(1, 2);
        let id3 = Id::new(1, 3);

        store.insert(id1);
        store.insert(id2);
        store.insert(id3);

        let mut e = EncoderV1::new();
        store.encode(&mut e, &EncodeContext::default());

        let mut d = e.decoder();
        let dd = IdStore::<Id>::decode(&mut d, &DecodeContext::default()).unwrap();

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
        store.encode(&mut e, &EncodeContext::default());

        let mut d = e.decoder();
        let dd = ClientStore::<Id>::decode(&mut d, &DecodeContext::default()).unwrap();

        assert_eq!(store, dd);
    }
}
