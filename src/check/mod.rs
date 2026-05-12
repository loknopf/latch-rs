use crate::{
    check::error::CheckError,
    parser::{Field, Register},
    state::{FieldId, State},
};

mod error;

pub(crate) fn check_field_overlap(state: &State, reg: &Register) -> Result<(), Vec<CheckError>> {
    let pairs: Vec<(&FieldId, &FieldId)> = reg
        .get_fields()
        .iter()
        .enumerate()
        .flat_map(|(i, a)| {
            reg.get_fields()[i + 1..]
                .iter()
                .filter(move |b| {
                    let field_a = state.get_field(*a);
                    let field_b = state.get_field(**b);
                    field_a.bits_overlap(field_b)
                })
                .map(move |b| (a, b))
        })
        .collect();
    if pairs.is_empty() {
        Ok(())
    } else {
        Err(pairs
            .iter()
            .map(|(a, b)| CheckError::FieldOverlap { a: **a, b: **b })
            .collect())
    }
}
