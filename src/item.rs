use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use crate::codec::decoder::{Decode, Decoder};
use crate::codec::encoder::{Encode, Encoder};
use crate::id::{Clock, Id, Split, WithId};
use crate::store::{DocStore, WeakStoreRef};

type ItemRefInner = Rc<RefCell<Item>>;
type WeakItemRefInner = Weak<RefCell<Item>>;

#[derive(Debug, Clone, Default)]
pub(crate) struct ItemRef {
    pub(crate) store: WeakStoreRef,
    pub(crate) item: ItemRefInner,
}

impl ItemRef {
    pub(crate) fn new(item: ItemRefInner, store: WeakStoreRef) -> Self {
        Self { item, store }
    }

    pub(crate) fn kind(&self) -> ItemKind {
        self.item.borrow().kind.clone()
    }

    pub(crate) fn delete(&self) {
        {
            // let store = self.store.upgrade().unwrap();
            // let mut store = store.write().unwrap();
            // let id = store.next_id_range(1);
            // let delete_item = DeleteItem::new(id, self.id().range(self.size).unwrap());
            // store.insert_delete(delete_item);
        }
        // let mut item = self.item.clone();
        // item.delete()

        // self.item.borrow_mut().flags |= 0x1
    }
}

impl Deref for ItemRef {
    type Target = ItemRefInner;

    fn deref(&self) -> &Self::Target {
        &self.item
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
        self.borrow().data.id
    }
}

impl Eq for ItemRef {}

impl PartialEq<Self> for ItemRef {
    fn eq(&self, other: &Self) -> bool {
        self.item
            .borrow()
            .id
            .compare_without_client(&other.item.borrow().id)
            == Ordering::Equal
    }
}

impl PartialOrd<Self> for ItemRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ItemRef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.item.borrow().data.id.cmp(&other.item.borrow().data.id)
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
                map.insert(field, item.clone());
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

impl ItemData {
    pub(crate) fn new(kind: ItemKind, id: Id) -> Self {
        Self {
            kind,
            id,
            parent_id: None,
            left_id: None,
            right_id: None,
            target_id: None,
            mover_id: None,
            field: None,
            content: Content::None,
        }
    }

    pub(crate) fn with_content(mut self, content: Content) -> Self {
        self.content = content;
        self
    }

    pub(crate) fn with_field(mut self, field: String) -> Self {
        self.field = Some(field);
        self
    }

    pub(crate) fn with_parent(mut self, parent: Id) -> Self {
        self.parent_id = Some(parent);
        self
    }

    pub(crate) fn with_left(mut self, left: Id) -> Self {
        self.left_id = Some(left);
        self
    }

    pub(crate) fn with_right(mut self, right: Id) -> Self {
        self.right_id = Some(right);
        self
    }

    pub(crate) fn with_target(mut self, target: Id) -> Self {
        self.target_id = Some(target);
        self
    }

    pub(crate) fn with_mover(mut self, mover: Id) -> Self {
        self.mover_id = Some(mover);
        self
    }
}

impl Split for ItemData {
    fn split(&self, at: Clock) -> Result<(Self, Self), String> {
        let mut left = self.clone();
        let mut right = self.clone();

        if self.kind != ItemKind::String {
            return Err("Cannot split root item".to_string());
        }

        let size = match &self.content {
            Content::String(s) => s.len(),
            _ => return Err("Cannot split non-string item".to_string()),
        };

        // split id
        let (left_range, right_range) = self.id.range(size as u32).split(at)?;
        left.id = left_range.start_id();
        right.id = right_range.start_id();

        left.right_id = Some(right_range.start_id());
        right.left_id = Some(left_range.end_id());

        // split content if it is a string
        if let Content::String(s) = &self.content {
            let (l, r) = s.split_at(at as usize);
            left.content = Content::String(l.to_string());
            right.content = Content::String(r.to_string());
        }

        Ok((left, right))
    }
}

impl From<ItemData> for Item {
    fn from(data: ItemData) -> Self {
        Self::new(data)
    }
}

impl From<ItemData> for ItemRefInner {
    fn from(data: ItemData) -> Self {
        Rc::new(RefCell::new(Item::new(data)))
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
