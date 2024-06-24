use std::cmp::PartialEq;
use std::ops::Deref;

use crate::decoder::Decoder;
use crate::encoder::{Encode, EncodeContext, Encoder};
use crate::item::{Content, ItemData, ItemKindFlags};

const VERSION: u8 = 1;
const BUF_STEP: usize = 1024;
const INIT_SIZE: usize = 1024;

#[derive(Debug, Clone)]
pub struct EncoderV1 {
    buf: Vec<u8>,
}

impl Default for EncoderV1 {
    fn default() -> Self {
        Self::with_capacity(INIT_SIZE)
    }
}

impl EncoderV1 {
    pub(crate) fn new() -> Self {
        Self::with_capacity(INIT_SIZE)
    }

    pub(crate) fn with_capacity(size: usize) -> Self {
        Self {
            buf: Vec::with_capacity(size),
        }
        .write_header()
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

    fn u16(&mut self, value: u16) {
        self.ensure_capacity(2);
        self.buf.extend_from_slice(&value.to_be_bytes());
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

    fn slice(&mut self, value: &[u8]) {
        self.buf.extend_from_slice(value);
    }

    fn item(&mut self, ctx: &EncodeContext, value: &ItemData) {
        encode_item(self, ctx, value);
    }

    fn trim(&mut self) {
        self.buf.shrink_to_fit();
    }

    fn decoder(self) -> Box<dyn Decoder> {
        Box::new(DecoderV1::new(self.buf))
    }

    fn buffer(self) -> Vec<u8> {
        self.buf
    }

    fn size(&self) -> usize {
        self.buf.len()
    }
}

impl Deref for EncoderV1 {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

fn encode_item(e: &mut EncoderV1, ctx: &EncodeContext, value: &ItemData) {
    // | kind, content, field, parent | left, right | ...

    let mut flags = ItemKindFlags::from(&value.kind).bits() << 4;
    if !matches!(value.content, Content::Null) {
        flags |= 1 << 3;
    }

    if value.field.is_some() {
        flags |= 1 << 2;
    }

    // if left_id is not None then we can get the parent_id from left item during integration,
    // so we don't need to store parent_id in the item
    if value.left_id.is_some() {
        flags |= 1 << 1;
    }

    if value.right_id.is_some() {
        flags |= 1;
    }

    e.u8(flags);

    if !matches!(value.content, Content::Null) {
        value.content.encode(e, ctx);
    }

    if let Some(field) = value.field {
        e.u32(field);
    }

    value.id.encode(e, ctx);
    if let Some(left_id) = value.left_id {
        left_id.encode(e, ctx);
    } else if let Some(parent_id) = value.parent_id {
        parent_id.encode(e, ctx);
    }

    if let Some(right_id) = value.right_id {
        right_id.encode(e, ctx);
    }

    if let Some(target_id) = value.target_id {
        target_id.encode(e, ctx);
    }

    if let Some(mover_id) = value.mover_id {
        mover_id.encode(e, ctx);
    }
}

pub struct DecoderV1 {
    buf: Vec<u8>,
    pos: usize,
}

impl DecoderV1 {
    pub(crate) fn new(buf: Vec<u8>) -> Self {
        let mut d = Self { buf, pos: 0 };

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

    fn u16(&mut self) -> Result<u16, String> {
        self.ensure_capacity(2);
        let value = u16::from_be_bytes([self.buf[self.pos], self.buf[self.pos + 1]]);
        self.pos += 2;
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
        let value = String::from_utf8(self.buf[self.pos..self.pos + len].to_vec())
            .map_err(|_| "decoder: invalid utf8 string".to_string())?;
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

    fn slice(&mut self, len: usize) -> Result<&[u8], String> {
        self.ensure_capacity(len);
        let value = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        Ok(value)
    }

    fn item(&mut self) -> Result<ItemData, String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encoder_v1() {
        let mut encoder = EncoderV1::with_capacity(0);
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
