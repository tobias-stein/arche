pub mod cli;
#[allow(dead_code)]
pub mod client;
pub mod config;
pub mod error;
pub mod output;

use arche_types::common::ProblemJson;
use arche_types::crud::BootstrapResponse;
use arche_types::generate::{AffixConstraints, ConstraintValue, GenerateRequest, GenerateResponse};
use clap::Parser;
use cli::{Cli, ClientCommand, Command, KeyCommand};
use config::Config;
use error::CliError;
use output::{print_verbose, OutputFormat};
use std::collections::HashMap;
use std::process::ExitCode;
use std::str::FromStr;

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

fn cmd_generate(args: &cli::GenerateArgs, config: &Config) -> Result<(), CliError> {
    let format = OutputFormat::from_str(&args.format).map_err(CliError::Args)?;

    let req = build_generate_request(args)?;

    let url = format!("{}/api/generate", config.api_url);
    print_verbose(&format!("Request: POST {url}"), config.verbose);
    if config.verbose {
        if let Ok(body) = serde_json::to_string(&req) {
            print_verbose(&format!("Request body: {body}"), config.verbose);
        }
    }

    let client = client::build_client(config);

    let rt = tokio::runtime::Runtime::new().unwrap();
    let response = rt
        .block_on(client.post(&url).json(&req).send())
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

    let generate_response: GenerateResponse = rt
        .block_on(response.json())
        .map_err(|e| CliError::Generic(format!("Failed to parse generate response: {e}")))?;

    let output_format = if config.quiet {
        OutputFormat::Quiet
    } else {
        format
    };

    let mut writer = output::OutputWriter::new(output_format, config.verbose);
    writer
        .write_generate(&generate_response)
        .map_err(|e| CliError::Generic(format!("Failed to write output: {e}")))?;

    let stdout_content = writer.stdout().contents();
    if !stdout_content.is_empty() {
        print!("{stdout_content}");
    }
    let stderr_content = writer.stderr().contents();
    if !stderr_content.is_empty() {
        eprint!("{stderr_content}");
    }

    Ok(())
}

