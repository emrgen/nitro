use std::collections::BTreeMap;

use crate::bimapid::{ClientId, ClientMap};
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::id::Clock;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub(crate) struct ClientState {
    pub(crate) clients: BTreeMap<ClientId, Clock>,
}

impl ClientState {
    pub(crate) fn new() -> ClientState {
        ClientState {
            clients: BTreeMap::new(),
        }
    }

    pub(crate) fn get(&self, client: &ClientId) -> Option<&Clock> {
        self.clients.get(client)
    }

    pub(crate) fn update(&mut self, client: ClientId, clock: Clock) {
        let current = *self.clients.entry(client).or_default();
        self.clients.insert(client, clock.max(current));
    }

    pub(crate) fn adjust(
        &self,
        other: &ClientState,
        before: &ClientMap,
        after: &ClientMap,
    ) -> Self {
        let mut adjust = ClientState::new();

        for (client, client_id) in after.entries() {
            let before_client_state = self.get(client_id).unwrap_or(&0);

            let other_client_id = before.get_client_id(client).unwrap();
            let after_client_state = other.get(other_client_id).unwrap_or(&0);

            let client_state = before_client_state.max(after_client_state);
            adjust.update(*client_id, *client_state);
        }

        adjust
    }
}

impl Encode for ClientState {
    fn encode<E: Encoder>(&self, e: &mut E, ctx: &EncodeContext) {
        e.u32(self.clients.len() as u32);
        for (client, clock) in &self.clients {
            e.u32(*client);
            e.u32(*clock);
        }
    }
}

impl Decode for ClientState {
    fn decode<D: Decoder>(d: &mut D, ctx: &DecodeContext) -> Result<ClientState, String> {
        let len = d.u32()? as usize;
        let mut clients = BTreeMap::new();
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
    use std::default::Default;

    use crate::codec_v1::EncoderV1;

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

    #[test]
    fn test_encode_decode_state() {
        let mut state = ClientState::new();
        state.update(1, 1);
        state.update(2, 2);

        let mut encoder = EncoderV1::default();
        state.encode(&mut encoder, &EncodeContext::default());

        let mut d = encoder.decoder();

        let decoded = ClientState::decode(&mut d, &DecodeContext::default()).unwrap();

        assert_eq!(state, decoded);
    }
}
