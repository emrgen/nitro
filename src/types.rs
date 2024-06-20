use crate::item::{ItemKey, ItemKind, ItemRef};
use crate::natom::NAtom;
use crate::nlist::NList;
use crate::nmap::NMap;
use crate::nstring::NString;
use crate::ntext::NText;

pub(crate) enum Type {
    List(NList),
    Map(NMap),
    Text(NText),
    String(NString),
    Atom(NAtom),
}

impl Type {
    pub(crate) fn field(&self) -> Option<String> {
        match self {
            Type::List(n) => n.field(),
            _ => panic!("field: not implemented"),
        }
    }

    pub(crate) fn size(&self) -> usize {
        match self {
            Type::List(n) => n.size(),
            _ => panic!("size: not implemented"),
        }
    }

    pub(crate) fn append(&mut self, item: Type) {
        match self {
            Type::List(n) => n.append(item.into()),
            _ => panic!("append: not implemented"),
        }
    }

    pub(crate) fn prepend(&mut self, item: Type) {
        match self {
            Type::List(n) => n.prepend(item.into()),
            _ => panic!("prepend: not implemented"),
        }
    }

    pub(crate) fn insert(&mut self, key: &ItemKey, item: Type) {
        match self {
            Type::List(n) => n.insert(key, item.into()),
            Type::Map(n) => n.set(key, item.into()),
            _ => panic!("insert: not implemented"),
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
        }
    }
}
