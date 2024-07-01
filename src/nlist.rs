use std::default::Default;
use std::ops::Deref;

use serde::ser::{Serialize, SerializeStruct};

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemData, ItemIterator, ItemKey, ItemKind, ItemRef, Linked};
use crate::store::WeakStoreRef;
use crate::tree::IndexTree;
use crate::types::Type;

#[derive(Clone, Debug, Default)]
pub(crate) struct NList {
    item: ItemRef,
    cache: Option<Vec<Type>>,
    list: IndexTree,
}

impl NList {
    pub(crate) fn on_insert(&self, child: &Type) {}
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
            cache: None,
            list: IndexTree::new(),
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

    pub(crate) fn insert(&self, offset: u32, item: impl Into<Type>) {
        let items = self.borrow().as_list();
        if offset == 0 {
            self.prepend(item.into());
        } else if offset >= items.len() as u32 {
            self.append(item.into());
        } else {
            let next = items[offset as usize].clone();

            next.insert_before(item.into());
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
        self.serialize_with(&mut s)?;

        let content = self
            .visible_item_iter()
            .map(|item| serde_json::to_value(item).unwrap_or_default())
            .collect::<Vec<_>>();

        s.serialize_field("content", &serde_json::Value::Array(content))?;

        s.end()
    }
}

impl WithId for NList {
    #[inline]
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
        Self {
            item,
            ..Default::default()
        }
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

        let expect = r#"id: (1, 1)
kind: list
parent_id: (0, 1)
content:
- content: a
  id: (1, 2)
  kind: atom
  parent_id: (1, 1)
- content: b
  id: (1, 3)
  kind: atom
  left_id: (1, 2)
  parent_id: (1, 1)
- content: c
  id: (1, 4)
  kind: atom
  left_id: (1, 3)
  parent_id: (1, 1)
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
        // println!("{}", yaml);

        let expect = r#"id: (0, 1)
kind: map
content:
  list:
    content:
    - content:
      - content: a
        id: (1, 3)
        kind: atom
        parent_id: (1, 2)
      id: (1, 2)
      kind: list
      parent_id: (1, 1)
    - content: b
      id: (1, 4)
      kind: atom
      left_id: (1, 2)
      parent_id: (1, 1)
    id: (1, 1)
    kind: list
    parent_id: (0, 1)
"#;
        assert_eq!(yaml, expect);

        // println!("{}", serde_yaml::to_string(doc).unwrap());
    }
}
