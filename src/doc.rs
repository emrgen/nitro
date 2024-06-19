use std::sync::Arc;

use crate::atom::NAtom;
use crate::clients::ClientId;
use crate::diff::Diff;
use crate::id::Clock;
use crate::item::Content;
use crate::list::NList;
use crate::map::NMap;
use crate::state::ClientState;
use crate::store::StoreRef;
use crate::string::NString;
use crate::text::NText;

#[derive(Clone, Debug)]
pub(crate) struct DocOpts {
    pub(crate) guid: String,
    pub(crate) client_id: ClientId,
}

impl Default for DocOpts {
    fn default() -> Self {
        Self {
            guid: "".to_string(),
            client_id: "".to_string(),
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
        let store = StoreRef::default();
        store
            .write()
            .unwrap()
            .update_client(opts.client_id.clone(), 0);

        let mut doc = Self {
            opts,
            store,
            ..Doc::default()
        };

        doc.root = Some(doc.map());

        doc
    }

    pub fn diff(&self, state: ClientState) -> Diff {
        let store = self.store.read().unwrap();
        store.diff(self.opts.guid.clone(), state)
    }

    pub(crate) fn from_diff(diff: &Diff, opts: DocOpts) -> Self {
        let store = StoreRef::default();
        store
            .write()
            .unwrap()
            .update_client(opts.client_id.clone(), 1);
        let mut doc = Doc {
            opts,
            store,
            ..Doc::default()
        };

        doc.apply(diff);
        doc
    }

    pub(crate) fn apply(&mut self, diff: &Diff) {
        // create a new tx
        // Tx::new();
        // let mut store = self.store.write().unwrap();
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
