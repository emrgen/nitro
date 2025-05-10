use btree_plus_store::map::Range;
use btree_plus_store::set::Range;

use crate::bimapid::ClientId;
use crate::{Clock, Id};

// Change represents a set of consecutive changes in the document by a client, which includes a range of clock ticks that are applied to the document.
// It is used to track the changes made by a client in an editor transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct Change {
    client: ClientId,
    range: Range<Clock>,
}

impl Change {
    pub fn new(client: ClientId, range: Range<Clock>) -> Self {
        Self { client, range }
    }

    pub fn start(&self) -> Id {
        self.range.min()
    }

    pub fn end(&self) -> Id {
        self.range.max()
    }
}
