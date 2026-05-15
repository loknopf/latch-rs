use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use toml::Value;

use crate::{
    ir::{Field, Register},
    state::{FieldId, RegId, State},
    toml::error::TomlError,
    types::{Access, BitRange},
};

#[derive(Debug, Serialize)]
pub(super) struct TomlFile {
    #[serde(flatten)]
    pub(super) registers: IndexMap<String, TomlRegister>,
}

impl TryFrom<toml::Value> for TomlFile {
    type Error = TomlError;
    fn try_from(value: toml::Value) -> Result<Self, Self::Error> {
        if let Value::Table(t) = value {
            let mut reg_map = IndexMap::default();
            for (key, value) in t {
                let toml_reg = TomlRegister::try_from(value)?;
                reg_map.insert(key, toml_reg);
            }
            Ok(Self { registers: reg_map })
        } else {
            Err(TomlError::msg(
                "latch-rs toml file may only contain top level tables",
            ))
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct TomlRegister {
    offset: u64,
    description: Option<String>,
    #[serde(flatten)]
    fields: IndexMap<String, TomlField>,
}

impl TomlRegister {
    pub(super) fn into_reg(self, name: String, state: &mut State) -> RegId {
        let field_ids: Vec<FieldId> = self
            .fields
            .into_iter()
            .map(|(f_name, field)| field.into_field(f_name, state))
            .collect();
        let reg = Register::new(self.offset, name, self.description, field_ids);
        state.insert_reg(reg)
    }

    pub(super) fn from_reg(reg: &Register, state: &State) -> Self {
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

impl TryFrom<toml::Value> for TomlRegister {
    type Error = TomlError;
    fn try_from(value: toml::Value) -> Result<Self, Self::Error> {
        let mut offset: Option<u64> = None;
        let mut description: Option<String> = None;
        let mut fields = IndexMap::default();
        if let Value::Table(t) = value {
            for (key, value) in t {
                match value {
                    Value::Table(_) => {
                        fields.insert(key, TomlField::try_from(value)?);
                    }
                    Value::String(s) => {
                        if key == "description" {
                            description = Some(s);
                        }
                    }
                    Value::Integer(k) => {
                        if key == "offset" {
                            offset = Some(k as u64)
                        }
                    }
                    _ => {
                        return Err(TomlError::msg(
                            "Register must contain only keys of type Table, String or Integer",
                        ));
                    }
                }
            }
            let offset =
                offset.ok_or_else(|| TomlError::msg("Register requires an offset member"))?;
            Ok(Self {
                offset: offset,
                description: description,
                fields: fields,
            })
        } else {
            Err(TomlError::msg("Register must be a table"))
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct TomlField {
    bits: BitRange,
    #[serde(skip_serializing_if = "Access::is_read_only")]
    access: Access,
    description: Option<String>,
    #[serde(rename = "enum")]
    enum_values: Option<Vec<String>>,
}

impl TomlField {
    pub(super) fn into_field(self, name: String, state: &mut State) -> FieldId {
        let field = Field {
            name: name,
            bits: self.bits,
            access: self.access,
            description: self.description,
            enum_values: self.enum_values,
        };
        state.insert_field(field)
    }

    pub(super) fn from_field(field_id: FieldId, state: &State) -> Self {
        let field = state.get_field(field_id);
        Self {
            bits: field.bits,
            access: field.access,
            description: field.description.to_owned(),
            enum_values: field.enum_values.to_owned(),
        }
    }
}

impl TryFrom<toml::Value> for TomlField {
    type Error = TomlError;
    fn try_from(value: toml::Value) -> Result<Self, Self::Error> {
        if let Value::Table(table) = value {
            let bits = table
                .get("bits")
                .ok_or_else(|| TomlError::msg("field is missing required member 'bits'"))?;
            let bits =
                BitRange::deserialize(bits.clone()).map_err(|e| TomlError::msg(e.to_string()))?;
            let access = match table.get("access") {
                Some(v) => {
                    Access::deserialize(v.clone()).map_err(|e| TomlError::msg(e.to_string()))?
                }
                None => Access::RO,
            };
            let description = table
                .get("description")
                .map(|v| match v {
                    Value::String(s) => Ok(s.clone()),
                    _ => Err(TomlError::msg("description member must be a string")),
                })
                .transpose()?;
            let enum_values = table
                .get("enum")
                .map(|v| match v {
                    Value::Array(arr) => arr
                        .iter()
                        .map(|item| match item {
                            Value::String(s) => Ok(s.clone()),
                            _ => Err(TomlError::msg("enum values must be strings")),
                        })
                        .collect::<Result<Vec<_>, _>>(),
                    _ => Err(TomlError::msg("enum values must be an array")),
                })
                .transpose()?;
            Ok(TomlField {
                bits,
                access,
                description,
                enum_values,
            })
        } else {
            Err(TomlError::msg("Expected a value table."))
        }
    }
}
