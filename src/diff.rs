use crate::bimapid::{ClientMap, FieldMap};
use crate::codec::decoder::{Decode, DecodeContext, Decoder};
use crate::codec::encoder::{Encode, EncodeContext, Encoder};
use crate::item::ItemData;
use crate::state::ClientState;
use crate::store::{DeleteItemStore, DocStore, ItemDataStore};

#[derive(Debug, Clone, Default)]
pub(crate) struct Diff {
    pub(crate) root: Option<ItemData>,
    pub(crate) fields: FieldMap,
    pub(crate) clients: ClientMap,
    pub(crate) state: ClientState,
    pub(crate) items: ItemDataStore,
    pub(crate) deletes: DeleteItemStore,
}

impl Diff {
    pub(crate) fn new() -> Diff {
        Default::default()
    }

    pub(crate) fn from(
        clients: ClientMap,
        fields: FieldMap,
        state: ClientState,
        items: ItemDataStore,
        deletes: DeleteItemStore,
    ) -> Diff {
        Diff {
            clients,
            fields,
            state,
            items,
            deletes,
            ..Default::default()
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
            clients.clone(),
            fields.clone(),
            state.clone(),
            items,
            deletes,
        )
    }
}

impl Encode for Diff {
    fn encode<E: Encoder>(&self, e: &mut E, ctx: &EncodeContext) {
        if self.root.is_some() {
            e.u8(1)
        } else {
            e.u8(0)
        }

        self.clients.encode(e, ctx);
        self.fields.encode(e, ctx);
        self.state.encode(e, ctx);
        self.items.encode(e, ctx);
        self.deletes.encode(e, ctx);
    }
}

impl Decode for Diff {
    fn decode<D: Decoder>(d: &mut D, ctx: &DecodeContext) -> Result<Diff, String> {
        let root = match d.u8()? {
            128 => Some(ItemData::decode(d, ctx)?),
            _ => None,
        };
        let clients = ClientMap::decode(d, ctx)?;
        let fields = FieldMap::decode(d, ctx)?;
        let state = ClientState::decode(d, ctx)?;
        let items = ItemDataStore::decode(d, ctx)?;
        let deletes = DeleteItemStore::decode(d, ctx)?;

        Ok(Diff {
            root,
            clients,
            fields,
            state,
            items,
            deletes,
        })
    }
}
