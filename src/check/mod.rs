use crate::{
    check::error::CheckError,
    parser::{Field, Register},
    state::{FieldId, RegId, State},
};

mod error;
#[cfg(test)]
mod test;

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
            .map(|(a, b)| CheckError::FieldOverlap {
                first: **a,
                other: **b,
            })
            .collect())
    }
}

pub(crate) fn check_reg_name_collisions(
    state: &State,
    reg_ids: &[RegId],
) -> Result<(), Vec<CheckError>> {
    let pairs: Vec<(RegId, RegId)> = reg_ids
        .iter()
        .enumerate()
        .flat_map(|(i, a_id)| {
            let reg_a = state.get_reg(*a_id);
            reg_ids[i + 1..]
                .iter()
                .filter(move |b_id| reg_a.get_name() == state.get_reg(**b_id).get_name())
                .map(move |b_id| (*a_id, *b_id))
        })
        .collect();
    if !pairs.is_empty() {
        Err(pairs
            .iter()
            .map(|(a, b)| CheckError::RegNameCollision {
                first: *a,
                other: *b,
            })
            .collect())
    } else {
        Ok(())
    }
}
