use std::str::FromStr;

use uuid::{Error, Uuid};

use nitro::Doc;
use nitro::encoder::{Encode, Encoder};

fn insert_text(doc: &Doc, text: &str) {
    doc.update_client();
    let text = doc.string(text);
    doc.get("text").unwrap().append(text);
}

// This example is used to test the size of the document after inserting 500 characters by 500 clients.
fn main() -> Result<(), Error> {
    let doc = Doc::default();
    let text = doc.text();
    doc.set("text", text.clone());

    let mut counter = 0;
    // for i in 0..1 {
    //     insert_text(&doc, "a");
    //     if i / 100 != counter {
    //         // println!("Counter: {}", counter);
    //     }
    //
    //     counter = i / 100;
    // }

    let mut encoder = nitro::codec_v1::EncoderV1::new();

    doc.encode(&mut encoder, &Default::default());

    let buf = encoder.buffer();

    println!("Doc size: {}", buf.len());
    println!(
        "Compressed size: {}",
        miniz_oxide::deflate::compress_to_vec(&buf, 1).len()
    );

    let mut encoder = nitro::codec_v1::EncoderV1::new();
    let v = doc.version();
    let clients = v.clients();

    encoder.u32(clients.len() as u32);
    for (client, id) in clients {
        println!("Client: {}", client);

        let uuid: Uuid = Uuid::from_str(client)?;
        let uuid = uuid.as_bytes().as_slice();
        encoder.uuid(uuid);
        println!("Client size: {}", uuid.len());
        encoder.u32(id);
    }

    println!("Version size: {}", encoder.buffer().len());

    Ok(())
}
