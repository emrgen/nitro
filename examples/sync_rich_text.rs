use nitro::{CloneDeep, RichText, sync_docs, SyncDirection, Type};

fn main() {
    let mut t1 = RichText::new();
    let mut t2 = t1.clone_deep();

    t1.insert(0, "w");
    t2.insert(0, "h");

    t1.sync(&t2);
    assert_eq!(t1, t2);

    // 26 letters
    let mut chars = vec![
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", //"u", "v", "w", "x", "y", "z",
    ];
    let mut size = 2;

    let mut s1: Option<Type> = None;
    for _ in 0..500 {
        for c in &chars {
            let item = t1.doc.string(c.to_string());
            // t1.text.insert(0, item.clone());
            t1.text.prepend(item);
        }
    }

    chars.reverse();

    s1 = None;
    for _ in 0..500 {
        for c in &chars {
            let item = t2.doc.string(c.to_string());
            // t2.text.insert(0, item.clone());
            t2.text.prepend(item);
        }
    }

    // t1.sync(&t2);
    // sync_docs(&t1.doc, &t2.doc, SyncDirection::LeftToRight);
    // sync_docs(&t1.doc, &t2.doc, SyncDirection::RightToLeft);
    sync_docs(&t1.doc, &t2.doc, SyncDirection::Both);

    println!("xxxxxxxxx");

    assert_eq!(t1, t2);

    // print_yaml(&t1);
    // print_yaml(&t2);
}
