use crate::item::{ItemKey, ItemRef};
use crate::list::NList;
use crate::map::NMap;
use crate::string::NString;
use crate::text::NText;

pub(crate) enum Type {
  List(NList),
  Map(NMap),
  Text(NText),
  String(NString),
}

impl Type {
  pub(crate) fn field(&self) -> Option<String> {
    match self {
      Type::List(n) => n.field(),
      _ => panic!("field: not implemented")
    }
  }

  pub(crate) fn size(&self) -> usize {
    match self {
      Type::List(n) => n.size(),
      _ => panic!("size: not implemented")
    }
  }

  pub(crate) fn append(&mut self, item: Type) {
    match self {
      Type::List(n) => n.append(item.into_item_ref()),
      _ => panic!("append: not implemented")
    }
  }

  pub(crate) fn prepend(&mut self, item: Type) {
    match self {
      Type::List(n) => n.prepend(item.into_item_ref()),
      _ => panic!("prepend: not implemented")
    }
  }

  pub(crate) fn insert(&mut self, key: &ItemKey, item: Type) {
    match self {
      Type::List(n) => n.insert(key, item.into_item_ref()),
      _ => panic!("insert: not implemented")
    }
  }

  fn into_item_ref(self) -> ItemRef {
    match self {
      Type::List(n) => n.into_item(),
      // Type::Map(n) => Type::Map(n),
      _ => panic!("into_inner: not implemented")
    }
  }
}