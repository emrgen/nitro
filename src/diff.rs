use std::cell::RefMut;
use std::ops::Add;

use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

use crate::bimapid::FieldMap;
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::id::Id;
use crate::item::{ItemData, Optimize};
use crate::state::ClientState;
use crate::store::{DeleteItemStore, DocStore, ItemDataStore};

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct Diff {
    pub(crate) guid: String,
    pub(crate) fields: FieldMap,
    pub(crate) state: ClientState,
    pub(crate) items: ItemDataStore,
    pub(crate) deletes: DeleteItemStore,
}

impl Diff {
    pub(crate) fn has_root(&self) -> bool {
        self.get_root().is_some()
    }

    pub(crate) fn get_root(&self) -> Option<ItemData> {
        let client = self.state.clients.get_client_id(&self.guid)?;
        self.items.find(&Id::new(*client, 1))
    }
}

impl Diff {
    pub(crate) fn new(guid: String) -> Diff {
        Self {
            guid,
            ..Default::default()
        }
    }

    pub(crate) fn from(
        guid: String,
        fields: FieldMap,
        state: ClientState,
        items: ItemDataStore,
        deletes: DeleteItemStore,
    ) -> Diff {
        Diff {
            guid,
            fields,
            state,
            items,
            deletes,
        }
    }

    pub(crate) fn from_deleted_items(deleted_items: DeleteItemStore) -> Diff {
        Diff {
            deletes: deleted_items,
            ..Default::default()
        }
    }

    pub(crate) fn from_items(items: ItemDataStore) -> Diff {
        Diff {
            items,
            ..Default::default()
        }
    }

    // adjust the diff to the current state of the store
    // this is used when applying a diff to a store
    pub(crate) fn adjust(&self, store: &RefMut<DocStore>) -> Diff {
        let state = self.state.adjust_min(&store.state);

        // let next_state = &self.state + &state;

        let fields = store.fields.adjust(&self.fields);
        let mut items = ItemDataStore::default();

        // println!("before clients: {:?}", self.state.clients);
        // println!("after clients: {:?}", self.items);

        for (_, id_store) in self.items.iter() {
            for (_, item) in id_store.iter() {
                let adjust =
                    item.adjust(&self.state.clients, &self.fields, &state.clients, &fields);
                items.insert(adjust);
            }
        }

        let mut deletes = DeleteItemStore::default();

        for (_, store) in self.deletes.clone().into_iter() {
            for (_, item) in store.into_iter() {
                let adjust = item.adjust(&self.state.clients, &state.clients);
                deletes.insert(adjust);
            }
        }

        let guid = self.guid.clone();

        Diff::from(guid, fields.clone(), state.clone(), items, deletes)
    }

    pub(crate) fn optimize(&mut self) {
        for (_, store) in self.items.items.iter_mut() {
            for (_, item) in store.iter_mut() {
                item.optimize();
            }
        }
    }
}

impl Serialize for Diff {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut s = serializer.serialize_struct("Diff", 6)?;
        s.serialize_field("guid", &self.guid)?;
        s.serialize_field("fields", &serde_json::to_value(&self.fields).unwrap())?;
        s.serialize_field("state", &serde_json::to_value(&self.state).unwrap())?;
        s.serialize_field("deletes", &serde_json::to_value(&self.deletes).unwrap())?;
        s.serialize_field("items", &serde_json::to_value(&self.items).unwrap())?;
        s.end()
    }
}

impl Encode for Diff {
    fn encode<E: Encoder>(&self, e: &mut E, ctx: &EncodeContext) {
        e.string(&self.guid);
        self.fields.encode(e, ctx);
        self.state.encode(e, ctx);
        self.deletes.encode(e, ctx);
        self.items.encode(e, ctx);
    }
}

impl Decode for Diff {
    fn decode<D: Decoder>(d: &mut D, ctx: &DecodeContext) -> Result<Diff, String> {
        let guid = d.string()?;

        let fields = FieldMap::decode(d, ctx)?;
        let state = ClientState::decode(d, ctx)?;
        let deletes = DeleteItemStore::decode(d, ctx)?;
        let items = ItemDataStore::decode(d, ctx)?;

        Ok(Diff {
            guid,
            fields,
            state,
            deletes,
            items,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::codec_v1::EncoderV1;
    use crate::decoder::Decode;
    use crate::diff::Diff;
    use crate::doc::Doc;
    use crate::encoder::{Encode, Encoder};
    use crate::state::ClientState;

    #[test]
    fn test_encode_decode_diff() {
        let doc = Doc::default();
        let text = doc.text();
        text.append(doc.string("hello"));

        doc.set("string", doc.string("str"));
        doc.set("text", text.clone());
        doc.set("props", doc.map());
        doc.set("k1", doc.atom("fe"));
        doc.set("k2", doc.list());

        let mut encoder = EncoderV1::default();

        let diff = doc.diff(ClientState::default());
        diff.encode(&mut encoder, &Default::default());

        let mut d = encoder.decoder();

        let decoded = Diff::decode(&mut d, &Default::default()).unwrap();

        assert_eq!(diff, decoded);
    }
}
