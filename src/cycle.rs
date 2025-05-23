use crate::id::WithId;
use crate::Type;

/// check if the current operation will create a cycle in the document tree
pub(crate) fn creates_cycle(parent: Type, child: Type) -> bool {
    // moving child to higher level in the tree does not create a cycle
    if child.depth() >= parent.depth() {
        return false;
    }

    let child_id = child.id();
    while let Some(parent) = parent.parent() {
        if parent.id().eq(&child_id) {
            return true;
        }
    }

    false
}
