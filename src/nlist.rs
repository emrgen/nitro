use std::ops::Deref;

use serde::ser::{Serialize, SerializeStruct};

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemData, ItemKey, ItemKind, ItemRef};
use crate::store::WeakStoreRef;
use crate::types::Type;

#[derive(Clone, Debug)]
pub(crate) struct NList {
    item: ItemRef,
}

impl NList {
    pub(crate) fn new(id: Id, store: WeakStoreRef) -> Self {
        let data = ItemData {
            id,
            kind: ItemKind::List,
            ..ItemData::default()
        };

        Self {
            item: ItemRef::new(data.into(), store),
        }
    }

    pub(crate) fn size(&self) -> u32 {
        self.borrow().as_list().len() as u32
    }

    pub(crate) fn field(&self) -> Option<String> {
        self.borrow().field(self.item_ref().store.clone())
    }

    pub(crate) fn content(&self) -> Content {
        let types = self.borrow().as_list();
        Content::Types(types)
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }
}

impl Deref for NList {
    type Target = ItemRef;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl NList {
    fn prepend(&self, item: Type) {
        self.item.append(item)
    }

    fn append(&self, item: impl Into<Type>) {
        self.item.append(item.into())
    }

    pub(crate) fn insert(&self, offset: usize, item: Type) {
        if offset == 0 {
            self.prepend(item);
        } else if offset >= self.size() as usize {
            self.append(item);
        } else {
            // self.item.insert(offset, item)
        }
    }

    fn remove(&self, key: ItemKey) {
        if let ItemKey::Number(offset) = key {
            if offset < self.size() {
                let items = self.borrow().as_list();
                let item = items[offset as usize].clone();
                item.delete();
            }
        }
    }

    pub(crate) fn delete(&self) {
        self.item.delete(1);
    }

    pub(crate) fn clear(&self) {
        let items = self.borrow().as_list();
        for item in items {
            item.delete();
        }
    }

    pub(crate) fn to_json(&self) -> serde_json::Value {
        let mut json = self.borrow().to_json();
        let items = self.borrow().as_list();

        let content = items.iter().map(|item| item.to_json()).collect();

        json.insert("content".to_string(), serde_json::Value::Array(content));

        serde_json::to_value(json).unwrap()
    }
}

impl Serialize for NList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut s = serializer.serialize_struct("List", self.borrow().serialize_size() + 1)?;
        self.borrow().serialize(&mut s)?;

        let items = self.borrow().as_list();
        let content = items
            .iter()
            .map(|item| serde_json::to_value(item).unwrap_or_default())
            .collect();
        s.serialize_field("content", &serde_json::Value::Array(content))?;

        s.end()
    }
}

impl WithId for NList {
    fn id(&self) -> Id {
        self.item.borrow().id()
    }
}

impl WithIdRange for NList {
    fn range(&self) -> IdRange {
        self.borrow().id().range(1)
    }
}

impl From<ItemRef> for NList {
    fn from(item: ItemRef) -> Self {
        Self { item }
    }
}

#[cfg(test)]
mod test {
    use crate::doc::Doc;

    #[test]
    fn test_nlist() {
        let doc = &Doc::default();

        let list = &doc.list();
        doc.set("list", list.clone());
        assert_eq!(list.size(), 0);

        ["a", "b", "c"]
            .iter()
            .map(|s| doc.atom(*s))
            .for_each(|atom| list.append(atom));

        assert_eq!(list.size(), 3);

        let yaml = serde_yaml::to_string(&list).unwrap();

        println!("{}", yaml);

        let expect = r#"id: (1, 0)
kind: list
parent_id: (0, 0)
content:
- content: a
  id: (1, 1)
  kind: atom
  parent_id: (1, 0)
- content: b
  id: (1, 2)
  kind: atom
  left_id: (1, 1)
  parent_id: (1, 0)
- content: c
  id: (1, 3)
  kind: atom
  left_id: (1, 2)
  parent_id: (1, 0)
"#;

        assert_eq!(yaml, expect);
    }

    #[test]
    fn test_list_of_list() {
        let doc = &Doc::default();

        let list1 = &doc.list();
        doc.set("list", list1.clone());
        assert_eq!(list1.size(), 0);

        let list2 = &doc.list();
        list1.append(list2.clone());

        assert_eq!(list1.size(), 1);

        let atom = doc.atom("a");
        list2.append(atom.clone());

        assert_eq!(list2.size(), 1);

        let atom = doc.atom("b");
        list1.append(atom.clone());

        assert_eq!(list1.size(), 2);

        let root = doc.root.clone();
        let yaml = serde_yaml::to_string(&root).unwrap();

        println!("{}", yaml);

        let expect = r#"id: (0, 0)
kind: map
content:
  list:
    content:
    - content:
      - content: a
        id: (1, 2)
        kind: atom
        parent_id: (1, 1)
      id: (1, 1)
      kind: list
      parent_id: (1, 0)
    - content: b
      id: (1, 3)
      kind: atom
      left_id: (1, 1)
      parent_id: (1, 0)
    id: (1, 0)
    kind: list
    parent_id: (0, 0)
"#;
        assert_eq!(yaml, expect);

        println!("{}", serde_yaml::to_string(doc).unwrap());
    }
}
