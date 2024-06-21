use std::ops::Deref;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemData, ItemKey, ItemRef};
use crate::store::WeakStoreRef;
use crate::types::Type;

#[derive(Debug, Clone)]
pub(crate) struct NProxy {
    pub(crate) item: ItemRef,
    pub(crate) target: Box<Option<Type>>,
}

impl NProxy {
    pub(crate) fn new(id: Id, mover_id: Id, target_id: Id, store: WeakStoreRef) -> NProxy {
        let data = ItemData {
            id,
            mover_id: Some(mover_id),
            target_id: Some(target_id),
            ..ItemData::default()
        };

        let target = store.upgrade().unwrap().borrow().find(target_id);

        Self {
            item: ItemRef::new(data.into(), store),
            target: Box::new(target),
        }
    }

    pub(crate) fn content(&self) -> Content {
        if let Some(target) = self.target.as_ref() {
            target.content()
        } else {
            Content::Null
        }
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }
}

impl NProxy {
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

    fn insert(&self, offset: usize, item: Type) {
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
        todo!()
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
