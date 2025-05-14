use crate::id::{Id, IdRange, WithId, WithIdRange, WithTarget};
use crate::item::{Content, ItemData, ItemIterator, ItemKey, ItemKind, ItemRef};
use crate::nlist::NList;
use crate::nproxy::NProxy;
use crate::store::WeakStoreRef;
use crate::Type;
use fake::Opt;
use serde::ser::SerializeStruct;
use serde::Serialize;
use serde_json::Value;
use std::ops::Deref;

/// NMove represents a move operation in the document.
/// It is similar to proxy but one instance is valid for multiple move operations.
#[derive(Debug, Clone)]
pub(crate) struct NMove {
    pub(crate) item: ItemRef,
}

impl NMove {
    pub(crate) fn new(id: Id, target: Type, store: WeakStoreRef) -> NMove {
        let data = ItemData {
            id,
            kind: ItemKind::Move,
            ..ItemData::default()
        };

        let mut item = ItemRef::new(data.into(), store);
        item.set_target(target);

        Self { item }
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }

    pub(crate) fn content(&self) -> Content {
        if let Some(target) = self.get_target().as_ref() {
            target.content()
        } else {
            Content::Null
        }
    }

    pub(crate) fn size(&self) -> u32 {
        if let Some(target) = self.get_target().as_ref() {
            target.size()
        } else {
            0
        }
    }

    fn get(&self, key: ItemKey) -> Option<Type> {
        if let Some(target) = self.get_target().as_ref() {
            target.get(key)
        } else {
            None
        }
    }

    fn set(&self, key: String, item: Type) {
        if let Some(target) = self.get_target().as_ref() {
            target.set(key, item);
        }
    }

    fn delete(&self) {
        self.item_ref().delete(1);
    }

    fn prepend(&self, item: Type) {
        if let Some(target) = self.get_target().as_ref() {
            target.prepend(item);
        }
    }

    fn append(&self, item: Type) {
        if let Some(target) = self.get_target().as_ref() {
            target.append(item);
        }
    }

    fn insert(&self, offset: u32, item: Type) {
        if let Some(target) = self.get_target().as_ref() {
            target.insert(offset, item);
        }
    }

    fn remove(&self, key: ItemKey) {
        if let Some(target) = self.get_target().as_ref() {
            target.remove(key);
        }
    }

    fn clear(&self) {
        if let Some(target) = self.get_target().as_ref() {
            target.clear();
        }
    }

    pub(crate) fn to_json(&self) -> serde_json::Value {
        if let Some(target) = self.get_target().as_ref() {
            serde_json::Value::Null
            // target.to_json()
        } else {
            serde_json::Value::Null
        }
    }
}

impl Serialize for NMove {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        self.get_target().serialize(serializer).into()
    }
}

impl WithId for NMove {
    #[inline]
    fn id(&self) -> Id {
        self.item.borrow().id()
    }
}

impl WithIdRange for NMove {
    fn range(&self) -> IdRange {
        self.borrow().id().range(1)
    }
}

impl From<ItemRef> for NMove {
    fn from(item: ItemRef) -> Self {
        Self { item }
    }
}

impl Deref for NMove {
    type Target = ItemRef;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{print_yaml, Doc};

    fn get_text(item: ItemRef) -> String {
        if let Some(target) = item.get_target() {
            target.text_content()
        } else {
            item.text_content()
        }
    }

    fn get_list_text(list: &NList) -> Vec<String> {
        list.visible_item_iter().map(get_text).collect()
    }

    #[test]
    fn test_move_atom_in_a_list() {
        let doc = Doc::default();
        let list = doc.list();
        doc.set("list", list.clone());
        let a = doc.atom("a");
        let b = doc.atom("b");
        let c = doc.atom("c");

        list.append(a.clone());
        list.append(b.clone());
        list.append(c.clone());

        print_yaml(&list);

        let at: Type = a.into();
        at.move_to(&list, 3);
        assert_eq!(get_list_text(&list), vec!["b", "c", "a"]);

        at.move_to(&list, 0);
        assert_eq!(get_list_text(&list), vec!["a", "b", "c"]);
    }

    #[test]
    fn test_move_atom_between_lists() {
        let doc = Doc::default();
        let l1 = doc.list();
        let l2 = doc.list();

        doc.set("l1", l1.clone());
        doc.set("l2", l2.clone());

        let a = doc.atom("a");
        let b = doc.atom("b");

        l1.append(a.clone());
        l1.append(b.clone());

        let c = doc.atom("c");
        let d = doc.atom("d");

        l2.append(c.clone());
        l2.append(d.clone());

        let a1: Type = a.into();
        a1.move_to(&l2, 0);

        assert_eq!(get_list_text(&l1), vec!["b"]);
        assert_eq!(get_list_text(&l2), vec!["a", "c", "d"]);

        let b1: Type = b.into();
        b1.move_to(&l2, 2);

        assert_eq!(get_list_text(&l1), vec![] as Vec<String>);
        assert_eq!(get_list_text(&l2), vec!["a", "c", "b", "d"]);
    }
}
