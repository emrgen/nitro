use std::cmp::Ordering;

use serde::Serialize;
use serde_json::Value;

use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::delete::DeleteItem;
use crate::doc::{Doc, DocOpts};
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::id::{Id, IdRange, Split, WithId, WithIdRange};
use crate::item::{Content, ItemKey, ItemKind, ItemRef};
use crate::mark::Mark;
use crate::natom::NAtom;
use crate::nlist::NList;
use crate::nmap::NMap;
use crate::nmark::NMark;
use crate::nmove::NMove;
use crate::nproxy::NProxy;
use crate::nstring::NString;
use crate::ntext::NText;
use crate::store::WeakStoreRef;

#[derive(Debug, Clone, Default)]
pub(crate) enum Type {
    List(NList),
    Map(NMap),
    Text(NText),
    String(NString),
    Atom(NAtom),
    Proxy(NProxy),
    Move(NMove),
    Mark(NMark),
    Doc(Doc),
    #[default]
    Identity,
}

impl Type {
    pub(crate) fn replace(&self, items: (Type, Type)) {
        let store = self.store().upgrade().unwrap();
        let mut store = store.borrow_mut();

        if let Some(left) = self.left() {
            left.set_right(Some(items.0.clone()));
        }

        if let Some(right) = self.right() {
            right.set_left(Some(items.1.clone()));
        }

        if let Some(parent) = self.parent() {
            items.0.set_parent(Some(parent.clone()));
            items.1.set_parent(Some(parent.clone()));
        }

        store.items.replace(&self.clone(), items);
    }
}

impl Type {
    pub(crate) fn slice(&self, start: u32, end: u32) -> Vec<Type> {
        match self {
            Type::Text(n) => n.slice(start, end),
            _ => panic!("slice: not implemented"),
        }
    }
}

