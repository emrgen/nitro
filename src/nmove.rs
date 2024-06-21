use std::ops::Deref;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{ItemData, ItemRef};
use crate::nproxy::NProxy;
use crate::store::WeakStoreRef;

#[derive(Debug, Clone)]
pub(crate) struct NMove {
    pub(crate) item: ItemRef,
}

impl NMove {
    pub(crate) fn new(id: Id, mover_id: Id, store: WeakStoreRef) -> NMove {
        let data = ItemData {
            id,
            mover_id: Some(mover_id),
            ..ItemData::default()
        };

        Self {
            item: ItemRef::new(data.into(), store),
        }
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

        let proxy = NProxy::new(proxy_id, mover_id, target_id, store.clone());
        let mover = NMove::new(mover_id, proxy_id, store.clone());

        let store = store.upgrade().unwrap();
        let mut store = store.borrow_mut();
        store.insert(mover.clone().into());
        store.insert(proxy.clone().into());

        (mover, proxy)
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
