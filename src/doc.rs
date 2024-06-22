use std::cell::RefCell;
use std::rc::Rc;

use serde::ser::SerializeStruct;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::bimapid::Client;
use crate::diff::Diff;
use crate::id::Id;
use crate::item::{Content, ItemKey};
use crate::mark::{Mark, MarkContent};
use crate::natom::NAtom;
use crate::nlist::NList;
use crate::nmap::NMap;
use crate::nmark::NMark;
use crate::nstring::NString;
use crate::ntext::NText;
use crate::state::ClientState;
use crate::store::{DocStore, StoreRef};
use crate::transaction::Transaction;
use crate::types::Type;

#[derive(Clone, Debug)]
pub(crate) struct DocOpts {
    pub(crate) id: String,
    pub(crate) client_id: Client,
    pub(crate) crated_by: Client,
}

impl Default for DocOpts {
    fn default() -> Self {
        let client_id = Uuid::new_v4().to_string();
        Self {
            id: Uuid::new_v4().to_string(),
            client_id: client_id.clone(),
            crated_by: client_id,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Doc {
    pub(crate) opts: DocOpts,
    pub(crate) root: Option<NMap>,
    pub(crate) store: StoreRef,
}

impl Doc {}

impl Doc {
    pub(crate) fn new(opts: DocOpts) -> Self {
        let mut store = DocStore::default();
        // doc is always created by the client with clock 0,
        // each doc is created by a new client
        store.update_client(&opts.crated_by, 0);
        if opts.client_id != opts.crated_by {
            store.update_client(&opts.client_id, 1);
        }

        let client = store.get_client(&opts.client_id);

        let root_id = Id::new(client, 0);
        let store = Rc::new(RefCell::new(store));
        let weak = Rc::downgrade(&store);
        let root = NMap::new(root_id, weak);
        store.borrow_mut().insert(root.clone().into());

        let mut doc = Self {
            opts,
            store,
            root: None,
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

    #[inline]
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

    pub fn atom(&self, content: impl Into<Content>) -> NAtom {
        let id = self.store.borrow_mut().next_id();
        let atom = NAtom::new(id, content.into(), Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(atom.clone().into());

        atom
    }

    pub fn text(&self) -> NText {
        let id = self.store.borrow_mut().next_id();
        let text = NText::new(id, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(text.clone().into());

        text
    }

    pub fn string(&self, value: impl Into<String>) -> NString {
        let id = self.store.borrow_mut().next_id();
        let string = NString::new(id, value.into(), Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(string.clone().into());

        string
    }

    pub fn mark(&self, content: impl Into<MarkContent>) -> NMark {
        let id = self.store.borrow_mut().next_id();
        let content = Content::Mark(Mark::new(content.into()));
        let mark = NMark::new(id, content, Rc::downgrade(&self.store));
        self.store.borrow_mut().insert(mark.clone().into());

        mark
    }
}

impl Doc {
    #[inline]
    pub(crate) fn add_mark(&self, mark: impl Into<NMark>) {
        self.root.as_ref().unwrap().item_ref().add_mark(mark.into())
    }

    #[inline]
    fn size(&self) -> usize {
        self.root.as_ref().unwrap().size()
    }

    #[inline]
    pub(crate) fn get(&self, key: impl Into<String>) -> Option<Type> {
        self.root.as_ref().unwrap().get(key.into())
    }

    #[inline]
    pub(crate) fn set(&self, key: impl Into<String>, item: impl Into<Type>) {
        let key = key.into();

        self.root.as_ref().unwrap().set(key, item.into());
    }

    fn remove(&self, key: ItemKey) {
        self.root.as_ref().unwrap().remove(key)
    }

    fn keys(&self) -> Vec<String> {
        self.root.as_ref().unwrap().keys()
    }

    fn values(&self) -> Vec<Type> {
        self.root.as_ref().unwrap().values()
    }

    pub(crate) fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();

        map.insert(
            "id".to_string(),
            serde_json::Value::String(self.opts.id.to_string()),
        );
        map.insert(
            "created_by".to_string(),
            serde_json::Value::String(self.opts.crated_by.to_string()),
        );

        if let Some(root) = self.root.as_ref() {
            map.insert("root".to_string(), root.to_json());
        }

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
        if let Some(root) = &self.root {
            size += root.borrow().serialize_size();
        }

        let mut s = serializer.serialize_struct("Doc", size + 1)?;
        s.serialize_field("id", &self.opts.id)?;
        s.serialize_field("created_by", &self.opts.crated_by)?;
        if let Some(root) = &self.root {
            root.borrow().serialize(&mut s)?;
            let map = root.borrow().as_map(Rc::downgrade(&self.store));
            let content = serde_json::to_value(map).unwrap_or_default();
            s.serialize_field("content", &content)?;
        }

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
