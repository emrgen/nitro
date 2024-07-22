use std::collections::BTreeMap;
use std::default::Default;
use std::ops::Add;

use serde::{Serialize, Serializer};
use serde::ser::SerializeStruct;

use crate::bimapid::{ClientId, ClientMap};
use crate::Client;
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::id::Clock;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct ClientState {
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

    pub fn clients(&self) -> Vec<(&Client, u32)> {
        self.clients.iter().map(|(k, v)| (k, *v)).collect()
    }

    pub fn state(&self) -> Vec<(u32, u32)> {
        self.state.clients.iter().map(|(k, v)| (*k, *v)).collect()
    }

    pub(crate) fn get(&self, client: &ClientId) -> Option<&Clock> {
        self.state.get(client)
    }

    pub(crate) fn remove(&mut self, client: &ClientId) {
        self.state.remove(client);
        self.state.remove(client);
        self.state.remove(client);
    }

    pub(crate) fn update(&mut self, client: ClientId, clock: Clock) {
        self.state.update_max(client, clock);
    }

    pub(crate) fn get_client(&self, client: &ClientId) -> Option<&Client> {
        self.clients.get_client(client)
    }

    pub(crate) fn get_client_id(&self, client: &Client) -> Option<&ClientId> {
        self.clients.get_client_id(client)
    }

    pub(crate) fn get_or_insert(&mut self, client: &Client) -> (ClientId, Clock) {
        let client_id = self.clients.get_or_insert(client);
        let clock = self.state.get(&client_id).unwrap_or(&0);
        (client_id, *clock)
    }

    pub(crate) fn adjust_max(&self, other: &ClientState) -> ClientState {
        let clients = self.clients.as_per(&other.clients);
        let mut state = ClientIdState::default();

        clients.iter().for_each(|(client, client_id)| {
            let self_clock = self
                .clients
                .get_client_id(client)
                .and_then(|id| self.get(id));
            let other_clock = other
                .clients
                .get_client_id(client)
                .and_then(|id| other.get(id));

            match (self_clock, other_clock) {
                (Some(self_clock), Some(other_clock)) => {
                    state.update(*client_id, *other_clock.max(self_clock));
                }
                (None, Some(other_clock)) => {
                    state.update(*client_id, *other_clock);
                }
                (Some(self_clock), None) => {
                    state.update(*client_id, *self_clock);
                }
                _ => {}
            }
        });

        ClientState { state, clients }
    }

    pub(crate) fn adjust_min(&self, other: &ClientState) -> ClientState {
        let clients = self.clients.as_per(&other.clients);
        let mut state = ClientIdState::default();

        clients.iter().for_each(|(client, client_id)| {
            let self_clock = self
                .clients
                .get_client_id(client)
                .and_then(|id| self.get(id));
            let other_clock = other
                .clients
                .get_client_id(client)
                .and_then(|id| other.get(id));

            match (self_clock, other_clock) {
                (Some(self_clock), Some(other_clock)) => {
                    if *self_clock < *other_clock {
                        state.update(*client_id, *self_clock);
                    } else {
                        state.update(*client_id, *other_clock);
                    }
                }
                (None, Some(other_clock)) => {
                    state.update(*client_id, *other_clock);
                }
                (Some(self_clock), None) => {
                    state.update(*client_id, *self_clock);
                }
                _ => {}
            }
        });

        ClientState { state, clients }
    }

    // get modified view of self as per other state. the final client ids should match with other state.
    pub(crate) fn as_per(&self, other: &ClientState) -> ClientState {
        let clients = self.clients.as_per(&other.clients);

        let mut state = ClientIdState::default();
        for (client, client_id) in clients.iter() {
            let self_clock = self
                .clients
                .get_client_id(client)
                .and_then(|id| self.state.get(id));
            let other_clock = other
                .clients
                .get_client_id(client)
                .and_then(|id| other.state.get(id));

            match (self_clock, other_clock) {
                (Some(self_clock), Some(other_clock)) => {
                    state.update(*client_id, *self_clock);
                }
                (None, Some(other_clock)) => {
                    // client id does not exists in self
                    state.update(*client_id, 0);
                }
                (Some(self_clock), None) => {
                    state.update(*client_id, *self_clock);
                }
                _ => {}
            }
        }

        ClientState { clients, state }
    }

    pub(crate) fn merge(&self, other: &ClientState) -> ClientState {
        let clients = self.clients.merge(&other.clients);
        let state = self
            .state
            .iter()
            .fold(other.state.clone(), |mut state, (client_id, clock)| {
                state.update_max(*client_id, *clock);
                state
            });

        ClientState { state, clients }
    }
}

