use std::collections::HashMap;

use crate::{ClientState, Diff, DocId};

pub trait DiffStore {
    type Error;

    fn get(&self, doc_id: &DocId, state: &ClientState) -> Result<&Diff, Self::Error>;
    fn put(&mut self, diff: Diff) -> Result<(), Self::Error>;
}

pub(crate) struct InMemoryDiffStore {
    diffs: HashMap<DocId, Diff>,
}

impl InMemoryDiffStore {
    pub fn new() -> Self {
        InMemoryDiffStore {
            diffs: HashMap::new(),
        }
    }
}

// in memory implementation of the DiffStore trait
impl DiffStore for InMemoryDiffStore {
    type Error = ();

    fn get(&self, doc_id: &DocId, state: &ClientState) -> Result<&Diff, Self::Error> {
        let diff = self.diffs.get(doc_id).unwrap();

        Ok(diff)
    }

    fn put(&mut self, diff: Diff) -> Result<(), Self::Error> {
        let doc_id = diff.doc_id.clone();

        let old_diff = self.diffs.get_mut(&doc_id);
        match old_diff {
            Some(old_diff) => {
                let adjusted = old_diff.adjust_diff(&diff);
                old_diff.merge(&adjusted);
            }
            None => {
                self.diffs.insert(doc_id, diff);
            }
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::Doc;

    use super::*;

    #[test]
    fn test_in_memory_diff_store() {
        let mut store = InMemoryDiffStore::new();

        let doc = Doc::default();
        let text = doc.atom("hello world");
        doc.set("world", text);

        let diff = doc.diff(ClientState::default());
        store.put(diff.clone()).unwrap();
        let diff = store
            .get(&diff.doc_id, &ClientState::default())
            .unwrap()
            .clone();

        // let doc2 = Doc::from_diff(diff).unwrap();
        //
        // assert_eq!(doc, doc2);

        let state = doc.state();
        doc.update_client();
        let text = doc.atom("hello earth!");
        doc.set("earth", text);

        let diff2 = doc.diff(&state);
        let diff3 = {
            let adjusted = diff.adjust_diff(&diff2);
            let mut merged = diff.clone();
            merged.merge(&adjusted);

            merged
        };

        store.put(diff3).unwrap();

        let diff = store.get(&diff.doc_id, &ClientState::default()).unwrap();

        let doc3 = Doc::from_diff(&diff).unwrap();

        assert_eq!(doc, doc3);
    }
}
