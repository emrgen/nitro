use std::cell::RefCell;
use std::rc::Rc;

use serde::ser::SerializeStruct;
use serde::Serialize;
use serde_json::Value;
use uuid::Uuid;

use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::diff::Diff;
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::id::Id;
use crate::item::{Content, DocProps, ItemKey};
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
use crate::{Client, ClockTick};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocOpts {
    pub id: DocId,
    pub crated_by: Client,
}

#[derive(Default, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DocId(Uuid);

impl From<&DocId> for DocId {
    fn from(value: &DocId) -> Self {
        value.clone()
    }
}

impl DocId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn from_bytes(bytes: &[u8; 16]) -> Self {
        let uuid = Uuid::from_slice(bytes).unwrap();
        Self(uuid)
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        Uuid::parse_str(s)
            .map(|uuid| Self(uuid))
            .map_err(|e| e.to_string())
    }

    pub fn to_string(&self) -> String {
        self.0.to_string()
    }

    pub fn as_bytes(&self) -> [u8; 16] {
        self.0.as_bytes().to_owned().try_into().unwrap()
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Encode for DocId {
    fn encode<T: Encoder>(&self, e: &mut T, ctx: &mut EncodeContext) {
        e.uuid(self.0.as_bytes().as_slice());
    }
}

impl Decode for DocId {
    fn decode<T: Decoder>(d: &mut T, ctx: &DecodeContext) -> Result<Self, String>
    where
        Self: Sized,
    {
        let bytes = d.uuid()?;
        let uuid = Uuid::from_slice(&bytes).map_err(|e| e.to_string())?;

        Ok(Self(uuid))
    }
}

impl Serialize for DocId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

impl Default for DocOpts {
    fn default() -> Self {
        let client_id = Uuid::new_v4().into();
        Self {
            id: DocId(Uuid::new_v4()),
            crated_by: client_id,
        }
    }
}

#[derive(Debug, Clone, Eq)]
pub struct Doc {
    pub(crate) opts: DocOpts,
    pub(crate) root: NMap,
    pub(crate) store: StoreRef,
}

impl Doc {
    pub(crate) fn state(&self) -> ClientState {
        let store = self.store.borrow();

        let state = &store.state;
        store.state.clone()
    }
}

impl Doc {
    pub(crate) fn new(opts: DocOpts) -> Self {
        let mut store = DocStore::default();

        store.doc_id = opts.id.clone();
        store.created_by = opts.crated_by.clone();

        // doc is always created by the client with clock 0,
        store.update_client(&opts.crated_by, 1);

        let client = store.get_client(&opts.crated_by);
        let root_id = store.next_id();

        // let client = Uuid::new_v4().to_string();
        // store.update_client(&client, 1);

        let store_ref = Rc::new(RefCell::new(store));
        let weak = Rc::downgrade(&store_ref);
        let root = NMap::new(root_id, weak);

        root.set_content(DocProps::new(opts.id.clone(), opts.crated_by.clone()));

        store_ref.borrow_mut().insert(root.clone());

        Self {
            opts,
            store: store_ref,
            root,
        }
    }

    pub fn id(&self) -> DocId {
        self.opts.id.clone()
    }

    // create a new doc from a diff
    pub(crate) fn from_diff(diff: &Diff) -> Option<Doc> {
        if let Some(root) = &diff.get_root() {
            if let Content::Doc(content) = &root.content {
                let doc = Doc::new(DocOpts {
                    id: content.id.clone(),
                    crated_by: content.created_by.clone().into(),
                });

                doc.apply(diff.clone());

                return Some(doc);
            }
        }

        None
    }

    #[inline]
    pub fn diff(&self, state: impl Into<ClientState>) -> Diff {
        let mut diff = self.store.borrow().diff(
            self.opts.id.clone(),
            self.opts.crated_by.clone(),
            state.into(),
        );
        diff.optimize();

        diff
    }

    pub(crate) fn apply(&self, diff: Diff) {
        let mut tx = Transaction::new(Rc::downgrade(&self.store.clone()), diff);
        tx.commit();
    }

    pub fn find_by_id(&self, id: &Id) -> Option<Type> {
        self.store.borrow().find(id)
    }

    pub fn update_client(&self) -> Client {
        let client_id = Uuid::new_v4().into();
        self.store.borrow_mut().update_client(&client_id, 1);

        client_id
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
        let content = value.into();
        let id = self
            .store
            .borrow_mut()
            .next_id_range(content.len() as ClockTick)
            .start_id();
        let string = NString::new(id, content, Rc::downgrade(&self.store));
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
    pub fn get(&self, key: impl Into<String>) -> Option<Type> {
        self.root.get(key.into())
    }

    #[inline]
    pub fn set(&self, key: impl Into<String>, item: impl Into<Type>) {
        let key = key.into();

        self.root.set(key, item.into());
    }

    #[inline]
    fn remove(&self, key: ItemKey) {
        self.root.remove(key)
    }

    fn keys(&self) -> Vec<String> {
        self.root.keys()
    }

    fn values(&self) -> Vec<Type> {
        self.root.values()
    }

    pub fn version(&self) -> ClientState {
        self.store.borrow().state.clone()
    }

    pub(crate) fn to_json(&self) -> Value {
        let mut map = serde_json::Map::new();

        map.insert(
            "id".to_string(),
            serde_json::Value::String(self.opts.id.0.to_string()),
        );
        map.insert(
            "created_by".to_string(),
            serde_json::Value::String(self.opts.crated_by.to_string()),
        );

        match self.root.to_json() {
            Value::Object(root) => {
                for (key, value) in root {
                    map.insert(key, value);
                }
            }
            _ => {}
        }

        serde_json::Value::Object(map)
    }
}

impl Default for Doc {
    fn default() -> Self {
        Doc::new(Default::default())
    }
}
impl From<Doc> for ClientState {
    fn from(value: Doc) -> Self {
        value.state()
    }
}

impl From<&Doc> for ClientState {
    fn from(value: &Doc) -> Self {
        value.state()
    }
}

impl PartialEq for Doc {
    fn eq(&self, other: &Self) -> bool {
        let d1 = self.diff(ClientState::default());
        let d2 = other.diff(ClientState::default());

        d1 == d2
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
        s.serialize_field("doc_id", &self.opts.id)?;
        s.serialize_field("created_by", &self.opts.crated_by)?;
        s.serialize_field("root", &root)?;

        s.end()
    }
}

impl Encode for Doc {
    fn encode<T: Encoder>(&self, e: &mut T, ctx: &mut EncodeContext) {
        let diff = self.diff(ClientState::default());
        diff.encode(e, ctx)
    }
}

pub trait CloneDeep {
    fn clone_deep(&self) -> Self;
}

impl CloneDeep for Doc {
    fn clone_deep(&self) -> Self {
        let doc = Doc::new(self.opts.clone());
        let diff = self.diff(ClientState::default());

        doc.apply(diff);

        doc
    }
}

#[cfg(test)]
mod test {
    use byte_unit::{AdjustedByte, Byte, Unit};
    use fake::faker::lorem::en::Words;
    use fake::Fake;
    use miniz_oxide::deflate::compress_to_vec;
    use rand::random;

    use crate::codec_v1::EncoderV1;
    use crate::doc::{CloneDeep, Doc};
    use crate::encoder::{Encode, Encoder};
    use crate::state::ClientState;

    #[test]
    fn test_create_doc() {
        let doc = Doc::new(Default::default());
        assert_eq!(doc.size(), 0);

        let atom = doc.atom("world");
        doc.set("hello", atom.clone());

        assert_eq!(doc.size(), 1);

        // let atom = doc.atom("hudrogen");
        // doc.set("el", atom.clone());
        //
        // let atom = doc.atom("oxygen");
        // doc.set("el", atom.clone());
        //
        // assert_eq!(doc.size(), 2);
        //
        // // let json_str = serde_json::to_string_pretty(&doc.to_json()).unwrap();
        // let yaml = serde_yaml::to_string(&doc.to_json()).unwrap();
        // println!("{}", yaml);
    }

    fn get_doc_encoding(lines: u32, words: u32) -> Vec<u8> {
        let words = words as usize;
        let doc = Doc::default();
        let list = doc.list();
        doc.set("list", list.clone());
        for _ in 0..lines {
            let text = doc.text();
            let words: Vec<String> = Words(words..words + 1).fake();
            words.iter().for_each(|word| {
                text.append(doc.string(word.to_string()));
            });

            let size = list.size();
            if size == 0 {
                list.append(text);
            } else {
                let offset = random::<u32>() % size;
                list.insert(offset, text);
            }
        }

        let mut encoder = EncoderV1::new();
        doc.encode(&mut encoder, &mut Default::default());

        encoder.buffer()
    }

    fn to_unit(size: usize) -> AdjustedByte {
        let byte = Byte::from_u64(size as u64);
        byte.get_adjusted_unit(Unit::KB)
    }

    #[test]
    fn test_encode_doc_size() {
        println!(
            "{:6}   {:6}   {:10}   {:10}   {:8}",
            "lines", "words", "size", "comp", "comp ratio"
        );
        println!(
            "{:6}   {:6}   {:10}   {:8}   {:8}",
            "------", "------", "----------", "----------", "----------"
        );
        for i in 0..20 {
            let lines = 10 * i;
            let words = 20;
            let buf = get_doc_encoding(lines, words);
            let comp = compress_to_vec(&buf, 1);
            println!(
                "{:6}   {:6}   {:10.2}   {:10.2}   {:8.2}%",
                lines,
                lines * words,
                to_unit(buf.len()),
                to_unit(comp.len()),
                100f32 * ((buf.len() as f32 - comp.len() as f32) / buf.len() as f32)
            );
        }
    }

    fn eq_doc(a: &Doc, b: &Doc) -> bool {
        let mut e1 = EncoderV1::new();
        let mut e2 = EncoderV1::new();

        let d1 = a.diff(ClientState::default());
        let d2 = b.diff(ClientState::default());

        // println!("d1: {:?}", d1);
        // println!("d2: {:?}", d2);

        d1 == d2
    }

    fn print_doc(doc: &Doc) {
        let yaml = serde_yaml::to_string(&doc).unwrap();
        println!("{}", yaml);
    }

    #[test]
    fn test_clone_doc_with_map() {
        let d1 = Doc::default();
        d1.set("a", d1.atom("a"));
        d1.set("b", d1.atom("b"));
        d1.set("c", d1.atom("c"));
        d1.set("d", d1.atom("d"));

        let d2 = d1.clone_deep();

        let left = serde_yaml::to_string(&d1).unwrap();
        let right = serde_yaml::to_string(&d2).unwrap();

        // println!("left: {}", left);
        // println!("right: {}", right);

        assert_eq!(left, right);
    }

    #[test]
    fn test_clone_doc_with_list() {
        let d1 = Doc::default();
        let list = d1.list();
        d1.set("list", list.clone());

        list.append(d1.atom("a"));
        list.append(d1.atom("b"));

        let d2 = d1.clone_deep();

        // print_yaml(&d1);
        // print_yaml(&d2);

        let left = serde_yaml::to_string(&d1).unwrap();
        let right = serde_yaml::to_string(&d2).unwrap();

        assert_eq!(left, right);
    }

    #[test]
    fn test_clone_doc_with_text() {
        let d1 = Doc::default();
        let text = d1.text();
        d1.set("text", text.clone());

        text.append(d1.string("a"));
        text.append(d1.string("b"));
        text.append(d1.string("c"));
        text.prepend(d1.string("d"));
        text.insert(1, d1.string("e"));

        let d2 = d1.clone_deep();

        let left = serde_yaml::to_string(&d1).unwrap();
        let right = serde_yaml::to_string(&d2).unwrap();

        // println!("left: {}", left);
        // println!("right: {}", right);

        assert_eq!(left, right);
    }

    #[test]
    fn test_item_depth() {
        let d1 = Doc::default();
        let list = d1.list();
        d1.set("list", list.clone());

        let a1 = d1.atom("a1");
        let a2 = d1.atom("a2");

        list.append(a1.clone());
        list.append(a2.clone());

        let list2 = d1.list();

        list.append(list2.clone());

        let a3 = d1.atom("a3");
        let a4 = d1.atom("a4");

        list2.append(a3.clone());
        list2.append(a4.clone());

        assert_eq!(list.depth(), 1);
        assert_eq!(a1.depth(), 2);
        assert_eq!(a3.depth(), 3);
    }
}
