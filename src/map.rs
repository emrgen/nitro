use crate::id::Id;
use crate::item::{ItemData, ItemKey, ItemKind, ItemRef};
use crate::store::WeakStoreRef;
use crate::types::Type;

#[derive(Clone, Debug, Default)]
pub(crate) struct NMap {
    pub(crate) item: ItemRef,
}

impl NMap {
    pub(crate) fn new(id: Id, store: WeakStoreRef) -> Self {
        let data = ItemData {
            kind: ItemKind::Map,
            id,
            ..ItemData::default()
        };
        let item_ref = ItemRef::new(data.into(), store);

        Self { item: item_ref }
    }

    pub fn set(&mut self, key: &ItemKey, value: ItemRef) {
        let item = self.item.borrow();
        // item.set(key, value);
    }

    pub fn get(&self, key: ItemKey) -> Option<Type> {
        let item = self.item.borrow();
        let map = item.as_map().unwrap();
        match key {
            ItemKey::String(key) => {
                let item = map.get(&key);
                item.map(|item| item.clone().into())
            }
            ItemKey::Number(key) => {
                let item = map.get(&key.to_string());
                item.map(|item| item.clone().into())
            }
        }
    }

    pub fn remove(&mut self, key: &ItemKey) {
        let map = self.item.borrow().as_map().unwrap();
        let value = map.get(&key.as_string());
        if let Some(value) = value {
            value.delete();
        }
    }

    pub fn keys(&self) -> Vec<ItemKey> {
        let item = self.item.borrow();
        let map = item.as_map().unwrap();
        map.keys().map(|key| key.clone().into()).collect()
    }

    pub fn values(&self) -> Vec<Type> {
        let item = self.item.borrow();
        let map = item.as_map().unwrap();
        map.values().map(|item| item.clone().into()).collect()
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }
}

impl From<ItemRef> for NMap {
    fn from(item: ItemRef) -> Self {
        Self { item }
    }
}
