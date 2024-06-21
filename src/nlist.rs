use std::ops::Deref;

use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{ItemData, ItemRef};
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

    pub(crate) fn field(&self) -> Option<String> {
        self.borrow().field()
    }

    fn get(&self, index: usize) -> Option<Type> {
        self.borrow().as_list().get(index).cloned()
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

impl IList for NList {
    fn size(&self) -> usize {
        0
    }

    fn prepend(&self, item: Type) {
        self.item.append(item)
    }

    fn append(&self, item: Type) {
        self.item.append(item)
    }

    fn insert(&self, offset: usize, item: Type) {
        if offset == 0 {
            self.prepend(item);
        } else if offset >= self.size() {
            self.append(item);
        } else {
            // self.item.insert(offset, item)
        }
    }

    fn remove(&self, offset: usize) {
        // self.item.append(item)
    }

    fn clear(&self) {
        todo!()
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

pub trait IList {
    fn size(&self) -> usize;
    fn prepend(&self, item: Type);
    fn append(&self, item: Type);
    fn insert(&self, offset: usize, item: Type);
    fn remove(&self, index: usize);
    fn clear(&self);
}
