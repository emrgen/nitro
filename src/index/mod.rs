mod ibtree;
mod rbtree;
mod sbtree;

pub(crate) use rbtree::IndexTree;

use crate::Type;

pub(crate) trait ItemListContainer {
    fn size(&self) -> u32;
    fn at_index(&self, index: u32) -> Option<&Type>;
    fn index_of(&self, item: &Type) -> u32;
    fn insert(&mut self, item: Type);
    fn append(&mut self, value: Type);
    fn prepend(&mut self, value: Type);
    fn remove(&mut self, item: &Type);
    fn delete(&mut self, item: &Type);
    fn undelete(&mut self, item: &Type);
    fn contains(&self, item: &Type) -> bool;
}
