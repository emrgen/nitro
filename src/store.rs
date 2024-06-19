use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock, Weak};

use crate::clients::{Client, ClientId, ClientMap};
use crate::codec::decoder::{Decode, Decoder};
use crate::codec::encoder::{Encode, Encoder};
use crate::delete::DeleteItem;
use crate::diff::Diff;
use crate::id::{Clock, Id, Split, WithId};
use crate::item::{ItemData, ItemRef};
use crate::state::ClientState;

pub(crate) type StoreRef = Arc<RwLock<DocStore>>;
pub(crate) type WeakStoreRef = Weak<RwLock<DocStore>>;

#[derive(Default, Debug, Clone)]
pub(crate) struct DocStore {
    pub(crate) client: Client,
    pub(crate) clock: Clock,

    pub(crate) clients: ClientMap,
    pub(crate) state: ClientState,
    pub(crate) items: ItemStore,
    pub(crate) deleted_items: DeleteItemStore,
    pub(crate) pending: PendingStore,
}

impl DocStore {
    pub(crate) fn update_client(&mut self, client: ClientId, clock: Clock) -> Client {
        self.client = self.clients.get_or_insert(client);
        self.clock = clock;

        self.client
    }

    pub(crate) fn take(&mut self, size: Clock) -> Id {
        let id = Id::new(self.client, self.clock, self.clock + size - 1);
        self.clock += size;
        id
    }

    pub(crate) fn find(&self, id: Id) -> Option<ItemRef> {
        self.items.find(id)
    }

    pub(crate) fn insert(&mut self, item: ItemRef) {
        self.items.insert(item);
    }

    pub(crate) fn remove(&mut self, id: &Id) {
        self.items.remove(id);
    }

    pub(crate) fn insert_delete(&mut self, item: DeleteItem) {
        self.deleted_items.insert(item);
    }

    pub(crate) fn replace(&mut self, item: ItemRef, items: (ItemRef, ItemRef)) {
        self.items.replace(item, items);
    }

    pub(crate) fn client(&mut self, client_id: ClientId) -> Client {
        self.clients.get_or_insert(client_id)
    }

    pub(crate) fn diff(&self, guid: String, state: ClientState) -> Diff {
        let items = self.items.diff(state.clone());
        let deletes = self.deleted_items.diff(state.clone());
        Diff::from(
            guid,
            self.clients.clone(),
            self.state.clone(),
            items,
            deletes,
        )
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct ReadyStore {
    pub(crate) queue: Vec<ItemData>,
    pub(crate) items: ItemDataStore,
    pub(crate) delete_items: DeleteItemStore,
}

#[derive(Default, Debug, Clone)]
pub(crate) struct PendingStore {
    pub(crate) items: ItemDataStore,
    pub(crate) delete_items: DeleteItemStore,
}

pub(crate) type ItemDataStore = ClientStore<ItemData>;
pub(crate) type DeleteItemStore = ClientStore<DeleteItem>;
pub(crate) type ItemStore = ClientStore<ItemRef>;

impl IdDiff for DeleteItemStore {
    type Target = DeleteItemStore;

    fn diff(&self, state: ClientState) -> DeleteItemStore {
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

impl IdDiff for ItemStore {
    type Target = ItemDataStore;

    fn diff(&self, state: ClientState) -> ItemDataStore {
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

trait IdDiff {
    type Target;
    fn diff(&self, state: ClientState) -> Self::Target;
}

#[derive(Default, Clone, Debug)]
pub struct ClientStore<T: WithId + Clone + Encode + Decode> {
    items: HashMap<Client, IdStore<T>>,
}

impl<T: WithId + Clone + Default + Encode + Decode> ClientStore<T> {
    pub(crate) fn find(&self, id: Id) -> Option<T> {
        self.items.get(&id.client).and_then(|store| store.get(&id))
    }

    pub(crate) fn insert(&mut self, item: T) {
        let id = item.id();
        let store = self.items.entry(id.client).or_default();
        store.insert(item);
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

impl<T: WithId + Clone + Default + Encode + Decode> Encode for ClientStore<T> {
    fn encode<E: Encoder>(&self, e: &mut E) {
        for (client, store) in self.items.iter() {
            e.u32(*client);
            store.encode(e);
        }
    }
}

impl<T: WithId + Clone + Default + Encode + Decode> Decode for ClientStore<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<ClientStore<T>, String> {
        let mut items = HashMap::new();
        while let Ok(client) = d.u32() {
            let store = IdStore::decode(d)?;
            items.insert(client, store);
        }
        Ok(ClientStore { items })
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct IdStore<T: WithId + Clone + Encode + Decode> {
    data: BTreeMap<Id, T>,
}

impl<T: WithId + Clone + Encode + Decode> IdStore<T> {
    pub(crate) fn insert(&mut self, value: T) {
        self.data.insert(value.id(), value);
    }

    pub(crate) fn get(&self, value: &Id) -> Option<T> {
        self.data.get(value).cloned()
    }

    pub(crate) fn remove(&mut self, value: &Id) -> Option<T> {
        self.data.remove(value)
    }

    pub(crate) fn contains(&self, value: &Id) -> bool {
        self.data.contains_key(value)
    }

    pub(crate) fn size(&self) -> usize {
        self.data.len()
    }
}

impl<T: Encode + Clone + WithId + Decode> Encode for IdStore<T> {
    fn encode<E: Encoder>(&self, e: &mut E) {
        e.u32(self.data.len() as u32);
        for (_, value) in self.data.iter() {
            value.encode(e);
        }
    }
}

impl<T: Encode + Clone + WithId + Decode> Decode for IdStore<T> {
    fn decode<D: Decoder>(d: &mut D) -> Result<IdStore<T>, String> {
        let len = d.u32()? as usize;
        let mut data = BTreeMap::new();
        for _ in 0..len {
            let value = T::decode(d)?;
            data.insert(value.id(), value);
        }
        Ok(IdStore { data })
    }
}

pub(crate) trait IdClockDiff {
    type Target;
    fn diff(&self, clock: Clock) -> Self::Target;
}

impl IdClockDiff for IdStore<ItemRef> {
    type Target = IdStore<ItemData>;

    fn diff(&self, clock: Clock) -> Self::Target {
        let mut items = IdStore::default();
        for (id, item) in self.data.iter() {
            let data = item.borrow().data.clone();
            if id.start > clock {
                items.insert(data)
            } else if id.end > clock && id.start < clock {
                let (_, r) = data.split(clock);
                items.insert(r);
            }
        }

        items
    }
}

impl IdClockDiff for IdStore<DeleteItem> {
    type Target = IdStore<DeleteItem>;

    fn diff(&self, clock: Clock) -> Self::Target {
        let mut items = IdStore::default();
        for (id, item) in self.data.iter() {
            if id.start > clock {
                items.insert(item.clone());
            }
        }

        items
    }
}

#[cfg(test)]
mod tests {
    use crate::id::Id;

    use super::*;

    #[test]
    fn test_id_store() {
        let mut store = IdStore::default();
        assert!(!store.contains(&Id::new(1, 1, 1)));
        store.insert(Id::new(1, 1, 1));
        assert!(store.contains(&Id::new(1, 1, 1)));

        store.insert(Id::new(1, 5, 8));
        assert!(store.contains(&Id::new(1, 6, 6)));
    }
}
