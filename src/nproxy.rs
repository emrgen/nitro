use crate::id::Id;
use crate::item::{ItemData, ItemRef};
use crate::store::WeakStoreRef;

pub(crate) struct NProxy {
    pub(crate) item: ItemRef,
}

impl NProxy {
    pub(crate) fn new(id: Id, mover_id: Id, target_id: Id, store: WeakStoreRef) -> NProxy {
        let data = ItemData {
            id,
            mover_id: Some(mover_id),
            target_id: Some(target_id),
            ..ItemData::default()
        };

        Self {
            item: ItemRef::new(data.into(), store),
        }
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }
}
