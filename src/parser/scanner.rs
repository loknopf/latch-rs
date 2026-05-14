pub(crate) enum AnnotationKind {
    Reg,
    Field,
}

pub(crate) struct RawAnnotation<'a> {
    pub(crate) line: usize,
    pub(crate) kind: AnnotationKind,
    pub(crate) content: &'a str,
}

impl<'a> RawAnnotation<'a> {
    pub(crate) fn pre_offset(&self) -> usize {
        match self.kind {
            AnnotationKind::Reg => REG_LINE.len(),
            AnnotationKind::Field => FIELD_LINE.len(),
        }
    }
}

const REG_LINE: &'static str = "-- @reg";
const FIELD_LINE: &'static str = "-- @field";

pub(crate) fn scan<'a>(input: &'a str) -> Vec<RawAnnotation<'a>> {
    let mut raw_annotations = Vec::new();
    let lines = input.lines();
    for (idx, line) in lines.enumerate() {
        if line.starts_with(REG_LINE) {
            raw_annotations.push(RawAnnotation {
                line: idx,
                kind: AnnotationKind::Reg,
                content: line.strip_prefix(REG_LINE).expect(
                    "This line was found to start with this prefix, thus it can always be stripped",
                ),
            });
        } else if line.starts_with(FIELD_LINE) {
            raw_annotations.push(RawAnnotation {
                line: idx,
                kind: AnnotationKind::Field,
                content: line.strip_prefix(FIELD_LINE).expect(
                    "This line was found to start with this prefix, thus it can always be stripped",
                ),
            });
        }
    }
    raw_annotations
}
