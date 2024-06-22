use serde::Serialize;
use serde_json::Value;

use crate::id::IdRange;

#[derive(Debug, Clone, Default)]
pub(crate) struct Mark {
    pub(crate) range: IdRange,
    pub(crate) data: MarkContent,
}

impl Mark {
    pub(crate) fn new(data: MarkContent) -> Self {
        Self {
            data,
            ..Default::default()
        }
    }

    // used for debugging
    pub(crate) fn key_value_with_range(&self) -> (String, Value) {
        match self.data {
            MarkContent::Bold => (
                "bold".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            MarkContent::Italic => (
                "italic".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            MarkContent::Underline => (
                "underline".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            MarkContent::StrikeThrough => (
                "strikethrough".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            MarkContent::Code => (
                "code".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            MarkContent::Subscript => (
                "subscript".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            MarkContent::Superscript => (
                "superscript".to_string(),
                serde_json::to_value((self.range.to_string(), true)).unwrap(),
            ),
            MarkContent::Color(ref color) => ("color".to_string(), color.to_string().into()),
            MarkContent::Background(ref color) => {
                ("background".to_string(), color.to_string().into())
            }
            MarkContent::Link(ref url) => ("link".to_string(), url.to_string().into()),
            MarkContent::Custom(ref name, ref json) => (name.to_string(), json.to_string().into()),
            MarkContent::None => ("_".to_string(), Value::Null),
        }
    }

    pub(crate) fn key_value_without_range(&self) -> (String, Value) {
        match self.data {
            MarkContent::Bold => ("bold".into(), true.into()),
            MarkContent::Italic => ("italic".into(), true.into()),
            MarkContent::Underline => ("underline".into(), true.into()),
            MarkContent::StrikeThrough => ("strikethrough".into(), true.into()),
            MarkContent::Code => ("code".into(), true.into()),
            MarkContent::Subscript => ("subscript".into(), true.into()),
            MarkContent::Superscript => ("superscript".into(), true.into()),
            MarkContent::Color(ref color) => ("color".into(), color.to_string().into()),
            MarkContent::Background(ref color) => ("background".into(), color.to_string().into()),
            MarkContent::Link(ref url) => ("link".into(), url.to_string().into()),
            MarkContent::Custom(ref name, ref json) => (name.to_string(), json.to_string().into()),
            MarkContent::None => ("_".into(), Value::Null),
        }
    }

    pub(crate) fn key_value(&self) -> (String, Value) {
        self.key_value_with_range()
    }
}

impl Serialize for Mark {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut map = serde_json::Map::new();
        match self.data {
            MarkContent::Bold => {
                map.insert("bold".to_string(), true.into());
            }
            MarkContent::Italic => {
                map.insert("italic".to_string(), true.into());
            }
            MarkContent::Underline => {
                map.insert("underline".to_string(), true.into());
            }
            MarkContent::StrikeThrough => {
                map.insert("strikethrough".to_string(), true.into());
            }
            MarkContent::Code => {
                map.insert("code".to_string(), true.into());
            }
            MarkContent::Subscript => {
                map.insert("subscript".to_string(), true.into());
            }
            MarkContent::Superscript => {
                map.insert("superscript".to_string(), true.into());
            }
            MarkContent::Color(ref color) => {
                map.insert("color".to_string(), color.to_string().into());
            }
            MarkContent::Background(ref color) => {
                map.insert("background".to_string(), color.to_string().into());
            }
            MarkContent::Link(ref url) => {
                map.insert("link".to_string(), url.to_string().into());
            }
            MarkContent::Custom(ref name, ref json) => {
                map.insert("name".to_string(), name.to_string().into());
                map.insert("json".to_string(), json.to_string().into());
            }
            MarkContent::None => {}
        }
        serde_json::Value::Object(map).serialize(serializer)
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) enum MarkContent {
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
