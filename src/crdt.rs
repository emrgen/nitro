use std::cmp::Ordering;
use std::collections::BTreeSet;

use crate::id::{Id, WithId};
use crate::item::{ItemData, ItemRef};
use crate::store::WeakStoreRef;
use crate::types::Type;

// integrate an item into the list of items
pub(crate) fn integrate<F>(
    data: ItemData,
    store: &WeakStoreRef,
    parent: Type,
    start: Option<Type>,
    set_start: F,
) -> Result<(), String>
where
    F: FnOnce(Option<Type>) -> Result<(), String>,
{
    let item: Type = ItemRef::new(data.into(), store.clone()).into();
    let mut left = item.left_origin();
    let right = item.right_origin();

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

    let mut conflict_items = BTreeSet::new();
    let mut items_before_origin = BTreeSet::new();
    let clients = store.upgrade().unwrap().borrow().clients.clone();

    let item_id = item.id();

    while conflict.is_some() && conflict != right {
        let curr_conflict = conflict.clone().unwrap();

        items_before_origin.insert(conflict.clone().unwrap().id());
        conflict_items.insert(conflict.clone().unwrap().id());

        let conflict_left_id = conflict.clone().unwrap().left().map(|l| l.id());
        let item_left_id = item.left().map(|l| l.id());
        if Id::eq_opt(&conflict_left_id, &item_left_id) {
            if curr_conflict.id().compare(&item_id, &clients) == Ordering::Greater {
                left = conflict.clone();
                conflict_items.insert(curr_conflict.id());
            } else if Id::eq_opt(&curr_conflict.right_id(), &item.right_id()) {
                break;
            }
        } else if conflict_left_id.is_some()
            && items_before_origin.contains(&conflict_left_id.unwrap())
        {
            if !conflict_items.contains(&conflict_left_id.unwrap()) {
                left = conflict.clone();
                conflict_items.clear();
            }
        } else {
            break;
        }
    }

    // if (left.is_none() && right.is_none()) {}

    // println!("integrate: left: {:?}, right: {:?}", left, right);

    if let Some(left) = &left {
        println!("-------------------------------");
        integrate_after(left.clone(), item.clone());
    } else {
        if let Some(start) = start {
            start.set_right(item.clone());
            item.set_left(start);
        }
        set_start(Some(item.clone()))?;
        item.set_parent(parent.clone());
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
