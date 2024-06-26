use crate::doc::Doc;

fn equal_docs(d1: &Doc, d2: &Doc) -> bool {
    let left = serde_json::to_string(d1).unwrap();
    let right = serde_json::to_string(d2).unwrap();

    // println!("left: {}", left);
    // println!("right: {}", right);

    left == right
}

fn sync_docs(d1: &Doc, d2: &Doc) {
    let diff1 = d1.diff(d2);
    let diff2 = d2.diff(d1);

    // println!("diff1");
    // print_yaml(&diff1);
    // println!("diff2");
    // print_yaml(&diff2);

    d1.apply(diff2);
    d2.apply(diff1);
}

fn sync_first_doc(d1: &Doc, d2: &Doc) {
    let diff1 = d2.diff(d1);
    d1.apply(diff1);
}

#[cfg(test)]
mod test {
    use crate::doc::{CloneDeep, Doc};
    use crate::sync::{equal_docs, sync_docs};

    #[test]
    fn test_sync() {
        let doc1 = Doc::default();
        let doc2 = doc1.clone_deep();
        doc2.update_client();

        doc1.set("a", doc1.string("hello"));
        doc2.set("b", doc2.string("world"));

        sync_docs(&doc1, &doc2);

        // print_yaml(&doc1);
        // print_yaml(&doc2);

        assert!(equal_docs(&doc1, &doc2));
    }

    #[test]
    fn test_sync2() {
        let doc1 = Doc::default();
        let doc2 = doc1.clone_deep();
        doc2.update_client();

        doc1.set("a", doc1.string("hello"));
        doc2.set("a", doc2.string("world"));

        sync_docs(&doc1, &doc2);

        assert!(equal_docs(&doc1, &doc2));
    }

    #[test]
    fn test_sync_with_list() {
        let doc1 = Doc::default();
        let doc2 = doc1.clone_deep();
        doc2.update_client();

        let list1 = doc1.list();
        doc1.set("list1", list1.clone());

        list1.append(doc1.string("a"));
        list1.append(doc1.string("b"));

        let list2 = doc2.list();
        doc2.set("list2", list2.clone());

        list2.append(doc2.string("a"));

        sync_docs(&doc1, &doc2);
        assert!(equal_docs(&doc1, &doc2));

        let list1 = doc2.get("list1").unwrap().as_list().unwrap();
        list1.append(doc2.string("c"));

        sync_docs(&doc1, &doc2);

        // print_yaml(&doc1);
        // print_yaml(&doc2);

        assert!(equal_docs(&doc1, &doc2));
    }

    #[test]
    fn test_sync_with_text() {
        let doc1 = Doc::default();
        let doc2 = doc1.clone_deep();
        doc2.update_client();

        let text1 = doc1.text();
        doc1.set("text", text1.clone());

        sync_docs(&doc1, &doc2);
        assert!(equal_docs(&doc1, &doc2));

        text1.insert(0, doc1.string("a"));

        sync_docs(&doc1, &doc2);
        assert!(equal_docs(&doc1, &doc2));

        let text2 = doc2.get("text").unwrap().as_text().unwrap();
        let text1 = doc1.get("text").unwrap().as_text().unwrap();

        text2.insert(1, doc2.string("b"));
        text2.insert(0, doc2.string("c"));

        text1.insert(1, doc1.string("d"));

        sync_docs(&doc1, &doc2);
        // print_yaml(&doc1);
        // print_yaml(&doc2);
        assert!(equal_docs(&doc1, &doc2));
    }
}
