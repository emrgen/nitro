use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use serde_json::Value;

use crate::codec::decoder::{Decode, Decoder};
use crate::codec::encoder::{Encode, Encoder};
use crate::id::{Clock, Id, Split, WithId};
use crate::store::WeakStoreRef;
use crate::types::Type;

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

    pub(crate) fn field(&self) -> Option<String> {
        self.item.borrow().field()
    }

    pub(crate) fn append(&self, item: Type) {
        if let Some(ref end) = self.borrow().end.clone() {
            end.item_ref().borrow_mut().right = Some(item.clone());
            item.item_ref().borrow_mut().left = Some(end.clone());
            self.borrow_mut().end = Some(item.clone());
            item.item_ref().borrow_mut().data.left_id = Some(end.end_id());
        } else {
            self.borrow_mut().start = Some(item.clone());
            self.borrow_mut().end = Some(item.clone());
        }
    }

    pub(crate) fn prepend(&self, item: Type) {
        if let Some(ref start) = self.borrow().start.clone() {
            start.item_ref().borrow_mut().left = Some(item.clone());
            item.item_ref().borrow_mut().right = Some(start.clone());
            self.borrow_mut().start = Some(item.clone());
            item.item_ref().borrow_mut().data.right_id = Some(start.id());
        } else {
            self.borrow_mut().start = Some(item.clone());
            self.borrow_mut().end = Some(item.clone());
        }
    }

    pub(crate) fn left_origin(&self) -> Option<Type> {
        self.item.borrow().left_origin(&self.store)
    }

    pub(crate) fn delete(&self) {
        self.borrow_mut().delete();
    }

    pub(crate) fn size(&self) -> usize {
        Type::from(self.clone()).size()
    }
}

pub(crate) trait GetItemRef {
    fn item_ref(&self) -> ItemRef;
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
    fn decode<D: Decoder>(d: &mut D) -> Result<Self, String> {
        Err("ItemRef::decode not implemented".to_string())
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
    pub(crate) data: ItemData,       // data for the item
    pub(crate) parent: Option<Type>, // parent link
    pub(crate) left: Option<Type>,   // left link
    pub(crate) right: Option<Type>,  // right link
    pub(crate) start: Option<Type>,  // linked children start
    pub(crate) end: Option<Type>,    // linked children end
    pub(crate) target: Option<Type>, // indirect item ref (proxy, mover)
    pub(crate) mover: Option<Type>,  // mover ref (proxy)
    pub(crate) movers: Option<Type>, // linked movers
    pub(crate) flags: u8,
}

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

    pub(crate) fn left_origin(&self, store: &WeakStoreRef) -> Option<Type> {
        self.data
            .left_id
            .and_then(|id| store.upgrade()?.borrow().find(id))
    }

    pub(crate) fn right_origin(&self, store: &WeakStoreRef) -> Option<Type> {
        self.data
            .right_id
            .and_then(|id| store.upgrade()?.borrow().find(id))
    }

    pub(crate) fn delete(&mut self) {
        self.flags |= 0x01;
    }

    pub(crate) fn set(&mut self, key: &ItemKey, _ref: ItemRef) {}

    pub(crate) fn as_map(&self) -> Option<HashMap<String, Type>> {
        let items = self.items();
        let mut map = HashMap::new();

        for item in items.clone() {
            if let Some(field) = item.item_ref().borrow().field() {
                map.insert(field, item.clone());
            }
        }

        // remove items that are moved or deleted
        for item in items.iter() {
            if item.item_ref().borrow().is_moved() || item.item_ref().borrow().is_deleted() {
                map.remove(&item.item_ref().borrow().field().unwrap());
            }
        }

        Some(map)
    }

    pub(crate) fn as_list(&self) -> Vec<Type> {
        let items = self.items();
        let mut list = vec![];

        for item in items.clone() {
            list.push(item.clone());
        }

        // remove items that are moved or deleted
        list.into_iter()
            .filter(|item| {
                return item.item_ref().borrow().is_moved()
                    || item.item_ref().borrow().is_deleted();
            })
            .collect()
    }

