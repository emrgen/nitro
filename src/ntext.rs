use crate::id::Id;
use crate::item::{ItemData, ItemRef};
use crate::store::WeakStoreRef;

pub(crate) struct NText {
    pub(crate) item: ItemRef,
}

impl NText {
    pub(crate) fn new(id: Id, store: WeakStoreRef) -> Self {
        let data = ItemData {
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

impl From<ItemRef> for NText {
    fn from(item: ItemRef) -> Self {
        Self { item }
    }
}
