use crate::change::Change;
use crate::natom::NAtom;
use crate::nlist::NList;
use crate::store::WeakStoreRef;
use crate::{Content, Id, Type};
use std::rc::Rc;

pub(crate) struct Transaction {
    store: WeakStoreRef,
    change: Change,
}

impl Transaction {
    pub fn new(store: WeakStoreRef) -> Self {
        let change = Change::default();

        Self { store, change }
    }

    /// Create a new atom with the given content.
    pub fn atom(&mut self, content: impl Into<Content>) -> NAtom {
        let atom = NAtom::new(self.next_id(), content.into(), self.store.clone());
        self.insert(atom.clone());

        atom
    }

    pub fn list(&mut self) -> NList {
        let list = NList::new(self.next_id(), self.store.clone());
        self.insert(list.clone());

        list
    }

    fn next_id(&mut self) -> Id {
        self.store.upgrade().unwrap().borrow_mut().next_id()
    }

    fn insert(&mut self, item: impl Into<Type>) {
        self.store.upgrade().unwrap().borrow_mut().insert(item);
    }

    /// Commit the transaction to the store.
    pub fn commit(&mut self) {
        let store = self.store.upgrade().unwrap();
        store.borrow_mut().changes.insert(self.change.id.clone());
    }
}

pub(crate) struct ChangeTx {
    pub(crate) store: WeakStoreRef,
    pub(crate) change: Change,
}

impl ChangeTx {
    fn new(store: WeakStoreRef, change: Change) -> Self {
        Self { store, change }
    }

    // apply the change items to the document store
    // the items should be applied in the tick order
    fn commit(&mut self) {
        let store = self.store.upgrade().unwrap().borrow_mut();
    }

    fn rollback(&mut self) {}
}
