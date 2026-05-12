use std::usize;

use pest_derive::Parser;

pub(crate) use crate::parser::{
    error::{LoweringError, ParseError},
    ir::{Field, Register},
    kv::parse_kv_pairs,
    scanner::{AnnotationKind, scan},
};
use crate::{
    error::LatchError,
    state::{FileId, Location, RegId, State},
};

mod error;
mod ir;
mod kv;
mod scanner;
#[cfg(test)]
mod tests;

#[derive(Parser)]
#[grammar = "annotation.pest"]
struct LatchParser;

pub(crate) fn parse(
    state: &mut State,
    input: &str,
    file_id: FileId,
) -> Result<Vec<RegId>, Vec<LatchError>> {
    let annotations = scan(input);
    let mut errors: Vec<LatchError> = Vec::new();
    let mut lowerable = Vec::new();

    for ann in annotations {
        match parse_kv_pairs(ann.content) {
            Ok(kv) => lowerable.push((ann, kv)),
            Err(e) => {
                errors.push(ParseError::from_pest_error(e, ann.line, ann.pre_offset()).into())
            }
        }
    }

    enum LowerState {
        Empty,
        Active { reg: Register, line: usize },
        Failed,
    }

    let mut registers: Vec<RegId> = Vec::new();
    let mut lower_state: LowerState = LowerState::Empty;

    for (ann, kv) in lowerable {
        match ann.kind {
            AnnotationKind::Reg => {
                if let LowerState::Active { reg, line } = lower_state {
                    if let Err(e) = empty_reg_guard(&reg, line) {
                        errors.push(e.into());
                    } else {
                        let reg_id = state.insert_reg(reg);
                        state.add_reg_loc(
                            reg_id,
                            Location {
                                line,
                                file: file_id,
                            },
                        );
                        registers.push(reg_id);
                    }
                }
                lower_state = match Register::from_kv_values(&kv, ann.line) {
                    Ok(reg) => LowerState::Active {
                        reg,
                        line: ann.line,
                    },
                    Err(e) => {
                        errors.push(e.into());
                        LowerState::Failed
                    }
                };
            }
            AnnotationKind::Field => match &mut lower_state {
                LowerState::Empty => errors.push(
                    LoweringError {
                        message: "field annotation must follow a reg annotation".to_string(),
                        line: ann.line,
                    }
                    .into(),
                ),
                LowerState::Active { reg, line } => {
                    //guard against orphaned -- @field lines that apear after a -- @reg but not directly after it or another -- @field
                    if ann.line.saturating_sub(*line + reg.get_fields().len()) != 1 {
                        errors.push(
                            LoweringError {
                                message: "field annotation must immediately follow a reg or field annotation"
                                    .to_string(),
                                line: ann.line,
                            }
                            .into(),
                        );
                    } else {
                        match Field::from_kv_values(&kv, ann.line) {
                            Ok(f) => {
                                let field_id = state.insert_field(f);
                                state.add_field_loc(
                                    field_id,
                                    Location {
                                        line: ann.line,
                                        file: file_id,
                                    },
                                );
                                reg.add_field(field_id);
                            }
                            Err(e) => errors.push(e.into()),
                        }
                    }
                }
                LowerState::Failed => {
                    continue;
                }
            },
        }
    }

    if let LowerState::Active { reg, line } = lower_state {
        if let Err(e) = empty_reg_guard(&reg, line) {
            errors.push(e.into());
        } else {
            let reg_id = state.insert_reg(reg);
            state.add_reg_loc(
                reg_id,
                Location {
                    line,
                    file: file_id,
                },
            );
            registers.push(reg_id);
        }
    }

    if errors.is_empty() {
        Ok(registers)
    } else {
        Err(errors)
    }
}

fn empty_reg_guard(reg: &Register, line: usize) -> Result<(), LoweringError> {
    if reg.get_fields().is_empty() {
        Err(LoweringError {
            message: "Registers require at least one @field.".into(),
            line,
        })
    } else {
        Ok(())
    }
}
