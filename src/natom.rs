use std::ops::Deref;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemData, ItemKind, ItemRef};
use crate::store::WeakStoreRef;

#[derive(Clone, Debug)]
pub(crate) struct NAtom {
    pub(crate) item: ItemRef,
}

impl NAtom {
    pub(crate) fn new(id: Id, content: Content, store: WeakStoreRef) -> Self {
        let data = ItemData {
            kind: ItemKind::Atom,
            id,
            content,
            ..ItemData::default()
        };
        Self {
            item: ItemRef::new(data.into(), store),
        }
    }

    pub(crate) fn size(&self) -> usize {
        1
    }

    pub(crate) fn content(&self) -> Content {
        self.borrow().content()
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }

    pub(crate) fn to_json(&self) -> serde_json::Value {
        self.borrow().to_json()
    }
}

impl WithId for NAtom {
    fn id(&self) -> Id {
        self.item.borrow().id()
    }
}

impl WithIdRange for NAtom {
    fn range(&self) -> IdRange {
        self.borrow().id().range(1)
    }
}

impl Deref for NAtom {
    type Target = ItemRef;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl From<ItemRef> for NAtom {
    fn from(item: ItemRef) -> Self {
        Self { item }
    }
}
