use crate::codec::encoder::Encoder;
use bimap::BiMap;

pub type Client = u64;
pub type ClientId = String;

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
                let client = self.map.len() as u64;
                self.insert(client_id, client);
                client
            }
        }
    }

    pub(crate) fn encode<T: Encoder>(&self, encoder: &mut T) {
        encoder.u32(self.map.len() as u32);
        for (client_id, client) in self.map.iter() {
            encoder.string(client_id);
            encoder.u64(*client);
        }
    }
}
