use std::collections::HashMap;

use crate::clients::Client;
use crate::codec::decoder::{Decode, Decoder};
use crate::codec::encoder::{Encode, Encoder};
use crate::id::Clock;

#[derive(Debug, Clone, Default)]
pub(crate) struct ClientState {
    pub(crate) clients: HashMap<Client, Clock>,
}

impl ClientState {
    pub(crate) fn new() -> ClientState {
        ClientState {
            clients: HashMap::new(),
        }
    }

    pub(crate) fn get(&self, client: &Client) -> Option<&Clock> {
        self.clients.get(client)
    }

    pub(crate) fn update(&mut self, client: Client, clock: Clock) {
        let current = *self.clients.entry(client).or_default();
        self.clients.insert(client, clock.max(current));
    }
}

impl Encode for ClientState {
    fn encode<E: Encoder>(&self, e: &mut E) {
        e.u32(self.clients.len() as u32);
        for (client, clock) in &self.clients {
            e.u32(*client);
            e.u32(*clock);
        }
    }
}

impl Decode for ClientState {
    fn decode<D: Decoder>(d: &mut D) -> Result<ClientState, String> {
        let len = d.u32()? as usize;
        let mut clients = HashMap::new();
        for _ in 0..len {
            let client = d.u32()?;
            let clock = d.u32()?;
            clients.insert(client, clock);
        }
        Ok(ClientState { clients })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_state() {
        let mut state = ClientState::new();
        state.update(1, 1);
        assert_eq!(state.clients.get(&1), Some(&1));
        state.update(1, 2);
        assert_eq!(state.clients.get(&1), Some(&2));
        state.update(2, 1);
        assert_eq!(state.clients.get(&2), Some(&1));
    }
}
