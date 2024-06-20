use crate::clients::ClientMap;
use crate::codec::decoder::{Decode, Decoder};
use crate::codec::encoder::{Encode, Encoder};
use crate::state::ClientState;
use crate::store::{DeleteItemStore, ItemDataStore};

#[derive(Debug, Clone, Default)]
pub(crate) struct Diff {
    pub(crate) guid: String,
    pub(crate) clients: ClientMap,
    pub(crate) state: ClientState,
    pub(crate) items: ItemDataStore,
    pub(crate) deletes: DeleteItemStore,
}

impl Diff {
    pub(crate) fn new(guid: String) -> Diff {
        Diff {
            guid,
            ..Default::default()
        }
    }

    pub(crate) fn from(
        guid: String,
        clients: ClientMap,
        state: ClientState,
        items: ItemDataStore,
        deletes: DeleteItemStore,
    ) -> Diff {
        Diff {
            guid,
            clients,
            state,
            items,
            deletes,
        }
    }

    pub(crate) fn from_deleted_items(guid: String, deleted_items: DeleteItemStore) -> Diff {
        Diff {
            guid,
            deletes: deleted_items,
            ..Default::default()
        }
    }

    pub(crate) fn from_items(guid: String, items: ItemDataStore) -> Diff {
        Diff {
            guid,
            items,
            ..Default::default()
        }
    }
}

impl Encode for Diff {
    fn encode<E: Encoder>(&self, e: &mut E) {
        e.string(&self.guid);
        self.clients.encode(e);
        self.state.encode(e);
        self.items.encode(e);
        self.deletes.encode(e);
    }
}

impl Decode for Diff {
    fn decode<D: Decoder>(d: &mut D) -> Result<Diff, String> {
        let guid = d.string()?;
        let clients = ClientMap::decode(d)?;
        let state = ClientState::decode(d)?;
        let items = ItemDataStore::decode(d)?;
        let deletes = DeleteItemStore::decode(d)?;

        Ok(Diff {
            guid,
            clients,
            state,
            items,
            deletes,
        })
    }
}
