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
    let regs: Vec<&Register> = reg_ids.iter().map(|id| state.get_reg(*id)).collect();
    // This should be using RegIds - the CheckError::NameCollision expects (RegId, RegId)
    let pairs: Vec<(&Register, &Register)> = regs
        .iter()
        .enumerate()
        .flat_map(|(i, reg_a)| {
            regs[i + 1..]
                .iter()
                .filter(move |reg_b| reg_a.get_name() == reg_b.get_name())
                .map(move |reg_b| (*reg_a, *reg_b))
        })
        .collect();

    Ok(())
}
