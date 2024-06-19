use std::sync::{Arc, RwLock};

use crate::clients::ClientId;
use crate::diff::Diff;
use crate::id::Clock;
use crate::item::Content;
use crate::natom::NAtom;
use crate::nlist::NList;
use crate::nmap::NMap;
use crate::nstring::NString;
use crate::ntext::NText;
use crate::state::ClientState;
use crate::store::{DocStore, StoreRef};
use crate::tx::Tx;

#[derive(Clone, Debug)]
pub(crate) struct DocOpts {
    pub(crate) guid: String,
    pub(crate) client_id: ClientId,
    pub(crate) crated_by: ClientId,
}

impl Default for DocOpts {
    fn default() -> Self {
        Self {
            guid: "".to_string(),
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
        let root_id = store.take(1);
        let store = Arc::new(RwLock::new(store));
        let root = NMap::new(root_id, Arc::downgrade(&store));
        store.write().unwrap().insert(root.item_ref());

        let mut doc = Self {
            opts,
            store,
            ..Doc::default()
        };

        doc.root = Some(root);

        doc
    }

    pub fn diff(&self, state: ClientState) -> Diff {
        let store = self.store.read().unwrap();
        store.diff(self.opts.guid.clone(), state)
    }

    pub(crate) fn from_diff(diff: Diff, opts: DocOpts) -> Self {
        let mut doc = Self::new(opts);
        doc.apply(diff);

        doc
    }

    pub(crate) fn apply(&mut self, diff: Diff) {
        let mut tx = Tx::new(Arc::downgrade(&self.store), diff);
        tx.commit();
    }

    pub fn list(&self) -> NList {
        let id = self.store.write().unwrap().take(1);
        let list = NList::new(id, Arc::downgrade(&self.store));
        self.store.write().unwrap().insert(list.item_ref());

        list
    }

    pub fn map(&self) -> NMap {
        let id = self.store.write().unwrap().take(1);
        let map = NMap::new(id, Arc::downgrade(&self.store));
        self.store.write().unwrap().insert(map.item_ref());
        map
    }

    pub fn atom(&self, content: Content) -> NAtom {
        let id = self.store.write().unwrap().take(1);
        let atom = NAtom::new(id, content, Arc::downgrade(&self.store));
        self.store.write().unwrap().insert(atom.item_ref());

        atom
    }

    pub fn text(&self) -> NText {
        let id = self.store.write().unwrap().take(1);
        let text = NText::new(id, Arc::downgrade(&self.store));
        self.store.write().unwrap().insert(text.item_ref());

        text
    }

    pub fn string(&self, string: String) -> NString {
        let id = self.store.write().unwrap().take(string.len() as Clock);
        let str = NString::new(id, string, Arc::downgrade(&self.store));
        self.store.write().unwrap().insert(str.item_ref());

        str
    }
}
