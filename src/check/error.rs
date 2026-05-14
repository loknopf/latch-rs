use crate::state::{FieldId, RegId};

#[derive(Debug)]
pub(crate) enum CheckError {
    FieldOverlap { first: FieldId, other: FieldId },
    RegNameCollision { first: RegId, other: RegId },
    FieldNameCollision { first: FieldId, other: FieldId },
}
