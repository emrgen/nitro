use miniz_oxide::deflate::compress_to_vec;
use nitro::codec_v1::EncoderV1;
use nitro::encoder::{Encode, Encoder};
use nitro::Doc;

fn main() {
    let doc = Doc::default();
    let text = doc.text();
    doc.set("text", text.clone());

    let chars = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z",
    ];

    for i in 0..6000 {
        text.prepend(doc.string(chars[i % 26]));
    }

    let now = std::time::Instant::now();
    let mut encoder = EncoderV1::new();
    doc.encode(&mut encoder, &mut Default::default());

    let comp = compress_to_vec(&encoder.buffer(), 1);
    println!("elapsed: {:?}", now.elapsed());

    println!("Doc size: {}", encoder.buffer().len());
    println!("Compressed size: {}", comp.len());
}
