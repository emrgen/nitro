use crate::id::Id;
use crate::item::{Item, ItemData, ItemRef};
use crate::store::{ClientStore, Store};

pub fn integrate(store: Store, item: ItemData, parent: ItemRef) {
    // get rw lock for the parent,
    // let mut left = item.left_id.and_then(|id| store.get(id));
    // let mut right = item.right_id.and_then(|id| store.get(id));

    // let mut conflict: Option<ItemRef> = None;

    // if (left.is_none() && right.is_none()) {}

    // println!("integrate: left: {:?}, right: {:?}", left, right);
}

fn integrate_after(prev: ItemRef, item: Item) {}
