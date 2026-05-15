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
