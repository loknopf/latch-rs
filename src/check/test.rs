use crate::{
    check::check_field_overlap,
    parser::parse,
    state::{FileId, State},
};

#[test]
fn test_field_bits_overlap() {
    let src = "\
        -- @reg name=tx_0 offset=0x03\n\
        -- @field name=bx_3 bits=0:6\n\
        -- @field name=bx_6 bits=3:9";
    let mut state = State::default();
    let reg_ids = parse(&mut state, src, FileId(0)).unwrap();
    let reg_0 = state.get_reg(reg_ids[0]);
    let error = check_field_overlap(&state, reg_0);
    assert_eq!(error.is_err(), true);
}
