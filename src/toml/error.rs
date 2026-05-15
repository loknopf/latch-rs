use std::ops::Range;

use crate::state::FileId;

#[derive(Debug)]
pub(crate) struct TomlError {
    message: String,
    span: Option<Range<usize>>,
    file: Option<FileId>,
}

impl From<toml::de::Error> for TomlError {
    fn from(value: toml::de::Error) -> Self {
        Self {
            message: value.message().to_string(),
            span: value.span(),
            file: None,
        }
    }
}

impl TomlError {
    pub(super) fn msg(message: impl Into<String>) -> Self {
        //TODO: see if we can pass a span value
        Self {
            message: message.into(),
            span: None,
            file: None,
        }
    }
    pub(super) fn with_file(self, file: FileId) -> Self {
        Self {
            message: self.message,
            span: self.span,
            file: Some(file),
        }
    }

    pub(super) fn with_span(self, span: Range<usize>) -> Self {
        Self {
            message: self.message,
            span: Some(span),
            file: self.file,
        }
    }
}
