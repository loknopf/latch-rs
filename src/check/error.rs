use crate::state::FieldId;

pub(crate) enum CheckError {
    FieldOverlap { a: FieldId, b: FieldId },
}
