use std::ops::Deref;

use serde::ser::SerializeStruct;
use serde::Serialize;
use serde_json::Value;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemData, ItemKind, ItemRef};
use crate::nmark::NMark;
use crate::store::WeakStoreRef;

#[derive(Clone, Debug)]
pub(crate) struct NString {
    pub(crate) item: ItemRef,
}

impl NString {
    pub(crate) fn new(id: Id, string: String, store: WeakStoreRef) -> Self {
        let data = ItemData {
            id,
            kind: ItemKind::String,
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

    pub(crate) fn delete(&self) {
        self.item.delete(self.size() as u32);
    }

    pub(crate) fn add_mark(&self, mark: NMark) {
        if let Content::Mark(m) = mark.item_ref().borrow_mut().content_mut() {
            m.range = self.id().range(self.size() as u32);
        }

        self.item_ref().add_mark(mark);
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }

    pub(crate) fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();

        map.insert("text".to_string(), self.borrow().content().to_json());

        map.into()
    }
}

impl Serialize for NString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut s = serializer.serialize_struct("String", self.borrow().serialize_size() + 1)?;
        self.borrow().serialize(&mut s)?;

        let content = serde_json::to_value(self.content()).unwrap_or_default();
        s.serialize_field("content", &content)?;

        s.end()
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
