use crate::id::Id;
use crate::item::{ItemData, ItemRef};
use crate::nproxy::NProxy;
use crate::store::WeakStoreRef;

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
        store.insert(mover.item_ref());
        store.insert(proxy.item_ref());

        (mover, proxy)
    }
}
