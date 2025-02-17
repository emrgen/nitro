#![feature(slice_take)] // allow warnings for now to make it easier to work on this
#![allow(warnings)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
#![allow(unused_mut)]
#![allow(unused_assignments)]
#![allow(unused_must_use)]

pub use crate::diff::*;
pub use crate::diffstore::*;
pub use crate::doc::*;
pub use crate::id::*;
pub use crate::item::*;
pub use crate::nstring::*;
pub use crate::ntext::*;
pub use crate::richtext::*;
pub use crate::state::*;
pub use crate::sync::*;
pub use crate::types::*;
pub use crate::utils::*;

use crate::index::*;

mod bimapid;
pub mod codec_v1;
mod crdt;
pub mod decoder;
mod delete;
mod diff;
pub mod diffstore;
mod doc;
pub mod encoder;
mod hash;
mod id;
mod id_store;
mod index;
mod item;
mod mark;
mod natom;
mod nlist;
mod nmap;
mod nmark;
mod nmove;
mod nproxy;
mod nstring;
mod ntext;
mod persist;
mod queue_store;
mod richtext;
mod skiplist;
mod state;
mod store;
mod sync;
mod table;
mod transaction;
mod types;
mod utils;
mod version;
mod frontier;
