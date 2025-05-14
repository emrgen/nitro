use std::ops::Deref;

use crate::id::{Id, IdRange, WithId, WithIdRange, WithTarget};
use crate::item::{Content, ItemData, ItemKey, ItemKind, ItemRef};
use crate::store::WeakStoreRef;
use crate::types::Type;

/// NProxy represents a proxy for another item in the document.
/// Multiple Proxy instances can point to the same target item.
/// It is used to manage references to items in a way that allows for
/// dynamic updates and changes.
#[derive(Debug, Clone)]
pub(crate) struct NProxy {
    pub(crate) item: ItemRef,
}

impl NProxy {
    pub(crate) fn new(id: Id, target: Type, store: WeakStoreRef) -> NProxy {
        let data = ItemData {
            id,
            kind: ItemKind::Proxy,
            content: Content::Id(target.id()),
            ..ItemData::default()
        };

        let mut item = ItemRef::new(data.into(), store);
        item.set_target(target);

        Self { item }
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }

    pub(crate) fn content(&self) -> Content {
        if let Some(target) = self.get_target().as_ref() {
            target.content()
        } else {
            Content::Null
        }
    }

    pub(crate) fn size(&self) -> u32 {
        if let Some(target) = self.get_target().as_ref() {
            target.size()
        } else {
            0
        }
    }

    fn get(&self, key: ItemKey) -> Option<Type> {
        if let Some(target) = self.get_target().as_ref() {
            target.get(key)
        } else {
            None
        }
    }

    fn set(&self, key: String, item: Type) {
        if let Some(target) = self.get_target().as_ref() {
            target.set(key, item);
        }
    }

    fn delete(&self) {
        self.item_ref().delete(1);
    }

    fn prepend(&self, item: Type) {
        if let Some(target) = self.get_target().as_ref() {
            target.prepend(item);
        }
    }

    fn append(&self, item: Type) {
        if let Some(target) = self.get_target().as_ref() {
            target.append(item);
        }
    }

    fn insert(&self, offset: u32, item: Type) {
        if let Some(target) = self.get_target().as_ref() {
            target.insert(offset, item);
        }
    }

    fn remove(&self, key: ItemKey) {
        if let Some(target) = self.get_target().as_ref() {
            target.remove(key);
        }
    }

    fn clear(&self) {
        if let Some(target) = self.get_target().as_ref() {
            target.clear();
        }
    }

    pub(crate) fn to_json(&self) -> serde_json::Value {
        if let Some(target) = self.get_target().as_ref() {
            target.to_json()
        } else {
            serde_json::Value::Null
        }
    }
}

impl WithId for NProxy {
    #[inline]
    fn id(&self) -> Id {
        self.item.borrow().id()
    }
}

impl WithIdRange for NProxy {
    fn range(&self) -> IdRange {
        self.borrow().id().range(1)
    }
}

impl Deref for NProxy {
    type Target = ItemRef;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}
