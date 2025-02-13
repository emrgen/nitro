use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::index::{BTreeIndex, IBTree, ItemIndexMap};
use crate::item::{Content, ItemData, ItemIterator, ItemKey, ItemKind, ItemRef, Linked};
use crate::store::WeakStoreRef;
use crate::types::Type;
use serde::ser::{Serialize, SerializeStruct};
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Clone, Debug, Default)]
pub struct NList {
    item: ItemRef,
    // TODO: dynamically create/destroy the list when the nlist is too big/small
    list: Rc<RefCell<IBTree>>,
}

impl NList {
    pub(crate) fn on_insert(&self, child: &Type) {
        self.list.borrow_mut().insert(child.clone());
    }
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
            list: Rc::new(RefCell::new(Default::default())),
        }
    }

    pub fn size(&self) -> u32 {
        self.list.borrow().size() as u32
    }

    pub fn get(&self, key: impl Into<ItemKey>) -> Option<Type> {
        if let ItemKey::Number(offset) = key.into() {
            return self.list.borrow().at_index(offset).map(|v| v.clone());
        }
        None
    }

    pub(crate) fn field(&self) -> Option<String> {
        self.borrow().field(self.item_ref().store.clone())
    }

    pub fn content(&self) -> Content {
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
    fn prepend(&self, item: impl Into<Type>) {
        let item = item.into();
        self.item.prepend(item.clone());
        Type::add_frac_index(&item);
        self.on_insert(&item);
    }

    pub(crate) fn append(&self, item: impl Into<Type>) {
        let item = item.into();
        self.item.append(item.clone());
        Type::add_frac_index(&item);
        self.on_insert(&item);
    }

    pub fn insert(&self, offset: u32, item: impl Into<Type>) {
        let size = self.list.borrow().size();
        let typ = item.into();

        if offset == 0 {
            self.prepend(typ);
        } else if offset >= size as u32 {
            self.append(typ);
        } else {
            let list = self.list.borrow();

            // quickly find the item at the offset index using the binary search
            let next = list.at_index(offset);
            if let Some(next) = next {
                next.insert_before(typ);
            } else {
                self.append(typ);
            }
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

        serde_json::Value::Array(content)
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

        let expect = r#"id: (0, 2)
kind: list
parent_id: (0, 1)
content:
- content: a
  id: (0, 3)
  kind: atom
  parent_id: (0, 2)
- content: b
  id: (0, 4)
  kind: atom
  left_id: (0, 3)
  parent_id: (0, 2)
- content: c
  id: (0, 5)
  kind: atom
  left_id: (0, 4)
  parent_id: (0, 2)
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
        id: (0, 4)
        kind: atom
        parent_id: (0, 3)
      id: (0, 3)
      kind: list
      parent_id: (0, 2)
    - content: b
      id: (0, 5)
      kind: atom
      left_id: (0, 3)
      parent_id: (0, 2)
    id: (0, 2)
    kind: list
    parent_id: (0, 1)
"#;
        assert_eq!(yaml, expect);

        // println!("{}", serde_yaml::to_string(doc).unwrap());
    }
}
