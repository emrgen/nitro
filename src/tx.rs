use log::log;

use crate::delete::DeleteItem;
use crate::diff::Diff;
use crate::doc::Doc;
use crate::item::ItemData;
use crate::store::{PendingStore, ReadyStore, WeakStoreRef};

#[derive(Debug, Clone, Default)]
pub(crate) struct Tx {
    store: WeakStoreRef,
    diff: Diff,
    ready: ReadyStore,
    pending: PendingStore,
    ops: Vec<TxOp>,
}

impl Tx {
    pub(crate) fn new(store: WeakStoreRef, diff: Diff) -> Tx {
        Tx {
            store,
            diff,
            ready: ReadyStore::default(),
            pending: PendingStore::default(),
            ops: Vec::default(),
        }
    }

    pub(crate) fn commit(&mut self) {
        self.prepare()
            .and_then(|_| self.apply())
            .unwrap_or_else(|err| {
                log::error!("Tx commit error: {}", err);
                self.rollback();
            });
    }

    pub(crate) fn prepare(&mut self) -> Result<(), String> {
        Ok(())
    }
    pub(crate) fn apply(&mut self) -> Result<(), String> {
        Ok(())
    }
    pub(crate) fn rollback(&mut self) {
        log::info!("Tx rollback");
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) enum TxOp {
    Insert(ItemData),
    Delete(DeleteItem),
    Split(ItemData, (ItemData, ItemData)),
    #[default]
    None,
}
