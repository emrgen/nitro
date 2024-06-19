use crate::codec::decoder::Decoder;
use crate::codec::encoder::Encoder;
use crate::item::ItemRef;

const VERSION: u8 = 1;
const BUF_STEP: usize = 1024;
const INIT_SIZE: usize = 1024;

#[derive(Debug)]
pub(crate) struct EncoderV1 {
  buf: Vec<u8>,
}

impl Default for EncoderV1 {
  fn default() -> Self {
    Self::new(INIT_SIZE)
  }
}

impl EncoderV1 {
  pub(crate) fn new(size: usize) -> Self {
    Self {
      buf: Vec::with_capacity(size),
    }.write_header()
  }

  fn ensure_capacity(&mut self, size: usize) {
    if self.buf.len() + size > self.buf.capacity() {
      self.buf.reserve(BUF_STEP);
    }
  }

  fn write_header(mut self) -> Self {
    self.u8(VERSION);

    self
  }
}

impl Encoder for EncoderV1 {
  fn u8(&mut self, value: u8) {
    self.ensure_capacity(1);
    self.buf.push(value);
  }

  fn u32(&mut self, value: u32) {
    self.ensure_capacity(4);
    self.buf.extend_from_slice(&value.to_be_bytes());
  }

  fn u64(&mut self, value: u64) {
    self.ensure_capacity(8);
    self.buf.extend_from_slice(&value.to_be_bytes());
  }

  fn string(&mut self, value: &str) {
    self.ensure_capacity(value.len() + 4);
    self.u32(value.len() as u32);
    self.buf.extend_from_slice(value.as_bytes());
  }

  fn bytes(&mut self, value: &[u8]) {
    self.ensure_capacity(value.len() + 4);
    self.u32(value.len() as u32);
    self.buf.extend_from_slice(value);
  }

  fn item(&mut self, value: ItemRef) {}

  fn decoder(self) -> Box<dyn Decoder> {
    Box::new(DecoderV1::new(self.buf))
  }

  fn trim(&mut self) {
    self.buf.shrink_to_fit();
  }

  fn buffer(self) -> Vec<u8> {
    self.buf
  }
}

pub(crate) struct DecoderV1 {
  buf: Vec<u8>,
  pos: usize,
}

impl DecoderV1 {
  pub(crate) fn new(buf: Vec<u8>) -> Self {
    let mut d = Self {
      buf,
      pos: 0,
    };

    if d.u8().unwrap() != VERSION {
      panic!("decoder: invalid version");
    }

    d
  }

  fn ensure_capacity(&mut self, size: usize) {
    if self.pos + size > self.buf.len() {
      panic!("decoder: out of bounds");
    }
  }
}

impl Decoder for DecoderV1 {
  fn u8(&mut self) -> Result<u8, String> {
    self.ensure_capacity(1);
    let value = self.buf[self.pos];
    self.pos += 1;
    Ok(value)
  }

  fn u32(&mut self) -> Result<u32, String> {
    self.ensure_capacity(4);
    let value = u32::from_be_bytes([
      self.buf[self.pos],
      self.buf[self.pos + 1],
      self.buf[self.pos + 2],
      self.buf[self.pos + 3],
    ]);
    self.pos += 4;
    Ok(value)
  }

  fn u64(&mut self) -> Result<u64, String> {
    self.ensure_capacity(8);
    let value = u64::from_be_bytes([
      self.buf[self.pos],
      self.buf[self.pos + 1],
      self.buf[self.pos + 2],
      self.buf[self.pos + 3],
      self.buf[self.pos + 4],
      self.buf[self.pos + 5],
      self.buf[self.pos + 6],
      self.buf[self.pos + 7],
    ]);
    self.pos += 8;
    Ok(value)
  }

  fn string(&mut self) -> Result<String, String> {
    let len = self.u32()? as usize;
    self.ensure_capacity(len);
    let value = String::from_utf8(self.buf[self.pos..self.pos + len].to_vec()).map_err(|_| "decoder: invalid utf8 string".to_string())?;
    self.pos += len;
    Ok(value)
  }

  fn bytes(&mut self) -> Result<Vec<u8>, String> {
    let len = self.u32()? as usize;
    self.ensure_capacity(len);
    let value = self.buf[self.pos..self.pos + len].to_vec();
    self.pos += len;
    Ok(value)
  }

  fn item(&mut self) -> Result<ItemRef, String> {
    todo!()
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_encoder_v1() {
    let mut encoder = EncoderV1::new(0);
    encoder.u8(1);
    encoder.u32(2);
    encoder.u64(3);
    encoder.string("hello");
    encoder.bytes(&[1, 2, 3, 4]);

    let buf = encoder.buffer();
    let mut decoder = DecoderV1::new(buf);
    assert_eq!(decoder.u8().unwrap(), 1);
    assert_eq!(decoder.u32().unwrap(), 2);
    assert_eq!(decoder.u64().unwrap(), 3);
    assert_eq!(decoder.string().unwrap(), "hello");
    assert_eq!(decoder.bytes().unwrap(), vec![1, 2, 3, 4]);
  }
}