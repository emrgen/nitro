use fake::Opt;
use serde_json::Value;
use std::ops::Deref;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemData, ItemKey, ItemKind, ItemRef};
use crate::nproxy::NProxy;
use crate::store::WeakStoreRef;
use crate::Type;

/// NMove represents a move operation in the document.
/// It is similar to proxy but one instance is valid for multiple move operations.
#[derive(Debug, Clone)]
pub(crate) struct NMove {
    pub(crate) item: ItemRef,
    pub(crate) target: Option<Box<Type>>,
}

impl NMove {
    pub(crate) fn new(id: Id, target: Option<Type>, store: WeakStoreRef) -> NMove {
        let data = ItemData {
            id,
            kind: ItemKind::Move,
            ..ItemData::default()
        };

        Self {
            item: ItemRef::new(data.into(), store),
            target: target.map(Box::new),
        }
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }

    pub(crate) fn content(&self) -> Content {
        if let Some(target) = self.target.as_ref() {
            target.content()
        } else {
            Content::Null
        }
    }

    pub(crate) fn size(&self) -> u32 {
        if let Some(target) = self.target.as_ref() {
            target.size()
        } else {
            0
        }
    }

    fn get(&self, key: ItemKey) -> Option<Type> {
        if let Some(target) = self.target.as_ref() {
            target.get(key)
        } else {
            None
        }
    }

    fn set(&self, key: String, item: Type) {
        if let Some(target) = self.target.as_ref() {
            target.set(key, item);
        }
    }

    fn delete(&self) {
        self.item_ref().delete(1);
    }

    fn prepend(&self, item: Type) {
        if let Some(target) = self.target.as_ref() {
            target.prepend(item);
        }
    }

    fn append(&self, item: Type) {
        if let Some(target) = self.target.as_ref() {
            target.append(item);
        }
    }

    fn insert(&self, offset: u32, item: Type) {
        if let Some(target) = self.target.as_ref() {
            target.insert(offset, item);
        }
    }

    fn remove(&self, key: ItemKey) {
        if let Some(target) = self.target.as_ref() {
            target.remove(key);
        }
    }

    fn clear(&self) {
        if let Some(target) = self.target.as_ref() {
            target.clear();
        }
    }

    pub(crate) fn to_json(&self) -> serde_json::Value {
        if let Some(target) = self.target.as_ref() {
            target.to_json()
        } else {
            serde_json::Value::Null
        }
    }
}

impl WithId for NMove {
    #[inline]
    fn id(&self) -> Id {
        self.item.borrow().id()
    }
}

impl WithIdRange for NMove {
    fn range(&self) -> IdRange {
        self.borrow().id().range(1)
    }
}

impl From<ItemRef> for NMove {
    fn from(item: ItemRef) -> Self {
        unimplemented!("This function is not implemented yet.");
        Self { item, target: None }
    }
}

impl Deref for NMove {
    type Target = ItemRef;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}
