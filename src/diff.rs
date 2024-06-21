use crate::bimapid::{ClientMap, FieldMap};
use crate::codec::decoder::{Decode, Decoder};
use crate::codec::encoder::{Encode, Encoder};
use crate::state::ClientState;
use crate::store::{DeleteItemStore, DocStore, ItemDataStore};

#[derive(Debug, Clone, Default)]
pub(crate) struct Diff {
    pub(crate) doc_id: String,
    pub(crate) fields: FieldMap,
    pub(crate) clients: ClientMap,
    pub(crate) state: ClientState,
    pub(crate) items: ItemDataStore,
    pub(crate) deletes: DeleteItemStore,
}

impl Diff {
    pub(crate) fn new(doc_id: String) -> Diff {
        Diff {
            doc_id,
            ..Default::default()
        }
    }

    pub(crate) fn from(
        doc_id: String,
        clients: ClientMap,
        fields: FieldMap,
        state: ClientState,
        items: ItemDataStore,
        deletes: DeleteItemStore,
    ) -> Diff {
        Diff {
            doc_id,
            clients,
            fields,
            state,
            items,
            deletes,
        }
    }

    pub(crate) fn from_deleted_items(guid: String, deleted_items: DeleteItemStore) -> Diff {
        Diff {
            doc_id: guid,
            deletes: deleted_items,
            ..Default::default()
        }
    }

    pub(crate) fn from_items(guid: String, items: ItemDataStore) -> Diff {
        Diff {
            doc_id: guid,
            items,
            ..Default::default()
        }
    }

    pub(crate) fn adjust(&mut self, store: &DocStore) -> Diff {
        let before_clients = store.clients.clone();
        let before_fields = store.fields.clone();

        let clients = store.clients.adjust(&self.clients);
        let fields = store.fields.adjust(&self.fields);
        let state = store.state.adjust(
            &self.state,
            &self.clients,
            &store.clients.merge(&self.clients),
        );

        let mut items = ItemDataStore::default();

        for (_, store) in self.items.clone().into_iter() {
            for (_, item) in store.into_iter() {
                let adjust = item.adjust(&before_clients, &before_fields, &clients, &fields);
                items.insert(adjust);
            }
        }

        let mut deletes = DeleteItemStore::default();

        for (_, store) in self.deletes.clone().into_iter() {
            for (_, item) in store.into_iter() {
                let adjust = item.adjust(&before_clients, &clients);
                deletes.insert(adjust);
            }
        }

        Diff::from(
            self.doc_id.clone(),
            clients.clone(),
            fields.clone(),
            state.clone(),
            items,
            deletes,
        )
    }
}

impl Encode for Diff {
    fn encode<E: Encoder>(&self, e: &mut E) {
        e.string(&self.doc_id);
        self.clients.encode(e);
        self.fields.encode(e);
        self.state.encode(e);
        self.items.encode(e);
        self.deletes.encode(e);
    }
}

impl Decode for Diff {
    fn decode<D: Decoder>(d: &mut D) -> Result<Diff, String> {
        let guid = d.string()?;
        let clients = ClientMap::decode(d)?;
        let fields = FieldMap::decode(d)?;
        let state = ClientState::decode(d)?;
        let items = ItemDataStore::decode(d)?;
        let deletes = DeleteItemStore::decode(d)?;

        Ok(Diff {
            doc_id: guid,
            clients,
            fields,
            state,
            items,
            deletes,
        })
    }
}
