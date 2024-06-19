use crate::item::ItemRef;

pub(crate) struct NList {
  item: ItemRef,
}

impl NList {
  pub(crate) fn new(item: ItemRef) -> Self {
    Self { item }
  }

  pub(crate) fn field(&self) -> Option<String> {
    self.item.borrow().field()
  }

  pub(crate) fn size(&self) -> usize {
    0
  }

  pub(crate) fn append(&mut self, _item: ItemRef) {
    // self.item.append(item)
  }

  pub(crate) fn prepend(&mut self, _item: ItemRef) {
    // self.item.append(item)
  }

  pub(crate) fn insert(&mut self, _item: ItemRef) {
    // self.item.append(item)
  }
}