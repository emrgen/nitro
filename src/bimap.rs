struct BiMap<L,R> {
  items: Vec<(L,R)>,
  left: HashMap<L, usize>,
  right: HashMap<R, usize>,
}

impl <L,R> BiMap<L,R> {
  fn new() -> Self {
    BiMap {
      items: Vec::new(),
      left: HashMap::new(),
      right: HashMap::new(),
    }
  }

  fn entry(&self) {}
}

#[derive(Default, Clone)]
struct ClientMap {
  clients: Vec<Client>,
  ids: HashMap<Client, usize>,
}

impl ClientMap {
  fn get_client(&self, client_id: ClientId) -> Option<Client> {}
  fn get_client_id(&self, client: Client) -> Option<ClientId> {}
}
