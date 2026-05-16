use crate::{
    check::{
        check_field_name_collision, check_field_overlap, check_reg_name_collisions,
        error::CheckError,
    },
    parser::parse,
    state::State,
};

// ── field overlap ────────────────────────────────────────────────────────────

#[test]
fn test_field_bits_overlap() {
    let src = "\
-- @reg name=tx_0 offset=0x03\n\
-- @field name=bx_3 bits=0:6\n\
-- @field name=bx_6 bits=3:9";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg_0 = state.get_reg(reg_ids[0]);
    let errors = check_field_overlap(&state, reg_0).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(matches!(errors[0], CheckError::FieldOverlap { .. }));
}

#[test]
fn test_field_no_overlap() {
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=lo bits=0:2\n\
-- @field name=hi bits=4:7";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg = state.get_reg(reg_ids[0]);
    assert!(check_field_overlap(&state, reg).is_ok());
}

#[test]
fn test_field_boundary_touch_is_overlap() {
    // Span(0,3) and Span(3,6) share bit 3
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=f0 bits=0:3\n\
-- @field name=f1 bits=3:6";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg = state.get_reg(reg_ids[0]);
    let errors = check_field_overlap(&state, reg).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(matches!(errors[0], CheckError::FieldOverlap { .. }));
}

#[test]
fn test_field_single_bit_inside_span() {
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=f0 bits=5\n\
-- @field name=f1 bits=3:7";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg = state.get_reg(reg_ids[0]);
    let errors = check_field_overlap(&state, reg).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(matches!(errors[0], CheckError::FieldOverlap { .. }));
}

#[test]
fn test_field_single_bit_outside_span() {
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=f0 bits=9\n\
-- @field name=f1 bits=3:7";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg = state.get_reg(reg_ids[0]);
    assert!(check_field_overlap(&state, reg).is_ok());
}

#[test]
fn test_field_multiple_overlap_pairs() {
    // f0=0:5, f1=3:8, f2=6:10 → (f0,f1) and (f1,f2) both overlap; (f0,f2) does not
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=f0 bits=0:5\n\
-- @field name=f1 bits=3:8\n\
-- @field name=f2 bits=6:10";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg = state.get_reg(reg_ids[0]);
    let errors = check_field_overlap(&state, reg).unwrap_err();
    assert_eq!(errors.len(), 2);
    assert!(
        errors
            .iter()
            .all(|e| matches!(e, CheckError::FieldOverlap { .. }))
    );
}

#[test]
fn test_field_single_field_no_overlap() {
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=f0 bits=0:7";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg = state.get_reg(reg_ids[0]);
    assert!(check_field_overlap(&state, reg).is_ok());
}

// ── field name collisions ────────────────────────────────────────────────────

#[test]
fn test_field_name_collision() {
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=dup bits=0:3\n\
-- @field name=dup bits=5:7";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg = state.get_reg(reg_ids[0]);
    let errors = check_field_name_collision(&state, reg).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(matches!(errors[0], CheckError::FieldNameCollision { .. }));
}

#[test]
fn test_field_name_no_collision() {
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=ctrl bits=0:3\n\
-- @field name=status bits=4:7";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg = state.get_reg(reg_ids[0]);
    assert!(check_field_name_collision(&state, reg).is_ok());
}

#[test]
fn test_field_name_single_field() {
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=only bits=0:7";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg = state.get_reg(reg_ids[0]);
    assert!(check_field_name_collision(&state, reg).is_ok());
}

#[test]
fn test_field_name_three_way_collision() {
    // three fields all named "dup" → 3 collision pairs: (0,1), (0,2), (1,2)
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=dup bits=0:2\n\
-- @field name=dup bits=3:5\n\
-- @field name=dup bits=6:8";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg = state.get_reg(reg_ids[0]);
    let errors = check_field_name_collision(&state, reg).unwrap_err();
    assert_eq!(errors.len(), 3);
    assert!(
        errors
            .iter()
            .all(|e| matches!(e, CheckError::FieldNameCollision { .. }))
    );
}

