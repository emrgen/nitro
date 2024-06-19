use crate::codec::decoder::Decoder;
use crate::item::{ItemData, ItemRef};

pub trait Encoder {
    fn u8(&mut self, value: u8);
    fn u32(&mut self, value: u32);
    fn u64(&mut self, value: u64);
    fn string(&mut self, value: &str);
    fn bytes(&mut self, value: &[u8]);
    fn item(&mut self, value: &ItemData);
    fn trim(&mut self);
    fn decoder(self) -> Box<dyn Decoder>;
    fn buffer(self) -> Vec<u8>;
}

pub trait Encode {
    fn encode<T: Encoder>(&self, e: &mut T);
}
