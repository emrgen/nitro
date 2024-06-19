use crate::clients::Client;
use crate::id::Clock;
use std::collections::HashMap;

pub(crate) struct ClientState {
    pub(crate) clients: HashMap<Client, Clock>,
}

impl ClientState {
    pub(crate) fn new() -> ClientState {
        ClientState {
            clients: HashMap::new(),
        }
    }

    pub(crate) fn update(&mut self, client: Client, clock: Clock) {
        let current = *self.clients.entry(client).or_default();
        self.clients.insert(client, clock.max(current));
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
