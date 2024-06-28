use std::cmp::Ordering;

use crate::bimapid::ClientMap;
use crate::id::{Id, WithId};
use crate::item::Linked;
use crate::store::ClientStore;
use crate::types::Type;

// integrate an item into the list of items
pub(crate) fn integrate<SF, EF>(
    item: &Type,
    client_map: &ClientMap,
    parent: &Type,
    start: Option<Type>,
    left: &mut Option<Type>,
    right: Option<Type>,
    set_start: SF,
    set_end: EF,
) -> Result<(i32), String>
where
    SF: FnOnce(Option<Type>) -> Result<(), String>,
    EF: FnOnce(Option<Type>) -> Result<(), String>,
{
    // let item: Type = ItemRef::new(data.into(), store.clone()).into();
    // print_yaml(&item);

    let left_conflict = || {
        let next = item.right();
        let next_id = next.map(|n| n.id());
        let right_id = right.as_ref().map(|r| r.id());

        !Id::eq_opt(&next_id, &right_id)
    };

    let right_conflict = || {
        let prev = item.left();
        let prev_id = prev.map(|p| p.id());
        let left_id = left.as_ref().map(|l| l.id());

        !Id::eq_opt(&prev_id, &left_id)
    };

    let mut conflict: Option<Type> = None;
    if left.is_none() && right.is_none() || left_conflict() || right_conflict() {
        if let Some(left) = &left {
            conflict.clone_from(&left.right());
        } else {
            conflict.clone_from(&start);
        }
    }

    let mut counter = 0;
    {
        let mut conflict_items = ClientStore::default();
        let mut items_before_origin = ClientStore::default();

        let item_id = item.id();

        // resolve conflicts
        while conflict.is_some() && right != conflict {
            // println!(
            //     "current conflict: {}",
            //     &conflict.as_ref().map(|c| c.id()).unwrap()
            // );
            // if right.is_some() {
            //     println!("right: {}", &right.as_ref().map(|c| c.id()).unwrap());
            // }

            counter += 1;
            let curr_conflict = conflict.as_ref().unwrap();
            let conflict_id = curr_conflict.id();

            if counter > 10_000_000 {
                println!(
                    "infinite loop: conflict: {}, right: {}",
                    conflict.as_ref().unwrap().id(),
                    right.as_ref().unwrap().id()
                );

                println!("right: {:?}", &curr_conflict.right_id());

                return Err("infinite loop".to_string());
            }

            items_before_origin.insert(conflict_id);
            conflict_items.insert(conflict_id);

            let conflict_left_id = conflict.as_ref().and_then(|c| c.left_id());
            let item_left_id = item.left_id();

            // println!("conflict_left_id: {:?}", conflict_left_id);
            // println!("item_left_id: {:?}", item_left_id);
            // println!("conflict: {:?}", curr_conflict.id());

            if Id::eq_opt(&conflict_left_id, &item_left_id) {
                if curr_conflict.id().compare(&item_id, client_map) == Ordering::Greater {
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
            integrate_after(left, item);
            // println!("integrated after left");
        } else {
            // println!("parent start: {:?}", parent.id());
            integrate_start(item, parent, start, set_start);
            // println!("integrated at start, {:?}", item.id());
        }

        if item.right().is_none() {
            set_end(Some(item.clone()))?;
        }
    }
    // store.upgrade().unwrap().borrow_mut().insert(item.clone());

    Ok((counter))
}

#[inline]
fn integrate_start<F>(item: &Type, parent: &Type, start: Option<Type>, set_start: F)
where
    F: FnOnce(Option<Type>) -> Result<(), String>,
{
    if let Some(start) = start {
        start.set_left(item.clone());
        item.set_right(start);
        // println!("has existing start item");
    }
    set_start(Some(item.clone())).expect("TODO: panic message");
    item.set_parent_id(parent.id());
    item.set_parent(parent.clone());
}

#[inline]
fn integrate_after(prev: &Type, item: &Type) {
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

    item.set_parent_id(parent.as_ref().map(|p| p.id()));
    item.set_parent(parent);
}
