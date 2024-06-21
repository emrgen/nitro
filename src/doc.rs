use std::cell::RefCell;
use std::rc::Rc;

use crate::clients::ClientId;
use crate::diff::Diff;
use crate::item::Content;
use crate::natom::NAtom;
use crate::nlist::NList;
use crate::nmap::{IMap, NMap};
use crate::nstring::NString;
use crate::ntext::NText;
use crate::state::ClientState;
use crate::store::{DocStore, StoreRef};
use crate::transaction::Transaction;
use crate::types::Type;

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
        // doc is always created by the client with clock 0,
        // each doc is created by a new client
        store.update_client(opts.crated_by.clone(), 0);
        if opts.client_id != opts.crated_by {
            store.update_client(opts.client_id.clone(), 1);
        }

        let root_id = store.next_id();
        let store = Rc::new(RefCell::new(store));
        let weak = Rc::downgrade(&store);
        let root = NMap::new(root_id, weak);
        store.borrow_mut().insert(root.clone().into());

        let mut doc = Self {
            opts,
            store,
            ..Doc::default()
        };

        doc.root = Some(root);

        doc
    }

    // create a new doc from a diff
    pub(crate) fn from_diff(diff: Diff, opts: DocOpts) -> Self {
        let mut doc = Self::new(opts);
        doc.apply(diff);

        doc
    }

    pub fn diff(&self, state: ClientState) -> Diff {
        self.store.borrow().diff(self.opts.id.clone(), state)
    }

    pub(crate) fn apply(&mut self, diff: Diff) {
        let mut tx = Transaction::new(Rc::downgrade(&self.store), diff);
        tx.commit();
    }

    pub fn list(&self) -> NList {
        let id = self.store.borrow_mut().next_id();
        let list = NList::new(id, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(list.clone().into());

        list
    }

    pub fn map(&self) -> NMap {
        let id = self.store.borrow_mut().next_id();
        let map = NMap::new(id, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(map.clone().into());

        map
    }

    pub fn atom(&self, content: Content) -> NAtom {
        let id = self.store.borrow_mut().next_id();
        let atom = NAtom::new(id, content, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(atom.clone().into());

        atom
    }

    pub fn text(&self) -> NText {
        let id = self.store.borrow_mut().next_id();
        let text = NText::new(id, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(text.clone().into());

        text
    }

    pub fn string(&self, string: String) -> NString {
        let id = self.store.borrow_mut().next_id();
        let string = NString::new(id, string, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(string.clone().into());

        string
    }
}

impl IMap for Doc {
    fn size(&self) -> usize {
        self.root.as_ref().unwrap().size()
    }

    fn get(&self, key: String) -> Option<Type> {
        self.root.as_ref().unwrap().get(key)
    }

    fn set(&self, key: String, item: Type) {
        self.root.as_ref().unwrap().set(key, item)
    }

    fn remove(&self, key: String) {
        self.root.as_ref().unwrap().remove(key)
    }

    fn keys(&self) -> Vec<String> {
        self.root.as_ref().unwrap().keys()
    }

    fn values(&self) -> Vec<Type> {
        self.root.as_ref().unwrap().values()
    }
}
