use crate::codec::decoder::{Decode, Decoder};
use crate::codec::encoder::{Encode, Encoder};
use crate::id::{Id, IdRange, WithId};

#[derive(Debug, Clone, Default)]
pub(crate) struct DeleteItem {
    id: Id,
    range: IdRange,
}

impl DeleteItem {
    pub(crate) fn new(id: Id, range: IdRange) -> DeleteItem {
        DeleteItem { id, range }
    }

    pub(crate) fn range(&self) -> IdRange {
        self.range
    }
}

impl Encode for DeleteItem {
    fn encode<E: Encoder>(&self, e: &mut E) {
        self.id.encode(e);
        self.range.encode(e);
    }
}

impl Decode for DeleteItem {
    fn decode<D: Decoder>(d: &mut D) -> Result<DeleteItem, String> {
        let id = Id::decode(d)?;
        let range = IdRange::decode(d)?;

        Ok(DeleteItem::new(id, range))
    }
}

impl WithId for DeleteItem {
    fn id(&self) -> Id {
        self.id
    }
}
