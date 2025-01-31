use nitro::{Doc, Type};

fn main() {
    let doc1 = Doc::default();
    let mut l1: Type = doc1.list().into();
    doc1.set("list1", l1.clone());

    // 26 letters
    let chars = vec![
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z",
    ];

    let mut size = 1;
    let now = std::time::Instant::now();

    for _ in 0..1000 {
        for c in &chars {
            let item = doc1.string(c.to_string());
            // random index
            let index = rand::random::<usize>() % size;

            l1.insert(index as u32, item);
            // l1.append(item);
            // l1.prepend(item);
            size += 1;
        }
    }

    println!("first doc: {:?}", now.elapsed());

    println!("{:?}", l1.size());

    // print_yaml(l1.content());
}
