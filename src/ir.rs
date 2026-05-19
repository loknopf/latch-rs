use crate::{
    parser::{KvPair, LoweringError},
    state::{FieldId, FileId},
    types::{Access, BitSpec, Value},
};

#[derive(Debug, Clone)]
pub(crate) struct Register {
    offset: u64,
    name: String,
    description: Option<String>,
    fields: Vec<FieldId>,
}

impl Register {
    pub(crate) fn new(
        offset: u64,
        name: String,
        description: Option<String>,
        fields: Vec<FieldId>,
    ) -> Self {
        Self {
            offset,
            name,
            description,
            fields,
        }
    }

    pub(crate) fn from_kv_values(
        values: &[KvPair],
        line: usize,
        file: FileId,
    ) -> Result<Self, LoweringError> {
        let offset = require(
            values,
            |v| v.key == "offset",
            LoweringError {
                message: "Require an offset key.".to_string(),
                line: line,
                file,
            },
        )?;
        let name = require(
            values,
            |v| v.key == "name",
            LoweringError {
                message: "Require a name key.".to_string(),
                line,
                file,
            },
        )?;

        let description = allow(values, |v| v.key == "description")
            .map(|v| {
                v.as_string().ok_or(LoweringError {
                    message: "description must be a string value.".to_string(),
                    line,
                    file,
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

    pub(crate) fn add_field(&mut self, field_id: FieldId) {
        self.fields.push(field_id);
    }

    pub(crate) fn get_fields(&self) -> &[FieldId] {
        &self.fields
    }

    pub(crate) fn get_name(&self) -> &str {
        &self.name
    }

    pub(crate) fn get_offset(&self) -> u64 {
        self.offset
    }

    pub(crate) fn get_description(&self) -> &Option<String> {
        &self.description
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Field {
    pub(crate) name: String,
    pub(crate) bits: BitSpec,
    pub(crate) access: Access,
    pub(crate) description: Option<String>,
    pub(crate) enum_values: Option<Vec<String>>,
}

impl Field {
    pub(crate) fn from_kv_values(
        values: &[KvPair],
        line: usize,
        file: FileId,
    ) -> Result<Self, LoweringError> {
        dbg!(values);
        let name = require(
            values,
            |v| v.key == "name",
            LoweringError {
                message: "Fields require a name.".to_string(),
                line,
                file,
            },
        )?;
        let bits = require(
            values,
            |v| v.key == "bits",
            LoweringError {
                message: "Fields require a bits declaration.".to_string(),
                line,
                file,
            },
        )?;
        let access = allow(values, |v| v.key == "access").unwrap_or(&Value::Access(Access::RO));
        let description = allow(values, |v| v.key == "description")
            .map(|v| {
                v.as_string().ok_or(LoweringError {
                    message: "description must be a string value.".to_string(),
                    line,
                    file,
                })
            })
            .transpose()?
            .cloned();

        let enum_values = allow(values, |v| v.key == "enum")
            .map(|v| {
                v.as_vec_string().ok_or(LoweringError {
                    message: "enum must be a quoted list of strings".to_string(),
                    line: line,
                    file,
                })
            })
            .transpose()?
            .cloned();

        Ok(Field {
            access: access.as_access().expect(
                "Access must be either RO, RW or WO. If absent it is replaced by RO per default",
            ),
            bits: bits.as_bit_spec().expect("Expecting a bit range."),
            description: description,
            enum_values: enum_values,
            name: name.as_string().expect("Expecting a name.").clone(),
        })
    }

    pub(crate) fn bits_overlap(&self, other: &Field) -> bool {
        match (self.bits, other.bits) {
            (BitSpec::Single(i), BitSpec::Single(o)) => i == o,
            (BitSpec::Single(i), BitSpec::Span(o, v))
            | (BitSpec::Span(o, v), BitSpec::Single(i)) => o <= i && i <= v,
            (BitSpec::Span(i, o), BitSpec::Span(k, v)) => i <= v && k <= o,
        }
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
