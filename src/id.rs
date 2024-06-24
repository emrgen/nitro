use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::{Add, Sub};

use serde::{Serialize, Serializer};

use crate::bimapid::{ClientId, ClientMap};
use crate::decoder::{Decode, DecodeContext, Decoder};
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::hash::calculate_hash;

pub(crate) type Clock = u32;

pub(crate) trait Split {
    type Target;
    fn split(&self, offset: u32) -> Result<(Self::Target, Self::Target), String>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Id {
    pub(crate) client: ClientId,
    pub(crate) clock: Clock,
}

impl Id {
    pub(crate) fn adjust(&self, before: &ClientMap, after: &ClientMap) -> Id {
        let client = before.get_client(&self.client).unwrap();
        let new_client = after.get_client_id(client).unwrap();

        Id::new(*new_client, self.clock)
    }
}

impl Id {
    pub(crate) fn new(client: ClientId, clock: Clock) -> Id {
        Id { client, clock }
    }

    pub(crate) fn eq_opt(a: Option<Id>, b: Option<Id>) -> bool {
        match (a, b) {
            (Some(a), Some(b)) => {
                a.client == b.client && a.compare_without_client(&b) == std::cmp::Ordering::Equal
            }
            (None, None) => true,
            _ => false,
        }
    }

    pub(crate) fn compare_without_client(&self, other: &Id) -> Ordering {
        self.clock.cmp(&other.clock)
    }

    pub(crate) fn compare(&self, other: &Id, clients: &ClientMap) -> Ordering {
        if self.client != other.client {
            if clients.size() == 0 {
                panic!("Cannot compare Ids from different clients without a client map")
            }

            let client = clients.get_client(&self.client).unwrap();
            let other_client = clients.get_client(&other.client).unwrap();
            return calculate_hash(client).cmp(&calculate_hash(other_client));
        }

        self.compare_without_client(other)
    }

    pub(crate) fn next(&self) -> Id {
        Id::new(self.client, self.clock + 1)
    }

    pub(crate) fn range(&self, size: u32) -> IdRange {
        IdRange::new(self.client, self.clock, self.clock + size - 1)
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        format!("Id({}, {})", self.client, self.clock).serialize(serializer)
    }
}

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.client, self.clock)
    }
}

impl WithId for Id {
    fn id(&self) -> Id {
        *self
    }
}

impl From<(ClientId, Clock)> for Id {
    fn from((client, clock): (ClientId, Clock)) -> Self {
        Id::new(client, clock)
    }
}

impl From<IdRange> for Id {
    fn from(value: IdRange) -> Self {
        Id::new(value.client, value.start)
    }
}

impl Sub<Clock> for Id {
    type Output = Id;

    fn sub(self, rhs: Clock) -> Self::Output {
        Id::new(self.client, self.clock - rhs)
    }
}

impl Add<Clock> for Id {
    type Output = Id;

    fn add(self, rhs: Clock) -> Self::Output {
        Id::new(self.client, self.clock + rhs)
    }
}

impl Add<Clock> for &Id {
    type Output = Id;

    fn add(self, rhs: Clock) -> Self::Output {
        Id::new(self.client, self.clock + rhs)
    }
}

impl PartialEq<Self> for Id {
    fn eq(&self, other: &Self) -> bool {
        self.client == other.client && self.clock == other.clock
    }
}

impl Eq for Id {}

impl PartialOrd<Self> for Id {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Id {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.compare(other, &ClientMap::new())
    }
}

impl Encode for Id {
    fn encode<T: Encoder>(&self, e: &mut T, ctx: &EncodeContext) {
        e.u32(self.client);
        e.u32(self.clock);
    }
}

impl Decode for Id {
    fn decode<T: Decoder>(d: &mut T, ctx: &DecodeContext) -> Result<Self, String> {
        let client = d.u32()?;
        let clock = d.u32()?;

        Ok(Id::new(client, clock))
    }
}

#[derive(Clone, Copy, Default)]
pub(crate) struct IdRange {
    pub(crate) client: ClientId,
    pub(crate) start: Clock,
    pub(crate) end: Clock,
}

impl IdRange {
    pub(crate) fn new(client: ClientId, start: Clock, end: Clock) -> IdRange {
        IdRange { client, start, end }
    }

    pub(crate) fn size(&self) -> Clock {
        self.end - self.start + 1
    }

    pub(crate) fn eq_opt(a: Option<&IdRange>, b: Option<&IdRange>) -> bool {
        match (a, b) {
            (Some(a), Some(b)) => {
                a.client == b.client && a.compare_without_client(b) == std::cmp::Ordering::Equal
            }
            (None, None) => true,
            _ => false,
        }
    }

    pub(crate) fn equals(&self, other: &IdRange) -> bool {
        self.client == other.client && self.start == other.start && self.end == other.end
    }

    pub(crate) fn start_id(&self) -> Id {
        Id::new(self.client, self.start)
    }

    pub(crate) fn end_id(&self) -> Id {
        Id::new(self.client, self.end)
    }

