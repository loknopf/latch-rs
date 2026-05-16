use crate::{
    error::LatchError,
    ir::Register,
    state::{FieldId, RegId, State},
};

pub(crate) use error::CheckError;

mod error;
#[cfg(test)]
mod test;

pub(crate) fn check_registers(regs: &[RegId], state: &State) -> Result<(), Vec<LatchError>> {
    let mut accum: Vec<CheckError> = check_reg_name_collisions(state, regs)
        .err()
        .unwrap_or_default();
    for r_id in regs {
        let reg = state.get_reg(*r_id);
        if let Err(mut errs) = check_field_overlap(state, reg) {
            accum.append(&mut errs);
        }
        if let Err(mut errs) = check_field_name_collision(state, reg) {
            accum.append(&mut errs);
        }
    }
    if !accum.is_empty() {
        Err(accum.into_iter().map(|e| LatchError::Check(e)).collect())
    } else {
        Ok(())
    }
}

fn check_field_overlap(state: &State, reg: &Register) -> Result<(), Vec<CheckError>> {
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

fn check_reg_name_collisions(state: &State, reg_ids: &[RegId]) -> Result<(), Vec<CheckError>> {
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

fn check_field_name_collision(state: &State, reg: &Register) -> Result<(), Vec<CheckError>> {
    let fields = reg.get_fields();
    let pairs: Vec<(FieldId, FieldId)> = fields
        .iter()
        .enumerate()
        .flat_map(|(i, a_id)| {
            let field_a = state.get_field(*a_id);
            fields[i + 1..]
                .iter()
                .filter(|b_id| field_a.name == state.get_field(**b_id).name)
                .map(|b_id| (*a_id, *b_id))
        })
        .collect();
    if !pairs.is_empty() {
        Err(pairs
            .iter()
            .map(|(a_id, b_id)| CheckError::FieldNameCollision {
                first: *a_id,
                other: *b_id,
            })
            .collect())
    } else {
        Ok(())
    }
}
