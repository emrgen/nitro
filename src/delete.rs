use crate::codec::decoder::Decoder;
use crate::codec::encoder::Encoder;
use crate::id::{Id, WithId};

#[derive(Debug, Clone, Default)]
pub(crate) struct DeleteItem {
    id: Id,
    range: Id,
}

impl DeleteItem {
    pub(crate) fn new(id: Id, range: Id) -> DeleteItem {
        DeleteItem { id, range }
    }

    pub(crate) fn id(&self) -> Id {
        self.id
    }

    pub(crate) fn range(&self) -> Id {
        self.range
    }

    fn encode<T: Encoder>(&self, e: &mut T) {
        self.id.encode(e);
        self.range.encode(e);
    }

    fn decode<T: Decoder>(d: &mut T) -> DeleteItem {
        let id = Id::decode(d);
        let range = Id::decode(d);
        DeleteItem::new(id, range)
    }
}

impl WithId for DeleteItem {
    fn id(&self) -> Id {
        self.id
    }
}
