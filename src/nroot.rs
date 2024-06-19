use crate::id::Id;
use crate::item::{ItemData, ItemKind, ItemRef};
use crate::store::WeakStoreRef;

pub(crate) struct NRoot {
    pub(crate) item: ItemRef,
}

impl NRoot {
    pub(crate) fn new(id: Id, store: WeakStoreRef) -> Self {
        let data = ItemData {
            kind: ItemKind::Root,
            id,
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
