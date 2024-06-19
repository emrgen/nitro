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
      // Type::Map(n) => n.field(),
      // Type::Text(n) => n.field(),
      // Type::String(n) => n.field(),
      _ => None,
    }
  }

  pub(crate) fn size(&self) -> usize {
    match self {
      Type::List(n) => n.size(),
      // Type::Map(n) => n.size(),
      // Type::Text(n) => n.size(),
      // Type::String(n) => n.size(),
      _ => 0,
    }
  }

  pub(crate) fn append(&mut self, item: Type) {
    match self {
      Type::List(n) => n.append(item),
      // Type::Map(n) => n.append(item),
      // Type::Text(n) => n.append(item),
      // Type::String(n) => n.append(item),
      _ => (),
    }
  }
}