use std::cmp::Ordering;
use crate::hash::calculate_hash;
use crate::state::ClientMap;

#[derive(Clone, Copy, Default)]
pub(crate) struct Id {
  pub(crate) client: u64,
  pub(crate) start: u64,
  pub(crate) end: u64,
}

impl Id {
  pub(crate) fn new(client: u64, start: u64, end: u64) -> Id {
    Id { client, start, end }
  }

  pub(crate) fn eq_opt(a: Option<&Id>, b: Option<&Id>) -> bool {
    match (a, b) {
      (Some(a), Some(b)) => a.client == b.client && a.compare_without_client(b) == std::cmp::Ordering::Equal,
      (None, None) => true,
      _ => false,
    }
  }

  pub(crate) fn size(&self) -> u64 {
    self.end - self.start + 1
  }

  pub(crate) fn equals(&self, other: &Id) -> bool {
    self.client == other.client && self.start == other.start && self.end == other.end
  }

  pub(crate) fn head(&self) -> Id {
    Id::new(self.client, self.start, self.start)
  }

  pub(crate) fn tail(&self) -> Id {
    Id::new(self.client, self.end, self.end)
  }

  // Compare two Ids, considering the client field if they are different
  pub(crate) fn compare(&self, other: &Id, clients: &ClientMap) -> std::cmp::Ordering {
    if self.client != other.client {
      let client = clients.get_by_client(&self.client).unwrap();
      let other_client = clients.get_by_client(&other.client).unwrap();
      return calculate_hash(client).cmp(&calculate_hash(other_client));
    }

    self.compare_without_client(other)
  }

  // Compare two Ids without considering the client field
  // e.g. [1...3] < [2..2] < [1...3] will help to find [1...3] using [2..2]
  pub(crate) fn compare_without_client(&self, other: &Id) -> std::cmp::Ordering {
    if self.end < other.start {
      std::cmp::Ordering::Less
    } else if other.end < self.start {
      std::cmp::Ordering::Greater
    } else {
      std::cmp::Ordering::Equal
    }
  }
}

impl std::fmt::Debug for Id {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "Id({:?}, {:?}, {:?})", self.client, self.start, self.end)
  }
}

impl PartialEq<Self> for Id {
  fn eq(&self, other: &Self) -> bool {
    self.compare_without_client(other) == std::cmp::Ordering::Equal
  }
}

impl Eq for Id {}

impl PartialOrd<Self> for Id {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(std::cmp::Ord::cmp(self, other)) }
}

impl Ord for Id {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.compare_without_client(other)
  }
}

impl std::hash::Hash for Id {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.client.hash(state);
    self.start.hash(state);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_compare() {
    let mut clients = ClientMap::new();
    clients.insert("client1".to_string(), 1);
    clients.insert("client2".to_string(), 2);

    let id1 = Id::new(1, 1, 1);
    let id2 = Id::new(1, 1, 1);
    let id3 = Id::new(1, 1, 2);
    let id4 = Id::new(1, 2, 2);
    let id5 = Id::new(2, 1, 1);
    let id6 = Id::new(2, 1, 2);
    let id7 = Id::new(2, 2, 2);

    assert_eq!(id1.compare(&id2, &clients), std::cmp::Ordering::Equal);
    assert_eq!(id1.compare(&id3, &clients), std::cmp::Ordering::Less);
    assert_eq!(id1.compare(&id4, &clients), std::cmp::Ordering::Less);
    assert_eq!(id1.compare(&id5, &clients), std::cmp::Ordering::Less);
    assert_eq!(id1.compare(&id6, &clients), std::cmp::Ordering::Less);
    assert_eq!(id1.compare(&id7, &clients), std::cmp::Ordering::Less);

    assert_eq!(id3.compare(&id1, &clients), std::cmp::Ordering::Greater);
    assert_eq!(id4.compare(&id1, &clients), std::cmp::Ordering::Greater);
    assert_eq!(id5.compare(&id1, &clients), std::cmp::Ordering::Greater);

    assert_eq!(id6.compare(&id1, &clients), std::cmp::Ordering::Greater);
    assert_eq!(id7.compare(&id1, &clients), std::cmp::Ordering::Greater);
  }
}