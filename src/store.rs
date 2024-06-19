use crate::clients::Client;
use crate::id::{Id, WithId};
use crate::item::{ItemKey, ItemRef};
use std::collections::{BTreeMap, HashMap};

pub struct ItemStore {
    items: HashMap<Client, Store<ItemRef>>,
}

impl ItemStore {
    pub(crate) fn find(&self, id: Id) -> Option<ItemRef> {
        self.items.get(&id.client).and_then(|store| store.get(&id))
    }

    pub(crate) fn insert(&mut self, item: ItemRef) {
        let id = item.borrow().id;
        let store = self.items.entry(id.client).or_default();
        store.insert(item);
    }

    pub(crate) fn replace(&mut self, item: ItemRef, items: (ItemRef, ItemRef)) {
        let id = item.borrow().id;
        let store = self.items.get_mut(&id.client).unwrap();
        store.remove(item);

        store.insert(items.0);
        store.insert(items.1);
    }
}

#[derive(Default, Debug)]
pub(crate) struct Store<T: WithId + Clone> {
    data: BTreeMap<Id, T>,
}

impl<T: WithId + Clone> Store<T> {
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
        let mut store = Store::default();
        assert!(!store.contains(&Id::new(1, 1, 1)));
        store.insert(Id::new(1, 1, 1));
        assert!(store.contains(&Id::new(1, 1, 1)));

        store.insert(Id::new(1, 5, 8));
        assert!(store.contains(&Id::new(1, 6, 6)));
    }
}
