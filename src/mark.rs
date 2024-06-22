use serde::Serialize;
use serde_json::Value;

use crate::id::IdRange;

#[derive(Debug, Clone, Default)]
pub(crate) struct MarkContent {
    pub(crate) range: IdRange,
    pub(crate) data: Mark,
}

impl MarkContent {
    pub(crate) fn new(range: IdRange, data: Mark) -> Self {
        Self { range, data }
    }

    pub(crate) fn size(&self) -> u32 {
        self.range.size()
    }

    pub(crate) fn split(&self, offset: u32) -> (MarkContent, MarkContent) {
        let (ld, rd) = self.range.split(offset).unwrap();
        let left = MarkContent::new(ld, self.data.clone());
        let right = MarkContent::new(rd, self.data.clone());
        (left, right)
    }

    // used for debugging
    pub(crate) fn key_value_with_range(&self) -> (String, Value) {
        match self.data {
            Mark::Bold => (
                "bold".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            Mark::Italic => (
                "italic".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            Mark::Underline => (
                "underline".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            Mark::StrikeThrough => (
                "strikethrough".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            Mark::Code => (
                "code".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            Mark::Subscript => (
                "subscript".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            Mark::Superscript => (
                "superscript".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            Mark::Color(ref color) => ("color".to_string(), color.to_string().into()),
            Mark::Background(ref color) => ("background".to_string(), color.to_string().into()),
            Mark::Link(ref url) => ("link".to_string(), url.to_string().into()),
            Mark::Custom(ref name, ref json) => (name.to_string(), json.to_string().into()),
            Mark::None => ("_".to_string(), Value::Null),
        }
    }

    pub(crate) fn key_value_without_range(&self) -> (String, Value) {
        match self.data {
            Mark::Bold => ("bold".into(), true.into()),
            Mark::Italic => ("italic".into(), true.into()),
            Mark::Underline => ("underline".into(), true.into()),
            Mark::StrikeThrough => ("strikethrough".into(), true.into()),
            Mark::Code => ("code".into(), true.into()),
            Mark::Subscript => ("subscript".into(), true.into()),
            Mark::Superscript => ("superscript".into(), true.into()),
            Mark::Color(ref color) => ("color".into(), color.to_string().into()),
            Mark::Background(ref color) => ("background".into(), color.to_string().into()),
            Mark::Link(ref url) => ("link".into(), url.to_string().into()),
            Mark::Custom(ref name, ref json) => (name.to_string(), json.to_string().into()),
            Mark::None => ("_".into(), Value::Null),
        }
    }

    pub(crate) fn key_value(&self) -> (String, Value) {
        self.key_value_with_range()
    }
}

impl Serialize for MarkContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut map = serde_json::Map::new();
        match self.data {
            Mark::Bold => {
                map.insert("bold".to_string(), true.into());
            }
            Mark::Italic => {
                map.insert("italic".to_string(), true.into());
            }
            Mark::Underline => {
                map.insert("underline".to_string(), true.into());
            }
            Mark::StrikeThrough => {
                map.insert("strikethrough".to_string(), true.into());
            }
            Mark::Code => {
                map.insert("code".to_string(), true.into());
            }
            Mark::Subscript => {
                map.insert("subscript".to_string(), true.into());
            }
            Mark::Superscript => {
                map.insert("superscript".to_string(), true.into());
            }
            Mark::Color(ref color) => {
                map.insert("color".to_string(), color.to_string().into());
            }
            Mark::Background(ref color) => {
                map.insert("background".to_string(), color.to_string().into());
            }
            Mark::Link(ref url) => {
                map.insert("link".to_string(), url.to_string().into());
            }
            Mark::Custom(ref name, ref json) => {
                map.insert("name".to_string(), name.to_string().into());
                map.insert("json".to_string(), json.to_string().into());
            }
            Mark::None => {}
        }
        serde_json::Value::Object(map).serialize(serializer)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) enum Mark {
    Bold,
    Italic,
    Underline,
    StrikeThrough,
    Code,
    Subscript,
    Superscript,
    Color(String),
    Background(String),
    Link(String),
    Custom(String, String),
    #[default]
    None,
}
