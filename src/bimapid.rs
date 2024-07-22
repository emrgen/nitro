use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;
use std::ops::Add;

use bimap::BiMap;
use serde::{Serialize, Serializer};

use crate::Client;
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::mark::Mark;

pub type ClientId = u32;

pub(crate) trait BiMapEntry:
    Encode + Decode + Clone + Default + Eq + PartialEq + Hash
{
}

pub(crate) trait EncoderMapEntry:
    Debug + Encode + Decode + Clone + Default + Eq + PartialEq + Hash
{
}

impl<T: Debug + Encode + Decode + Clone + Default + Eq + PartialEq + Hash> EncoderMapEntry for T {}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct EncoderMap<L: EncoderMapEntry> {
    map: BiMap<L, u32>,
}

impl<T: EncoderMapEntry> EncoderMap<T> {
    pub fn new() -> EncoderMap<T> {
        EncoderMap { map: BiMap::new() }
    }

    pub fn size(&self) -> usize {
        self.map.len()
    }

    pub fn remove_by_right(&mut self, right: &u32) {
        self.map.remove_by_right(right);
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

    // insert self clients into other clients
    pub fn as_per(&self, other: &EncoderMap<T>) -> EncoderMap<T> {
        let mut clone = other.clone();
        let mut entries = self.map.iter().collect::<Vec<_>>();
        entries.sort_by(|a, b| a.1.cmp(b.1));

        for (l, _) in entries {
            clone.get_or_insert(l);
        }

        // println!("final: {:?}", clone);
        clone
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

impl<T: EncoderMapEntry> Add<EncoderMap<T>> for EncoderMap<T> {
    type Output = EncoderMap<T>;

    fn add(self, rhs: EncoderMap<T>) -> Self::Output {
        &self + &rhs
    }
}

impl<T: EncoderMapEntry> Add<&EncoderMap<T>> for &EncoderMap<T> {
    type Output = EncoderMap<T>;

    fn add(self, rhs: &EncoderMap<T>) -> Self::Output {
        let mut clone = self.clone();

        for (client, client_id) in rhs.map.iter() {
            clone.insert(client.clone(), *client_id);
        }

        clone
    }
}

// impl<T: PartialEq + Default + Eq + Hash + Clone + Encode + Decode> PartialEq for EncoderMap<T> {
//     fn eq(&self, other: &Self) -> bool {
//         return true;
//         let self_keys: Vec<&T> = self.map.left_values().collect();
//         let other_keys: Vec<&T> = other.map.left_values().collect();
//
//         if self_keys.len() != other_keys.len() {
//             return false;
//         }
//
//         for key in self_keys {
//             if !other.map.contains_left(key) {
//                 return false;
//             }
//         }
//
//         true
//     }
// }

impl Encode for EncoderMap<Client> {
    fn encode<E: Encoder>(&self, e: &mut E, _ctx: &EncodeContext) {
        e.u32(self.size() as u32);
        for (client, client_id) in self.map.iter() {
            client.encode(e, _ctx);
            e.u32(*client_id);
        }
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
    fn encode<T: Encoder>(&self, e: &mut T, ctx: &EncodeContext) {
        let len = self.map.len();
        e.u32(len as u32);
        if len > u16::MAX as usize {
            for (mark, mark_id) in self.map.iter() {
                mark.encode(e, ctx);
                e.u32(*mark_id);
            }
        } else if len > u8::MAX as usize {
            for (mark, mark_id) in self.map.iter() {
                mark.encode(e, ctx);
                e.u16(*mark_id as u16);
            }
        } else {
            for (mark, mark_id) in self.map.iter() {
                mark.encode(e, ctx);
                e.u8(*mark_id as u8);
            }
        }
    }
}

impl Decode for EncoderMap<Mark> {
    fn decode<D: Decoder>(d: &mut D, _ctx: &DecodeContext) -> Result<EncoderMap<Mark>, String> {
        let len = d.u32()? as usize;
        let mut map = BiMap::new();
        if len > u16::MAX as usize {
            for _ in 0..len {
                let mark = Mark::decode(d, _ctx)?;
                let mark_id = d.u32()?;
                map.insert(mark, mark_id as u32);
            }
        } else if len > u8::MAX as usize {
            for _ in 0..len {
                let mark = Mark::decode(d, _ctx)?;
                let mark_id = d.u16()?;
                map.insert(mark, mark_id as u32);
            }
        } else {
            for _ in 0..len {
                let mark = Mark::decode(d, _ctx)?;
                let mark_id = d.u8()?;
                map.insert(mark, mark_id as u32);
            }
        }

        Ok(Self { map })
    }
}

// #[derive(Debug, Clone, Default)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ClientMap {
    map: EncoderMap<Client>,
}

impl ClientMap {
    pub(crate) fn extend(&mut self, other: &ClientMap) {
        for (client, client_id) in other.iter() {
            self.map.insert(client.clone(), *client_id);
        }
    }
}

impl ClientMap {
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&Client, &u32)> {
        self.map.map.iter()
    }
}

impl ClientMap {
    pub(crate) fn remove(&mut self, id: &ClientId) {
        self.map.remove_by_right(id);
    }
}

impl ClientMap {
    pub(crate) fn new() -> ClientMap {
        ClientMap {
            map: EncoderMap::new(),
        }
    }

    pub(crate) fn size(&self) -> u32 {
        self.map.size() as u32
    }
    pub(crate) fn insert(&mut self, client: Client, client_id: ClientId) {
        self.map.map.insert(client, client_id);
    }

    pub(crate) fn get_or_insert(&mut self, client: &Client) -> ClientId {
        self.map.get_or_insert(client)
    }

    pub(crate) fn contains_client(&self, client: &Client) -> bool {
        self.map.get(client).is_some()
    }

    pub(crate) fn remove_client(&mut self, client: &Client) {
        self.map.map.remove_by_left(client);
    }

    pub(crate) fn get_client_id(&self, client_id: &Client) -> Option<&ClientId> {
        self.map.get(client_id)
    }

    pub(crate) fn get_client(&self, client: &ClientId) -> Option<&Client> {
        self.map.get_key(client)
    }

    pub(crate) fn as_per(&self, other: &ClientMap) -> ClientMap {
        let map = self.map.as_per(&other.map);
        ClientMap { map }
    }

    pub(crate) fn merge(&self, other: &ClientMap) -> Self {
        let map = self.map.merge(&other.map);
        ClientMap { map }
    }

    pub(crate) fn entries(&self) -> impl Iterator<Item = (&Client, &u32)> {
        self.map.map.iter()
    }
}

impl Serialize for ClientMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let map = self.map.map.iter().collect::<HashMap<_, _>>();
        map.serialize(serializer)
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
            let client = Client::decode(decoder, _ctx)?;
            let client_id = decoder.u32()?;
            map.insert(client, client_id);
        }
        Ok(Self {
            map: EncoderMap { map },
        })
    }
}

