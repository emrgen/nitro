use std::ops::Deref;

use serde::Serialize;
use serde_json::Value;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemData, ItemKind, ItemRef};
use crate::store::WeakStoreRef;

#[derive(Clone, Debug)]
pub(crate) struct NMark {
    item: ItemRef,
}

impl NMark {
    pub(crate) fn new(id: Id, content: Content, store: WeakStoreRef) -> Self {
        let data = ItemData {
            kind: ItemKind::Mark,
            id,
            content,
            ..ItemData::default()
        };

        let item = ItemRef::new(data.into(), store);

        Self { item }
    }

    pub(crate) fn size(&self) -> usize {
        let marks = self.borrow().get_marks();

        marks.len()
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }

    pub(crate) fn to_json(&self) -> Value {
        serde_json::to_value(self.clone()).unwrap()
    }
}

impl NMark {
    pub(crate) fn content(&self) -> Content {
        self.item_ref().borrow().content.clone()
    }
}

impl Serialize for NMark {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let content = self.content();
        println!("{:?}", content);
        if let Content::Mark(mark) = &content {
            serializer.serialize_some(mark)
        } else {
            panic!("NMark content is not a mark type");
        }
    }
}

impl Deref for NMark {
    type Target = ItemRef;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl WithId for NMark {
    fn id(&self) -> Id {
        self.item.borrow().id()
    }
}

impl WithIdRange for NMark {
    fn range(&self) -> IdRange {
        self.borrow().id().range(1)
    }
}

impl From<ItemRef> for NMark {
    fn from(item: ItemRef) -> Self {
        Self { item }
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::Doc;
    use crate::id::IdRange;
    use crate::mark::MarkContent;

    #[test]
    fn test_nmark() {
        let doc = Doc::default();
        let m1 = doc.mark(IdRange::default(), MarkContent::Bold);

        println!("{}", serde_json::to_string(&m1).unwrap());

        doc.add_mark(m1);

        let yaml = serde_yaml::to_string(&doc).unwrap();
        println!("{}", yaml);
    }
}
