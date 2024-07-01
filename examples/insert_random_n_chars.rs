use miniz_oxide::deflate::compress_to_vec;
use rand::{Rng, thread_rng};

use nitro::codec_v1::EncoderV1;
use nitro::Doc;
use nitro::encoder::{Encode, Encoder};

fn main() {
    let mut indexes = Vec::new();
    for _ in 0..6000 {
        let index = thread_rng().gen_range(0..indexes.len() + 1);
        indexes.push(index);
    }

    let now = std::time::Instant::now();
    let doc = Doc::default();
    let list = doc.list();
    doc.set("list", list.clone());

    let chars = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z",
    ];

    let mut size = 0;

    for i in 0..6000 {
        // random index
        let index = indexes[i];
        list.insert(index as u32, doc.string(chars[i % 26]));
    }

    let mut encoder = EncoderV1::new();
    doc.encode(&mut encoder, &Default::default());

    let comp = compress_to_vec(&encoder.buffer(), 1);

    println!("Doc size: {}", encoder.buffer().len());
    println!("Compressed size: {}", comp.len());

    println!("elapsed: {:?}", now.elapsed());

    // print_yaml(&doc);
}
