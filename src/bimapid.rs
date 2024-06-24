use std::hash::Hash;

use bimap::BiMap;

use crate::codec::decoder::{Decode, DecodeContext, Decoder};
use crate::codec::encoder::{Encode, EncodeContext, Encoder};
use crate::mark::Mark;

pub type Client = String;
pub type ClientId = u32;

pub(crate) trait BiMapEntry:
    Encode + Decode + Clone + Default + Eq + PartialEq + Hash
{
}

#[derive(Debug, Clone, Default)]
pub(crate) struct EncoderMap<T: Clone + Default + PartialEq + Eq + Hash> {
    pub map: BiMap<T, u32>,
}

impl<T: Clone + Default + PartialEq + Eq + Hash> EncoderMap<T> {
    pub fn new() -> EncoderMap<T> {
        EncoderMap { map: BiMap::new() }
    }

    pub fn size(&self) -> usize {
        self.map.len()
    }

    pub fn insert(&mut self, key: T, value: u32) {
        self.map.insert(key, value);
    }

    pub fn get_or_insert(&mut self, key: &T) -> u32 {
        match self.map.get_by_left(key) {
            Some(&id) => id,
            None => {
                let id = self.map.len() as u32;
                self.map.insert(key.clone(), id);
                id
            }
        }
    }

    pub fn get(&self, key: &T) -> Option<&u32> {
        self.map.get_by_left(key)
    }

    pub fn get_key(&self, id: &u32) -> Option<&T> {
        self.map.get_by_right(id)
    }

    pub fn adjust(&self, other: &EncoderMap<T>) -> EncoderMap<T> {
        let mut adjust = EncoderMap::new();
        let mut clone = self.clone();
        for (key, _) in other.map.iter() {
            let id = clone.get_or_insert(key);
            adjust.map.insert(key.clone(), id);
        }

        adjust
    }

    pub(crate) fn merge(&self, other: &EncoderMap<T>) -> Self {
        let mut merged = Self::default();
        for (client, client_id) in self.map.iter() {
            merged.insert(client.clone(), *client_id);
        }

        for (client, client_id) in other.map.iter() {
            merged.insert(client.clone(), *client_id);
        }

        merged
    }
}

impl Encode for EncoderMap<String> {
    fn encode<E: Encoder>(&self, e: &mut E, _ctx: &EncodeContext) {
        e.u32(self.size() as u32);
        for (client_id, client) in self.map.iter() {
            let size = client_id.len();
            let client_id = client_id.as_bytes();
            e.u8(size as u8);
            e.slice(client_id);
            e.u32(*client);
        }
    }
}

impl Decode for EncoderMap<String> {
    fn decode<D: Decoder>(
        decoder: &mut D,
        _ctx: &DecodeContext,
    ) -> Result<EncoderMap<String>, String> {
        let len = decoder.u32()? as usize;
        let mut map = BiMap::new();
        for _ in 0..len {
            let size = decoder.u8()? as usize;
            let slice = decoder.slice(size)?;
            let client_id = String::from_utf8(slice.into()).map_err(|e| e.to_string())?;
            let client = decoder.u32()?;
            map.insert(client_id, client);
        }
        Ok(Self { map })
    }
}

impl Encode for EncoderMap<Mark> {
    fn encode<T: Encoder>(&self, e: &mut T, _ctx: &EncodeContext) {
        e.u32(self.size() as u32);
        for (client_id, client) in self.map.iter() {
            // let size = client_id.len();
            // let client_id = client_id.as_bytes();
            // e.u8(size as u8);
            // e.slice(client_id);
            // e.u32(*client);
        }
    }
}

impl Decode for EncoderMap<Mark> {
    fn decode<D: Decoder>(d: &mut D, _ctx: &DecodeContext) -> Result<EncoderMap<Mark>, String> {
        todo!()
    }
}

// #[derive(Debug, Clone, Default)]
#[derive(Clone, Debug, Default)]
pub(crate) struct ClientMap {
    map: EncoderMap<String>,
}

impl ClientMap {
    pub(crate) fn new() -> ClientMap {
        ClientMap {
            map: EncoderMap::new(),
        }
    }

    fn insert(&mut self, client_id: Client, client: ClientId) {
        self.map.map.insert(client_id, client);
    }

    pub(crate) fn get_or_insert(&mut self, client_id: &Client) -> ClientId {
        self.map.get_or_insert(client_id)
    }

    pub(crate) fn get_client_id(&self, client_id: &Client) -> Option<&ClientId> {
        self.map.get(client_id)
    }

    pub(crate) fn get_client(&self, client: &ClientId) -> Option<&Client> {
        self.map.get_key(client)
    }

    pub(crate) fn adjust(&self, other: &ClientMap) -> ClientMap {
        let map = self.map.adjust(&other.map);
        ClientMap { map }
    }

    pub(crate) fn merge(&self, other: &ClientMap) -> Self {
        let map = self.map.merge(&other.map);
        ClientMap { map }
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = (&String, &u32)> {
        self.map.map.iter()
    }
}

impl Encode for ClientMap {
    fn encode<E: Encoder>(&self, encoder: &mut E, ctx: &EncodeContext) {
        self.map.encode(encoder, ctx);
    }
}

impl Decode for ClientMap {
    fn decode<D: Decoder>(decoder: &mut D, _ctx: &DecodeContext) -> Result<ClientMap, String> {
        let len = decoder.u32()? as usize;
        let mut map = BiMap::new();
        for _ in 0..len {
            let size = decoder.u8()? as usize;
            let slice = decoder.slice(size)?;
            let client_id = String::from_utf8(slice.into()).map_err(|e| e.to_string())?;
            let client = decoder.u32()?;
            map.insert(client_id, client);
        }
        Ok(Self {
            map: EncoderMap { map },
        })
    }
}

pub(crate) type Field = String;
pub(crate) type FieldId = u32;

#[derive(Clone, Debug, Default)]
pub(crate) struct FieldMap {
    map: EncoderMap<String>,
}

impl FieldMap {
    pub(crate) fn new() -> FieldMap {
        FieldMap {
            map: EncoderMap::new(),
        }
    }

    fn insert(&mut self, client_id: Client, client: ClientId) {
        self.map.map.insert(client_id, client);
    }

    pub(crate) fn get_or_insert(&mut self, client_id: &Field) -> FieldId {
        self.map.get_or_insert(client_id)
    }

    pub(crate) fn get_field_id(&self, field_id: &Field) -> Option<&FieldId> {
        self.map.get(field_id)
    }

    pub(crate) fn get_field(&self, client: &FieldId) -> Option<&Field> {
        self.map.get_key(client)
    }

    pub(crate) fn adjust(&self, other: &FieldMap) -> FieldMap {
        let map = self.map.adjust(&other.map);
        FieldMap { map }
    }

    pub(crate) fn merge(&self, other: &FieldMap) -> Self {
        let map = self.map.merge(&other.map);
        FieldMap { map }
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = (&String, &u32)> {
        self.map.map.iter()
    }
}

impl Encode for FieldMap {
    fn encode<E: Encoder>(&self, encoder: &mut E, ctx: &EncodeContext) {
        self.map.encode(encoder, ctx);
    }
}

impl Decode for FieldMap {
    fn decode<D: Decoder>(decoder: &mut D, ctx: &DecodeContext) -> Result<FieldMap, String> {
        let map = EncoderMap::decode(decoder, ctx)?;
        Ok(FieldMap { map })
    }
}
