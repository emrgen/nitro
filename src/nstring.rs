use std::ops::Deref;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemData, ItemRef};
use crate::store::WeakStoreRef;

#[derive(Clone, Debug)]
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
    pub(crate) fn content(&self) -> Content {
        self.borrow().content()
    }

    pub(crate) fn size(&self) -> usize {
        match self.borrow().content {
            Content::String(ref s) => s.len(),
            _ => panic!("NString has invalid content"),
        }
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }
}

impl WithId for NString {
    fn id(&self) -> Id {
        self.item.borrow().id()
    }
}

impl WithIdRange for NString {
    fn range(&self) -> IdRange {
        self.item.borrow().id().range(self.size() as u32)
    }
}

impl Deref for NString {
    type Target = ItemRef;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl From<ItemRef> for NString {
    fn from(item: ItemRef) -> Self {
        Self { item }
    }
}
