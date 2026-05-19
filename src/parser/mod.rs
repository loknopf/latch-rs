use std::usize;

use pest_derive::Parser;

pub(crate) use crate::parser::{
    error::{LoweringError, ParseError},
    kv::{KvPair, parse_kv_pairs},
    scanner::{AnnotationKind, scan},
};
use crate::{
    error::LatchError,
    ir::Register,
    parser::lower::LowerCtx,
    state::{FileId, RegId, State},
};

mod error;
mod kv;
mod lower;
mod scanner;
#[cfg(test)]
mod tests;

#[derive(Parser)]
#[grammar = "annotation.pest"]
struct LatchParser;

enum LowerState {
    Empty,
    Active {
        reg: Register,
        line: usize,
        failed_fields: usize,
    },
    Failed,
}

struct LoweredAnnotation {
    line: usize,
    kind: AnnotationKind,
}

pub(crate) fn parse(state: &mut State, file_id: FileId) -> Result<Vec<RegId>, Vec<LatchError>> {
    //This block scopes the lifetime of the immutable borrow of the &str inside the `RawAnnotation`
    let parsed: Vec<_> = {
        let input = state
            .get_file(file_id)
            .expect("Expecting a file to exist for a given FileId")
            .source();
        //Scan the input and convert into RawAnnotations; convert them int their atomics to avoid borrow issue
        scan(input)
            .into_iter()
            .map(|ann| {
                let pre_offset = ann.pre_offset();
                let result = parse_kv_pairs(ann.content);
                (ann.kind, ann.line, pre_offset, result)
            })
            .collect()
    };

    let mut ctx = LowerCtx::new(file_id);
    for (kind, line, pre_offset, result) in parsed {
        match result {
            Err(e) => ctx.on_parse_error(kind, line, pre_offset, e),
            Ok(kv) => match kind {
                AnnotationKind::Reg => ctx.on_reg(&kv, line, state),
                AnnotationKind::Field => ctx.on_field(&kv, line, state),
            },
        }
    }
    ctx.finish(state)
}
