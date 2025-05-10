use crate::change::Change;
use crate::Id;
use std::collections::HashSet;

pub struct Frontier {
    changes: HashSet<Change>,
}
