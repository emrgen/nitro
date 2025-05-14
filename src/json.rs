use crate::Doc;

/// Json utility to create a doc from json
pub(crate) struct JsonDoc {
    value: serde_json::Value,
}

impl JsonDoc {
    pub(crate) fn new(value: serde_json::Value) -> Self {
        JsonDoc { value }
    }

    pub(crate) fn to_doc(&self) -> Doc {
        let mut doc = Doc::default();
        if let Some(target) = self.value.as_object() {
            for (key, value) in target.iter() {}
        }
        doc
    }
}
