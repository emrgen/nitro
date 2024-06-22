use serde::Serialize;
use serde_json::Value;

use crate::id::IdRange;

#[derive(Debug, Clone)]
pub(crate) struct Mark {
    pub(crate) range: IdRange,
    pub(crate) data: MarkContent,
}

impl Mark {
    pub(crate) fn new(range: IdRange, data: MarkContent) -> Self {
        Self { range, data }
    }

    pub(crate) fn bold(range: IdRange) -> Self {
        Self::new(range, MarkContent::Bold)
    }

    pub(crate) fn italic(range: IdRange) -> Self {
        Self::new(range, MarkContent::Italic)
    }

    pub(crate) fn underline(range: IdRange) -> Self {
        Self::new(range, MarkContent::Underline)
    }

    pub(crate) fn strikethrough(range: IdRange) -> Self {
        Self::new(range, MarkContent::StrikeThrough)
    }

    pub(crate) fn code(range: IdRange) -> Self {
        Self::new(range, MarkContent::Code)
    }

    pub(crate) fn subscript(range: IdRange) -> Self {
        Self::new(range, MarkContent::Subscript)
    }

    pub(crate) fn superscript(range: IdRange) -> Self {
        Self::new(range, MarkContent::Superscript)
    }

    pub(crate) fn link(range: IdRange, url: String) -> Self {
        Self::new(range, MarkContent::Link(url))
    }

    pub(crate) fn color(range: IdRange, color: String) -> Self {
        Self::new(range, MarkContent::Color(color))
    }

    pub(crate) fn background(range: IdRange, color: String) -> Self {
        Self::new(range, MarkContent::Background(color))
    }

    pub(crate) fn custom(range: IdRange, name: String, json: String) -> Self {
        Self::new(range, MarkContent::Custom(name, json))
    }

    pub(crate) fn key_value(&self) -> (String, Value) {
        match self.data {
            MarkContent::Bold => ("bold".to_string(), true.into()),
            MarkContent::Italic => ("italic".to_string(), true.into()),
            MarkContent::Underline => ("underline".to_string(), true.into()),
            MarkContent::StrikeThrough => ("strikethrough".to_string(), true.into()),
            MarkContent::Code => ("code".to_string(), true.into()),
            MarkContent::Subscript => ("subscript".to_string(), true.into()),
            MarkContent::Superscript => ("superscript".to_string(), true.into()),
            MarkContent::Color(ref color) => ("color".to_string(), color.to_string().into()),
            MarkContent::Background(ref color) => {
                ("background".to_string(), color.to_string().into())
            }
            MarkContent::Link(ref url) => ("link".to_string(), url.to_string().into()),
            MarkContent::Custom(ref name, ref json) => {
                ("name".to_string(), name.to_string().into())
            }
        }
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
        }
        serde_json::Value::Object(map).serialize(serializer)
    }
}

#[derive(Debug, Clone)]
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
}
