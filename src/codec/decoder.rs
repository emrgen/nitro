use crate::item::{ItemData, ItemRef};

pub trait Decoder {
    fn u8(&mut self) -> Result<u8, String>;
    fn u32(&mut self) -> Result<u32, String>;
    fn u64(&mut self) -> Result<u64, String>;
    fn string(&mut self) -> Result<String, String>;
    fn bytes(&mut self) -> Result<Vec<u8>, String>;
    fn item(&mut self) -> Result<ItemData, String>;
}

pub trait Decode {
    fn decode<T: Decoder>(d: &mut T) -> Result<Self, String>
    where
        Self: Sized;
}
