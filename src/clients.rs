use crate::codec::decoder::{Decode, Decoder};
use crate::codec::encoder::{Encode, Encoder};
use bimap::BiMap;

pub type Client = u32;
pub type ClientId = String;

#[derive(Debug, Clone, Default)]
pub(crate) struct ClientMap {
    pub(crate) map: BiMap<ClientId, Client>,
}

impl ClientMap {
    pub(crate) fn new() -> ClientMap {
        ClientMap { map: BiMap::new() }
    }

    pub(crate) fn insert(&mut self, client_id: ClientId, client: Client) {
        self.map.insert(client_id, client);
    }

    pub(crate) fn get_by_client_id(&self, client_id: &ClientId) -> Option<&Client> {
        self.map.get_by_left(client_id)
    }

    pub(crate) fn get_by_client(&self, client: &Client) -> Option<&ClientId> {
        self.map.get_by_right(client)
    }

    pub(crate) fn get_or_insert(&mut self, client_id: ClientId) -> Client {
        match self.get_by_client_id(&client_id) {
            Some(client) => *client,
            None => {
                let client = self.map.len() as Client;
                self.insert(client_id, client);
                client
            }
        }
    }
}

impl Encode for ClientMap {
    fn encode<E: Encoder>(&self, encoder: &mut E) {
        encoder.u32(self.map.len() as Client);
        for (client_id, client) in self.map.iter() {
            encoder.string(client_id);
            encoder.u32(*client);
        }
    }
}

impl Decode for ClientMap {
    fn decode<D: Decoder>(decoder: &mut D) -> Result<ClientMap, String> {
        let len = decoder.u32()? as usize;
        let mut map = BiMap::new();
        for _ in 0..len {
            let client_id = decoder.string()?;
            let client = decoder.u32()?;
            map.insert(client_id, client);
        }
        Ok(ClientMap { map })
    }
}
