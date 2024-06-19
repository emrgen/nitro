use crate::clients::ClientMap;
use crate::codec::decoder::{Decode, Decoder};
use crate::codec::encoder::{Encode, Encoder};
use crate::state::ClientState;
use crate::store::{DeleteItemStore, ItemDataStore};

#[derive(Debug, Clone, Default)]
pub(crate) struct Diff {
    pub(crate) clients: ClientMap,
    pub(crate) state: ClientState,
    pub(crate) items: ItemDataStore,
    pub(crate) deletes: DeleteItemStore,
}

impl Diff {
    pub(crate) fn new() -> Diff {
        Diff {
            clients: ClientMap::new(),
            state: ClientState::new(),
            items: ItemDataStore::default(),
            deletes: DeleteItemStore::default(),
        }
    }

    pub(crate) fn from_deleted_items(deleted_items: DeleteItemStore) -> Diff {
        Diff {
            clients: ClientMap::new(),
            state: ClientState::new(),
            items: ItemDataStore::default(),
            deletes: deleted_items,
        }
    }

    pub(crate) fn from_items(items: ItemDataStore) -> Diff {
        Diff {
            clients: ClientMap::new(),
            state: ClientState::new(),
            items,
            deletes: DeleteItemStore::default(),
        }
    }
}

impl Encode for Diff {
    fn encode<E: Encoder>(&self, e: &mut E) {
        self.clients.encode(e);
        self.state.encode(e);
        self.items.encode(e);
        self.deletes.encode(e);
    }
}

impl Decode for Diff {
    fn decode<D: Decoder>(d: &mut D) -> Result<Diff, String> {
        let clients = ClientMap::decode(d)?;
        let state = ClientState::decode(d)?;
        let items = ItemDataStore::decode(d)?;
        let deletes = DeleteItemStore::decode(d)?;

        Ok(Diff {
            clients,
            state,
            items,
            deletes,
        })
    }
}
