use std::ops::Range;

use crate::bimapid::ClientId;
use crate::{ClockTick, Id};

// Change represents a set of consecutive changes in the document by a client, which includes a range of clock ticks that are applied to the document.
// It is used to track the changes made by a client in an editor transaction.
#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct Change {
    client: ClientId,
    range: Range<ClockTick>,
}

impl Change {
    pub fn new(client: ClientId, range: Range<ClockTick>) -> Self {
        Self { client, range }
    }

    pub fn start(&self) -> ClockTick {
        self.range.start
    }

    pub fn end(&self) -> ClockTick {
        self.range.end
    }
}
