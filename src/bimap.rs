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
