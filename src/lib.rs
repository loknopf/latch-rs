use std::path::PathBuf;

use crate::{check::check_registers, error::LatchError, state::State};

mod check;
mod diagnostics;
mod error;
mod ir;
mod parser;
mod state;
mod toml;
mod types;

pub fn check(file: PathBuf) -> String {
    let name = file.display().to_string();
    let content_res = std::fs::read_to_string(file);
    if let Err(e) = content_res {
        return e.to_string();
    }
    let content = content_res.unwrap();
    let mut state = State::default();
    let file = state.add_file(name, content.clone());
    let parse_result = parser::parse(&mut state, file);
    if let Err(errors) = parse_result {
        return render_to_codespan(errors, &state);
    }
    let regs = parse_result.unwrap();
    let check_errors = check_registers(&regs, &state);
    if let Err(errs) = check_errors {
        return render_to_codespan(errs, &state);
    }
    "Check done - 0 errors found".to_string()
}

fn render_to_codespan(errors: Vec<LatchError>, state: &State) -> String {
    use codespan_reporting::term::{self, Config};

    let config = Config::default();
    let mut writer = String::new();

    for error in errors {
        let diagnostic = diagnostics::to_diagnostic(error, state);
        term::emit_to_string(&mut writer, &config, state.get_files(), &diagnostic).unwrap();
    }
    writer
}
