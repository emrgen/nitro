use crate::Id;

pub trait Frontier {
    fn frontier(&self) -> u128;
    fn advance(&mut self, client: Id);
}
