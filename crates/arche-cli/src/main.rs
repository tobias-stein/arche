pub mod cli;
#[allow(dead_code)]
pub mod client;
pub mod config;
pub mod error;
pub mod output;

use arche_types::common::ProblemJson;
use arche_types::crud::BootstrapResponse;
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
        output::print_error(&err.display_message());
        return ExitCode::from(err.exit_code() as u8);
    }

    ExitCode::SUCCESS
}

fn run(cli: Cli, config: &Config) -> Result<(), CliError> {
    print_verbose(&format!("API URL: {}", config.api_url), config.verbose);
    let key_status = if config.api_key.is_some() { "configured" } else { "not set" };
    print_verbose(&format!("API key: {key_status}"), config.verbose);

    match &cli.command {
        Command::Init => cmd_init(config),
        Command::Generate(args) => cmd_generate(args, config),
        Command::Export(args) => cmd_export(args, config),
        Command::Import(args) => cmd_import(args, config),
        Command::Key(sub) => cmd_key(sub, config),
        Command::Client(sub) => cmd_client(sub, config),
    }
}

fn cmd_init(config: &Config) -> Result<(), CliError> {
    let url = format!("{}/api/bootstrap", config.api_url);
    print_verbose(&format!("Request: GET {url}"), config.verbose);

    let client = client::build_client(config);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let response = rt
        .block_on(client.get(&url).send())
        .map_err(|e| CliError::Generic(format!("Failed to connect to API: {e}")))?;

    let status = response.status();
    print_verbose(
        &format!(
            "Response: {} {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        ),
        config.verbose,
    );

    if !status.is_success() {
        let problem: Result<ProblemJson, _> = rt.block_on(response.json());
        if let Ok(problem) = problem {
            return Err(CliError::Api(problem));
        }
        return Err(CliError::Api(ProblemJson {
            type_: format!("/errors/http-{}", status.as_u16()),
            title: status.canonical_reason().unwrap_or("HTTP Error").to_string(),
            status: status.as_u16(),
            detail: None,
        }));
    }

    let bootstrap: BootstrapResponse = rt
        .block_on(response.json())
        .map_err(|e| CliError::Generic(format!("Failed to parse bootstrap response: {e}")))?;

    if bootstrap.bootstrapped {
        if let Some(key) = &bootstrap.key {
            if config.quiet {
                println!("{key}");
            } else {
                println!();
                println!("╔══════════════════════════════════════════════════════════════╗");
                println!("║               === SUPER ADMIN API KEY ===                  ║");
                println!("║                                                            ║");
                println!("║  {:<58}║", key);
                println!("║                                                            ║");
                println!("║  Store this key securely. It will not be shown again.      ║");
                println!("╚══════════════════════════════════════════════════════════════╝");
                println!();
            }
        }
    } else if !config.quiet {
        let msg = bootstrap.message.as_deref().unwrap_or(
            "Already bootstrapped. Super admin key available in server logs.",
        );
        println!("{msg}");
    }

    Ok(())
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

#[cfg(test)]
mod init_tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn make_config(api_url: &str, quiet: bool, verbose: bool) -> Config {
        Config {
            api_url: api_url.to_string(),
            api_key: None,
            verbose,
            quiet,
        }
    }

    async fn run_init(config: Config) -> Result<(), CliError> {
        tokio::task::spawn_blocking(move || cmd_init(&config))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_init_returns_key_on_success() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/bootstrap"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "bootstrapped": true,
                "key": "arche_k_testkey123"
            })))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_init(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_already_bootstrapped() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/bootstrap"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "bootstrapped": false,
                "message": "Already bootstrapped. Key available in server logs."
            })))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_init(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_quiet_mode_outputs_only_key() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/bootstrap"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "bootstrapped": true,
                "key": "arche_k_testkey123"
            })))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, true, false);
        let result = run_init(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_verbose_shows_request_details() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/bootstrap"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "bootstrapped": true,
                "key": "arche_k_vkey"
            })))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, true);
        let result = run_init(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_init_server_error_returns_api_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/bootstrap"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_init(config).await;
        assert!(result.is_err());
        match result {
            Err(CliError::Api(_)) => {}
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_init_api_error_uses_problem_json_detail() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/bootstrap"))
            .respond_with(
                ResponseTemplate::new(400).set_body_json(serde_json::json!({
                    "type": "/errors/bad-request",
                    "title": "Bad Request",
                    "status": 400,
                    "detail": "something went wrong"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_init(config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 400);
                assert_eq!(problem.detail, Some("something went wrong".into()));
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[test]
    fn test_init_connection_refused_returns_generic_error() {
        let config = make_config("http://127.0.0.1:1", false, false);
        let result = cmd_init(&config);
        assert!(result.is_err());
        match &result {
            Err(CliError::Generic(msg)) => {
                assert!(msg.contains("Failed to connect"), "expected connect error, got: {msg}");
            }
            _ => panic!("expected Generic error, got {:?}", result),
        }
    }

    #[test]
    fn test_init_exit_codes_map_correctly() {
        assert_eq!(CliError::Generic("x".into()).exit_code(), 1);
        assert_eq!(
            CliError::Api(arche_types::common::ProblemJson {
                type_: "/x".into(),
                title: "x".into(),
                status: 500,
                detail: None,
            })
            .exit_code(),
            3
        );
        assert_eq!(CliError::Args("x".into()).exit_code(), 2);
    }
}
