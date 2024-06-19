use crate::id::Id;
use crate::item::{Content, ItemData, ItemRef};
use crate::store::WeakStoreRef;

pub(crate) struct NString {
    pub(crate) item: ItemRef,
}

impl NString {
    pub(crate) fn new(id: Id, string: String, store: WeakStoreRef) -> Self {
        let data = ItemData {
            id,
            content: Content::String(string),
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

impl From<ItemRef> for NString {
    fn from(item: ItemRef) -> Self {
        Self { item }
    }
}