pub(crate) type Field = String;
pub(crate) type FieldId = u32;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FieldMap {
    map: EncoderMap<String>,
}

impl FieldMap {
    pub(crate) fn extend(&mut self, other: &FieldMap) {
        for (field, field_id) in other.iter() {
            self.map.insert(field.clone(), *field_id);
        }
    }
}

impl FieldMap {
    pub(crate) fn new() -> FieldMap {
        FieldMap {
            map: EncoderMap::new(),
        }
    }

    fn insert(&mut self, field: String, field_id: FieldId) {
        self.map.map.insert(field, field_id);
    }

    pub(crate) fn get_or_insert(&mut self, client_id: &Field) -> FieldId {
        self.map.get_or_insert(client_id)
    }

    pub(crate) fn get_field_id(&self, field_id: &Field) -> Option<&FieldId> {
        self.map.get(field_id)
    }

    pub(crate) fn get_field(&self, field_id: &FieldId) -> Option<&Field> {
        self.map.get_key(field_id)
    }

    // append self clients into other clients and return new clients
    pub(crate) fn as_per(&self, other: &FieldMap) -> FieldMap {
        let map = self.map.as_per(&other.map);
        FieldMap { map }
    }

    pub(crate) fn merge(&self, other: &FieldMap) -> Self {
        let map = self.map.merge(&other.map);
        FieldMap { map }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&String, &u32)> {
        self.map.map.iter()
    }
}

impl Serialize for FieldMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let map = self.map.map.iter().collect::<HashMap<_, _>>();
        map.serialize(serializer)
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

#[cfg(test)]
mod test {
    use crate::bimapid::ClientMap;
    use crate::Client;
    use crate::codec_v1::EncoderV1;
    use crate::decoder::{Decode, DecodeContext};
    use crate::encoder::{Encode, EncodeContext, Encoder};

    #[test]
    fn test_encode_decode_client_map() {
        let mut map = ClientMap::default();

        map.insert(Client::default(), 1);
        map.insert(Client::default(), 2);
        map.insert(Client::default(), 3);

        let mut e = EncoderV1::new();
        map.encode(&mut e, &EncodeContext::default());

        let mut d = e.decoder();

        let dd = ClientMap::decode(&mut d, &DecodeContext::default()).unwrap();

        assert_eq!(map, dd);
    }
}
