use crate::{
    check::CheckError,
    parser::{LoweringError, ParseError},
    toml::TomlError,
};

#[derive(Debug)]
pub(crate) enum LatchError {
    Parse(ParseError),
    Lowering(LoweringError),
    Check(CheckError),
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

impl From<CheckError> for LatchError {
    fn from(e: CheckError) -> Self {
        LatchError::Check(e)
    }
}
