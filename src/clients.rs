// ClientTree is a CRDT
#[derive(Debug, Clone, Default)]
struct ClientTree {
  content: Option<ClientVersionRef>,
}

impl ClientTree {
  fn insert(&mut self, item: ClientVersion) -> Self {
    let mut tree = ClientTree::default();
    let item_ref = Rc::new(RefCell::new(item));
    if let Some(content) = &self.content {
      // find the
    } else {
      tree.content = Some(item_ref);
    }
  }

  fn integrate(&mut self, item: ClientVersion) {}
}

// ClientVersionRef is a reference to a ClientVersion
type ClientVersionRef = Rc<RefCell<ClientTree>>;

// ClientVersion is a CRDT Item in the ClientTree
struct ClientVersion {
  client_id: u128,
  version: u32,
  left: Option<ClientVersionRef>,
  right: Option<ClientVersionRef>,
}

impl Ord for ClientVersion {
  fn cmp(&self, other: &Self) -> Ordering {
    self.version.cmp(&other.version)
  }
}
