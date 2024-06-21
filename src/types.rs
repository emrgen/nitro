use serde::Serialize;
use serde_json::Value;

use crate::codec::decoder::{Decode, Decoder};
use crate::codec::encoder::{Encode, Encoder};
use crate::id::{Id, IdRange, WithId, WithIdRange};
use crate::item::{Content, ItemKey, ItemKind, ItemRef};
use crate::natom::NAtom;
use crate::nlist::NList;
use crate::nmap::NMap;
use crate::nmove::NMove;
use crate::nproxy::NProxy;
use crate::nstring::NString;
use crate::ntext::NText;

#[derive(Debug, Clone, Default)]
pub(crate) enum Type {
    List(NList),
    Map(NMap),
    Text(NText),
    String(NString),
    Atom(NAtom),
    Proxy(NProxy),
    Move(NMove),
    #[default]
    Identity,
}

impl Type {}

impl Type {
    pub fn kind(&self) -> ItemKind {
        self.item_ref().kind()
    }

    pub fn field(&self) -> Option<String> {
        self.item_ref().field()
    }

    pub fn size(&self) -> usize {
        match self {
            Type::List(n) => n.size(),
            Type::Map(n) => n.size(),
            Type::Text(n) => n.size(),
            Type::String(n) => n.size(),
            Type::Atom(n) => n.size(),
            Type::Proxy(n) => n.size(),
            Type::Move(n) => n.size(),
            _ => panic!("size: not implemented"),
        }
    }

    pub(crate) fn content(&self) -> Content {
        match self {
            Type::String(n) => n.content(),
            Type::Atom(n) => n.content(),
            Type::Text(n) => n.content(),
            Type::Proxy(n) => n.content(),
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

    pub fn insert(&self, offset: usize, item: Type) {
        match self {
            Type::List(n) => n.insert(offset, item),
            _ => panic!("insert: not implemented"),
        }
    }

    pub fn set(&self, key: String, item: Type) {
        match self {
            Type::Map(n) => n.set(key, item),
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
            _ => panic!("clear: not implemented"),
        }
    }

    pub(crate) fn start_id(&self) -> Id {
        self.item_ref().id().range(0).start_id()
    }

    pub(crate) fn end_id(&self) -> Id {
        self.item_ref().id().range(self.size() as u32).end_id()
    }

    pub(crate) fn insert_after(&self, item: Type) {
        let parent = self.item_ref().borrow().parent.clone();
        let next = self.item_ref().borrow().right.clone();

        let item_ref = item.item_ref();
        let mut item_mut = item_ref.borrow_mut();

        item_mut.data.parent_id = parent.clone().map(|p| p.id());
        item_mut.data.left_id = Some(self.id());
        item_mut.data.right_id = next.clone().map(|n| n.id());

        item_mut.parent.clone_from(&parent);
        item_mut.left = Some(self.clone());
        item_mut.right.clone_from(&next);

        self.item_ref().borrow_mut().right = Some(item.clone());
        if let Some(next) = next {
            next.item_ref().borrow_mut().left = Some(item.clone());
        } else if let Some(ref parent) = parent {
            parent.item_ref().borrow_mut().end = Some(item.clone());
        }
    }

    pub(crate) fn insert_before(&self, item: Type) {
        let parent = self.item_ref().borrow().parent.clone();
        let prev = self.item_ref().borrow().left.clone();

        let item_ref = item.item_ref();
        let mut item_mut = item_ref.borrow_mut();

        item_mut.data.parent_id = parent.clone().map(|p| p.id());
        item_mut.data.left_id = prev.clone().map(|p| p.id());
        item_mut.data.right_id = Some(self.id());

        item_mut.parent.clone_from(&parent);
        item_mut.left.clone_from(&prev);
        item_mut.right = Some(self.clone());

        self.item_ref().borrow_mut().left = Some(item.clone());
        if let Some(prev) = prev {
            prev.item_ref().borrow_mut().right = Some(item.clone());
        } else if let Some(ref parent) = parent {
            parent.item_ref().borrow_mut().start = Some(item.clone());
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
            Type::Identity => panic!("item_ref: not implemented"),
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
            Type::Identity => panic!("to_json: not implemented"),
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
            // Type::Map(n) => n.serialize(serializer),
            // Type::Text(n) => n.serialize(serializer),
            // Type::String(n) => n.serialize(serializer),
            Type::Atom(n) => n.serialize(serializer),
            // Type::Proxy(n) => n.serialize(serializer),
            // Type::Move(n) => n.serialize(serializer),
            _ => panic!("serialize: not implemented"),
        }
    }
}

impl Encode for Type {
    fn encode<T: Encoder>(&self, e: &mut T) {
        e.item(&self.item_ref().borrow().data.clone());
    }
}

impl Decode for Type {
    fn decode<T: Decoder>(d: &mut T) -> Result<Self, String> {
        Err("Type::decode: not implemented".to_string())
    }
}

impl WithId for Type {
    fn id(&self) -> Id {
        self.item_ref().id()
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
        Self::Map(n)
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
            Type::Identity => panic!("Type::into(ItemRef): not implemented"),
        }
    }
}
