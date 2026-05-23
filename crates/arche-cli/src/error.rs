use std::process;

use arche_types::common::ProblemJson;

#[derive(Debug)]
pub enum CliError {
    Args(String),
    Api(ProblemJson),
    Generic(String),
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Args(_) => 2,
            CliError::Api(_) => 3,
            CliError::Generic(_) => 1,
        }
    }

    pub fn display_message(&self) -> String {
        match self {
            CliError::Args(msg) => format!("error: {msg}"),
            CliError::Api(problem) => problem
                .detail
                .clone()
                .unwrap_or_else(|| problem.title.clone()),
            CliError::Generic(msg) => format!("error: {msg}"),
        }
    }
}

pub fn exit_with_error(error: CliError) -> ! {
    eprintln!("{}", error.display_message());
    process::exit(error.exit_code())
}