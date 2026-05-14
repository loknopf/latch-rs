use std::ops::Range;

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

use crate::{
    ir::{Field, Register},
    state::{FieldId, FileId, RegId, State},
    types::{Access, BitRange},
};

#[derive(Debug)]
pub(crate) struct TomlError {
    message: String,
    span: Option<Range<usize>>,
    file: Option<FileId>,
}

impl From<toml::de::Error> for TomlError {
    fn from(value: toml::de::Error) -> Self {
        Self {
            message: value.message().to_string(),
            span: value.span(),
            file: None,
        }
    }
}

impl TomlError {
    fn with_file(self, file: FileId) -> Self {
        Self {
            message: self.message,
            span: self.span,
            file: Some(file),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlFile {
    #[serde(flatten)]
    registers: IndexMap<String, TomlRegister>,
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlRegister {
    offset: u64,
    description: Option<String>,
    #[serde(flatten)]
    fields: IndexMap<String, TomlField>,
}

impl TomlRegister {
    fn into_reg(self, name: String, state: &mut State) -> RegId {
        let field_ids: Vec<FieldId> = self
            .fields
            .into_iter()
            .map(|(f_name, field)| field.into_field(f_name, state))
            .collect();
        let reg = Register::new(self.offset, name, self.description, field_ids);
        state.insert_reg(reg)
    }

    fn from_reg(reg: &Register, state: &State) -> Self {
        Self {
            offset: reg.get_offset(),
            description: reg.get_description().to_owned(),
            fields: reg
                .get_fields()
                .iter()
                .map(|f_id| {
                    (
                        state.get_field(*f_id).name.clone(),
                        TomlField::from_field(*f_id, state),
                    )
                })
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlField {
    bits: BitRange,
    #[serde(skip_serializing_if = "Access::is_read_only")]
    access: Access,
    description: Option<String>,
    enum_values: Option<Vec<String>>,
}

impl TomlField {
    fn into_field(self, name: String, state: &mut State) -> FieldId {
        let field = Field {
            name: name,
            bits: self.bits,
            access: self.access,
            description: self.description,
            enum_values: self.enum_values,
        };
        state.insert_field(field)
    }

    fn from_field(field_id: FieldId, state: &State) -> Self {
        let field = state.get_field(field_id);
        Self {
            bits: field.bits,
            access: field.access,
            description: field.description.to_owned(),
            enum_values: field.enum_values.to_owned(),
        }
    }
}

pub(crate) fn from_toml(
    state: &mut State,
    src: &str,
    file: FileId,
) -> Result<Vec<RegId>, TomlError> {
    let file_result: Result<TomlFile, toml::de::Error> = toml::from_str(src);
    match file_result {
        Ok(f) => Ok(f
            .registers
            .into_iter()
            .map(|(reg_name, reg)| reg.into_reg(reg_name, state))
            .collect()),
        Err(e) => Err(TomlError::from(e).with_file(file)),
    }
}

pub(crate) fn to_toml(state: &State) -> Result<String, toml::ser::Error> {
    let file = TomlFile {
        registers: state
            .get_regs()
            .iter()
            .map(|reg| {
                (
                    reg.get_name().clone().to_string(),
                    TomlRegister::from_reg(reg, state),
                )
            })
            .collect(),
    };
    toml::to_string_pretty(&file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        parser::parse,
        state::{FileId, State},
    };

    fn state_from_src(src: &str) -> State {
        let mut state = State::default();
        parse(&mut state, src, FileId(0)).expect("annotation parse failed");
        state
    }

    // Snapshot the TOML produced for a single register with span and single-bit
    // fields. Single register avoids SlotMap ordering non-determinism.
    #[test]
    fn test_serialize_single_reg() {
        let state = state_from_src(
            "-- @reg name=ctrl offset=0x00\n\
             -- @field name=enable bits=0 access=RW\n\
             -- @field name=mode bits=1:3 access=RO",
        );
        let toml = to_toml(&state).expect("serialization failed");
        insta::assert_snapshot!(toml);
    }

    // Snapshot the TOML produced for a register with all optional fields filled.
    #[test]
    fn test_serialize_with_enum_and_description() {
        let state = state_from_src(
            "-- @reg name=status offset=0x04\n\
             -- @field name=mode bits=0:1 access=RO enum=\"fast,slow,idle\"",
        );
        let toml = to_toml(&state).expect("serialization failed");
        insta::assert_snapshot!(toml);
    }

    // Serialize then deserialize and verify the structural content is preserved.
    #[test]
    fn test_round_trip_structure() {
        let state1 = state_from_src(
            "-- @reg name=ctrl offset=0x00\n\
             -- @field name=enable bits=0 access=RW\n\
             -- @field name=mode bits=1:3 access=RO",
        );
        let toml_str = to_toml(&state1).expect("serialization failed");

        let mut state2 = State::default();
        from_toml(&mut state2, &toml_str, FileId(0)).expect("deserialization failed");

        let regs = state2.get_regs();
        assert_eq!(regs.len(), 1);
        let reg = regs[0];
        assert_eq!(reg.get_name(), "ctrl");
        assert_eq!(reg.get_offset(), 0x00);
        assert_eq!(reg.get_fields().len(), 2);

        let fields: Vec<_> = reg
            .get_fields()
            .iter()
            .map(|id| state2.get_field(*id))
            .collect();
        let enable = fields
            .iter()
            .find(|f| f.name == "enable")
            .expect("enable field missing");
        let mode = fields
            .iter()
            .find(|f| f.name == "mode")
            .expect("mode field missing");
        assert_eq!(enable.bits, crate::types::BitRange::Single(0));
        assert_eq!(enable.access, crate::types::Access::RW);
        assert_eq!(mode.bits, crate::types::BitRange::Span(1, 3));
        assert_eq!(mode.access, crate::types::Access::RO);
    }

    // Re-serialize after round-trip and assert the TOML string is identical.
    #[test]
    fn test_round_trip_string_stability() {
        let state1 = state_from_src(
            "-- @reg name=ctrl offset=0x00\n\
             -- @field name=enable bits=0 access=RW\n\
             -- @field name=mode bits=1:3 access=RO",
        );
        let toml1 = to_toml(&state1).expect("first serialization failed");

        let mut state2 = State::default();
        from_toml(&mut state2, &toml1, FileId(0)).expect("deserialization failed");
        let toml2 = to_toml(&state2).expect("second serialization failed");

        assert_eq!(toml1, toml2);
    }
}
