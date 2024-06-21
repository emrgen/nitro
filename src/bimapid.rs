use std::hash::Hash;

use bimap::BiMap;

use crate::codec::decoder::{Decode, Decoder};
use crate::codec::encoder::{Encode, Encoder};

pub type Client = String;
pub type ClientId = u32;

pub(crate) trait BiMapEntry:
    Encode + Decode + Clone + Default + Eq + PartialEq + Hash
{
}

#[derive(Debug, Clone, Default)]
pub(crate) struct EncoderMap {
    pub map: BiMap<String, u32>,
}

impl EncoderMap {
    pub fn new() -> EncoderMap {
        EncoderMap { map: BiMap::new() }
    }

    pub fn insert(&mut self, key: String, value: u32) {
        self.map.insert(key, value);
    }

    pub fn get_or_insert(&mut self, key: &str) -> u32 {
        match self.map.get_by_left(key) {
            Some(&id) => id,
            None => {
                let id = self.map.len() as u32;
                self.map.insert(key.to_string(), id);
                id
            }
        }
    }

    pub fn get(&self, key: &str) -> Option<&u32> {
        self.map.get_by_left(key)
    }

    pub fn get_key(&self, id: &u32) -> Option<&String> {
        self.map.get_by_right(id)
    }

    pub fn adjust(&self, other: &EncoderMap) -> EncoderMap {
        let mut adjust = EncoderMap::new();
        let mut clone = self.clone();
        for (key, _) in other.map.iter() {
            let id = clone.get_or_insert(key);
            adjust.map.insert(key.clone(), id);
        }

        adjust
    }

    pub(crate) fn merge(&self, other: &EncoderMap) -> Self {
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

impl Encode for EncoderMap {
    fn encode<E: Encoder>(&self, encoder: &mut E) {
        encoder.u32(self.map.len() as ClientId);
        for (client_id, client) in self.map.iter() {
            encoder.string(client_id);
            encoder.u32(*client);
        }
    }
}

impl Decode for EncoderMap {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<EncoderMap, String> {
        let len = decoder.u32()? as usize;
        let mut map = BiMap::new();
        for _ in 0..len {
            let client_id = decoder.string()?;
            let client = decoder.u32()?;
            map.insert(client_id, client);
        }
        Ok(Self { map })
    }
}

// #[derive(Debug, Clone, Default)]
#[derive(Clone, Debug, Default)]
pub(crate) struct ClientMap {
    map: EncoderMap,
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
    fn encode<E: Encoder>(&self, encoder: &mut E) {
        self.map.encode(encoder);
    }
}

impl Decode for ClientMap {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<ClientMap, String> {
        let map = EncoderMap::decode(decoder)?;
        Ok(ClientMap { map })
    }
}

pub(crate) type Field = String;
pub(crate) type FieldId = u32;

#[derive(Clone, Debug, Default)]
pub(crate) struct FieldMap {
    map: EncoderMap,
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
    fn encode<E: Encoder>(&self, encoder: &mut E) {
        self.map.encode(encoder);
    }
}

impl Decode for FieldMap {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<FieldMap, String> {
        let map = EncoderMap::decode(decoder)?;
        Ok(FieldMap { map })
    }
}
