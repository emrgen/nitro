use std::ops::Deref;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{GetItemRef, ItemData, ItemKind, ItemRef};
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

    fn field(&self) -> Option<String> {
        self.borrow().field()
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }
}

impl Deref for NMap {
    type Target = ItemRef;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl IMap for NMap {
    fn size(&self) -> usize {
        let item = self.borrow();
        let map = item.as_map().unwrap();
        map.len()
    }

    fn get(&self, key: String) -> Option<Type> {
        let item = self.borrow();
        let map = item.as_map().unwrap();

        let item = map.get(&key);
        item.map(|item| item.clone().into())
    }

    fn set(&self, field: String, item: Type) {
        let item_ref = item.clone().item_ref();
        item_ref.borrow_mut().data.field = Some(field);
        self.item_ref().append(item)
    }

    fn remove(&self, key: String) {
        let map = self.borrow().as_map().unwrap();
        let value = map.get(&key);
        if let Some(value) = value {
            value.delete();
        }
    }

    fn keys(&self) -> Vec<String> {
        let item = self.borrow();
        let map = item.as_map().unwrap();
        map.keys().map(|key| key.clone().into()).collect()
    }

    fn values(&self) -> Vec<Type> {
        let item = self.borrow();
        let map = item.as_map().unwrap();
        map.values().map(|item| item.clone().into()).collect()
    }
}

impl WithId for NMap {
    fn id(&self) -> Id {
        self.item.borrow().id()
    }
}

impl WithIdRange for NMap {
    fn range(&self) -> IdRange {
        self.item.borrow().id().range(1)
    }
}

impl From<ItemRef> for NMap {
    fn from(item: ItemRef) -> Self {
        Self { item }
    }
}

pub trait IMap {
    fn size(&self) -> usize;
    fn get(&self, key: String) -> Option<Type>;
    fn set(&self, key: String, item: Type);
    fn remove(&self, key: String);
    fn keys(&self) -> Vec<String>;
    fn values(&self) -> Vec<Type>;
}
