use crate::parser::{
    error::LoweringError,
    kv::{Access, BitRange, KvPair, Value},
};

pub(crate) struct Register {
    offset: u64,
    name: String,
    description: Option<String>,
    fields: Vec<Field>,
}

impl Register {
    pub(crate) fn from_kv_values(values: &[KvPair], line: usize) -> Result<Self, LoweringError> {
        let offset = require(
            values,
            |v| v.key == "offset",
            LoweringError {
                message: "Require an offset key.".to_string(),
                line: line,
            },
        )?;
        let name = require(
            values,
            |v| v.key == "name",
            LoweringError {
                message: "Require a name key.".to_string(),
                line,
            },
        )?;

        let description = allow(values, |v| v.key == "description")
            .map(|v| {
                v.as_string().ok_or(LoweringError {
                    message: "description must be a string value.".to_string(),
                    line,
                })
            })
            .transpose()?
            .cloned();

        Ok(Register {
            offset: offset
                .as_u64()
                .expect("Expecting offset to be of number value."),
            name: name
                .as_string()
                .expect("Expecting name to be a bare value.")
                .clone(),
            description,
            fields: Vec::new(),
        })
    }

    pub(crate) fn add_field(&mut self, field: Field) {
        self.fields.push(field);
    }

    pub(crate) fn get_fields(&self) -> &Vec<Field> {
        &self.fields
    }
}

pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) bits: BitRange,
    pub(crate) access: Access,
    pub(crate) description: Option<String>,
    pub(crate) enum_values: Option<Vec<String>>,
}

impl Field {
    pub(crate) fn from_kv_values(values: &[KvPair], line: usize) -> Result<Self, LoweringError> {
        let name = require(
            values,
            |v| v.key == "name",
            LoweringError {
                message: "Fields require a name.".to_string(),
                line,
            },
        )?;
        let bits = require(
            values,
            |v| v.key == "bits",
            LoweringError {
                message: "Fields require a bits declaration.".to_string(),
                line,
            },
        )?;
        let access = allow(values, |v| v.key == "access").unwrap_or(&Value::Access(Access::RO));
        let description = allow(values, |v| v.key == "description")
            .map(|v| {
                v.as_string().ok_or(LoweringError {
                    message: "description must be a string value.".to_string(),
                    line,
                })
            })
            .transpose()?
            .cloned();

        let enum_values = allow(values, |v| v.key == "enum")
            .map(|v| {
                v.as_vec_string().ok_or(LoweringError {
                    message: "enum must be a quoted list of strings".to_string(),
                    line: line,
                })
            })
            .transpose()?
            .cloned();

        Ok(Field {
            access: access.as_access().expect(
                "Access must be either RO, RW or WO. If absent it is replaced by RO per default",
            ),
            bits: bits.as_bit_range().expect("Expecting a bit range."),
            description: description,
            enum_values: enum_values,
            name: name.as_string().expect("Expecting a name.").clone(),
        })
    }
}

fn require(
    values: &[KvPair],
    pred: impl Fn(&KvPair) -> bool,
    err: LoweringError,
) -> Result<&Value, LoweringError> {
    values
        .iter()
        .find_map(|v| if pred(v) { Some(&v.value) } else { None })
        .ok_or_else(|| err)
}

fn allow(values: &[KvPair], pred: impl Fn(&KvPair) -> bool) -> Option<&Value> {
    values
        .iter()
        .find_map(|v| if pred(v) { Some(&v.value) } else { None })
}