    // Compare two Ids, considering the client field if they are different
    pub(crate) fn compare(&self, other: &IdRange, clients: &ClientMap) -> std::cmp::Ordering {
        if self.client != other.client {
            let client = clients.get_client(&self.client).unwrap();
            let other_client = clients.get_client(&other.client).unwrap();
            return calculate_hash(client).cmp(&calculate_hash(other_client));
        }

        self.compare_without_client(other)
    }

    // Compare two Ids without considering the client field
    // e.g. [1...3] < [2..2] < [1...3] will help to find [1...3] using [2..2]
    pub fn compare_without_client(&self, other: &IdRange) -> std::cmp::Ordering {
        if self.end < other.start {
            std::cmp::Ordering::Less
        } else if other.end < self.start {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Equal
        }
    }

    pub(crate) fn split(&self, offset: u32) -> Result<(IdRange, IdRange), String> {
        if offset == 0 || offset >= self.size() {
            return Err("Cannot split IdRange at invalid position".to_string());
        }

        Ok((
            IdRange::new(self.client, self.start, self.start + offset - 1),
            IdRange::new(self.client, self.start + offset, self.end),
        ))
    }

    pub(crate) fn adjust(&self, before: &ClientMap, after: &ClientMap) -> IdRange {
        self.id().adjust(before, after).range(self.size())
    }
}

impl Display for IdRange {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}, {})", self.client, self.start, self.end)
    }
}

impl Serialize for IdRange {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        format!("Range({}, {}, {})", self.client, self.start, self.end).serialize(serializer)
    }
}

impl Encode for IdRange {
    fn encode<T: Encoder>(&self, e: &mut T, ctx: &EncodeContext) {
        e.u32(self.client);
        e.u32(self.start);
        e.u32(self.size());
    }
}

impl Decode for IdRange {
    fn decode<T: Decoder>(d: &mut T, ctx: &DecodeContext) -> Result<Self, String> {
        let client = d.u32()?;
        let start = d.u32()?;
        let size = d.u32()?;

        Ok(IdRange::new(client, start, start + size - 1))
    }
}

impl WithId for IdRange {
    fn id(&self) -> Id {
        self.clone().into()
    }
}

impl Add<IdRange> for IdRange {
    type Output = IdRange;

    fn add(self, other: IdRange) -> IdRange {
        if (self.client != other.client) || (self.end + 1 != other.start) {
            panic!("Cannot add non-adjacent Ids")
        }

        IdRange::new(
            self.client,
            self.start.min(other.start),
            self.end.max(other.end),
        )
    }
}

pub(crate) trait WithId {
    fn id(&self) -> Id;
}

pub(crate) trait WithIdRange {
    fn range(&self) -> IdRange;
}

impl Split for IdRange {
    type Target = IdRange;
    fn split(&self, at: Clock) -> Result<(IdRange, IdRange), String> {
        self.split(at)
    }
}

impl std::fmt::Debug for IdRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Id({:?}, {:?}, {:?})", self.client, self.start, self.end)
    }
}

impl From<Id> for IdRange {
    fn from(value: Id) -> Self {
        IdRange::new(value.client, value.clock, value.clock)
    }
}

impl PartialEq<Self> for IdRange {
    fn eq(&self, other: &Self) -> bool {
        self.compare_without_client(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for IdRange {}

impl PartialOrd<Self> for IdRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(std::cmp::Ord::cmp(self, other))
    }
}

impl Ord for IdRange {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.compare_without_client(other)
    }
}

impl std::hash::Hash for IdRange {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.client.hash(state);
        self.start.hash(state);
    }
}

#[cfg(test)]
mod tests {
    use crate::bimapid::ClientMap;

    use super::*;

    #[test]
    fn test_compare() {
        let mut clients = ClientMap::new();
        clients.get_or_insert(&"client1".to_string());
        clients.get_or_insert(&"client2".to_string());

        let id1 = IdRange::new(0, 1, 1);
        let id2 = IdRange::new(0, 1, 1);
        let id3 = IdRange::new(0, 1, 2);
        let id4 = IdRange::new(0, 2, 2);
        let id5 = IdRange::new(1, 1, 1);
        let id6 = IdRange::new(1, 1, 2);
        let id7 = IdRange::new(1, 2, 2);

        assert_eq!(id1.compare(&id2, &clients), std::cmp::Ordering::Equal);
        assert_eq!(id1.compare(&id3, &clients), std::cmp::Ordering::Equal);
        assert_eq!(id1.compare(&id4, &clients), std::cmp::Ordering::Less);
        assert_eq!(id1.compare(&id5, &clients), std::cmp::Ordering::Less);
        assert_eq!(id1.compare(&id6, &clients), std::cmp::Ordering::Less);
        assert_eq!(id1.compare(&id7, &clients), std::cmp::Ordering::Less);

        assert_eq!(id3.compare(&id1, &clients), std::cmp::Ordering::Equal);
        assert_eq!(id4.compare(&id1, &clients), std::cmp::Ordering::Greater);
        assert_eq!(id5.compare(&id1, &clients), std::cmp::Ordering::Greater);

        assert_eq!(id6.compare(&id1, &clients), std::cmp::Ordering::Greater);
        assert_eq!(id7.compare(&id1, &clients), std::cmp::Ordering::Greater);
    }
}
