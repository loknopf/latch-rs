use crate::{
    error::LatchError,
    ir::{Field, Register},
    state::State,
    types::{Access, BitSpec},
};

use super::parse;

// helpers

fn ok_registers(state: &mut State, input: &str) -> Vec<Register> {
    let id = state.add_file("ok_reg".to_string(), input.to_string());
    parse(state, id)
        .unwrap_or_else(|errors| {
            let rendered = crate::render_to_codespan(
                errors.into_iter().map(LatchError::from).collect(),
                state,
            );
            panic!("expected successful parse:\n{rendered}")
        })
        .iter()
        .map(|reg| state.get_reg(*reg))
        .cloned()
        .collect()
}

fn err_count(input: &str) -> usize {
    let mut state = State::default();
    let id = state.add_file("err_count".to_string(), input.to_string());
    match parse(&mut state, id) {
        Ok(_) => panic!("expected parse errors but got Ok"),
        Err(e) => e.len(),
    }
}

// tests

#[test]
fn empty_input_yields_no_registers() {
    let mut state = State::default();
    assert_eq!(ok_registers(&mut state, "").len(), 0);
}

#[test]
fn register_with_two_fields() {
    let src = "\
-- @reg offset=0x04 name=status\n\
-- @field bits=0 name=tx_en access=RW\n\
-- @field bits=1 name=rx_en access=RO\n";
    let mut state = State::default();
    let registers = ok_registers(&mut state, src);
    let field_ids = registers[0].get_fields().clone();
    let field_0 = state.get_field(field_ids[0]).clone();
    let field_1 = state.get_field(field_ids[1]).clone();
    assert_eq!(registers.len(), 1);
    assert_eq!(registers[0].get_fields().len(), 2);
    assert_eq!(field_0.name, "tx_en");
    assert_eq!(field_0.bits, BitSpec::Single(0));
    assert_eq!(field_0.access, Access::RW);
    assert_eq!(field_1.name, "rx_en");
    assert_eq!(field_1.bits, BitSpec::Single(1));
    assert_eq!(field_1.access, Access::RO);
}

#[test]
fn two_consecutive_registers() {
    let src = "\
-- @reg offset=0x00 name=ctrl\n\
-- @field bits=0:6 name=rx_in access=RO\n\
-- @reg offset=0x04 name=status\n\
-- @field bits=0:6 name=tx_out access=WO\n\
-- @field bits=7 name=rx_in access=RW\n";
    let mut state = State::default();
    let registers = ok_registers(&mut state, src);
    let reg0_fields: Vec<Field> = registers[0]
        .get_fields()
        .iter()
        .map(|&id| state.get_field(id).clone())
        .collect();
    let reg1_fields: Vec<Field> = registers[1]
        .get_fields()
        .iter()
        .map(|&id| state.get_field(id).clone())
        .collect();
    assert_eq!(registers.len(), 2);
    assert_eq!(registers[0].get_fields().len(), 1);
    assert_eq!(registers[1].get_fields().len(), 2);
    assert_eq!(reg0_fields[0].name, "rx_in");
    assert_eq!(reg0_fields[0].bits, BitSpec::Span(0, 6));
    assert_eq!(reg0_fields[0].access, Access::RO);
    assert_eq!(reg1_fields[0].name, "tx_out");
    assert_eq!(reg1_fields[0].bits, BitSpec::Span(0, 6));
    assert_eq!(reg1_fields[0].access, Access::WO);
    assert_eq!(reg1_fields[1].name, "rx_in");
    assert_eq!(reg1_fields[1].bits, BitSpec::Single(7));
    assert_eq!(reg1_fields[1].access, Access::RW);
}

