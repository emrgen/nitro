use std::ops::Deref;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{ItemData, ItemRef};
use crate::store::WeakStoreRef;

#[derive(Clone, Debug)]
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
impl WithId for NText {
    fn id(&self) -> Id {
        self.item.borrow().id()
    }
}

impl WithIdRange for NText {
    fn range(&self) -> IdRange {
        self.borrow().id().range(1)
    }
}

impl Deref for NText {
    type Target = ItemRef;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl From<ItemRef> for NText {
    fn from(item: ItemRef) -> Self {
        Self { item }
    }
}
