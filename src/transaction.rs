use std::cell::Ref;
use std::collections::{BTreeMap, HashMap};

use crate::bimapid::ClientId;
use crate::crdt::integrate;
use crate::delete::DeleteItem;
use crate::diff::Diff;
use crate::id::WithId;
use crate::item::ItemData;
use crate::store::{
    ClientStore, DocStore, ItemDataStore, ItemStore, PendingStore, ReadyStore, WeakStoreRef,
};

#[derive(Debug, Clone, Default)]
pub(crate) struct Transaction {
    store: WeakStoreRef,
    ready: ReadyStore,
    pending: PendingStore,

    diff: Diff,
    ops: Vec<TxOp>,
}

impl Transaction {
    pub(crate) fn new(store: WeakStoreRef, diff: Diff) -> Transaction {
        let mut_store = store.upgrade().unwrap();
        let store_ref = mut_store.borrow_mut();
        let diff = diff.adjust(&store_ref);
        Transaction {
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
        let store = self.store.upgrade().unwrap();
        let store = store.borrow();

        for (_, store) in self.diff.items.iter() {
            for (_, data) in store.iter() {
                self.pending.insert(data.clone());
            }
        }

        for (_, store) in self.diff.deletes.iter() {
            for (_, data) in store.iter() {
                self.pending.insert_delete(data.clone());
            }
        }

        // if all item dependencies are satisfied put the item in ready store
        let mut stage: BTreeMap<ClientId, ItemData> = BTreeMap::new();
        for (client, store) in self.pending.iter_items() {
            // take the first item from pending store
            if store.is_empty() {
                continue;
            }

            let (_, data) = store.iter().next().unwrap();
            stage.insert(*client, data.clone());
        }

        for (_, data) in &stage {
            self.pending.remove(&data.id);
        }

        let mut progress = false;
        let mut count = 0;
        loop {
            if stage.is_empty() {
                break;
            }

            let clients = stage.keys().cloned().collect::<Vec<_>>();

            for client_id in clients {
                if let Some(item) = stage.get(&client_id) {
                    let id = item.id;
                    let clone = item.clone();

                    if self.is_integrated(item, &store) {
                        progress = true;
                    } else if self.is_ready(item, &store) {
                        progress = true;
                        self.ready.insert(item.clone());
                    } else if self.is_orphan(item) {
                        self.pending.insert(item.clone());
                        progress = true;
                    }

                    if progress {
                        if let Some(data) = self.pending.take_first(&client_id) {
                            stage.insert(client_id, data);
                        } else {
                            stage.remove(&client_id);
                        }
                    }

                    count += 1;
                    if count > 6 {
                        println!("Item: {:?}", id);
                        panic!("Infinite loop while collecting client ready items");
                    }
                }
            }

            // if no progress is made, break the loop
            if !progress {
                break;
            }

            if count > 1000 {
                panic!("Infinite loop while collecting ready items");
            }

            progress = false;
        }
        //
        // // remaining items has unmet dependencies and are put in pending store
        for (_, data) in stage.iter() {
            self.pending.insert(data.clone());
        }

        // now that all ready items are collected, collected the ready delete items
        for (_, store) in self.pending.iter_delete_items() {
            for (_, data) in store.iter() {
                let id = data.range().id();
                if self.ready.contains(&id) || store.contains(&id) {
                    self.ready.insert_delete(data.clone());
                }
            }
        }

        Ok(())
    }
    pub(crate) fn apply(&mut self) -> Result<(), String> {
        println!("ready count: {}", self.ready.queue.len());

        let fields = self.store.upgrade().unwrap().borrow().fields.clone();

        self.store.upgrade().unwrap().borrow_mut().fields = fields.adjust(&self.diff.fields);

        self.ready.queue.reverse();

        while let Some(data) = self.ready.queue.pop() {
            let parent = {
                let store = self.store.upgrade().unwrap();
                let store = store.borrow();
                if let Some(parent_id) = &data.parent_id {
                    store.find(*parent_id)
                } else if let Some(left_id) = &data.left_id {
                    store.find(*left_id).and_then(|item| item.parent())
                } else if let Some(right_id) = &data.right_id {
                    store.find(*right_id).and_then(|item| item.parent())
                } else {
                    None
                }
            };

            if let Some(parent) = parent {
                integrate(
                    data,
                    &self.store.clone(),
                    parent.clone(),
                    parent.start(),
                    |start| {
                        parent.set_start(start);
                        Ok(())
                    },
                )?;
            }
        }

        Ok(())
    }

    pub(crate) fn rollback(&mut self) {
        log::info!("Tx rollback");
    }

    pub(crate) fn is_ready(&self, data: &ItemData, store: &Ref<DocStore>) -> bool {
        if data.is_root() {
            return true;
        }

        if let Some(parent_id) = data.parent_id {
            if !(self.ready.contains(&parent_id) || store.contains(&parent_id)) {
                return false;
            }
        }

        if let Some(left_id) = data.left_id {
            if !(self.ready.contains(&left_id) || store.contains(&left_id)) {
                return false;
            }
        }

        if let Some(right_id) = data.right_id {
            if !(self.ready.contains(&right_id) || store.contains(&right_id)) {
                return false;
            }
        }

        true
    }

    pub(crate) fn is_orphan(&self, data: &ItemData) -> bool {
        if data.is_root() {
            return false;
        }

        data.parent_id.is_none() || data.left_id.is_none() || data.right_id.is_none()
    }

    pub(crate) fn is_integrated(&self, data: &ItemData, store: &Ref<DocStore>) -> bool {
        store.contains(&data.id)
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
