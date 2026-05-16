use std::ops::Range;

use pest::error::Error;

use crate::{parser::Rule, state::FileId};

#[derive(Debug)]
pub(crate) struct ParseError {
    pub(crate) message: String,
    pub(crate) line: usize,
    pub(crate) offset: Range<usize>,
    pub(crate) file: FileId,
}

impl ParseError {
    pub(crate) fn new(message: String, line: usize, offset: Range<usize>, file: FileId) -> Self {
        ParseError {
            message,
            line,
            offset,
            file,
        }
    }

    pub(crate) fn from_pest_error(
        pest_error: Error<Rule>,
        line: usize,
        pre_offset: usize,
        file: FileId,
    ) -> Self {
        let message = pest_error.variant.message().into_owned();
        let offset = match pest_error.location {
            pest::error::InputLocation::Pos(o) => pre_offset + o..o + 1,
            pest::error::InputLocation::Span((start, end)) => pre_offset + start..end,
        };
        ParseError {
            message,
            line,
            offset,
            file,
        }
    }
}

#[derive(Debug)]
pub(crate) struct LoweringError {
    pub(crate) message: String,
    pub(crate) line: usize,
    pub(crate) file: FileId,
}
