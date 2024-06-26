use std::fmt::Debug;
use std::hash::Hash;

use bimap::BiMap;

use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::mark::Mark;

pub type Client = String;
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

    pub fn adjust(&self, other: &EncoderMap<T>) -> EncoderMap<T> {
        let mut clone = other.clone();
        let mut entries = other.map.iter().collect::<Vec<_>>();
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
    map: EncoderMap<String>,
}

impl ClientMap {
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&String, &u32)> {
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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

#[cfg(test)]
mod test {
    use crate::bimapid::ClientMap;
    use crate::codec_v1::EncoderV1;
    use crate::decoder::{Decode, DecodeContext};
    use crate::encoder::{Encode, EncodeContext, Encoder};

    #[test]
    fn test_encode_decode_client_map() {
        let mut map = ClientMap::default();

        map.insert("client1".to_string(), 1);
        map.insert("client2".to_string(), 2);
        map.insert("client3".to_string(), 3);

        let mut e = EncoderV1::new();
        map.encode(&mut e, &EncodeContext::default());

        let mut d = e.decoder();

        let dd = ClientMap::decode(&mut d, &DecodeContext::default()).unwrap();

        assert_eq!(map, dd);
    }
}
