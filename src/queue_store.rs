use std::collections::BTreeMap;
use std::fmt::Debug;

use serde::Serialize;

use crate::bimapid::ClientId;
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::id::WithId;

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub(crate) struct ClientQueryStore<T: QueryStoreEntry> {
    pub(crate) items: BTreeMap<ClientId, QueueStore<T>>,
}

impl<T: QueryStoreEntry> ClientQueryStore<T> {
    pub(crate) fn new() -> Self {
        Self {
            items: BTreeMap::new(),
        }
    }

    pub(crate) fn size(&self, client_id: ClientId) -> usize {
        self.items
            .get(&client_id)
            .map_or(0, |store| store.vec.len())
    }

    pub(crate) fn take_first(&mut self, client_id: ClientId) -> Option<T> {
        self.get_store(client_id).pop().cloned()
    }

    pub(crate) fn insert(&mut self, entry: T) {
        let client_id = entry.id().client;
        self.get_store(client_id).push(entry);
    }

    pub(crate) fn pop(&mut self, client_id: ClientId) -> Option<&T> {
        self.get_store(client_id).pop()
    }

    fn get_store(&mut self, client_id: ClientId) -> &mut QueueStore<T> {
        self.items.entry(client_id).or_insert_with(QueueStore::new)
    }

    pub(crate) fn remove(&mut self, client_id: ClientId) {
        self.items.remove(&client_id);
    }

    pub(crate) fn clear(&mut self) {
        self.items.clear();
    }
}

impl<T: QueryStoreEntry> Encode for ClientQueryStore<T> {
    fn encode<E: Encoder>(&self, e: &mut E, ctx: &EncodeContext) {
        e.u32(self.items.len() as u32);
        for (client_id, store) in &self.items {
            e.u32(*client_id);
            store.encode(e, ctx)
        }
    }
}

impl<T: QueryStoreEntry> Decode for ClientQueryStore<T> {
    fn decode<D: Decoder>(d: &mut D, ctx: &DecodeContext) -> Result<Self, String> {
        let mut items = BTreeMap::new();
        let len = d.u32()? as usize;
        for _ in 0..len {
            let client_id = d.u32()?;
            let store = QueueStore::decode(d, ctx)?;
            items.insert(client_id, store);
        }
        Ok(Self { items })
    }
}

pub(crate) trait QueryStoreEntry:
    Clone + Debug + Default + Encode + Decode + Serialize + WithId
{
}
impl<T: Debug + Default + Encode + Decode + Serialize + Clone + WithId> QueryStoreEntry for T {}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub(crate) struct QueueStore<T: QueryStoreEntry> {
    vec: Vec<T>,
    pub(crate) pos: usize,
}

impl<T: QueryStoreEntry> QueueStore<T> {
    pub(crate) fn new() -> Self {
        Self {
            vec: Vec::new(),
            pos: 0,
        }
    }

    pub(crate) fn push(&mut self, entry: T) {
        self.vec.push(entry);
    }

    pub(crate) fn pop(&mut self) -> Option<&T> {
        if self.pos < self.vec.len() {
            let entry = &self.vec[self.pos];
            self.pos += 1;
            Some(entry)
        } else {
            None
        }
    }

    pub(crate) fn reset(&mut self) {
        self.pos = 0;
    }

    pub(crate) fn clear(&mut self) {
        self.vec.clear();
        self.pos = 0;
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &T> {
        self.vec.iter()
    }
}

impl<T: QueryStoreEntry> Encode for QueueStore<T> {
    fn encode<E: Encoder>(&self, e: &mut E, ctx: &EncodeContext) {
        e.u32(self.vec.len() as u32);
        for entry in &self.vec {
            entry.encode(e, ctx);
        }
    }
}

impl<T: QueryStoreEntry> Decode for QueueStore<T> {
    fn decode<D: Decoder>(d: &mut D, ctx: &DecodeContext) -> Result<Self, String> {
        let len = d.u32()? as usize;
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::decode(d, ctx)?);
        }

        Ok(Self { vec, pos: 0 })
    }
}

#[cfg(test)]
mod test {
    use crate::id::Id;
    use crate::queue_store::QueueStore;
    use crate::store::IdStore;

    #[test]
    fn test_queue_vs_store() {
        let now = std::time::Instant::now();
        let mut queue = QueueStore::default();
        for i in 0..50000 {
            queue.push(Id::new(0, i as u32));
        }

        println!("queue time: {:?}", now.elapsed());

        let now = std::time::Instant::now();
        let mut store = IdStore::default();
        for i in 0..50000 {
            store.insert(Id::new(0, i as u32));
        }
        println!("store time: {:?}", now.elapsed());
    }
}
