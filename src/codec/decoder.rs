use crate::item::ItemData;

pub trait Decoder {
    fn u8(&mut self) -> Result<u8, String>;
    fn u32(&mut self) -> Result<u32, String>;
    fn u64(&mut self) -> Result<u64, String>;
    fn string(&mut self) -> Result<String, String>;
    fn bytes(&mut self) -> Result<Vec<u8>, String>;
    fn slice(&mut self, len: usize) -> Result<&[u8], String>;
    fn item(&mut self) -> Result<ItemData, String>;
}

pub(crate) struct DecodeContext {
    pub(crate) version: u8,
}

pub trait Decode {
    fn decode<T: Decoder>(d: &mut T, ctx: &DecodeContext) -> Result<Self, String>
    where
        Self: Sized;
}
