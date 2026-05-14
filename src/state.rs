use slotmap::{SecondaryMap, SlotMap, new_key_type};

use crate::ir::{Field, Register};

new_key_type! { pub(crate) struct RegId; }
new_key_type! { pub(crate) struct FieldId; }

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub(crate) struct FileId(pub u32);

#[derive(Debug)]
pub(crate) struct Location {
    pub(crate) line: usize,
    pub(crate) file: FileId,
}

#[derive(Debug, Default)]
pub(crate) struct State {
    registers: SlotMap<RegId, Register>,
    fields: SlotMap<FieldId, Field>,
    reg_locations: SecondaryMap<RegId, Location>,
    field_locations: SecondaryMap<FieldId, Location>,
}

impl State {
    pub(crate) fn insert_reg(&mut self, reg: Register) -> RegId {
        self.registers.insert(reg)
    }

    pub(crate) fn insert_field(&mut self, field: Field) -> FieldId {
        self.fields.insert(field)
    }

    pub(crate) fn add_reg_loc(&mut self, reg_id: RegId, loc: Location) {
        self.reg_locations.insert(reg_id, loc);
    }

    pub(crate) fn add_field_loc(&mut self, field_id: FieldId, loc: Location) {
        self.field_locations.insert(field_id, loc);
    }

    pub(crate) fn get_reg(&self, reg_id: RegId) -> &Register {
        self.registers
            .get(reg_id)
            .expect("Expecting a RegId to be valid")
    }

    pub(crate) fn get_regs(&self) -> Vec<&Register> {
        self.registers.iter().map(|(_, v)| v).collect()
    }

    pub(crate) fn get_field(&self, field_id: FieldId) -> &Field {
        self.fields
            .get(field_id)
            .expect("Expecting a FieldId to be valid.")
    }

    pub(crate) fn get_fields(&self) -> Vec<&Field> {
        self.fields.iter().map(|(_, v)| v).collect()
    }

    pub(crate) fn get_reg_loc(&self, reg_id: RegId) -> Option<&Location> {
        self.reg_locations.get(reg_id)
    }

    pub(crate) fn get_field_loc(&self, field_id: FieldId) -> Option<&Location> {
        self.field_locations.get(field_id)
    }
}
