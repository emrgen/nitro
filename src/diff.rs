use std::cell::RefMut;
use std::ops::Add;

use serde::ser::SerializeStruct;
use serde::{Serialize, Serializer};

use crate::bimapid::FieldMap;
use crate::change::Change;
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::doc::DocId;
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::id::Id;
use crate::item::{ItemData, Optimize};
use crate::state::ClientState;
use crate::store::{DeleteItemStore, DocStore, IdDiff, ItemDataStore};
use crate::Client;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct Diff {
    pub created_by: Client,
    pub doc_id: DocId,
    pub fields: FieldMap,
    pub changes: Vec<Change>,
    pub state: ClientState,
    pub items: ItemDataStore,
    pub deletes: DeleteItemStore,
}

impl Diff {
    pub(crate) fn has_root(&self) -> bool {
        self.get_root().is_some()
    }

    pub(crate) fn get_root(&self) -> Option<ItemData> {
        let client = self.state.clients.get_client_id(&self.created_by)?;
        self.items.find(&Id::new(*client, 1))
    }
}

impl Diff {
    pub(crate) fn new(doc_id: DocId, created_by: Client) -> Diff {
        Self {
            doc_id,
            created_by,
            ..Default::default()
        }
    }

    pub(crate) fn from(
        doc_id: DocId,
        created_by: Client,
        fields: FieldMap,
        state: ClientState,
        items: ItemDataStore,
        deletes: DeleteItemStore,
    ) -> Diff {
        Diff {
            created_by,
            doc_id,
            fields,
            changes: vec![],
            state,
            items,
            deletes,
        }
    }

    // create a diff from a diff
    pub fn diff(&self, state: &ClientState) -> Diff {
        Diff {
            doc_id: self.doc_id.clone(),
            created_by: self.created_by.clone(),
            fields: self.fields.clone(),
            changes: vec![],
            state: self.state.clone(),
            items: self.items.diff(state),
            deletes: self.deletes.diff(state),
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
    pub fn adjust(&self, store: &RefMut<DocStore>) -> Diff {
        let state = self.state.adjust_max(&store.state);

        // let next_state = &self.state + &state;

        let fields = self.fields.as_per(&store.fields);
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

        Diff::from(
            self.doc_id.clone(),
            self.created_by.clone(),
            fields.clone(),
            state.clone(),
            items,
            deletes,
        )
    }

    // adjust the diff to the current state of the store
    pub fn adjust_diff(&self, other: &Diff) -> Diff {
        // print_yaml(&self.state);
        // print_yaml(&other.state);

        // merge client states
        let state = self.state.adjust_max(&other.state);

        // print_yaml(&state);

        // merge fields
        let fields = self.fields.as_per(&other.fields);

        let mut items = ItemDataStore::default();

        // adjust items
        for (_, store) in self.items.iter() {
            for (_, item) in store.iter() {
                let adjust =
                    item.adjust(&self.state.clients, &self.fields, &state.clients, &fields);
                items.insert(adjust);
            }
        }

        // adjust deletes
        let mut deletes = DeleteItemStore::default();

        for (_, store) in self.deletes.clone().into_iter() {
            for (_, item) in store.into_iter() {
                let adjust = item.adjust(&self.state.clients, &state.clients);
                deletes.insert(adjust);
            }
        }

        Diff::from(
            self.doc_id.clone(),
            self.created_by.clone(),
            fields,
            state,
            items,
            deletes,
        )
    }

    // merge two diffs together into self
    pub fn merge(&mut self, other: &Diff) {
        if self.doc_id != other.doc_id {
            panic!("cannot merge diffs with different doc ids");
        }
        if self.created_by != other.created_by {
            panic!("cannot merge diffs with different created_by");
        }

        self.fields = self.fields.merge(&other.fields);
        self.state = self.state.merge(&other.state);
        self.items = self.items.merge(&other.items);
        self.deletes = self.deletes.merge(&other.deletes);
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
        s.serialize_field("doc_id", &self.doc_id)?;
        s.serialize_field("created_by", &self.created_by)?;
        s.serialize_field("fields", &self.fields)?;
        s.serialize_field("state", &self.state)?;
        s.serialize_field("deletes", &self.deletes)?;
        s.serialize_field("items", &self.items)?;
        s.end()
    }
}

impl Encode for Diff {
    fn encode<E: Encoder>(&self, e: &mut E, cx: &mut EncodeContext) {
        self.doc_id.encode(e, cx);
        self.created_by.encode(e, cx);
        self.fields.encode(e, cx);
        self.state.encode(e, cx);
        self.deletes.encode(e, cx);
        self.items.encode(e, cx);
    }
}

impl Decode for Diff {
    fn decode<D: Decoder>(d: &mut D, ctx: &DecodeContext) -> Result<Diff, String> {
        let doc_id = DocId::decode(d, ctx)?;
        let created_by = Client::decode(d, ctx)?;
        let fields = FieldMap::decode(d, ctx)?;
        let state = ClientState::decode(d, ctx)?;
        let deletes = DeleteItemStore::decode(d, ctx)?;
        let items = ItemDataStore::decode(d, ctx)?;

        Ok(Diff {
            doc_id,
            created_by,
            fields,
            changes: vec![],
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
        diff.encode(&mut encoder, &mut Default::default());

        let mut d = encoder.decoder();

        let decoded = Diff::decode(&mut d, &Default::default()).unwrap();

        assert_eq!(diff, decoded);
    }
}
