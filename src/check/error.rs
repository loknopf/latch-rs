use crate::state::{FieldId, RegId};

pub(crate) enum CheckError {
    FieldOverlap { first: FieldId, other: FieldId },
    RegNameCollision { first: RegId, other: RegId },
    FieldNameCollision { first: FieldId, other: FieldId },
}
