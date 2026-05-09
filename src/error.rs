use crate::parser::{LoweringError, ParseError};

pub(crate) enum LatchError {
    Parse(ParseError),
    Lowering(LoweringError),
}

impl From<ParseError> for LatchError {
    fn from(e: ParseError) -> Self {
        LatchError::Parse(e)
    }
}

impl From<LoweringError> for LatchError {
    fn from(e: LoweringError) -> Self {
        LatchError::Lowering(e)
    }
}
