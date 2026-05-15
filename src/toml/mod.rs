use crate::state::{FileId, RegId, State};
pub(crate) use error::TomlError;
use types::{TomlFile, TomlRegister};

mod error;
#[cfg(test)]
mod test;
mod types;

pub(crate) fn from_toml(
    state: &mut State,
    src: &str,
    file: FileId,
) -> Result<Vec<RegId>, TomlError> {
    let value: toml::Value = toml::from_str(src)?;
    let toml_file = TomlFile::try_from(value)?;
    Ok(toml_file
        .registers
        .into_iter()
        .map(|(reg_name, reg)| reg.into_reg(reg_name, state))
        .collect())
}

pub(crate) fn to_toml(state: &State) -> Result<String, toml::ser::Error> {
    let file = TomlFile {
        registers: state
            .get_regs()
            .iter()
            .map(|reg| {
                (
                    reg.get_name().to_string(),
                    TomlRegister::from_reg(reg, state),
                )
            })
            .collect(),
    };
    toml::to_string_pretty(&file)
}
