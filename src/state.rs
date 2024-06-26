use std::collections::BTreeMap;
use std::default::Default;
use std::ops::Add;

use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

use crate::bimapid::{Client, ClientId, ClientMap};
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::id::Clock;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct ClientState {
    pub(crate) state: ClientIdState,
    pub(crate) clients: ClientMap,
}

impl ClientState {
    pub(crate) fn new() -> ClientState {
        ClientState {
            state: ClientIdState::new(),
            clients: ClientMap::new(),
        }
    }

    pub(crate) fn get(&self, client: &ClientId) -> Option<&Clock> {
        self.state.get(client)
    }

    pub(crate) fn remove(&mut self, client: &ClientId) {
        self.state.remove(client);
    }

    pub(crate) fn update(&mut self, client: ClientId, clock: Clock) {
        self.state.update(client, clock);
    }

    pub(crate) fn get_or_insert(&mut self, client: &Client) -> Clock {
        let client_id = self.clients.get_or_insert(client);
        let clock = self.state.get(&client_id).unwrap_or(&0);
        *clock
    }

    pub(crate) fn adjust(&self, other: &ClientState) -> ClientState {
        let clients = self.clients.adjust(&other.clients);
        let state = self.state.adjust(&other.state);

        ClientState { state, clients }
    }
}

impl Add<ClientState> for ClientState {
    type Output = ClientState;

    fn add(self, rhs: ClientState) -> Self::Output {
        &self + &rhs
    }
}

impl Serialize for ClientState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ClientState", 2)?;
        state.serialize_field("state", &self.state)?;
        state.serialize_field("clients", &self.clients)?;
        state.end()
    }
}

impl Encode for ClientState {
    fn encode<E: Encoder>(&self, e: &mut E, ctx: &EncodeContext) {
        self.state.encode(e, ctx);
        self.clients.encode(e, ctx);
    }
}

impl Decode for ClientState {
    fn decode<D: Decoder>(d: &mut D, ctx: &DecodeContext) -> Result<ClientState, String> {
        let state = ClientIdState::decode(d, ctx)?;
        let clients = ClientMap::decode(d, ctx)?;
        Ok(ClientState { state, clients })
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct ClientIdState {
    pub(crate) clients: BTreeMap<ClientId, Clock>,
}

impl ClientIdState {
    pub(crate) fn new() -> ClientIdState {
        ClientIdState {
            clients: BTreeMap::new(),
        }
    }

    pub(crate) fn get(&self, client: &ClientId) -> Option<&Clock> {
        self.clients.get(client)
    }

    pub(crate) fn remove(&mut self, client: &ClientId) {
        self.clients.remove(client);
    }

    pub(crate) fn update(&mut self, client: ClientId, clock: Clock) {
        let current = *self.clients.entry(client).or_default();
        self.clients.insert(client, clock.max(current));
    }

    pub(crate) fn adjust(&self, other: &ClientIdState) -> Self {
        let mut adjust = ClientIdState::new();

        for (client, clock) in &self.clients {
            let other_clock = other.get(client).unwrap_or(&0);
            adjust.update(*client, *clock.min(other_clock));
        }

        adjust
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&ClientId, &Clock)> {
        self.clients.iter()
    }
}

impl Serialize for ClientIdState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.clients.serialize(serializer)
    }
}

impl Encode for ClientIdState {
    fn encode<E: Encoder>(&self, e: &mut E, _ctx: &EncodeContext) {
        e.u32(self.clients.len() as u32);
        for (client, clock) in &self.clients {
            e.u32(*client);
            e.u32(*clock);
        }
    }
}

impl Decode for ClientIdState {
    fn decode<D: Decoder>(d: &mut D, ctx: &DecodeContext) -> Result<ClientIdState, String> {
        let len = d.u32()? as usize;
        let mut clients = BTreeMap::new();
        for _ in 0..len {
            let client = d.u32()?;
            let clock = d.u32()?;
            clients.insert(client, clock);
        }
        Ok(ClientIdState { clients })
    }
}

#[cfg(test)]
mod tests {
    use std::default::Default;

    use crate::codec_v1::EncoderV1;

    use super::*;

    #[test]
    fn test_client_state() {
        let mut state = ClientIdState::new();
        state.update(1, 1);
        assert_eq!(state.clients.get(&1), Some(&1));
        state.update(1, 2);
        assert_eq!(state.clients.get(&1), Some(&2));
        state.update(2, 1);
        assert_eq!(state.clients.get(&2), Some(&1));
    }

    #[test]
    fn test_encode_decode_state() {
        let mut state = ClientIdState::new();
        state.update(1, 1);
        state.update(2, 2);

        let mut encoder = EncoderV1::default();
        state.encode(&mut encoder, &EncodeContext::default());

        let mut d = encoder.decoder();

        let decoded = ClientIdState::decode(&mut d, &DecodeContext::default()).unwrap();

        assert_eq!(state, decoded);
    }
}
