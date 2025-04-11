use crate::bimapid::ClientId;
use crate::Id;

// Change represents a change in the document, which includes a range of IDs that are applied to the document.
// It is used to track the changes made by a client in an editor transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct Change {
    start: Id,
    end: Id,
}
