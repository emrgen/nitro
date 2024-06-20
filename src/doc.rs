use std::cell::RefCell;
use std::rc::Rc;

use crate::clients::ClientId;
use crate::diff::Diff;
use crate::item::Content;
use crate::natom::NAtom;
use crate::nlist::NList;
use crate::nmap::NMap;
use crate::nstring::NString;
use crate::ntext::NText;
use crate::state::ClientState;
use crate::store::{DocStore, StoreRef};
use crate::transaction::Transaction;

#[derive(Clone, Debug)]
pub(crate) struct DocOpts {
    pub(crate) id: String,
    pub(crate) client_id: ClientId,
    pub(crate) crated_by: ClientId,
}

impl Default for DocOpts {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            client_id: "".to_string(),
            crated_by: "".to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Doc {
    pub(crate) opts: DocOpts,
    pub(crate) root: Option<NMap>,
    pub(crate) store: StoreRef,
}

impl Doc {
    pub(crate) fn new(opts: DocOpts) -> Self {
        let mut store = DocStore::default();
        store.update_client(opts.crated_by.clone(), 0);
        let root_id = store.next_id();
        let store = Rc::new(RefCell::new(store));
        let weak = Rc::downgrade(&store);
        let root = NMap::new(root_id, weak);
        store.borrow_mut().insert(root.item_ref());

        let mut doc = Self {
            opts,
            store,
            ..Doc::default()
        };

        doc.root = Some(root);

        doc
    }

    pub fn diff(&self, state: ClientState) -> Diff {
        self.store.borrow().diff(self.opts.id.clone(), state)
    }

    pub(crate) fn from_diff(diff: Diff, opts: DocOpts) -> Self {
        let mut doc = Self::new(opts);
        doc.apply(diff);

        doc
    }

    pub(crate) fn apply(&mut self, diff: Diff) {
        let mut tx = Transaction::new(Rc::downgrade(&self.store), diff);
        tx.commit();
    }

    pub fn list(&self) -> NList {
        let id = self.store.borrow_mut().next_id();
        let list = NList::new(id, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(list.item_ref());

        list
    }

    pub fn map(&self) -> NMap {
        let id = self.store.borrow_mut().next_id();
        let map = NMap::new(id, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(map.item_ref());

        map
    }

    pub fn atom(&self, content: Content) -> NAtom {
        let id = self.store.borrow_mut().next_id();
        let atom = NAtom::new(id, content, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(atom.item_ref());

        atom
    }

    pub fn text(&self) -> NText {
        let id = self.store.borrow_mut().next_id();
        let text = NText::new(id, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(text.item_ref());

        text
    }

    pub fn string(&self, string: String) -> NString {
        let id = self.store.borrow_mut().next_id();
        let string = NString::new(id, string, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(string.item_ref());

        string
    }
}