#[test]
fn test_field_name_partial_collision() {
    // f0, f1, f0 → only (f0_a, f0_b) collide
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=f0 bits=0:2\n\
-- @field name=f1 bits=3:5\n\
-- @field name=f0 bits=6:8";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg = state.get_reg(reg_ids[0]);
    let errors = check_field_name_collision(&state, reg).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(matches!(errors[0], CheckError::FieldNameCollision { .. }));
}

#[test]
fn test_field_name_collision_is_case_sensitive() {
    // "F0" and "f0" are distinct names — no collision
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=F0 bits=0:3\n\
-- @field name=f0 bits=4:7";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let reg = state.get_reg(reg_ids[0]);
    assert!(check_field_name_collision(&state, reg).is_ok());
}

// ── register name collisions ─────────────────────────────────────────────────

#[test]
fn test_reg_name_collision() {
    let src = "\
-- @reg name=tx_0 offset=0x03\n\
-- @field name=bx_3 bits=0:6\n\
signal my_signal: std_logic_vector(19);\n\
-- @reg name=tx_0 offset=0x03\n\
-- @field name=bx_3 bits=0:6";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    let errors = check_reg_name_collisions(&state, &reg_ids).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(matches!(errors[0], CheckError::RegNameCollision { .. }));
}

#[test]
fn test_reg_name_no_collision() {
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=f0 bits=0:3\n\
signal x: std_logic;\n\
-- @reg name=r1 offset=0x04\n\
-- @field name=f1 bits=0:3";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    assert!(check_reg_name_collisions(&state, &reg_ids).is_ok());
}

#[test]
fn test_reg_name_single_reg() {
    let src = "\
-- @reg name=only offset=0x00\n\
-- @field name=f0 bits=0:3";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    assert_eq!(reg_ids.len(), 1);
    assert!(check_reg_name_collisions(&state, &reg_ids).is_ok());
}

#[test]
fn test_reg_name_three_way_collision() {
    // three regs with same name → 3 collision pairs: (0,1), (0,2), (1,2)
    let src = "\
-- @reg name=dup offset=0x00\n\
-- @field name=f0 bits=0:3\n\
signal a: std_logic;\n\
-- @reg name=dup offset=0x04\n\
-- @field name=f1 bits=0:3\n\
signal b: std_logic;\n\
-- @reg name=dup offset=0x08\n\
-- @field name=f2 bits=0:3";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    assert_eq!(reg_ids.len(), 3);
    let errors = check_reg_name_collisions(&state, &reg_ids).unwrap_err();
    assert_eq!(errors.len(), 3);
    assert!(
        errors
            .iter()
            .all(|e| matches!(e, CheckError::RegNameCollision { .. }))
    );
}

#[test]
fn test_reg_name_partial_collision() {
    // r0, r1, r0 → only (r0_a, r0_b) collide
    let src = "\
-- @reg name=r0 offset=0x00\n\
-- @field name=f0 bits=0:3\n\
signal a: std_logic;\n\
-- @reg name=r1 offset=0x04\n\
-- @field name=f1 bits=0:3\n\
signal b: std_logic;\n\
-- @reg name=r0 offset=0x08\n\
-- @field name=f2 bits=0:3";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    assert_eq!(reg_ids.len(), 3);
    let errors = check_reg_name_collisions(&state, &reg_ids).unwrap_err();
    assert_eq!(errors.len(), 1);
    assert!(matches!(errors[0], CheckError::RegNameCollision { .. }));
}

#[test]
fn test_reg_name_collision_is_case_sensitive() {
    // "TX_0" and "tx_0" are distinct names — no collision
    let src = "\
-- @reg name=TX_0 offset=0x00\n\
-- @field name=f0 bits=0:3\n\
signal x: std_logic;\n\
-- @reg name=tx_0 offset=0x04\n\
-- @field name=f1 bits=0:3";
    let mut state = State::default();
    let id = state.add_file("test".to_string(), src.to_string());
    let reg_ids = parse(&mut state, id).unwrap();
    assert!(check_reg_name_collisions(&state, &reg_ids).is_ok());
}
