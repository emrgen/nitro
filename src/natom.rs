use std::ops::Deref;

use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemData, ItemKind, ItemRef};
use crate::store::WeakStoreRef;

#[derive(Clone, Debug)]
pub struct NAtom {
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

    pub(crate) fn depth(&self) -> u32 {
        self.item.depth()
    }

    pub(crate) fn size(&self) -> u32 {
        1
    }

    pub(crate) fn content(&self) -> Content {
        self.borrow().content()
    }

    pub(crate) fn delete(&self) {
        self.item.delete(1);
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }

    pub(crate) fn to_json(&self) -> serde_json::Value {
        self.content().to_json()
    }
}

impl Serialize for NAtom {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut s = serializer.serialize_struct("Atom", self.borrow().serialize_size() + 1)?;

        self.serialize_with(&mut s)?;

        s.serialize_field("content", &self.content())?;

        s.end()
    }
}

impl WithId for NAtom {
    #[inline]
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
