mod cli;
#[allow(dead_code)]
mod client;
mod config;
mod error;
mod output;

use clap::Parser;
use cli::{Cli, ClientCommand, Command, KeyCommand};
use config::Config;
use error::CliError;
use output::print_verbose;
use std::process::ExitCode;

fn main() -> ExitCode {
    let cli = Cli::parse();
    let config = Config::from_global_opts(&cli.global_opts);

    if let Err(err) = run(cli, &config) {
        output::print_error(&err.to_string());
        return err.exit_code();
    }

    ExitCode::SUCCESS
}

fn run(cli: Cli, config: &Config) -> Result<(), CliError> {
    print_verbose(&format!("API URL: {}", config.api_url), config.verbose);
    if config.api_key.is_some() {
        print_verbose("API key: configured", config.verbose);
    } else {
        print_verbose("API key: not set", config.verbose);
    }

    match &cli.command {
        Command::Init => cmd_init(config),
        Command::Generate(args) => cmd_generate(args, config),
        Command::Export(args) => cmd_export(args, config),
        Command::Import(args) => cmd_import(args, config),
        Command::Key(sub) => cmd_key(sub, config),
        Command::Client(sub) => cmd_client(sub, config),
    }
}

fn cmd_init(_config: &Config) -> Result<(), CliError> {
    Err(CliError::Generic("Not yet implemented".into()))
}

fn cmd_generate(_args: &cli::GenerateArgs, _config: &Config) -> Result<(), CliError> {
    Err(CliError::Generic("Not yet implemented".into()))
}

fn cmd_export(_args: &cli::ExportArgs, _config: &Config) -> Result<(), CliError> {
    Err(CliError::Generic("Not yet implemented".into()))
}

fn cmd_import(_args: &cli::ImportArgs, _config: &Config) -> Result<(), CliError> {
    Err(CliError::Generic("Not yet implemented".into()))
}

fn cmd_key(_sub: &KeyCommand, _config: &Config) -> Result<(), CliError> {
    Err(CliError::Generic("Not yet implemented".into()))
}

fn cmd_client(_sub: &ClientCommand, _config: &Config) -> Result<(), CliError> {
    Err(CliError::Generic("Not yet implemented".into()))
}
