use pest_derive::Parser;

pub(crate) use crate::parser::{
    error::{LoweringError, ParseError},
    ir::Register,
    kv::parse_kv_pairs,
    scanner::{AnnotationKind, scan},
};
use crate::{error::LatchError, parser::ir::Field};

mod error;
mod ir;
mod kv;
mod scanner;
#[cfg(test)]
mod tests;

#[derive(Parser)]
#[grammar = "annotation.pest"]
struct LatchParser;

pub(crate) fn parse(input: &str) -> Result<Vec<Register>, Vec<LatchError>> {
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

    enum State {
        Empty,
        Active { reg: Register, line: usize },
        Failed,
    }

    let mut registers: Vec<Register> = Vec::new();
    let mut state: State = State::Empty;

    for (ann, kv) in lowerable {
        match ann.kind {
            AnnotationKind::Reg => {
                if let State::Active { reg, line } = state {
                    if let Err(e) = empty_reg_guard(&reg, line) {
                        errors.push(e.into());
                    } else {
                        registers.push(reg);
                    }
                }
                state = match Register::from_kv_values(&kv, ann.line) {
                    Ok(reg) => State::Active {
                        reg,
                        line: ann.line,
                    },
                    Err(e) => {
                        errors.push(e.into());
                        State::Failed
                    }
                };
            }
            AnnotationKind::Field => match &mut state {
                State::Empty => errors.push(
                    LoweringError {
                        message: "field annotation must follow a reg annotation".to_string(),
                        line: ann.line,
                    }
                    .into(),
                ),
                State::Active { reg, line } => {
                    //guard against orphaned -- @field lines that apear after a -- @reg but not directly after it or another -- @field
                    if ann.line.saturating_sub(*line) != 1 {
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
                                reg.add_field(f);
                                *line = ann.line;
                            }
                            Err(e) => errors.push(e.into()),
                        }
                    }
                }
                State::Failed => {
                    continue;
                }
            },
        }
    }

    if let State::Active { reg, line } = state {
        if let Err(e) = empty_reg_guard(&reg, line) {
            errors.push(e.into());
        } else {
            registers.push(reg);
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