#[test]
fn single_register_exists_in_source_map() {
    let src = "\
-- @reg offset=0x04 name=status\n\
-- @field bits=0 name=tx_en access=RW\n\
-- @field bits=1 name=rx_en access=RO\n";
    let mut state = State::default();
    let id = state.add_file("single_reg_exists".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let field_ids = state.get_reg(reg_ids[0]).get_fields().clone();
    assert_eq!(reg_ids.len(), 1);
    assert_eq!(field_ids.len(), 2);
    assert_eq!(state.get_reg_loc(reg_ids[0]).map(|l| l.line), Some(0));
    assert_eq!(state.get_field_loc(field_ids[0]).map(|l| l.line), Some(1));
    assert_eq!(state.get_field_loc(field_ids[1]).map(|l| l.line), Some(2));
}

#[test]
fn multiple_registers_exist_in_source_map() {
    let src = "\
-- @reg offset=0x00 name=ctrl\n\
-- @field bits=0:6 name=rx_in access=RO\n\
-- @reg offset=0x04 name=status\n\
-- @field bits=0:6 name=tx_out access=WO\n\
-- @field bits=7 name=rx_in access=RW\n";
    let mut state = State::default();
    let id = state.add_file("multiple_registers_source_map".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg1_field_ids = state.get_reg(reg_ids[1]).get_fields().clone();
    assert_eq!(reg_ids.len(), 2);
    assert_eq!(state.get_reg_loc(reg_ids[0]).map(|l| l.line), Some(0));
    assert_eq!(state.get_reg_loc(reg_ids[1]).map(|l| l.line), Some(2));
    assert_eq!(
        state.get_field_loc(reg1_field_ids[0]).map(|l| l.line),
        Some(3)
    );
    assert_eq!(
        state.get_field_loc(reg1_field_ids[1]).map(|l| l.line),
        Some(4)
    );
}

#[test]
fn bit_spec_fields_different_number_formats() {
    let src = "\
-- @reg offset=0x04 name=status\n\
-- @field bits=0x00 name=tx_en access=RW\n\
-- @field bits=0x01:0x03 name=tx_en access=RW\n\
-- @field bits=0b100 name=tx_en access=RW\n\
-- @field bits=0b101:8 name=tx_en access=RW\n\
-- @field bits=0x09:0b1100 name=rx_en access=RO\n\
-- @field bits=13:0x10 name=tx_en access=RW\n";
    let mut state = State::default();
    let registers = ok_registers(&mut state, src);
    let field_ids = registers[0].get_fields();
    let field_0 = state.get_field(field_ids[0]);
    let field_1 = state.get_field(field_ids[1]);
    let field_2 = state.get_field(field_ids[2]);
    let field_3 = state.get_field(field_ids[3]);
    let field_4 = state.get_field(field_ids[4]);
    let field_5 = state.get_field(field_ids[5]);

    assert_eq!(registers.len(), 1);
    assert_eq!(registers[0].get_fields().len(), 6);
    assert_eq!(field_0.bits, BitSpec::Single(0));
    assert_eq!(field_1.bits, BitSpec::Span(1, 3));
    assert_eq!(field_2.bits, BitSpec::Single(4));
    assert_eq!(field_3.bits, BitSpec::Span(5, 8));
    assert_eq!(field_4.bits, BitSpec::Span(9, 12));
    assert_eq!(field_5.bits, BitSpec::Span(13, 16));
}

// error cases

#[test]
fn single_register_no_fields() {
    let src = "-- @reg offset=0x00 name=ctrl\n";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    assert!(parse(&mut state, id).is_err());
}

#[test]
fn field_without_preceding_register_is_error() {
    let src = "-- @field bits=0 name=en access=RW\n";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    assert!(parse(&mut state, id).is_err());
}

#[test]
fn register_missing_offset_is_error() {
    let src = "-- @reg name=ctrl\n";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    assert!(parse(&mut state, id).is_err());
}

#[test]
fn register_missing_name_is_error() {
    let src = "-- @reg offset=0x00\n";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    assert!(parse(&mut state, id).is_err());
}

#[test]
fn field_missing_name_is_error() {
    let src = "\
-- @reg offset=0x00 name=ctrl\n\
-- @field bits=0 access=RW\n";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    assert!(parse(&mut state, id).is_err());
}

#[test]
fn field_missing_bits_is_error() {
    let src = "\
-- @reg offset=0x00 name=ctrl\n\
-- @field name=en access=RW\n";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    assert!(parse(&mut state, id).is_err());
}

#[test]
fn non_adjacent_field_is_error() {
    let src = "\
-- @reg offset=0x00 name=ctrl\n\
\n\
-- @field bits=0 name=en\n";
    assert_eq!(err_count(src), 2);
}

#[test]
fn non_annotation_lines_are_ignored() {
    let src = "\
signal s_foo : std_logic;\n\
-- @reg offset=0x00 name=ctrl\n\
-- some unrelated comment\n\
-- @field bits=0 name=en access=RW\n";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    assert!(parse(&mut state, id).is_err());
}
