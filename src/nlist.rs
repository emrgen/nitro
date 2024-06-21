use std::ops::Deref;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemData, ItemKey, ItemRef};
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
            ..ItemData::default()
        };

        Self {
            item: ItemRef::new(data.into(), store),
        }
    }

    pub(crate) fn size(&self) -> usize {
        self.borrow().as_list().len()
    }

    pub(crate) fn field(&self) -> Option<String> {
        self.borrow().field()
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

    fn append(&self, item: Type) {
        self.item.append(item)
    }

    pub(crate) fn insert(&self, offset: usize, item: Type) {
        if offset == 0 {
            self.prepend(item);
        } else if offset >= self.size() {
            self.append(item);
        } else {
            // self.item.insert(offset, item)
        }
    }

    fn remove(&self, key: ItemKey) {
        match key {
            ItemKey::Number(offset) => {
                if offset < self.size() {
                    let items = self.borrow().as_list();
                    let item = items[offset].clone();
                    item.delete();
                }
            }
            _ => {}
        }
    }

    fn clear(&self) {
        let items = self.borrow().as_list();
        for item in items {
            item.delete();
        }
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
