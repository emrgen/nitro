use crate::id::Id;
use crate::item::{ItemData, ItemKey, ItemRef};
use crate::store::WeakStoreRef;

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
        self.item.borrow().field()
    }

    pub(crate) fn size(&self) -> usize {
        0
    }

    pub(crate) fn append(&mut self, _item: ItemRef) {
        // self.item.append(item)
    }

    pub(crate) fn prepend(&mut self, _item: ItemRef) {
        // self.item.append(item)
    }

    pub(crate) fn insert(&mut self, key: &ItemKey, _item: ItemRef) {
        match key {
            ItemKey::Number(offset) => {
                if *offset == 0 {
                    self.prepend(_item);
                } else if *offset >= self.size() {
                    self.append(_item);
                } else {
                    panic!("insert: not implemented")
                }
            }
            ItemKey::String(_) => panic!("insert: not implemented"),
        }
    }

    fn insert_at(&mut self, offset: usize, _item: ItemRef) {
        // self.item.append(item)
    }

    pub(crate) fn remove(&mut self, offset: usize) {
        // self.item.append(item)
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }
}

impl From<ItemRef> for NList {
    fn from(item: ItemRef) -> Self {
        Self { item }
    }
}
