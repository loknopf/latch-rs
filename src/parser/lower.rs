use crate::{
    error::LatchError,
    ir::{Field, Register},
    parser::{AnnotationKind, KvPair, LoweringError, ParseError, Rule},
    state::{FileId, Location, RegId, State},
};

enum LowerState {
    Empty,
    Active {
        reg: Register,
        line: usize,
        failed_fields: usize,
    },
    Failed,
}

pub(super) struct LowerCtx {
    state: LowerState,
    errors: Vec<LatchError>,
    registers: Vec<RegId>,
    file_id: FileId,
}

impl LowerCtx {
    pub(super) fn new(file: FileId) -> Self {
        Self {
            state: LowerState::Empty,
            errors: Vec::default(),
            registers: Vec::default(),
            file_id: file,
        }
    }

    pub(super) fn on_parse_error(
        &mut self,
        kind: AnnotationKind,
        line: usize,
        pre_offset: usize,
        e: pest::error::Error<Rule>,
    ) {
        let parse_error = ParseError::from_pest_error(e, line, pre_offset, self.file_id);
        self.errors.push(LatchError::from(parse_error));
        match kind {
            AnnotationKind::Reg => {
                self.state = LowerState::Failed;
            }
            AnnotationKind::Field => match &mut self.state {
                LowerState::Active { failed_fields, .. } => {
                    *failed_fields += 1;
                }
                _ => {}
            },
        }
    }

    pub(super) fn on_reg(&mut self, kv: &Vec<KvPair>, line: usize, state: &mut State) {
        self.flush_active(state);
        match Register::from_kv_values(kv, line, self.file_id) {
            Ok(reg) => {
                self.state = LowerState::Active {
                    reg,
                    line,
                    failed_fields: 0,
                }
            }
            Err(e) => {
                self.state = LowerState::Failed;
                self.errors.push(LatchError::from(e));
            }
        }
    }

    pub(super) fn on_field(&mut self, kv: &Vec<KvPair>, field_line: usize, state: &mut State) {
        match &mut self.state {
            LowerState::Empty => self.errors.push(
                LoweringError {
                    message: "field annotation must follow a reg annotation".to_string(),
                    line: field_line,
                    file: self.file_id,
                }
                .into(),
            ),
            //The register this field belongs to has failed to successfully parse - skipping it
            // FIXME: A corner-case might arise when a a @reg annotation fails and a lonely @field annotation is placed somewhere after it
            LowerState::Failed => {}
            LowerState::Active {
                reg,
                line,
                failed_fields,
            } => {
                // Guard against lone @field annotations - failed fields are counted as to not cascade lone-field errors
                if field_line.saturating_sub(*line + reg.get_fields().len() + *failed_fields) != 1 {
                    self.errors.push(
                        LoweringError {
                            message:
                                "field annotation must immediately follow a reg or field annotation"
                                    .to_string(),
                            line: field_line,
                            file: self.file_id,
                        }
                        .into(),
                    );
                } else {
                    match Field::from_kv_values(&kv, field_line, self.file_id) {
                        Ok(f) => {
                            let field_id = state.insert_field(f);
                            state.add_field_loc(
                                field_id,
                                Location {
                                    line: field_line,
                                    file: self.file_id,
                                },
                            );
                            reg.add_field(field_id);
                        }
                        Err(e) => {
                            self.errors.push(e.into());
                            *failed_fields += 1;
                        }
                    }
                }
            }
        }
    }

    pub(super) fn finish(&mut self, state: &mut State) -> Result<Vec<RegId>, Vec<LatchError>> {
        self.flush_active(state);
        if !self.errors.is_empty() {
            Err(std::mem::take(&mut self.errors))
        } else {
            Ok(std::mem::take(&mut self.registers))
        }
    }

    fn flush_active(&mut self, state: &mut State) {
        if let LowerState::Active {
            reg,
            line,
            failed_fields,
        } = self.take_state()
        {
            if reg.get_fields().is_empty() {
                //Emit an error for no subsequent fields ONLY IF no fields failed to parse
                if failed_fields == 0 {
                    self.errors.push(LatchError::from(LoweringError {
                        message: "Registers require at least one @field.".into(),
                        line: line,
                        file: self.file_id,
                    }));
                }
            }
            let reg_id = state.insert_reg(reg);
            self.registers.push(reg_id);
            state.add_reg_loc(
                reg_id,
                Location {
                    file: self.file_id,
                    line: line,
                },
            );
        }
    }

    fn take_state(&mut self) -> LowerState {
        std::mem::replace(&mut self.state, LowerState::Empty)
    }
}
