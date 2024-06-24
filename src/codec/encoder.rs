use crate::codec::decoder::Decoder;
use crate::item::ItemData;

pub trait Encoder: Clone {
    fn u8(&mut self, value: u8);
    fn u32(&mut self, value: u32);
    fn u64(&mut self, value: u64);
    fn string(&mut self, value: &str);
    fn bytes(&mut self, value: &[u8]);
    fn slice(&mut self, value: &[u8]);
    fn item(&mut self, ctx: &EncodeContext, value: &ItemData);
    fn trim(&mut self);
    fn decoder(self) -> Box<dyn Decoder>;
    fn buffer(self) -> Vec<u8>;
    fn size(&self) -> usize;
}

#[derive(Clone, Default, Debug)]
pub(crate) struct EncodeContext {
    pub(crate) version: u8,
}

pub trait Encode {
    fn encode<T: Encoder>(&self, e: &mut T, ctx: &EncodeContext);
}