    pub(crate) fn items(&self) -> Vec<Type> {
        self.all_items()
            .into_iter()
            .filter(|item| {
                return item.item_ref().borrow().is_moved()
                    || item.item_ref().borrow().is_deleted();
            })
            .collect()
    }

    pub(crate) fn all_items(&self) -> Vec<Type> {
        let mut items: Vec<Type> = vec![];
        let mut item = self.start.clone();
        while item.is_some() {
            items.push(item.clone().unwrap().into());
            item = item.unwrap().item_ref().borrow().right.clone();
        }

        items
    }

    pub(crate) fn to_json(&self) -> serde_json::Value {
        let mut map = serde_json::Map::new();
        map.insert("id".to_string(), self.data.id.to_string().into());
        map.insert("kind".to_string(), self.data.kind.to_string().into());
        map.insert("content".to_string(), self.data.content.to_json());
        map.insert(
            "field".to_string(),
            self.data.field.clone().unwrap_or("".to_string()).into(),
        );

        if let Some(parent) = &self.parent {
            map.insert("parent".to_string(), parent.id().to_string().into());
        }

        if let Some(left) = &self.left {
            map.insert("left".to_string(), left.id().to_string().into());
        }

        if let Some(right) = &self.right {
            map.insert("right".to_string(), right.id().to_string().into());
        }

        if let Some(target) = &self.target {
            map.insert("target".to_string(), target.id().to_string().into());
        }

        if let Some(mover) = &self.mover {
            map.insert("mover".to_string(), mover.id().to_string().into());
        }

        serde_json::Value::Object(map)
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

impl Display for ItemKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Root => write!(f, "Root"),
            Self::Map => write!(f, "Map"),
            Self::List => write!(f, "List"),
            Self::Text => write!(f, "Text"),
            Self::String => write!(f, "String"),
            Self::Atom => write!(f, "Atom"),
            Self::Proxy => write!(f, "Proxy"),
            Self::Move => write!(f, "Move"),
        }
    }
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

impl Content {
    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::Binary(b) => Value::String(serde_json::to_string(b).unwrap()),
            Self::String(s) => Value::String(s.clone()),
            Self::Embed(a) => a.to_json(),
            Self::Doc(d) => Value::String(d.guid.clone()),
            Self::None => Value::Null,
        }
    }
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

impl Any {
    pub(crate) fn to_json(&self) -> Value {
        match self {
            Self::True => Value::Bool(true),
            Self::False => Value::Bool(false),
            Self::Float32(f) => Value::Number(serde_json::Number::from_f64(*f as f64).unwrap()),
            Self::Float64(f) => Value::Number(serde_json::Number::from_f64(*f).unwrap()),
            Self::Int8(i) => Value::Number(serde_json::Number::from(*i)),
            Self::Int16(i) => Value::Number(serde_json::Number::from(*i)),
            Self::Int32(i) => Value::Number(serde_json::Number::from(*i)),
            Self::Int64(i) => Value::Number(serde_json::Number::from(*i)),
            Self::Uint8(u) => Value::Number(serde_json::Number::from(*u)),
            Self::Uint16(u) => Value::Number(serde_json::Number::from(*u)),
            Self::Uint32(u) => Value::Number(serde_json::Number::from(*u)),
            Self::Uint64(u) => Value::Number(serde_json::Number::from(*u)),
            Self::String(s) => Value::String(s.clone()),
            Self::Binary(b) => Value::String(serde_json::to_string(b).unwrap()),
            Self::Array(a) => Value::Array(a.iter().map(|a| a.to_json()).collect()),
            Self::Map(m) => {
                let mut map = serde_json::Map::new();
                for (k, v) in m.iter() {
                    map.insert(k.clone(), v.to_json());
                }
                Value::Object(map)
            }
        }
    }
}