impl Type {
    pub(crate) fn as_list(&self) -> Option<NList> {
        match self {
            Type::List(n) => Some(n.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_map(&self) -> Option<NMap> {
        match self {
            Type::Map(n) => Some(n.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_text(&self) -> Option<NText> {
        match self {
            Type::Text(n) => Some(n.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_string(&self) -> Option<NString> {
        match self {
            Type::String(n) => Some(n.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_atom(&self) -> Option<NAtom> {
        match self {
            Type::Atom(n) => Some(n.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_proxy(&self) -> Option<NProxy> {
        match self {
            Type::Proxy(n) => Some(n.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_move(&self) -> Option<NMove> {
        match self {
            Type::Move(n) => Some(n.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_mark(&self) -> Option<NMark> {
        match self {
            Type::Mark(n) => Some(n.clone()),
            _ => None,
        }
    }

    pub(crate) fn as_doc(&self) -> Option<Doc> {
        match self {
            Type::Doc(n) => Some(n.clone()),
            _ => None,
        }
    }
}

impl Type {
    pub(crate) fn store(&self) -> WeakStoreRef {
        self.item_ref().store.clone()
    }
    pub(crate) fn right_origin(&self) -> Option<Type> {
        self.item_ref().borrow().right_origin(self.store())
    }

    pub(crate) fn left_origin(&self) -> Option<Type> {
        self.item_ref().borrow().left_origin(self.store())
    }

    pub(crate) fn parent(&self) -> Option<Type> {
        self.item_ref().borrow().parent.clone()
    }

    pub(crate) fn parent_id(&self) -> Option<Id> {
        self.item_ref().borrow().data.parent_id
    }

    pub(crate) fn left(&self) -> Option<Type> {
        self.item_ref().borrow().left.clone()
    }

    pub(crate) fn right(&self) -> Option<Type> {
        self.item_ref().borrow().right.clone()
    }

    pub(crate) fn left_id(&self) -> Option<Id> {
        self.item_ref().borrow().data.left_id
    }

    pub(crate) fn right_id(&self) -> Option<Id> {
        self.item_ref().borrow().data.right_id
    }

    pub(crate) fn start(&self) -> Option<Type> {
        self.item_ref().borrow().start.clone()
    }

    pub(crate) fn end(&self) -> Option<Type> {
        self.item_ref().borrow().end.clone()
    }

    pub(crate) fn start_id(&self) -> Id {
        self.item_ref().id().range(1).start_id()
    }

    pub(crate) fn end_id(&self) -> Id {
        if let ItemKind::String = self.kind() {
            self.item_ref().id().range(self.size()).end_id()
        } else {
            self.item_ref().id().range(1).end_id()
        }
    }

    pub(crate) fn set_parent(&self, parent: impl Into<Option<Type>>) {
        self.item_ref()
            .borrow_mut()
            .parent
            .clone_from(&parent.into());
    }

    pub(crate) fn set_parent_id(&self, parent_id: impl Into<Option<Id>>) {
        self.item_ref()
            .borrow_mut()
            .parent_id
            .clone_from(&parent_id.into());
    }

    pub(crate) fn set_left(&self, left: impl Into<Option<Type>>) {
        self.item_ref().borrow_mut().left.clone_from(&left.into());
    }

    pub(crate) fn set_left_id(&self, left_id: impl Into<Option<Id>>) {
        self.item_ref()
            .borrow_mut()
            .left_id
            .clone_from(&left_id.into());
    }

    pub(crate) fn set_right(&self, right: impl Into<Option<Type>>) {
        self.item_ref().borrow_mut().right.clone_from(&right.into());
    }

    pub(crate) fn set_right_id(&self, right_id: impl Into<Option<Id>>) {
        self.item_ref()
            .borrow_mut()
            .data
            .right_id
            .clone_from(&right_id.into());
    }

    pub(crate) fn set_start(&self, start: impl Into<Option<Type>>) {
        self.item_ref().borrow_mut().start.clone_from(&start.into());
    }

    pub(crate) fn set_end(&self, end: impl Into<Option<Type>>) {
        self.item_ref().borrow_mut().end.clone_from(&end.into());
    }

    pub(crate) fn insert_after(&self, item: Type) {
        let parent = self.parent();
        let next = self.right();

        item.set_parent_id(parent.clone().map(|p| p.id()));
        item.set_left_id(Some(self.id()));
        item.set_right_id(next.clone().map(|n| n.id()));

        item.set_parent(parent.clone());
        item.set_left(self.clone());
        item.set_right(next.clone());

        self.set_right(item.clone());

        if let Some(next) = next {
            next.set_left(item.clone());
        } else if let Some(ref parent) = parent {
            parent.set_end(item.clone());
        }
    }

    pub(crate) fn insert_before(&self, item: Type) {
        let parent = self.parent();
        let prev = self.left();

        item.set_parent_id(parent.clone().map(|p| p.id()));
        item.set_left_id(prev.clone().map(|p| p.id()));
        item.set_right_id(Some(self.id()));

        item.set_parent(parent.clone());
        item.set_left(prev.clone());
        item.set_right(self.clone());

        self.set_left(item.clone());

        if let Some(prev) = prev {
            prev.set_right(item.clone());
        } else if let Some(ref parent) = parent {
            parent.set_start(item.clone());
        }
    }

    pub(crate) fn item_ref(&self) -> ItemRef {
        match self {
            Type::List(n) => n.item_ref(),
            Type::Map(n) => n.item_ref(),
            Type::Text(n) => n.item_ref(),
            Type::String(n) => n.item_ref(),
            Type::Atom(n) => n.item_ref(),
            Type::Proxy(n) => n.item_ref(),
            Type::Move(n) => n.item_ref(),
            Type::Mark(n) => n.item_ref(),
            Type::Doc(n) => n.root.item_ref(),
            Type::Identity => panic!("item_ref: not implemented"),
        }
    }

    pub(crate) fn is_moved(&self) -> bool {
        self.item_ref().borrow().is_moved()
    }

    pub(crate) fn is_deleted(&self) -> bool {
        self.item_ref().borrow().is_deleted()
    }

    pub(crate) fn is_visible(&self) -> bool {
        !self.is_moved() && !self.is_deleted()
    }
}

impl Type {}

impl Type {
    pub fn kind(&self) -> ItemKind {
        self.item_ref().kind()
    }

    pub fn field(&self) -> Option<String> {
        self.item_ref().field()
    }

    pub(crate) fn add_mark(&self, mark: Mark) {
        match self {
            // Type::List(n) => n.add_mark(mark),
            Type::Map(n) => n.add_mark(mark),
            // Type::Text(n) => n.add_mark(mark),
            Type::String(n) => n.add_mark(mark),
            // Type::Atom(n) => n.add_mark(mark),
            // Type::Proxy(n) => n.add_mark(mark),
            // Type::Move(n) => n.add_mark(mark),
            // Type::Mark(n) => n.add_mark(mark),
            Type::Identity => panic!("add_mark: not implemented"),
            _ => panic!("add_mark: not implemented"),
        }
    }

    pub(crate) fn remove_mark(&self, mark: Mark) {
        let id = self.store().upgrade().unwrap().borrow_mut().next_id();
        let marks = self.item_ref().borrow().get_marks();
        let item = DeleteItem::new(id, self.range());
    }

    pub fn size(&self) -> u32 {
        match self {
            Type::List(n) => n.size(),
            Type::Map(n) => n.size(),
            Type::Text(n) => n.size(),
            Type::String(n) => n.size(),
            Type::Atom(n) => n.size(),
            Type::Proxy(n) => n.size(),
            Type::Move(n) => n.size(),
            Type::Mark(n) => n.size(),
            _ => panic!("size: not implemented"),
        }
    }

    pub(crate) fn content(&self) -> Content {
        match self {
            Type::String(n) => n.content(),
            Type::Atom(n) => n.content(),
            Type::Text(n) => n.content(),
            Type::Proxy(n) => n.content(),
            Type::Move(n) => n.content(),
            Type::Mark(n) => n.content(),
            _ => panic!("content: not implemented"),
        }
    }

    pub fn append(&self, item: Type) {
        match self {
            Type::List(n) => n.append(item),
            _ => panic!("append: not implemented"),
        }
    }

    pub fn prepend(&self, item: Type) {
        match self {
            Type::List(n) => n.prepend(item),
            _ => panic!("prepend: not implemented"),
        }
    }

    pub fn insert(&self, offset: u32, item: impl Into<Type>) {
        match self {
            Type::List(n) => n.insert(offset, item),
            Type::Text(n) => n.insert(offset, item),
            _ => panic!("insert: not implemented"),
        }
    }

    pub fn set(&self, key: impl Into<String>, item: impl Into<Type>) {
        match self {
            Type::Map(n) => n.set(key.into(), item.into()),
            _ => panic!("set: not implemented"),
        }
    }

    pub fn get(&self, key: String) -> Option<Type> {
        match self {
            Type::Map(n) => n.get(key),
            _ => panic!("get: not implemented"),
        }
    }

    pub fn remove(&self, key: ItemKey) {
        match self {
            Type::Map(n) => n.remove(key),
            _ => panic!("remove: not implemented"),
        }
    }

    pub fn delete(&self) {
        self.item_ref().delete(1);
    }

    pub(crate) fn clear(&self) {
        match self {
            Type::List(n) => n.clear(),
            Type::Map(n) => n.clear(),
            Type::Text(n) => n.clear(),
            _ => panic!("clear: not implemented"),
        }
    }

    pub(crate) fn split(&self, offset: u32) -> (Type, Type) {
        match self {
            Type::String(n) => n.split(offset).unwrap(),
            Type::Mark(n) => n.split(offset).unwrap(),
            _ => panic!("split: not implemented for {:?}", self.kind()),
        }
    }

    pub(crate) fn to_json(&self) -> Value {
        match self {
            Type::List(n) => n.to_json(),
            Type::Map(n) => n.to_json(),
            Type::Text(n) => n.to_json(),
            Type::String(n) => n.to_json(),
            Type::Atom(n) => n.to_json(),
            Type::Proxy(n) => n.to_json(),
            Type::Move(n) => n.to_json(),
            Type::Mark(n) => n.to_json(),
            Type::Doc(n) => n.to_json(),
            Type::Identity => panic!("to_json: not implemented for identity"),
        }
    }
}

impl Serialize for Type {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Type::List(n) => n.serialize(serializer),
            Type::Map(n) => n.serialize(serializer),
            Type::Text(n) => n.serialize(serializer),
            Type::String(n) => n.serialize(serializer),
            Type::Atom(n) => n.serialize(serializer),
            Type::Mark(n) => n.serialize(serializer),
            // Type::Proxy(n) => n.serialize(serializer),
            // Type::Move(n) => n.serialize(serializer),
            _ => panic!("Type: serialize: not implemented for {:?}", self),
        }
    }
}

impl Encode for Type {
    fn encode<T: Encoder>(&self, e: &mut T, ctx: &EncodeContext) {
        e.item(ctx, &self.item_ref().borrow().data.clone());
    }
}

impl Decode for Type {
    fn decode<T: Decoder>(_d: &mut T, _ctx: &DecodeContext) -> Result<Self, String> {
        Err("Type::decode: not implemented".to_string())
    }
}

impl WithId for Type {
    fn id(&self) -> Id {
        self.item_ref().id()
    }
}

impl From<Type> for Option<Id> {
    fn from(value: Type) -> Self {
        Some(value.id())
    }
}
impl From<&Type> for Option<Id> {
    fn from(value: &Type) -> Self {
        Some(value.id())
    }
}

impl WithIdRange for Type {
    fn range(&self) -> IdRange {
        match &self {
            Type::List(n) => n.range(),
            Type::Map(n) => n.range(),
            Type::Text(n) => n.range(),
            Type::String(n) => n.range(),
            Type::Atom(n) => n.range(),
            Type::Proxy(n) => n.range(),
            Type::Move(n) => n.range(),
            Type::Mark(n) => n.range(),
            Type::Doc(n) => n.root.range(),
            Type::Identity => panic!("range: not implemented for identity"),
        }
    }
}

impl From<NList> for Type {
    fn from(n: NList) -> Self {
        Self::List(n)
    }
}

impl From<NMap> for Type {
    fn from(n: NMap) -> Self {
        if let Content::Doc(d) = n.content() {
            Type::Doc(Doc::new(DocOpts {
                guid: d.guid,
                crated_by: d.created_by,
            }))
        } else {
            Self::Map(n)
        }
    }
}

impl From<NText> for Type {
    fn from(n: NText) -> Self {
        Self::Text(n)
    }
}

impl From<NString> for Type {
    fn from(n: NString) -> Self {
        Self::String(n)
    }
}

impl From<NAtom> for Type {
    fn from(n: NAtom) -> Self {
        Self::Atom(n)
    }
}

impl From<NProxy> for Type {
    fn from(n: NProxy) -> Self {
        Self::Proxy(n)
    }
}

impl From<NMove> for Type {
    fn from(n: NMove) -> Self {
        Self::Move(n)
    }
}

impl From<NMark> for Type {
    fn from(n: NMark) -> Self {
        Self::Mark(n)
    }
}

impl From<ItemRef> for Type {
    fn from(item: ItemRef) -> Self {
        let kind = item.borrow().kind.clone();
        match kind {
            ItemKind::List => Self::List(item.into()),
            ItemKind::Map => Self::Map(item.into()),
            ItemKind::Text => Self::Text(item.into()),
            ItemKind::String => Self::String(item.into()),
            ItemKind::Atom => Self::Atom(item.into()),
            _ => panic!("Type::from(ItemRef): not implemented"),
        }
    }
}

impl From<Type> for ItemRef {
    fn from(t: Type) -> Self {
        match t {
            Type::List(n) => n.item_ref(),
            Type::Map(n) => n.item_ref(),
            Type::Text(n) => n.item_ref(),
            Type::String(n) => n.item_ref(),
            Type::Atom(n) => n.item_ref(),
            Type::Proxy(n) => n.item_ref(),
            Type::Move(n) => n.item_ref(),
            Type::Mark(n) => n.item_ref(),
            Type::Doc(n) => n.root.item_ref(),
            Type::Identity => panic!("Type::into(ItemRef): not implemented"),
        }
    }
}

impl Ord for Type {
    fn cmp(&self, other: &Self) -> Ordering {
        let store = self.store().upgrade().unwrap();
        let store = store.borrow();
        self.id().compare(&other.id(), &store.state.clients)
    }
}
impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq<Self> for Type {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for Type {}
