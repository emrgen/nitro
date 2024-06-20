use crate::id::Id;
use crate::item::{Content, ItemData, ItemKind, ItemRef};
use crate::store::WeakStoreRef;

pub(crate) struct NAtom {
    pub(crate) item: ItemRef,
}

impl NAtom {
    pub(crate) fn new(id: Id, content: Content, store: WeakStoreRef) -> Self {
        let data = ItemData {
            kind: ItemKind::Atom,
            id,
            content,
            ..ItemData::default()
        };
        Self {
            item: ItemRef::new(data.into(), store),
        }
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        self.item.clone()
    }
}

impl From<ItemRef> for NAtom {
    fn from(item: ItemRef) -> Self {
        Self { item }
    }
}