impl From<&ClientState> for ClientState {
    fn from(state: &ClientState) -> Self {
        state.clone()
    }
}

impl Add<ClientState> for ClientState {
    type Output = ClientState;

    fn add(self, rhs: ClientState) -> Self::Output {
        &self + &rhs
    }
}

impl Add<&ClientState> for &ClientState {
    type Output = ClientState;

    fn add(self, rhs: &ClientState) -> Self::Output {
        self.adjust_max(rhs)
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

    pub(crate) fn update_max(&mut self, client: ClientId, clock: Clock) {
        let current = *self.clients.entry(client).or_default();
        self.clients.insert(client, clock.max(current));
    }

    pub(crate) fn update_min(&mut self, client: ClientId, clock: Clock) {
        let current = *self.clients.entry(client).or_default();
        self.clients.insert(client, clock.min(current));
    }

    pub(crate) fn update(&mut self, client: ClientId, clock: Clock) {
        self.clients.insert(client, clock);
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

    use miniz_oxide::deflate::compress_to_vec;
    use uuid::Uuid;

    use crate::codec_v1::EncoderV1;

    use super::*;

    #[test]
    fn test_client_state() {
        let mut state = ClientIdState::new();
        state.update_max(1, 1);
        assert_eq!(state.clients.get(&1), Some(&1));
        state.update_max(1, 2);
        assert_eq!(state.clients.get(&1), Some(&2));
        state.update_max(2, 1);
        assert_eq!(state.clients.get(&2), Some(&1));
    }

    #[test]
    fn test_encode_decode_state() {
        let mut state = ClientIdState::new();
        state.update_max(1, 1);
        state.update_max(2, 2);

        let mut encoder = EncoderV1::default();
        state.encode(&mut encoder, &EncodeContext::default());

        let mut d = encoder.decoder();

        let decoded = ClientIdState::decode(&mut d, &DecodeContext::default()).unwrap();

        assert_eq!(state, decoded);
    }

    #[test]
    fn test_client_state_size() {
        let mut state = ClientState::new();

        for _ in 0..10000 {
            let id = Uuid::new_v4().into();
            state.clients.get_or_insert(&id);
        }

        let mut encoder = EncoderV1::default();
        state.encode(&mut encoder, &EncodeContext::default());

        let buf = encoder.buffer();
        let comp = compress_to_vec(&buf, 1);
        println!("ClientState size: {}", buf.len());
    }

    #[test]
    fn test_client_state_as_per() {
        let mut s1 = ClientState::default();
        let mut s2 = ClientState::default();

        let c1 = s1.get_or_insert(&Client::default());
        let c2 = s1.get_or_insert(&Client::default());
        // println!("c1: {}, c2: {}", c1.0, c2.0);
        s1.update(c1.0, 5);
        s1.update(c2.0, 5);

        let c3 = s2.get_or_insert(&Client::default());
        let c4 = s2.get_or_insert(&Client::default());
        // println!("c3: {}, c4: {}", c3.0, c4.0);
        s2.update(c3.0, 10);
        s2.update(c4.0, 10);

        // print_yaml(&s1);
        // print_yaml(&s2);

        let s3 = s1.as_per(&s2);
        // print_yaml(&s3);

        assert_eq!(s3.get(&0).unwrap(), &0);
        assert_eq!(s3.get(&1).unwrap(), &0);
        assert_eq!(s3.get(&2).unwrap(), &5);
        assert_eq!(s3.get(&3).unwrap(), &5);
    }
}
