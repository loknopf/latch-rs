use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::Files,
};

use crate::{
    check::CheckError,
    error::LatchError,
    parser::{LoweringError, ParseError},
    state::State,
    toml::TomlError,
};

pub(crate) fn to_diagnostic(error: LatchError, state: &State) -> Diagnostic<u32> {
    match error {
        LatchError::Parse(e) => parse_diagnostic(e, state),
        LatchError::Lowering(e) => lowering_diagnostic(e, state),
        LatchError::Check(e) => check_diagnostic(e, state),
        LatchError::Toml(e) => toml_diagnostic(e, state),
        _ => todo!(),
    }
}

fn parse_diagnostic(error: ParseError, state: &State) -> Diagnostic<u32> {
    let file_id = error.file.0;
    let line_range = state
        .get_files()
        .line_range(file_id as usize, error.line)
        .unwrap();
    let abs_start = line_range.start + error.offset.start;
    let abs_end = line_range.start + error.offset.end;
    Diagnostic::error()
        .with_message(&error.message)
        .with_label(Label::primary(file_id, abs_start..abs_end))
}

fn lowering_diagnostic(error: LoweringError, state: &State) -> Diagnostic<u32> {
    let file_id = error.file.0;
    let line_range = state
        .get_files()
        .line_range(file_id as usize, error.line)
        .unwrap();
    Diagnostic::error()
        .with_message(&error.message)
        .with_label(Label::primary(file_id, line_range).with_message("lowering error occured here"))
}

fn check_diagnostic(error: CheckError, state: &State) -> Diagnostic<u32> {
    match error {
        CheckError::FieldNameCollision { first, other } => {
            let first_loc = state
                .get_field_loc(first)
                .expect("Expecting a field to have a location");
            let other_loc = state
                .get_field_loc(other)
                .expect("Expecting a field to have a location");
            let message = "Two field names are colliding within the same @reg block!";
            let first_span = first_loc.to_line_range(state);
            let other_span = other_loc.to_line_range(state);
            Diagnostic::error()
                .with_message(message)
                .with_label(Label::primary(other_loc.file.0, other_span).with_message(
                    "this fields name collides with another field in the same register",
                ))
                .with_label(
                    Label::secondary(first_loc.file.0, first_span)
                        .with_message("first defined here"),
                )
        }
        CheckError::FieldOverlap { first, other } => {
            let first_loc = state
                .get_field_loc(first)
                .expect("Expecting a field to have a location");
            let other_loc = state
                .get_field_loc(other)
                .expect("Expecting a field to have a location");
            let first_span = first_loc.to_line_range(state);
            let other_span = other_loc.to_line_range(state);
            Diagnostic::error()
                .with_message("Two fields have overlapping bit ranges!")
                .with_label(
                    Label::primary(first_loc.file.0, first_span)
                        .with_message("overlap encountered here"),
                )
                .with_label(
                    Label::secondary(other_loc.file.0, other_span)
                        .with_message("overlap encountered here"),
                )
        }
        CheckError::RegNameCollision { first, other } => {
            let first_loc = state
                .get_reg_loc(first)
                .expect("Expecting a register to have a location.");
            let other_loc = state
                .get_reg_loc(other)
                .expect("Expecting a register to have a location.");
            let first_span = first_loc.to_line_range(state);
            let other_span = other_loc.to_line_range(state);
            Diagnostic::error()
                .with_message("Two registers have the same name!")
                .with_label(
                    Label::primary(other_loc.file.0, other_span)
                        .with_message("Name collision encountered here"),
                )
                .with_label(
                    Label::secondary(first_loc.file.0, first_span)
                        .with_message("first declared here"),
                )
        }
    }
}

fn toml_diagnostic(error: TomlError, _state: &State) -> Diagnostic<u32> {
    let mut d = Diagnostic::error().with_message(&error.message);
    if let (Some(file), Some(span)) = (error.file, error.span) {
        d = d.with_label(Label::primary(file.0, span).with_message("TOML error occoured here"));
    }
    d
}
