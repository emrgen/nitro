use crate::id::{Id, WithId};
use crate::item::{Item, ItemData, ItemRef};
use crate::store::WeakStoreRef;
use crate::types::Type;

pub(crate) fn integrate<F>(
    store: &WeakStoreRef,
    data: ItemData,
    start: Option<Type>,
    set_start: F,
) -> Result<(), String>
where
    F: FnOnce(Option<Type>) -> Result<(), String>,
{
    let item: Type = ItemRef::new(data.into(), store.clone()).into();
    let parent = item.parent();
    let left = item.left_origin();
    let right = item.right_origin();

    let left_conflict = {
        let next = item.right();
        let next_id = next.map(|n| n.id());
        let right_id = right.clone().map(|r| r.id());

        !Id::eq_opt(next_id, right_id)
    };

    let right_conflict = {
        let prev = item.left();
        let prev_id = prev.map(|p| p.id());
        let left_id = left.clone().map(|l| l.id());

        !Id::eq_opt(prev_id, left_id)
    };

    let mut conflict: Option<Type> = None;
    if left.is_none() && right.is_none() || left_conflict || right_conflict {
        if let Some(left) = &left {
            conflict.clone_from(&left.right());
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

    if left.is_none() {
        if let Some(start) = start {
            start.set_right(item.clone());
            item.set_left(start);
        }
        set_start(Some(item.clone()))?;
        item.set_parent(parent.clone());
    } else {
    }

    Ok(())
}

fn integrate_after(prev: ItemRef, item: Item) {}
