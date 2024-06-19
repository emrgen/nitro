use std::cmp::Ordering;
use std::ops::Deref;
use std::rc::Rc;
use crate::doc::Doc;
use crate::id::{Id, WithId};
use crate::store::ItemStore;

#[derive(Debug, Clone, Default)]
pub(crate) struct ItemRef {
  pub(crate) doc: Doc,
  pub(crate) item: Rc<Item>,
}

impl ItemRef {
  pub(crate) fn new(doc: Doc, item: Item) -> Self {
    Self {
      doc,
      item: Rc::new(item),
    }
  }

  pub(crate) fn borrow(&self) -> Rc<Item> {
    Rc::clone(&self.item)
  }

  pub(crate) fn borrow_mut(&mut self) -> &mut Item {
    Rc::make_mut(&mut self.item)
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
    Some(self.item.id.compare_without_client(&other.item.id))
  }
}

impl Ord for ItemRef {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.item.data.id.cmp(&other.item.data.id)
  }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Item {
  pub(crate) data: ItemData,
  pub(crate) parent: Option<ItemRef>,
  pub(crate) left: Option<ItemRef>,
  pub(crate) right: Option<ItemRef>,
  pub(crate) start: Option<ItemRef>,
  pub(crate) target: Option<ItemRef>,
  pub(crate) mover: Option<ItemRef>,
}

impl Item {
  pub(crate) fn new(data: ItemData) -> Self {
    Self {
      data,
      parent: None,
      left: None,
      right: None,
      start: None,
      target: None,
      mover: None,
    }
  }

  pub(crate) fn field(&self) -> Option<String> {
    self.data.field.clone()
  }

  pub(crate) fn left_origin(&mut self, store: &ItemStore) -> Option<ItemRef> {
    self.data.left_id.and_then(|id| store.get(id))
  }

  pub(crate) fn right_origin(&mut self, store: &ItemStore) -> Option<ItemRef> {
    self.data.right_id.and_then(|id| store.get(id))
  }
}

impl Deref for Item {
  type Target = ItemData;

  fn deref(&self) -> &Self::Target {
    &self.data
  }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct ItemData {
  pub(crate) id: Id,
  pub(crate) parent_id: Option<Id>,
  pub(crate) left_id: Option<Id>,
  pub(crate) right_id: Option<Id>,

  pub(crate) target_id: Option<Id>, // for proxy & move
  pub(crate) mover_id: Option<Id>, // for proxy

  pub(crate) field: Option<String>,
  pub(crate) content: Content,
}

pub(crate) enum ItemKey {
  Number(usize),
  String(String),
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
