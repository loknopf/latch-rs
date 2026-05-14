use std::ops::Range;

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
struct TomlRegister {
    offset: u64,
    name: String,
    description: Option<String>,
    fields: Vec<TomlField>,
}

impl TomlRegister {
    fn into_reg(self, state: &mut State) -> RegId {
        let field_ids: Vec<FieldId> = self
            .fields
            .into_iter()
            .map(|field| field.into_field(state))
            .collect();
        let reg = Register::new(self.offset, self.name, self.description, field_ids);
        state.insert_reg(reg)
    }

    fn from_reg(reg: &Register, state: &State) -> Self {
        Self {
            offset: reg.get_offset(),
            name: reg.get_name().to_string(),
            description: reg.get_description().to_owned(),
            fields: reg
                .get_fields()
                .iter()
                .map(|f_id| TomlField::from_field(*f_id, state))
                .collect(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct TomlField {
    name: String,
    bits: BitRange,
    access: Access,
    description: Option<String>,
    enum_values: Option<Vec<String>>,
}

impl TomlField {
    fn into_field(self, state: &mut State) -> FieldId {
        let field = Field {
            name: self.name,
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
            name: field.name.clone(),
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
    let toml_result: Result<Vec<TomlRegister>, toml::de::Error> = toml::from_str(src);
    match toml_result {
        Ok(regs) => Ok(regs.into_iter().map(|reg| reg.into_reg(state)).collect()),
        Err(e) => Err(TomlError::from(e).with_file(file)),
    }
}

pub(crate) fn to_toml(state: &State) -> Result<String, toml::ser::Error> {
    let regs: Vec<TomlRegister> = state
        .get_regs()
        .iter()
        .map(|reg| TomlRegister::from_reg(reg, state))
        .collect();
    toml::to_string_pretty(&regs)
}
