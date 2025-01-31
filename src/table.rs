use crate::bimapid::ClientId;
use crate::{Id, ItemData};
use serde::{Deserialize, Serialize};
use serde_columnar::columnar;
use std::default::Default;

#[columnar(vec, ser, de)]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Data {
    #[columnar(strategy = "Rle")]
    id_client: ClientId,
    #[columnar(strategy = "Rle")]
    id_clock: u32,
    #[columnar(strategy = "Rle")]
    parent_id_client: ClientId,
    #[columnar(strategy = "Rle")]
    parent_id_clock: u32,
    #[columnar(strategy = "Rle")]
    left_id_client: ClientId,
    left_id_clock: u32,
    #[columnar(strategy = "Rle")]
    right_id_client: ClientId,
    #[columnar(strategy = "Rle")]
    right_id_clock: u32,
    #[columnar(strategy = "Rle")]
    target_id: ClientId,
    #[columnar(strategy = "Rle")]
    target_id_clock: u32,
    #[columnar(strategy = "Rle")]
    mover_id: ClientId,
    #[columnar(strategy = "Rle")]
    mover_id_clock: u32,
}

#[columnar]
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Table {
    pub(crate) data: Vec<Data>,
}

impl Table {
    pub(crate) fn add(&mut self, item: &ItemData) {
        let mut data = Data {
            id_client: item.id.client,
            id_clock: item.id.clock,
            ..Default::default()
        };

        if let Some(left_id) = item.left_id {
            data.left_id_client = left_id.client;
            data.left_id_clock = left_id.clock;
        } else if let Some(parent_id) = item.parent_id {
            data.parent_id_client = parent_id.client;
            data.parent_id_clock = parent_id.clock;
        }

        if let Some(right_id) = item.right_id {
            data.right_id_client = right_id.client;
            data.right_id_clock = right_id.clock;
        }

        if let Some(target_id) = item.target_id {
            data.target_id = target_id.client;
            data.target_id_clock = target_id.clock;
        }

        if let Some(mover_id) = item.mover_id {
            data.mover_id = mover_id.client;
            data.mover_id_clock = mover_id.clock;
        }

        self.data.push(data);
    }

    pub(crate) fn buffer(&self) -> Vec<u8> {
        let bytes = serde_columnar::to_vec(&self).unwrap();

        bytes
    }
}