fn build_generate_request(args: &cli::GenerateArgs) -> Result<GenerateRequest, CliError> {
    let constraints = args.constraints.as_deref().map(|json_str| {
        serde_json::from_str::<HashMap<String, ConstraintValue>>(json_str)
            .map_err(|e| CliError::Args(format!("Invalid constraints JSON: {e}")))
    }).transpose()?;

    let affixes = args.affixes.as_deref().map(|json_str| {
        serde_json::from_str::<AffixConstraints>(json_str)
            .map_err(|e| CliError::Args(format!("Invalid affixes JSON: {e}")))
    }).transpose()?;

    Ok(GenerateRequest {
        archetype: args.archetype.clone(),
        seed: args.seed,
        constraints,
        affixes,
    })
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

#[cfg(test)]
mod generate_tests {
    use super::*;
    use arche_types::generate::{AffixAttributeEntry, NameParts};
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use std::collections::BTreeMap;

    fn make_config(api_url: &str, quiet: bool, verbose: bool) -> Config {
        Config {
            api_url: api_url.to_string(),
            api_key: None,
            verbose,
            quiet,
        }
    }

    fn make_args(
        archetype: Option<&str>,
        constraints: Option<&str>,
        seed: Option<u64>,
        affixes: Option<&str>,
        format: &str,
    ) -> cli::GenerateArgs {
        cli::GenerateArgs {
            archetype: archetype.map(|s| s.to_string()),
            constraints: constraints.map(|s| s.to_string()),
            seed,
            affixes: affixes.map(|s| s.to_string()),
            format: format.to_string(),
        }
    }

    fn sample_response() -> GenerateResponse {
        GenerateResponse {
            seed: 12345,
            name: "Fire Longsword of the Bear".into(),
            name_parts: NameParts {
                base: "Longsword".into(),
                prefixes: vec!["Fire".into()],
                suffixes: vec!["of the Bear".into()],
            },
            blueprint_id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            blueprint_attributes: {
                let mut m = BTreeMap::new();
                m.insert("damage".into(), serde_json::json!(27.3));
                m.insert("weight".into(), serde_json::json!(3.5));
                m
            },
            affix_attributes: vec![AffixAttributeEntry {
                affix_id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                affix_name: "Fire".into(),
                attributes: serde_json::json!({"fireDamage": 12.7}),
            }],
        }
    }

    async fn run_generate(args: cli::GenerateArgs, config: Config) -> Result<(), CliError> {
        tokio::task::spawn_blocking(move || cmd_generate(&args, &config))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_generate_no_flags_returns_success() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::to_value(sample_response()).unwrap()))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_args(None, None, None, None, "json");
        let result = run_generate(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_with_archetype_filter() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::to_value(sample_response()).unwrap()))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_args(Some("sword"), None, None, None, "json");
        let result = run_generate(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_with_constraints() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::to_value(sample_response()).unwrap()))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_args(None, Some(r#"{"damage":{"gte":15}}"#), None, None, "json");
        let result = run_generate(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_with_seed() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::to_value(sample_response()).unwrap()))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_args(None, None, Some(12345), None, "json");
        let result = run_generate(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_with_affixes() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::to_value(sample_response()).unwrap()))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_args(None, None, None, Some(r#"{"minPrefixes":1,"maxPrefixes":2}"#), "json");
        let result = run_generate(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_format_json_outputs_json() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::to_value(sample_response()).unwrap()))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_args(None, None, None, None, "json");
        let result = run_generate(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_format_pretty_outputs_colored() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::to_value(sample_response()).unwrap()))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_args(None, None, None, None, "pretty");
        let result = run_generate(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_404_returns_api_error_exit_code_3() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(
                ResponseTemplate::new(404).set_body_json(serde_json::json!({
                    "type": "/errors/no-matching-blueprints",
                    "title": "No Blueprints Found",
                    "status": 404,
                    "detail": "No blueprint satisfies archetype 'nonexistent'"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_args(Some("nonexistent"), None, None, None, "json");
        let result = run_generate(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 404);
                assert_eq!(CliError::Api(problem.clone()).exit_code(), 3);
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_generate_invalid_constraints_json_returns_args_error() {
        let config = make_config("http://localhost:1", false, false);
        let args = make_args(None, Some("not json"), None, None, "json");
        let result = run_generate(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(msg)) => {
                assert!(msg.contains("Invalid constraints JSON"));
                assert_eq!(CliError::Args(msg.clone()).exit_code(), 2);
            }
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_generate_invalid_affixes_json_returns_args_error() {
        let config = make_config("http://localhost:1", false, false);
        let args = make_args(None, None, None, Some("{bad json"), "json");
        let result = run_generate(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(msg)) => {
                assert!(msg.contains("Invalid affixes JSON"));
                assert_eq!(CliError::Args(msg.clone()).exit_code(), 2);
            }
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_generate_invalid_format_returns_args_error() {
        let config = make_config("http://localhost:1", false, false);
        let args = make_args(None, None, None, None, "invalid");
        let result = run_generate(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(_)) => {}
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_generate_all_flags_together() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::to_value(sample_response()).unwrap()))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_args(
            Some("sword"),
            Some(r#"{"damage":{"gte":15}}"#),
            Some(42),
            Some(r#"{"minPrefixes":1}"#),
            "pretty",
        );
        let result = run_generate(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_verbose_shows_request_details() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::to_value(sample_response()).unwrap()))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, true);
        let args = make_args(None, None, None, None, "json");
        let result = run_generate(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_generate_quiet_mode_suppresses_output() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/generate"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::to_value(sample_response()).unwrap()))
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, true, false);
        let args = make_args(None, None, None, None, "json");
        let result = run_generate(args, config).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_connection_refused_returns_generic_error() {
        let config = make_config("http://127.0.0.1:1", false, false);
        let args = make_args(None, None, None, None, "json");
        let result = cmd_generate(&args, &config);
        assert!(result.is_err());
        match &result {
            Err(CliError::Generic(msg)) => {
                assert!(msg.contains("Failed to connect"), "expected connect error, got: {msg}");
            }
            _ => panic!("expected Generic error, got {:?}", result),
        }
    }

    #[test]
    fn test_build_generate_request_empty() {
        let args = make_args(None, None, None, None, "json");
        let req = build_generate_request(&args).unwrap();
        assert!(req.archetype.is_none());
        assert!(req.seed.is_none());
        assert!(req.constraints.is_none());
        assert!(req.affixes.is_none());
    }

    #[test]
    fn test_build_generate_request_all_fields() {
        let args = make_args(
            Some("sword"),
            Some(r#"{"damage":{"gte":15}}"#),
            Some(12345),
            Some(r#"{"minPrefixes":1,"maxPrefixes":2}"#),
            "json",
        );
        let req = build_generate_request(&args).unwrap();
        assert_eq!(req.archetype.as_deref(), Some("sword"));
        assert_eq!(req.seed, Some(12345));
        assert!(req.constraints.is_some());
        assert!(req.affixes.is_some());
    }
}
