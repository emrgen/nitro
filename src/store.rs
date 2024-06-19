use std::collections::{BTreeMap, BTreeSet, HashMap};
use crate::id::{Id, WithId};
use crate::item::{Item, ItemRef};
use crate::state::Client;

pub struct ItemStore {
  items: HashMap<Client, Store<ItemRef>>,
}

impl ItemStore {
  pub fn get(&self, id: Id) -> Option<ItemRef> {
    self.items.get(&id.client).and_then(|store| store.get(&id))
  }

  pub fn put(&mut self, item: ItemRef) {
    let id = item.borrow().id;
    let store = self.items.entry(id.client).or_insert_with(Store::default);
    store.insert(item);
  }
}


#[derive(Default, Debug)]
pub(crate) struct Store<T: WithId> {
  data: BTreeMap<Id, T>,
}

impl<T: WithId> Store<T> {
  pub(crate) fn insert(&mut self, value: T) {
    self.data.insert(value.id(), value);
  }

  pub(crate) fn contains(&self, value: &Id) -> bool {
    self.data.contains_key(value)
  }

  pub(crate) fn get(&self, value: &Id) -> Option<T> {
    // self.data.get(value).cloned()
    None
  }

  pub(crate) fn put(&mut self, value: T) {
    self.data.insert(value.id(), value);
  }

  pub(crate) fn remove(&mut self, value: T) -> Option<T> {
    self.data.remove(&value.id())
  }
}

#[cfg(test)]
mod tests {
  use crate::id::Id;
  use super::*;

  #[test]
  fn test_id_store() {
    let mut store = Store::default();
    assert!(!store.contains(&Id::new(1, 1, 1)));
    store.insert(Id::new(1, 1, 1));
    assert!(store.contains(&Id::new(1, 1, 1)));


    store.insert(Id::new(1, 5, 8));
    assert!(store.contains(&Id::new(1, 6, 6)));
  }
}