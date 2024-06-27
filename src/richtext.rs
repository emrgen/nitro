use serde::ser::SerializeStruct;
use serde::Serialize;

use crate::doc::{CloneDeep, Doc};
use crate::item::Content;
use crate::sync::sync_docs;
use crate::types::Type;

#[derive(Clone, Debug)]
pub(crate) struct RichText {
    pub(crate) doc: Doc,
    pub(crate) text: Type,
}

impl RichText {
    pub(crate) fn new() -> RichText {
        let doc = Doc::default();
        let text = doc.text();
        doc.set("text", text.clone());
        RichText {
            doc,
            text: text.into(),
        }
    }

    pub(crate) fn sync(&mut self, other: &RichText) {
        sync_docs(&mut self.doc, &other.doc);
    }

    pub(crate) fn insert(&mut self, index: usize, text: &str) {
        let text = self.doc.string(text);
        self.text.insert(index as u32, text);
    }

    pub(crate) fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

impl PartialEq<Self> for RichText {
    fn eq(&self, other: &Self) -> bool {
        self.to_string() == other.to_string()
    }
}

impl Eq for RichText {}

impl Serialize for RichText {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let text = self
            .text
            .item_ref()
            .borrow()
            .items()
            .iter()
            .map(|item| {
                if let Content::String(s) = item.content() {
                    s
                } else {
                    "".to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("");

        text.serialize(serializer)
    }
}

impl CloneDeep for RichText {
    fn clone_deep(&self) -> Self {
        let doc = self.doc.clone_deep();
        doc.update_client();
        let text = doc.get("text").unwrap().clone();
        RichText { doc, text }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rich_text() {
        let mut t1 = RichText::new();
        let mut t2 = t1.clone_deep();

        t1.insert(0, "w");
        t2.insert(0, "h");

        t1.sync(&t2);
        assert_eq!(t1, t2);

        // 26 letters
        let mut chars = vec![
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q",
            "r", "s", "t", "u", "v", "w", "x", "y", "z",
        ];
        for c in &chars {
            let index = rand::random::<usize>() % (t1.text.size() as usize + 1);
            t1.insert(index, &c.to_string());
        }

        chars.reverse();

        for c in &chars {
            let index = rand::random::<usize>() % (t1.text.size() as usize + 1);
            t2.insert(index, &c.to_string());
        }

        t1.sync(&t2);

        assert_eq!(t1, t2);

        // print_yaml(&t1);
        // print_yaml(&t2);
    }
}
