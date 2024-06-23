use std::cell::RefCell;
use std::rc::Rc;

use serde::ser::SerializeStruct;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::bimapid::Client;
use crate::diff::Diff;
use crate::id::Id;
use crate::item::{Content, DocContent, ItemKey};
use crate::mark::Mark;
use crate::natom::NAtom;
use crate::nlist::NList;
use crate::nmap::NMap;
use crate::nstring::NString;
use crate::ntext::NText;
use crate::state::ClientState;
use crate::store::{DocStore, StoreRef};
use crate::transaction::Transaction;
use crate::types::Type;

#[derive(Clone, Debug)]
pub(crate) struct DocOpts {
    pub(crate) guid: String,
    pub(crate) crated_by: Client,
}

impl Default for DocOpts {
    fn default() -> Self {
        let client_id = Uuid::new_v4().to_string();
        Self {
            guid: Uuid::new_v4().to_string(),
            crated_by: client_id,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Doc {
    pub(crate) opts: DocOpts,
    pub(crate) root: NMap,
    pub(crate) store: StoreRef,
}

impl Doc {}

impl Doc {
    pub(crate) fn new(opts: DocOpts) -> Self {
        let mut store = DocStore::default();
        // doc is always created by the client with clock 0,
        // each doc is created by a new client

        let client = store.get_client(&opts.crated_by);
        let root_id = Id::new(client, 0);

        let client = Uuid::new_v4().to_string();
        store.update_client(&client, 0);

        let store_ref = Rc::new(RefCell::new(store));
        let weak = Rc::downgrade(&store_ref);
        let root = NMap::new(root_id, weak);

        root.set_content(DocContent::new(opts.guid.clone(), opts.crated_by.clone()));

        store_ref.borrow_mut().insert(root.clone());

        Self {
            opts,
            store: store_ref,
            root,
        }
    }

    // create a new doc from a diff
    pub(crate) fn from_diff(diff: Diff, opts: DocOpts) -> Self {
        let mut doc = Self::new(opts);
        doc.apply(diff);

        doc
    }

    #[inline]
    pub fn diff(&self, state: ClientState) -> Diff {
        self.store.borrow().diff(self.opts.guid.clone(), state)
    }

    pub(crate) fn apply(&mut self, diff: Diff) {
        let mut tx = Transaction::new(Rc::downgrade(&self.store), diff);
        tx.commit();
    }

    pub fn find_by_id(&self, id: Id) -> Option<Type> {
        self.store.borrow().find(id)
    }

    pub fn list(&self) -> NList {
        let id = self.store.borrow_mut().next_id();
        let list = NList::new(id, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(list.clone());

        list
    }

    pub fn map(&self) -> NMap {
        let id = self.store.borrow_mut().next_id();
        let map = NMap::new(id, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(map.clone());

        map
    }

    pub fn atom(&self, content: impl Into<Content>) -> NAtom {
        let id = self.store.borrow_mut().next_id();
        let atom = NAtom::new(id, content.into(), Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(atom.clone());

        atom
    }

    pub fn text(&self) -> NText {
        let id = self.store.borrow_mut().next_id();
        let text = NText::new(id, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(text.clone());

        text
    }

    pub fn string(&self, value: impl Into<String>) -> NString {
        let id = self.store.borrow_mut().next_id();
        let string = NString::new(id, value.into(), Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(string.clone());

        string
    }
}

impl Doc {
    #[inline]
    pub(crate) fn add_mark(&self, mark: Mark) {
        self.root.add_mark(mark);
    }

    #[inline]
    fn size(&self) -> u32 {
        self.root.size()
    }

    #[inline]
    pub(crate) fn get(&self, key: impl Into<String>) -> Option<Type> {
        self.root.get(key.into())
    }

    #[inline]
    pub(crate) fn set(&self, key: impl Into<String>, item: impl Into<Type>) {
        let key = key.into();

        self.root.set(key, item.into());
    }

    fn remove(&self, key: ItemKey) {
        self.root.remove(key)
    }

    fn keys(&self) -> Vec<String> {
        self.root.keys()
    }

    fn values(&self) -> Vec<Type> {
        self.root.values()
    }

    pub(crate) fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();

        map.insert(
            "id".to_string(),
            serde_json::Value::String(self.opts.guid.to_string()),
        );
        map.insert(
            "created_by".to_string(),
            serde_json::Value::String(self.opts.crated_by.to_string()),
        );

        map.insert("root".to_string(), self.root.to_json());

        serde_json::Value::Object(map)
    }
}

impl Default for Doc {
    fn default() -> Self {
        Doc::new(Default::default())
    }
}

impl Serialize for Doc {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut size = 2;
        let root = self.root.clone();
        size += root.borrow().serialize_size();

        let mut s = serializer.serialize_struct("Doc", size + 1)?;
        s.serialize_field("guid", &self.opts.guid)?;
        s.serialize_field("created_by", &self.opts.crated_by)?;
        s.serialize_field("root", &root)?;

        s.end()
    }
}

#[cfg(test)]
mod test {
    use crate::doc::Doc;

    #[test]
    fn test_create_doc() {
        let doc = Doc::new(Default::default());
        assert_eq!(doc.size(), 0);

        let atom = doc.atom("world");
        doc.set("hello", atom.clone());

        assert_eq!(doc.size(), 1);

        let atom = doc.atom("hudrogen");
        doc.set("el", atom.clone());

        let atom = doc.atom("oxygen");
        doc.set("el", atom.clone());

        assert_eq!(doc.size(), 2);

        // let json_str = serde_json::to_string_pretty(&doc.to_json()).unwrap();
        let yaml = serde_yaml::to_string(&doc.to_json()).unwrap();
        println!("{}", yaml);
    }
}
