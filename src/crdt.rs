use std::cmp::Ordering;

use crate::id::{Id, WithId};
use crate::item::{ItemData, ItemRef};
use crate::store::{ClientStore, WeakStoreRef};
use crate::types::Type;

// integrate an item into the list of items
pub(crate) fn integrate<SF, EF>(
    data: ItemData,
    store: &WeakStoreRef,
    parent: Type,
    start: Option<Type>,
    set_start: SF,
    set_end: EF,
) -> Result<(), String>
where
    SF: FnOnce(Option<Type>) -> Result<(), String>,
    EF: FnOnce(Option<Type>) -> Result<(), String>,
{
    let item: Type = ItemRef::new(data.into(), store.clone()).into();
    let mut left = item.left_origin();
    let right = item.right_origin();

    // print_yaml(&item);

    let left_conflict = {
        let next = item.right();
        let next_id = next.map(|n| n.id());
        let right_id = right.clone().map(|r| r.id());

        !Id::eq_opt(&next_id, &right_id)
    };

    let right_conflict = {
        let prev = item.left();
        let prev_id = prev.map(|p| p.id());
        let left_id = left.clone().map(|l| l.id());

        !Id::eq_opt(&prev_id, &left_id)
    };

    let mut conflict: Option<Type> = None;
    if left.is_none() && right.is_none() || left_conflict || right_conflict {
        if let Some(left) = &left {
            conflict.clone_from(&left.right());
        } else {
            conflict.clone_from(&start);
        }
    }

    let mut conflict_items = ClientStore::default();
    let mut items_before_origin = ClientStore::default();
    let clients = store.upgrade().unwrap().borrow().state.clone();

    let item_id = item.id();

    while conflict.is_some() && conflict != right {
        let curr_conflict = conflict.clone().unwrap();

        items_before_origin.insert(conflict.clone().unwrap().id());
        conflict_items.insert(conflict.clone().unwrap().id());

        let conflict_left_id = conflict.clone().unwrap().left().map(|l| l.id());
        let item_left_id = item.left_id();

        // println!("conflict_left_id: {:?}", conflict_left_id);
        // println!("item_left_id: {:?}", item_left_id);
        // println!("conflict: {:?}", curr_conflict.id());

        if Id::eq_opt(&conflict_left_id, &item_left_id) {
            if curr_conflict.id().compare(&item_id, &clients.clients) == Ordering::Greater {
                left.clone_from(&conflict);
                conflict_items.clear();
            } else if Id::eq_opt(&curr_conflict.right_id(), &item.right_id()) {
                break;
            }
        } else if conflict_left_id.is_some()
            && items_before_origin.contains(&conflict_left_id.unwrap())
        {
            if !conflict_items.contains(&conflict_left_id.unwrap()) {
                left.clone_from(&conflict);
                conflict_items.clear();
            }
        } else {
            break;
        }

        conflict.clone_from(&curr_conflict.right());
    }

    if let Some(left) = &left {
        integrate_after(left.clone(), item.clone());
        // println!("integrated after left");
    } else {
        // println!("parent start: {:?}", parent.id());
        if let Some(start) = parent.start() {
            start.set_left(item.clone());
            item.set_right(start);
            // println!("has existing start item");
        }
        set_start(Some(item.clone()))?;
        item.set_parent(parent.clone());
        // println!("integrated at start, {:?}", item.id());
    }

    if item.right().is_none() {
        set_end(Some(item.clone()))?;
    }
    store.upgrade().unwrap().borrow_mut().insert(item.clone());

    Ok(())
}

fn integrate_after(prev: Type, item: Type) {
    let next = prev.right();
    let parent = prev.parent();

    prev.set_right(item.clone());

    if let Some(next) = &next {
        next.set_left(item.clone());
        item.set_right(next.clone());
    }

    item.set_left(prev.clone());
    if let Some(next) = &next {
        next.set_left(item.clone());
    }

    item.set_parent_id(parent.clone().map(|p| p.id()));
    item.set_parent(parent);
}
