use std::ops::Deref;

use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemData, ItemKey, ItemKind, ItemRef};
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

    pub(crate) fn size(&self) -> usize {
        let item = self.borrow();
        let map = item.as_map().unwrap();
        map.len()
    }

    fn field(&self) -> Option<String> {
        self.borrow().field()
    }

    pub(crate) fn content(&self) -> Content {
        let types = self.borrow().as_list();
        Content::Types(types)
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

impl NMap {
    pub(crate) fn get(&self, key: String) -> Option<Type> {
        let item = self.borrow();
        let map = item.as_map().unwrap();

        let item = map.get(&key);
        item.map(|item| item.clone().into())
    }

    pub(crate) fn set(&self, field: String, item: Type) {
        let item_ref = item.clone().item_ref();
        item_ref.borrow_mut().data.field = Some(field.clone());
        self.item_ref().append(item);
    }

    pub(crate) fn remove(&self, key: ItemKey) {
        let map = self.borrow().as_map().unwrap();
        let value = map.get(&key.as_string());
        if let Some(value) = value {
            value.delete();
        }
    }

    pub(crate) fn keys(&self) -> Vec<String> {
        let item = self.borrow();
        let map = item.as_map().unwrap();
        map.keys().map(|key| key.clone().into()).collect()
    }

    pub(crate) fn values(&self) -> Vec<Type> {
        let item = self.borrow();
        let map = item.as_map().unwrap();
        map.values().map(|item| item.clone().into()).collect()
    }

    pub(crate) fn clear(&self) {
        let item = self.borrow();
        let map = item.as_map().unwrap();
        for item in map.values() {
            item.delete();
        }
    }

    pub(crate) fn delete(&self) {
        self.item.delete(1);
    }

    pub(crate) fn to_json(&self) -> serde_json::Value {
        let mut json = self.borrow().to_json();
        let item = self.borrow();
        let map = item.as_map().unwrap();
        let mut content = serde_json::Map::new();
        for (key, value) in map.iter() {
            content.insert(key.clone(), value.to_json());
        }

        json.insert("content".to_string(), serde_json::Value::Object(content));

        serde_json::to_value(map).unwrap()
    }
}

impl Serialize for NMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut s = serializer.serialize_struct("Doc", self.borrow().serialize_size() + 1)?;
        self.borrow().serialize(&mut s)?;

        let map = self.borrow().as_map().unwrap();
        let content = serde_json::to_value(map).unwrap_or_default();
        s.serialize_field("content", &content)?;

        s.end()
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
