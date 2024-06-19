use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

use crate::codec::decoder::{Decode, Decoder};
use crate::codec::encoder::{Encode, Encoder};
use crate::delete::DeleteItem;
use crate::id::{Clock, Id, Split, WithId};
use crate::store::{DocStore, WeakStoreRef};

type ItemRefInner = Rc<Item>;

#[derive(Debug, Clone, Default)]
pub(crate) struct ItemRef {
    pub(crate) store: WeakStoreRef,
    pub(crate) item: ItemRefInner,
}

impl ItemRef {
    pub(crate) fn new(item: ItemRefInner, store: WeakStoreRef) -> Self {
        Self { item, store }
    }

    pub(crate) fn borrow(&self) -> &Item {
        panic!("")
    }

    pub(crate) fn borrow_mut(&mut self) -> &mut Item {
        panic!("")
    }

    pub(crate) fn delete(&self) {
        {
            let store = self.store.upgrade().unwrap();
            let mut store = store.write().unwrap();
            let id = store.take(1);
            let delete_item = DeleteItem::new(id, self.id().clone());
            store.insert_delete(delete_item);
        }

        // let mut item = self.item.clone();
        // item.delete()

        // self.item.borrow_mut().flags |= 0x1
    }
}

impl Encode for ItemRef {
    fn encode<E: Encoder>(&self, e: &mut E) {
        self.borrow().data.encode(e);
    }
}

impl Decode for ItemRef {
    fn decode<D: Decoder>(d: &mut D) -> Result<ItemRef, String> {
        Err("ItemRef decode not implemented".to_string())
    }
}

impl WithId for ItemRef {
    fn id(&self) -> Id {
        self.item.data.id
    }
}

impl Eq for ItemRef {}

impl PartialEq<Self> for ItemRef {
    fn eq(&self, other: &Self) -> bool {
        self.item.id.compare_without_client(&other.item.id) == Ordering::Equal
    }
}

impl PartialOrd<Self> for ItemRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ItemRef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.item.data.id.cmp(&other.item.data.id)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Item {
    pub(crate) data: ItemData,          // data for the item
    pub(crate) parent: Option<ItemRef>, // parent link
    pub(crate) left: Option<ItemRef>,   // left link
    pub(crate) right: Option<ItemRef>,  // right link
    pub(crate) start: Option<ItemRef>,  // linked children start
    pub(crate) end: Option<ItemRef>,    // linked children end
    pub(crate) target: Option<ItemRef>, // indirect item ref (proxy, mover)
    pub(crate) mover: Option<ItemRef>,  // mover ref (proxy)
    pub(crate) movers: Option<ItemRef>, // linked movers
    pub(crate) flags: u8,
}

impl Item {}

impl Item {
    pub(crate) fn new(data: ItemData) -> Self {
        Self {
            data,
            parent: None,
            left: None,
            right: None,
            start: None,
            end: None,
            target: None,
            mover: None,
            movers: None,
            flags: 0,
        }
    }

    pub(crate) fn is_moved(&self) -> bool {
        self.flags & 0x02 == 0x02
    }

    pub(crate) fn is_deleted(&self) -> bool {
        self.flags & 0x01 == 0x01
    }

    pub(crate) fn field(&self) -> Option<String> {
        self.data.field.clone()
    }

    pub(crate) fn left_origin(&mut self, store: &DocStore) -> Option<ItemRef> {
        self.data.left_id.and_then(|id| store.find(id))
    }

    pub(crate) fn right_origin(&mut self, store: &DocStore) -> Option<ItemRef> {
        self.data.right_id.and_then(|id| store.find(id))
    }

    pub(crate) fn delete(&mut self) {
        self.flags |= 0x01;
    }

    pub(crate) fn set(&mut self, key: &ItemKey, _ref: ItemRef) {}

    pub(crate) fn as_map(&self) -> Option<HashMap<String, ItemRef>> {
        let items = self.items();
        let mut map = HashMap::new();

        for item in items {
            if let Some(field) = item.borrow().field() {
                map.insert(field, item);
            }
        }

        Some(map)
    }
    pub(crate) fn insert_after(&mut self, item: ItemRef) {}
    pub(crate) fn insert_before(&mut self, item: ItemRef) {}
    pub(crate) fn items(&self) -> Vec<ItemRef> {
        self.all_items()
            .into_iter()
            .filter(|item| {
                return item.borrow().is_moved() || item.borrow().is_deleted();
            })
            .collect()
    }
    pub(crate) fn all_items(&self) -> Vec<ItemRef> {
        let mut items: Vec<ItemRef> = vec![];
        let mut item = self.start.clone();
        while item.is_some() {
            items.push(item.clone().unwrap());
            item = item.unwrap().borrow().right.clone();
        }

        items
    }
}

impl Deref for Item {
    type Target = ItemData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

// item data is encoded and saved into persistent storage
#[derive(Debug, Clone, Default)]
pub(crate) struct ItemData {
    pub(crate) kind: ItemKind,
    pub(crate) id: Id,
    pub(crate) parent_id: Option<Id>,
    pub(crate) left_id: Option<Id>,
    pub(crate) right_id: Option<Id>,

    pub(crate) target_id: Option<Id>, // for proxy & move
    pub(crate) mover_id: Option<Id>,  // for proxy

    pub(crate) field: Option<String>,
    pub(crate) content: Content,
}

impl Split for ItemData {
    fn split(&self, at: Clock) -> (Self, Self) {
        let mut left = self.clone();
        let mut right = self.clone();

        // split id
        let (lid, rid) = self.id.split(at);
        left.id = lid;
        right.id = rid;

        left.right_id = Some(right.id.head());
        right.left_id = Some(left.id.tail());

        // split content
        match &self.content {
            Content::String(s) => {
                let (l, r) = s.split_at(at as usize);
                left.content = Content::String(l.to_string());
                right.content = Content::String(r.to_string());
            }
            _ => {}
        }

        (left, right)
    }
}

impl From<ItemData> for Item {
    fn from(data: ItemData) -> Self {
        Self::new(data)
    }
}

impl From<ItemData> for Rc<Item> {
    fn from(data: ItemData) -> Self {
        Rc::new(Item::new(data))
    }
}

impl Encode for ItemData {
    fn encode<E: Encoder>(&self, e: &mut E) {
        e.item(self)
    }
}

impl Decode for ItemData {
    fn decode<D: Decoder>(d: &mut D) -> Result<ItemData, String> {
        let item = d.item()?;
        Ok(item)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ItemKind {
    Root,
    Map,
    List,
    Text,
    String,
    Atom,
    Proxy,
    Move,
}

impl Default for ItemKind {
    fn default() -> Self {
        Self::Atom
    }
}

impl WithId for ItemData {
    fn id(&self) -> Id {
        self.id
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ItemKey {
    Number(usize),
    String(String),
}

impl ItemKey {
    pub(crate) fn as_string(&self) -> String {
        match self {
            Self::String(s) => s.clone(),
            Self::Number(n) => n.to_string(),
        }
    }
}

impl From<String> for ItemKey {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<usize> for ItemKey {
    fn from(n: usize) -> Self {
        Self::Number(n)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Content {
    Binary(Vec<u8>),
    String(String),
    Embed(Any),
    Doc(DocOpts),
    None,
}

impl Default for Content {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DocOpts {
    pub(crate) guid: String,
    pub(crate) opts: Any,
}

#[derive(Debug, Clone)]
pub(crate) enum Any {
    True,
    False,
    Float32(f32),
    Float64(f64),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    String(String),
    Binary(Vec<u8>),
    Array(Vec<Any>),
    Map(Vec<(String, Any)>),
}
