use crate::clients::Client;
use crate::delete::DeleteItem;
use crate::id::{Id, WithId};
use crate::item::{ItemData, ItemRef};

use std::collections::{BTreeMap, HashMap};

#[derive(Default, Debug, Clone)]
pub(crate) struct Store {
    pub(crate) items: ItemStore,
    pub(crate) deleted_items: DeleteItemStore,
    pub(crate) pending: PendingStore,
}

impl Store {
    pub(crate) fn find(&self, id: Id) -> Option<ItemRef> {
        self.items.find(id)
    }

    pub(crate) fn insert(&mut self, item: ItemRef) {
        self.items.insert(item);
    }

    pub(crate) fn replace(&mut self, item: ItemRef, items: (ItemRef, ItemRef)) {
        self.items.replace(item, items);
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

#[derive(Default, Clone, Debug)]
pub struct ClientStore<T: WithId + Clone> {
    items: HashMap<Client, IdStore<T>>,
}

impl<T: WithId + Clone + Default> ClientStore<T> {
    pub(crate) fn find(&self, id: Id) -> Option<T> {
        self.items.get(&id.client).and_then(|store| store.get(&id))
    }

    pub(crate) fn insert(&mut self, item: T) {
        let id = item.id();
        let store = self.items.entry(id.client).or_default();
        store.insert(item);
    }

    pub(crate) fn replace(&mut self, item: T, items: (T, T)) {
        let id = item.id();
        let store = self.items.get_mut(&id.client).unwrap();
        store.remove(item);

        store.insert(items.0);
        store.insert(items.1);
    }
}

#[derive(Default, Debug, Clone)]
pub(crate) struct IdStore<T: WithId + Clone> {
    data: BTreeMap<Id, T>,
}

impl<T: WithId + Clone> IdStore<T> {
    pub(crate) fn insert(&mut self, value: T) {
        self.data.insert(value.id(), value);
    }

    pub(crate) fn get(&self, value: &Id) -> Option<T> {
        self.data.get(value).cloned()
    }

    pub(crate) fn remove(&mut self, value: T) -> Option<T> {
        self.data.remove(&value.id())
    }

    pub(crate) fn contains(&self, value: &Id) -> bool {
        self.data.contains_key(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::id::Id;

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
