use crate::{
    parser::{LoweringError, ParseError},
    toml::TomlError,
};

#[derive(Debug)]
pub(crate) enum LatchError {
    Parse(ParseError),
    Lowering(LoweringError),
    Toml(TomlError),
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
