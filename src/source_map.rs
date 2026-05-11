use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct FileId(pub u32);

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SourceMap {
    /// (file, register_index) -> source line
    pub registers: HashMap<(FileId, usize), usize>,
    /// (file, register_index, field_index) -> source line
    pub fields: HashMap<(FileId, usize, usize), usize>,
}

impl SourceMap {
    pub fn insert_register(&mut self, file: FileId, reg_idx: usize, line: usize) {
        self.registers.insert((file, reg_idx), line);
    }

    pub fn insert_field(&mut self, file: FileId, reg_idx: usize, field_idx: usize, line: usize) {
        self.fields.insert((file, reg_idx, field_idx), line);
    }

    pub fn register_line(&self, file: FileId, reg_idx: usize) -> Option<usize> {
        self.registers.get(&(file, reg_idx)).copied()
    }

    pub fn field_line(&self, file: FileId, reg_idx: usize, field_idx: usize) -> Option<usize> {
        self.fields.get(&(file, reg_idx, field_idx)).copied()
    }

    pub fn extend(&mut self, other: Self) {
        self.registers.extend(other.registers);
        self.fields.extend(other.fields);
    }
}
