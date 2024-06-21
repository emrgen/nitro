use crate::id::{Id, WithId};
use crate::item::{Item, ItemData, ItemRef};
use crate::store::WeakStoreRef;
use crate::types::Type;

pub(crate) fn integrate(
    store: &WeakStoreRef,
    data: ItemData,
    start: Option<Type>,
) -> Result<(), String> {
    let item: Type = ItemRef::new(data.into(), store.clone()).into();
    let left = item.item_ref().borrow().left_origin(store);
    let right = item.item_ref().borrow().right_origin(store);

    let left_conflict = {
        let next = item.item_ref().borrow().right.clone();
        let next_id = next.map(|n| n.id());
        let right_id = right.clone().map(|r| r.id());

        !Id::eq_opt(next_id, right_id)
    };

    let right_conflict = {
        let prev = item.item_ref().borrow().left.clone();
        let prev_id = prev.map(|p| p.id());
        let left_id = left.clone().map(|l| l.id());

        !Id::eq_opt(prev_id, left_id)
    };

    let mut conflict: Option<Type> = None;
    if left.is_none() && right.is_none() || left_conflict || right_conflict {
        if let Some(left) = left {
            conflict.clone_from(&left.item_ref().borrow().right);
        } else {
            conflict.clone_from(&start);
        }
    }

    loop {
        match (conflict.clone(), right.clone()) {
            (Some(c), Some(r)) => {
                if c.id().eq(&r.id()) {
                    break;
                }
            }
            (None, _) => {
                break;
            }
            _ => {}
        }
    }

    // if (left.is_none() && right.is_none()) {}

    // println!("integrate: left: {:?}, right: {:?}", left, right);

    Ok(())
}

fn integrate_after(prev: ItemRef, item: Item) {}
