use crate::doc::Doc;

fn equal_docs(d1: &Doc, d2: &Doc) -> bool {
    let left = serde_json::to_string(d1).unwrap();
    let right = serde_json::to_string(d2).unwrap();

    println!("left: {}", left);
    // println!("right: {}", right);

    left == right
}

fn sync_docs(d1: &Doc, d2: &Doc) {
    let diff1 = d1.diff(d2);
    // let diff2 = d2.diff(d1);

    println!("xxxxxx {:?}", diff1.items);
    println!("xxxxxx {:?}", diff1.state.clients);

    println!("{}", serde_yaml::to_string(&diff1).unwrap());

    // d1.apply(diff2);
    // d2.apply(diff2);
}

#[cfg(test)]
mod test {
    use crate::doc::{CloneDeep, Doc};
    use crate::sync::sync_docs;

    #[test]
    fn test_sync() {
        let doc1 = Doc::default();
        let doc2 = doc1.clone_deep();
        doc2.update_client();

        doc1.set("a", doc1.string("hello"));
        doc2.set("b", doc2.string("world"));

        sync_docs(&doc1, &doc2);

        // let left = serde_yaml::to_string(&doc1).unwrap();
        // println!("left: {}", left);

        let right = serde_yaml::to_string(&doc2).unwrap();
        println!("right: {}", right);

        // assert_eq!(doc1, doc2);
        // assert!(equal_docs(&doc1, &doc2));
    }
}
