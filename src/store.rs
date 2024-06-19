use std::collections::{BTreeSet, HashMap};
use crate::id::Id;
use crate::item::{Item, ItemRef};
use crate::state::Client;

pub struct ItemStore {
  items: HashMap<Client, Store<ItemRef>>,
}

impl ItemStore {
  pub fn get(&self, id: Id) -> Option<ItemRef> {
    self.items.get(&id.client).and_then(|store| store.get(id))
  }
  
  pub fn put(&mut self, item: Item) {
    let store = self.items.entry(item.id.client).or_insert_with(Store::default);
    store.insert(ItemRef::new(item));
  }
}


#[derive(Default, Debug)]
pub(crate) struct Store<T: Ord> {
  data: BTreeSet<T>,
}

impl<T: Ord> Store<T> {
  pub(crate) fn insert(&mut self, value: T) {
    self.data.insert(value);
  }

  pub(crate) fn contains(&self, value: &T) -> bool {
    self.data.contains(value)
  }
}

#[cfg(test)]
mod tests {
  use crate::id::Id;
  use super::*;

  #[test]
  fn test_store() {
    let mut store = Store::default();
    assert!(!store.contains(&1));
    store.insert(1);
    assert!(store.contains(&1));
  }

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