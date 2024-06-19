use crate::clients::ClientMap;
use crate::codec::encoder::Encoder;
use crate::state::ClientState;
use crate::store::{DeleteItemStore, ItemDataStore, ItemStore};

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

    pub(crate) fn encode<T: Encoder>(&self, e: &mut T) {
        self.clients.encode(e);
        self.state.encode(e);
    }
}
