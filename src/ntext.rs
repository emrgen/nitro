use std::ops::Deref;

use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemData, ItemKind, ItemRef};
use crate::store::WeakStoreRef;
use crate::types::Type;

#[derive(Clone, Debug)]
pub(crate) struct NText {
    pub(crate) item: ItemRef,
}

impl NText {
    pub(crate) fn new(id: Id, store: WeakStoreRef) -> Self {
        let data = ItemData {
            id,
            kind: ItemKind::Text,
            ..ItemData::default()
        };

        Self {
            item: ItemRef::new(data.into(), store),
        }
    }

    pub(crate) fn clear(&self) {
        self.item_ref()
            .borrow()
            .items()
            .iter()
            .for_each(|item| item.delete());
    }

    pub(crate) fn content(&self) -> Content {
        let items = self.borrow().as_list();
        Content::Types(items)
    }

    pub(crate) fn size(&self) -> u32 {
        self.borrow().as_list().iter().map(|item| item.size()).sum()
    }

    pub(crate) fn insert(&self, offset: u32, item: impl Into<Type>) {
        let items = self.borrow().as_list();
        let item = item.into();

        if offset == 0 {
            self.prepend(item);
        } else if offset >= self.size() {
            self.append(item);
        } else {
            // find the target item offset
            let mut target_offset = 0;
            let mut target = None;

            for item in items.iter() {
                let size = item.size();
                if target_offset + size > offset {
                    target = Some(item);
                    break;
                }
                target_offset += size;
            }

            if let Some(target) = target {
                let target_offset = offset - target_offset;
                if target_offset == 0 {
                    target.insert_before(item);
                } else if target_offset >= target.size() {
                    target.insert_after(item);
                } else {
                    let items = target.split(target_offset);
                    self.store
                        .upgrade()
                        .unwrap()
                        .borrow_mut()
                        .replace(target, items.clone());

                    items.0.insert_after(item);
                }
            }
        }
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }

    pub(crate) fn to_json(&self) -> serde_json::Value {
        let items = self.borrow().as_list();
        let items: Vec<_> = items.iter().map(|item| item.to_json()).collect();

        items.into()
    }
}

impl Serialize for NText {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut s = serializer.serialize_struct("Text", self.borrow().serialize_size() + 1)?;
        self.borrow().serialize_with(&mut s)?;

        let items = self.borrow().as_list();
        let content = items
            .iter()
            .map(|item| serde_json::to_value(item).unwrap_or_default())
            .collect();
        s.serialize_field("content", &serde_json::Value::Array(content))?;

        s.end()
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

#[cfg(test)]
mod tests {
    use crate::doc::Doc;

    #[test]
    fn test_text() {
        let doc = Doc::default();
        let text = doc.text();
        doc.set("text", text.clone());

        assert_eq!(text.size(), 0);

        let s1 = doc.string("hello");
        text.append(s1.clone());

        let s2 = doc.string("world");
        text.prepend(s2.clone());

        let yaml = serde_yaml::to_string(&text).unwrap();
        println!("{}", yaml);
    }
}
