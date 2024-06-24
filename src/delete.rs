use crate::bimapid::ClientMap;
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::encoder::{Encode, EncodeContext, Encoder};
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

    pub(crate) fn adjust(&self, before: &ClientMap, after: &ClientMap) -> DeleteItem {
        let mut adjust = self.clone();

        adjust.id = self.id.adjust(before, after);
        adjust.range = self.range.adjust(before, after);

        adjust
    }
}

impl Encode for DeleteItem {
    fn encode<E: Encoder>(&self, e: &mut E, ctx: &EncodeContext) {
        self.id.encode(e, ctx);
        self.range.encode(e, ctx);
    }
}

impl Decode for DeleteItem {
    fn decode<D: Decoder>(d: &mut D, ctx: &DecodeContext) -> Result<DeleteItem, String> {
        let id = Id::decode(d, ctx)?;
        let range = IdRange::decode(d, ctx)?;

        Ok(DeleteItem::new(id, range))
    }
}

impl WithId for DeleteItem {
    fn id(&self) -> Id {
        self.id
    }
}
