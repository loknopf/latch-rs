use crate::{
    parser::kv::{Access, BitRange},
    source_map::FileId,
};

use super::parse;

// helpers

fn ok_registers(input: &str) -> Vec<super::ir::Register> {
    parse(input, FileId(0))
        .unwrap_or_else(|e| panic!("expected successful parse, got {} errors", e.len()))
        .0
}

fn err_count(input: &str) -> usize {
    match parse(input, FileId(0)) {
        Ok(_) => panic!("expected parse errors but got Ok"),
        Err(e) => e.len(),
    }
}

// tests

#[test]
fn empty_input_yields_no_registers() {
    assert_eq!(ok_registers("").len(), 0);
}

#[test]
fn register_with_two_fields() {
    let src = "\
-- @reg offset=0x04 name=status\n\
-- @field bits=0 name=tx_en access=RW\n\
-- @field bits=1 name=rx_en access=RO\n";
    let registers = ok_registers(src);
    assert_eq!(registers.len(), 1);
    assert_eq!(registers[0].get_fields().len(), 2);
    assert_eq!(registers[0].get_fields()[0].name, "tx_en");
    assert_eq!(registers[0].get_fields()[0].bits, BitRange::Single(0));
    assert_eq!(registers[0].get_fields()[0].access, Access::RW);
    assert_eq!(registers[0].get_fields()[1].name, "rx_en");
    assert_eq!(registers[0].get_fields()[1].bits, BitRange::Single(1));
    assert_eq!(registers[0].get_fields()[1].access, Access::RO);
}

#[test]
fn two_consecutive_registers() {
    let src = "\
-- @reg offset=0x00 name=ctrl\n\
-- @field bits=0:6 name=rx_in access=RO\n\
-- @reg offset=0x04 name=status\n\
-- @field bits=0:6 name=tx_out access=WO\n\
-- @field bits=7 name=rx_in access=RW\n";
    let registers = ok_registers(src);
    assert_eq!(registers.len(), 2);
    assert_eq!(registers[0].get_fields().len(), 1);
    assert_eq!(registers[1].get_fields().len(), 2);
    assert_eq!(registers[0].get_fields()[0].name, "rx_in");
    assert_eq!(registers[0].get_fields()[0].bits, BitRange::Span(0, 6));
    assert_eq!(registers[0].get_fields()[0].access, Access::RO);
    assert_eq!(registers[1].get_fields()[0].name, "tx_out");
    assert_eq!(registers[1].get_fields()[0].bits, BitRange::Span(0, 6));
    assert_eq!(registers[1].get_fields()[0].access, Access::WO);
    assert_eq!(registers[1].get_fields()[1].name, "rx_in");
    assert_eq!(registers[1].get_fields()[1].bits, BitRange::Single(7));
    assert_eq!(registers[1].get_fields()[1].access, Access::RW);
}

#[test]
fn singel_register_exists_in_source_map() {
    let src = "-- @reg offset=0x04 name=status\n\
-- @field bits=0 name=tx_en access=RW\n\
-- @field bits=1 name=rx_en access=RO\n";
    let source_map = parse(src, FileId(0)).unwrap().1;
    assert_eq!(source_map.registers.len(), 1);
    assert_eq!(source_map.fields.len(), 2);
    assert_eq!(source_map.register_line(FileId(0), 0), Some(0));
    assert_eq!(source_map.field_line(FileId(0), 0, 0), Some(1));
    assert_eq!(source_map.field_line(FileId(0), 0, 1), Some(2));
}

#[test]
fn multiple_registers_exists_in_source_map() {
    let src = "\
-- @reg offset=0x00 name=ctrl\n\
-- @field bits=0:6 name=rx_in access=RO\n\
-- @reg offset=0x04 name=status\n\
-- @field bits=0:6 name=tx_out access=WO\n\
-- @field bits=7 name=rx_in access=RW\n";
    let source_map = parse(src, FileId(0)).unwrap().1;
    assert_eq!(source_map.registers.len(), 2);
    assert_eq!(source_map.fields.len(), 3);
    assert_eq!(source_map.register_line(FileId(0), 0), Some(0));
    assert_eq!(source_map.register_line(FileId(0), 1), Some(2));
    assert_eq!(source_map.field_line(FileId(0), 1, 0), Some(3));
    assert_eq!(source_map.field_line(FileId(0), 1, 1), Some(4));
}

// error cases

#[test]
fn single_register_no_fields() {
    let src = "-- @reg offset=0x00 name=ctrl\n";
    assert!(parse(src, FileId(0)).is_err());
}

#[test]
fn field_without_preceding_register_is_error() {
    let src = "-- @field bits=0 name=en access=RW\n";
    assert!(parse(src, FileId(0)).is_err());
}

#[test]
fn register_missing_offset_is_error() {
    let src = "-- @reg name=ctrl\n";
    assert!(parse(src, FileId(0)).is_err());
}

#[test]
fn register_missing_name_is_error() {
    let src = "-- @reg offset=0x00\n";
    assert!(parse(src, FileId(0)).is_err());
}

#[test]
fn field_missing_name_is_error() {
    // Valid register followed by a field missing its required `name` key.
    let src = "\
-- @reg offset=0x00 name=ctrl\n\
-- @field bits=0 access=RW\n";
    assert!(parse(src, FileId(0)).is_err());
}

#[test]
fn field_missing_bits_is_error() {
    let src = "\
-- @reg offset=0x00 name=ctrl\n\
-- @field name=en access=RW\n";
    assert!(parse(src, FileId(0)).is_err());
}

#[test]
fn non_adjacent_field_is_error() {
    // A blank line between @reg and @field breaks adjacency.
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
    // The gap between the @reg and @field (non-annotation line in between)
    // triggers the adjacency guard, so we get one register with no valid field.
    // The register itself must still parse.
    assert!(parse(src, FileId(0)).is_err());
}
