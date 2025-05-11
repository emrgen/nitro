use std::ops::Deref;

use serde_json::Value;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemData, ItemKind, ItemRef};
use crate::nproxy::NProxy;
use crate::store::WeakStoreRef;

/// NMove represents a move operation in the document.
#[derive(Debug, Clone)]
pub(crate) struct NMove {
    pub(crate) item: ItemRef,
}

impl NMove {
    pub(crate) fn new(id: Id, target_id: Id, store: WeakStoreRef) -> NMove {
        let data = ItemData {
            id,
            kind: ItemKind::Move,
            target_id: Some(target_id),
            ..ItemData::default()
        };

        Self {
            item: ItemRef::new(data.into(), store),
        }
    }

    pub(crate) fn delete(&self) {
        self.item.delete(1);
    }

    pub(crate) fn content(&self) -> Content {
        Content::Null
    }

    pub(crate) fn size(&self) -> u32 {
        1
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }

    pub(crate) fn create_pair(target_id: Id, store: WeakStoreRef) -> (NMove, NProxy) {
        let (mover_id, proxy_id) = {
            let store = store.upgrade().unwrap();
            let mut store = store.borrow_mut();

            let mover_id = store.next_id();
            let proxy_id = store.next_id();

            (mover_id, proxy_id)
        };

        let mover_store = store.clone();
        let proxy_store = store.clone();

        let store = store.upgrade().unwrap();
        let mut store = store.borrow_mut();

        let mover = NMove::new(mover_id, proxy_id, mover_store);
        store.insert(mover.clone());

        let proxy = NProxy::new(proxy_id, mover_id, target_id, proxy_store);
        store.insert(proxy.clone());

        (mover, proxy)
    }

    pub(crate) fn to_json(&self) -> Value {
        todo!()
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
        Self { item }
    }
}

impl Deref for NMove {
    type Target = ItemRef;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}
