use std::path::PathBuf;

use crate::{
    check::{check_field_name_collision, check_field_overlap, check_reg_name_collisions},
    error::LatchError,
    state::State,
};

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
    let parse_result = parser::parse(&mut state, &content, file);
    if let Err(errors) = parse_result {
        return render_to_codespan(errors, &state);
    }
    let regs = parse_result.unwrap();
    let name_collisions = check_reg_name_collisions(&state, &regs);
    if let Err(errors) = name_collisions {
        return render_to_codespan(
            errors.into_iter().map(|f| LatchError::from(f)).collect(),
            &state,
        );
    }
    for reg in regs.into_iter().map(|id| state.get_reg(id)) {
        let overlap = check_field_overlap(&state, reg);
        if let Err(errors) = overlap {
            return render_to_codespan(
                errors.into_iter().map(|f| LatchError::from(f)).collect(),
                &state,
            );
        }
        let field_name_collision = check_field_name_collision(&state, reg.get_fields());
        if let Err(errors) = field_name_collision {
            return render_to_codespan(
                errors.into_iter().map(|f| LatchError::from(f)).collect(),
                &state,
            );
        }
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
