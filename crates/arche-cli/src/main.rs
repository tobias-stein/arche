pub mod cli;
#[allow(dead_code)]
pub mod client;
pub mod config;
pub mod error;
pub mod output;

use arche_types::common::{PaginatedResponse, ProblemJson};
use arche_types::crud::{BootstrapResponse, ClientResponse, CreateClientRequest};
use arche_types::export_import::{
    ConflictAttribute, ConflictDetail, ConflictResolutionRequest, ExportRequest,
    ImportConflictResponse, ImportSuccessResponse, ResolutionStrategy, ResourceResolution,
};
use arche_types::generate::{AffixConstraints, ConstraintValue, GenerateRequest, GenerateResponse};
use clap::Parser;
use cli::{Cli, ClientCommand, Command, KeyCommand};
use config::Config;
use error::CliError;
use output::{print_verbose, OutputFormat};
use reqwest::Response;
use std::collections::{HashMap, HashSet};
use std::io::{self, Write};
use std::process::ExitCode;
use std::str::FromStr;
use uuid::Uuid;

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
                println!("║  {key:<58}║");
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

    writer.flush_to_stdio();

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
        client_id: None,
    })
}

fn cmd_export(args: &cli::ExportArgs, config: &Config) -> Result<(), CliError> {
    let clients_str = args.clients.as_deref().unwrap_or("");
    let client_ids: Vec<Uuid> = if clients_str.trim().is_empty() {
        Vec::new()
    } else {
        clients_str
            .split(',')
            .map(|s| {
                let s = s.trim();
                Uuid::parse_str(s)
                    .map_err(|e| CliError::Args(format!("Invalid client UUID '{s}': {e}")))
            })
            .collect::<Result<Vec<_>, _>>()?
    };

    let req = ExportRequest {
        client_ids,
        include_api_keys: args.include_api_keys,
        include_audit_log: args.include_audit_log,
        inline_global_refs: args.inline_refs,
    };

    let url = format!("{}/api/export", config.api_url);
    print_verbose(&format!("Request: POST {url}"), config.verbose);
    if let Ok(body) = serde_json::to_string(&req) {
        print_verbose(&format!("Request body: {body}"), config.verbose);
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
        return Err(api_error_from_response(response, &rt));
    }

    let data = rt
        .block_on(response.bytes())
        .map_err(|e| CliError::Generic(format!("Failed to read response: {e}")))?;

    if let Some(path) = &args.output {
        output::write_binary(&data, Some(path))
            .map_err(|e| CliError::Generic(format!("Failed to write export file '{path}': {e}")))?;
        if !config.quiet {
            let size = data.len();
            println!("Export saved to {path} ({size} bytes)");
        }
    } else {
        output::write_binary(&data, None)
            .map_err(|e| CliError::Generic(format!("Failed to write to stdout: {e}")))?;
    }

    Ok(())
}

pub(crate) trait ImportPrompter {
    fn prompt_strategy(
        &mut self,
        detail: &ConflictDetail,
        index: usize,
        total: usize,
        apply_all: &mut Option<ResolutionStrategy>,
    ) -> io::Result<ResolutionStrategy>;

    fn prompt_attribute(
        &mut self,
        attr: &ConflictAttribute,
        apply_all_attr: &mut Option<ResolutionStrategy>,
    ) -> io::Result<ResolutionStrategy>;
}

pub(crate) struct DialoguerPrompter {
    theme: dialoguer::theme::ColorfulTheme,
}

impl DialoguerPrompter {
    fn new() -> Self {
        Self {
            theme: dialoguer::theme::ColorfulTheme::default(),
        }
    }
}

impl ImportPrompter for DialoguerPrompter {
    fn prompt_strategy(
        &mut self,
        detail: &ConflictDetail,
        index: usize,
        total: usize,
        apply_all: &mut Option<ResolutionStrategy>,
    ) -> io::Result<ResolutionStrategy> {
        let _ = writeln!(
            io::stderr(),
            "\n\u{2500}\u{2500}\u{2500} Conflict {} of {}: {} ({}) \u{2500}\u{2500}\u{2500}",
            index + 1,
            total,
            detail.resource_name,
            detail.resource_type
        );
        let _ = writeln!(io::stderr(), "  ID: {}", detail.resource_id);

        for attr in &detail.attributes {
            let _ = writeln!(
                io::stderr(),
                "  {}  old: {}",
                attr.key,
                format_attr_val(&attr.old_value)
            );
            let _ = writeln!(
                io::stderr(),
                "  {}  new: {}",
                " ".repeat(attr.key.len()),
                format_attr_val(&attr.new_value)
            );
        }
        let _ = writeln!(io::stderr());

        let mut items = vec![
            "Keep existing (keepOld)".to_string(),
            "Use imported version (keepNew)".to_string(),
            "Choose per-attribute (perAttribute)".to_string(),
        ];

        if *apply_all != Some(ResolutionStrategy::PerAttribute) {
            items.push("Apply keepOld to ALL remaining".to_string());
            items.push("Apply keepNew to ALL remaining".to_string());
        }

        let selection = dialoguer::Select::with_theme(&self.theme)
            .with_prompt("Select resolution strategy")
            .items(&items)
            .default(0)
            .interact()
            .map_err(io::Error::other)?;

        let result = match selection {
            0 => ResolutionStrategy::KeepOld,
            1 => ResolutionStrategy::KeepNew,
            2 => ResolutionStrategy::PerAttribute,
            3 => {
                *apply_all = Some(ResolutionStrategy::KeepOld);
                ResolutionStrategy::KeepOld
            }
            4 => {
                *apply_all = Some(ResolutionStrategy::KeepNew);
                ResolutionStrategy::KeepNew
            }
            _ => unreachable!(),
        };

        Ok(result)
    }

    fn prompt_attribute(
        &mut self,
        attr: &ConflictAttribute,
        apply_all_attr: &mut Option<ResolutionStrategy>,
    ) -> io::Result<ResolutionStrategy> {
        let _ = writeln!(
            io::stderr(),
            "\n  Attribute: {}  (old: {} \u{2192} new: {})",
            attr.key,
            format_attr_val(&attr.old_value),
            format_attr_val(&attr.new_value)
        );

        let mut items = vec![
            "Keep old value (keepOld)".to_string(),
            "Keep new value (keepNew)".to_string(),
        ];

        if *apply_all_attr != Some(ResolutionStrategy::PerAttribute) {
            items.push("Apply keepOld to ALL remaining attributes".to_string());
            items.push("Apply keepNew to ALL remaining attributes".to_string());
        }

        let selection = dialoguer::Select::with_theme(&self.theme)
            .with_prompt(format!("Resolve attribute '{}'", attr.key))
            .items(&items)
            .default(0)
            .interact()
            .map_err(io::Error::other)?;

        match selection {
            0 => Ok(ResolutionStrategy::KeepOld),
            1 => Ok(ResolutionStrategy::KeepNew),
            2 => {
                *apply_all_attr = Some(ResolutionStrategy::KeepOld);
                Ok(ResolutionStrategy::KeepOld)
            }
            3 => {
                *apply_all_attr = Some(ResolutionStrategy::KeepNew);
                Ok(ResolutionStrategy::KeepNew)
            }
            _ => unreachable!(),
        }
    }
}

fn format_attr_val(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

fn resolve_interactively(
    conflict_response: &ImportConflictResponse,
    prompter: &mut dyn ImportPrompter,
) -> io::Result<ConflictResolutionRequest> {
    let _ = writeln!(
        io::stderr(),
        "\n{} conflict(s) detected. Interactive resolution required.\n",
        conflict_response.conflicts.len()
    );

    let mut resolutions: HashMap<Uuid, ResourceResolution> = HashMap::new();
    let mut apply_all: Option<ResolutionStrategy> = None;
    let mut apply_all_attr: Option<ResolutionStrategy> = None;
    let total = conflict_response.conflicts.len();

    for (idx, conflict) in conflict_response.conflicts.iter().enumerate() {
        let strategy = if let Some(strat) = &apply_all {
            strat.clone()
        } else {
            prompter.prompt_strategy(conflict, idx, total, &mut apply_all)?
        };

        let resource_resolution = match &strategy {
            ResolutionStrategy::KeepOld | ResolutionStrategy::KeepNew => ResourceResolution {
                strategy,
                attributes: None,
            },
            ResolutionStrategy::PerAttribute => {
                let mut attrs: HashMap<String, ResolutionStrategy> = HashMap::new();
                for attr in &conflict.attributes {
                    let attr_strategy = match &apply_all_attr {
                        Some(s) if *s != ResolutionStrategy::PerAttribute => s.clone(),
                        _ => prompter.prompt_attribute(attr, &mut apply_all_attr)?,
                    };
                    attrs.insert(attr.key.clone(), attr_strategy);
                }
                ResourceResolution {
                    strategy: ResolutionStrategy::PerAttribute,
                    attributes: Some(attrs),
                }
            }
        };

        resolutions.insert(conflict.resource_id, resource_resolution);
    }

    Ok(ConflictResolutionRequest {
        import_token: conflict_response.import_token.clone(),
        resolutions,
    })
}

fn cmd_import(args: &cli::ImportArgs, config: &Config) -> Result<(), CliError> {
    let data = std::fs::read(&args.path).map_err(|e| {
        CliError::Args(format!("Failed to read import file '{}': {e}", args.path))
    })?;

    let url = format!("{}/api/import", config.api_url);
    print_verbose(&format!("Request: POST {url}"), config.verbose);
    print_verbose(
        &format!("File: {} ({} bytes)", args.path, data.len()),
        config.verbose,
    );

    let client = client::build_client(config);
    let rt = tokio::runtime::Runtime::new().unwrap();

    let response = rt
        .block_on(client.post(&url).body(data).send())
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

    match status.as_u16() {
        200 => {
            let success: ImportSuccessResponse = rt
                .block_on(response.json())
                .map_err(|e| CliError::Generic(format!("Failed to parse import response: {e}")))?;

            if args.dry_run {
                if !config.quiet {
                    println!("No conflicts");
                }
            } else if !config.quiet {
                println!(
                    "Import successful: {} clients created, {} resources imported",
                    success.clients_created, success.resources_imported
                );
            }
            Ok(())
        }
        409 => {
            let conflict_response: ImportConflictResponse = rt
                .block_on(response.json())
                .map_err(|e| {
                    CliError::Generic(format!("Failed to parse conflict response: {e}"))
                })?;

            if args.dry_run {
                let mut writer =
                    output::OutputWriter::new(OutputFormat::Json, config.verbose);
                writer
                    .write_import_conflicts(&conflict_response)
                    .map_err(|e| CliError::Generic(format!("Failed to write output: {e}")))?;

                writer.flush_to_stdio();
                return Ok(());
            }

            if let Some(resolve_path) = &args.resolve_file {
                let resolution_data =
                    std::fs::read_to_string(resolve_path).map_err(|e| {
                        CliError::Args(format!(
                            "Failed to read resolution file '{resolve_path}': {e}"
                        ))
                    })?;

                let resolution_request: ConflictResolutionRequest =
                    serde_json::from_str(&resolution_data).map_err(|e| {
                        CliError::Args(format!("Invalid resolution file: {e}"))
                    })?;

                validate_resolutions(&resolution_request, &conflict_response)?;

                return submit_resolve(config, &client, &rt, &resolution_request);
            }

            handle_conflicts_interactively(
                &conflict_response,
                &mut DialoguerPrompter::new(),
                config,
                &client,
                &rt,
            )
        }
        _ => Err(api_error_from_response(response, &rt)),
    }
}

fn handle_conflicts_interactively(
    conflict_response: &ImportConflictResponse,
    prompter: &mut dyn ImportPrompter,
    config: &Config,
    client: &reqwest::Client,
    rt: &tokio::runtime::Runtime,
) -> Result<(), CliError> {
    let resolution_request = resolve_interactively(conflict_response, prompter)
        .map_err(|e| CliError::Generic(format!("Interactive resolution failed: {e}")))?;

    submit_resolve(config, client, rt, &resolution_request)
}

fn submit_resolve(
    config: &Config,
    client: &reqwest::Client,
    rt: &tokio::runtime::Runtime,
    resolution_request: &ConflictResolutionRequest,
) -> Result<(), CliError> {
    let resolve_url = format!("{}/api/import/resolve", config.api_url);
    print_verbose(
        &format!("Request: POST {resolve_url}"),
        config.verbose,
    );

    let resolve_response = rt
        .block_on(client.post(&resolve_url).json(resolution_request).send())
        .map_err(|e| CliError::Generic(format!("Failed to connect to API: {e}")))?;

    let resolve_status = resolve_response.status();
    print_verbose(
        &format!(
            "Response: {} {}",
            resolve_status.as_u16(),
            resolve_status.canonical_reason().unwrap_or("")
        ),
        config.verbose,
    );

    if !resolve_status.is_success() {
        return Err(api_error_from_response(resolve_response, rt));
    }

    let resolve_result: ImportSuccessResponse = rt
        .block_on(resolve_response.json())
        .map_err(|e| CliError::Generic(format!("Failed to parse resolve response: {e}")))?;

    if !config.quiet {
        println!(
            "Import successful: {} clients created, {} resources imported",
            resolve_result.clients_created, resolve_result.resources_imported
        );
    }

    Ok(())
}

fn validate_resolutions(
    req: &ConflictResolutionRequest,
    conflict_response: &ImportConflictResponse,
) -> Result<(), CliError> {
    if req.import_token != conflict_response.import_token {
        return Err(CliError::Args(format!(
            "Resolution file import_token '{}' does not match conflict import_token '{}'",
            req.import_token, conflict_response.import_token
        )));
    }

    let conflict_ids: HashSet<_> = conflict_response
        .conflicts
        .iter()
        .map(|c| c.resource_id)
        .collect();

    let conflict_attrs: HashMap<_, _> = conflict_response
        .conflicts
        .iter()
        .map(|c| (c.resource_id, c.attributes.iter().map(|a| a.key.as_str()).collect::<HashSet<_>>()))
        .collect();

    for (id, resource_resolution) in &req.resolutions {
        if !conflict_ids.contains(id) {
            return Err(CliError::Args(format!(
                "Resource ID '{id}' in resolutions not found in conflicts"
            )));
        }

        if resource_resolution.strategy == ResolutionStrategy::PerAttribute {
            let attrs = resource_resolution.attributes.as_ref().ok_or_else(|| {
                CliError::Args(format!(
                    "PerAttribute strategy for resource '{id}' must include an attributes map"
                ))
            })?;
            if let Some(valid_keys) = conflict_attrs.get(id) {
                for key in attrs.keys() {
                    if !valid_keys.contains(key.as_str()) {
                        return Err(CliError::Args(format!(
                            "Attribute key '{key}' for resource '{id}' not found in conflicts"
                        )));
                    }
                }
            }
        }
    }

    Ok(())
}

fn api_error_from_response(
    response: Response,
    rt: &tokio::runtime::Runtime,
) -> CliError {
    let status = response.status();
    let problem: Result<ProblemJson, _> = rt.block_on(response.json());
    match problem {
        Ok(problem) => CliError::Api(problem),
        Err(_) => CliError::Api(ProblemJson {
            type_: format!("/errors/http-{}", status.as_u16()),
            title: status.canonical_reason().unwrap_or("HTTP Error").to_string(),
            status: status.as_u16(),
            detail: None,
        }),
    }
}

fn cmd_key(sub: &KeyCommand, config: &Config) -> Result<(), CliError> {
    match sub {
        KeyCommand::List { client_id } => cmd_key_list(client_id, config),
        KeyCommand::Create {
            client_id,
            name,
            permissions,
        } => cmd_key_create(client_id, name, permissions, config),
        KeyCommand::Revoke { client_id, key_id } => cmd_key_revoke(client_id, key_id, config),
    }
}

fn cmd_key_list(client_id: &str, config: &Config) -> Result<(), CliError> {
    let url = format!("{}/api/clients/{client_id}/keys", config.api_url);
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
        return Err(api_error_from_response(response, &rt));
    }

    let keys: Vec<arche_types::crud::ApiKeySummary> = rt
        .block_on(response.json())
        .map_err(|e| CliError::Generic(format!("Failed to parse key list response: {e}")))?;

    let output_format = if config.quiet {
        OutputFormat::Quiet
    } else {
        OutputFormat::Json
    };

    let mut writer = output::OutputWriter::new(output_format, config.verbose);
    writer
        .write_key_list(&keys)
        .map_err(|e| CliError::Generic(format!("Failed to write output: {e}")))?;

    writer.flush_to_stdio();

    Ok(())
}

fn cmd_key_create(
    client_id: &str,
    name: &str,
    permissions: &str,
    config: &Config,
) -> Result<(), CliError> {
    let perms = parse_permissions(permissions)?;

    if perms.is_empty() {
        return Err(CliError::Args(
            "At least one permission is required".into(),
        ));
    }

    let req = arche_types::crud::CreateApiKeyRequest {
        name: name.to_string(),
        permissions: perms,
    };

    let url = format!("{}/api/clients/{client_id}/keys", config.api_url);
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
        return Err(api_error_from_response(response, &rt));
    }

    let create_response: arche_types::crud::CreateApiKeyResponse = rt
        .block_on(response.json())
        .map_err(|e| {
            CliError::Generic(format!("Failed to parse key create response: {e}"))
        })?;

    let output_format = if config.quiet {
        OutputFormat::Quiet
    } else {
        OutputFormat::Json
    };

    let mut writer = output::OutputWriter::new(output_format, config.verbose);
    writer
        .write_key_created(&create_response)
        .map_err(|e| CliError::Generic(format!("Failed to write output: {e}")))?;

    let stdout_content = writer.stdout().contents();
    if !stdout_content.is_empty() {
        print!("{stdout_content}");
    }

    eprintln!("This key will not be shown again. Copy it now.");

    let stderr_content = writer.stderr().contents();
    if !stderr_content.is_empty() {
        eprint!("{stderr_content}");
    }

    Ok(())
}

fn cmd_key_revoke(client_id: &str, key_id: &str, config: &Config) -> Result<(), CliError> {
    let url = format!("{}/api/clients/{client_id}/keys/{key_id}", config.api_url);
    print_verbose(&format!("Request: DELETE {url}"), config.verbose);

    let client = client::build_client(config);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let response = rt
        .block_on(client.delete(&url).send())
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
        return Err(api_error_from_response(response, &rt));
    }

    let output_format = if config.quiet {
        OutputFormat::Quiet
    } else {
        OutputFormat::Json
    };

    let mut writer = output::OutputWriter::new(output_format, config.verbose);
    writer
        .write_key_revoked()
        .map_err(|e| CliError::Generic(format!("Failed to write output: {e}")))?;

    writer.flush_to_stdio();

    Ok(())
}

fn parse_permissions(permissions: &str) -> Result<Vec<arche_types::Permission>, CliError> {
    permissions
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| match s.to_lowercase().as_str() {
            "read" => Ok(arche_types::Permission::Read),
            "write" => Ok(arche_types::Permission::Write),
            "delete" => Ok(arche_types::Permission::Delete),
            "generate" => Ok(arche_types::Permission::Generate),
            "admin" => Ok(arche_types::Permission::Admin),
            other => Err(CliError::Args(format!(
                "Invalid permission: '{other}'. Valid permissions: read, write, delete, generate, admin"
            ))),
        })
        .collect()
}

fn cmd_client(sub: &ClientCommand, config: &Config) -> Result<(), CliError> {
    match sub {
        ClientCommand::List => cmd_client_list(config),
        ClientCommand::Create { name } => cmd_client_create(name, config),
        ClientCommand::Delete { client_id } => cmd_client_delete(client_id, config),
    }
}

fn cmd_client_list(config: &Config) -> Result<(), CliError> {
    let url = format!("{}/api/clients", config.api_url);
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
        return Err(api_error_from_response(response, &rt));
    }

    let paginated: PaginatedResponse<ClientResponse> = rt
        .block_on(response.json())
        .map_err(|e| CliError::Generic(format!("Failed to parse response: {e}")))?;

    let format = if config.quiet {
        OutputFormat::Quiet
    } else {
        OutputFormat::Json
    };

    let mut writer = output::OutputWriter::new(format, config.verbose);
    writer
        .write_client_list(&paginated.data)
        .map_err(|e| CliError::Generic(format!("Failed to write output: {e}")))?;

    output::emit_output(&writer);

    Ok(())
}

fn cmd_client_create(name: &str, config: &Config) -> Result<(), CliError> {
    let url = format!("{}/api/clients", config.api_url);
    print_verbose(&format!("Request: POST {url}"), config.verbose);
    print_verbose(&format!("Request body: name={name}"), config.verbose);

    let client = client::build_client(config);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let req = CreateClientRequest {
        name: name.to_string(),
    };
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
        return Err(api_error_from_response(response, &rt));
    }

    let client_response: ClientResponse = rt
        .block_on(response.json())
        .map_err(|e| CliError::Generic(format!("Failed to parse response: {e}")))?;

    let format = if config.quiet {
        OutputFormat::Quiet
    } else {
        OutputFormat::Json
    };

    let mut writer = output::OutputWriter::new(format, config.verbose);
    writer
        .write_client_created(&client_response)
        .map_err(|e| CliError::Generic(format!("Failed to write output: {e}")))?;

    output::emit_output(&writer);

    Ok(())
}

fn cmd_client_delete(client_id: &str, config: &Config) -> Result<(), CliError> {
    let url = format!("{}/api/clients/{client_id}", config.api_url);
    print_verbose(&format!("Request: DELETE {url}"), config.verbose);

    let client = client::build_client(config);
    let rt = tokio::runtime::Runtime::new().unwrap();
    let response = rt
        .block_on(client.delete(&url).send())
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
        return Err(api_error_from_response(response, &rt));
    }

    let format = if config.quiet {
        OutputFormat::Quiet
    } else {
        OutputFormat::Json
    };

    let mut writer = output::OutputWriter::new(format, config.verbose);
    writer
        .write_client_deleted()
        .map_err(|e| CliError::Generic(format!("Failed to write output: {e}")))?;

    output::emit_output(&writer);

    Ok(())
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
        let args = make_args(None, None, None, Some(r#"{"min_prefixes":1,"max_prefixes":2}"#), "json");
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
            Some(r#"{"min_prefixes":1}"#),
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
            Some(r#"{"min_prefixes":1,"max_prefixes":2}"#),
            "json",
        );
        let req = build_generate_request(&args).unwrap();
        assert_eq!(req.archetype.as_deref(), Some("sword"));
        assert_eq!(req.seed, Some(12345));
        assert!(req.constraints.is_some());
        assert!(req.affixes.is_some());
    }
}

#[cfg(test)]
mod import_tests {
    use super::*;
    use arche_types::export_import::{ConflictAttribute, ConflictDetail};
    use arche_types::ValueType;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use uuid::Uuid;

    fn make_config(api_url: &str, quiet: bool, verbose: bool) -> Config {
        Config {
            api_url: api_url.to_string(),
            api_key: None,
            verbose,
            quiet,
        }
    }

    fn make_import_args(
        path: &str,
        dry_run: bool,
        resolve_file: Option<&str>,
    ) -> cli::ImportArgs {
        cli::ImportArgs {
            path: path.to_string(),
            dry_run,
            resolve_file: resolve_file.map(|s| s.to_string()),
        }
    }

    fn write_temp_file(name: &str, content: &[u8]) -> std::path::PathBuf {
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, content).unwrap();
        path
    }

    fn conflict_response() -> ImportConflictResponse {
        ImportConflictResponse {
            problem: ProblemJson {
                type_: "/errors/import-conflict".into(),
                title: "Import conflicts require resolution".into(),
                status: 409,
                detail: Some("2 conflicts found".into()),
            },
            conflicts: vec![
                ConflictDetail {
                    resource_type: "blueprint".into(),
                    resource_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001")
                        .unwrap(),
                    resource_name: "Longsword".into(),
                    attributes: vec![ConflictAttribute {
                        key: "damage".into(),
                        old_value: serde_json::json!({"min": 10}),
                        new_value: serde_json::json!({"min": 15}),
                        value_type: ValueType::Range,
                    }],
                },
                ConflictDetail {
                    resource_type: "affix".into(),
                    resource_id: Uuid::parse_str("00000000-0000-0000-0000-000000000002")
                        .unwrap(),
                    resource_name: "Fire".into(),
                    attributes: vec![ConflictAttribute {
                        key: "weight".into(),
                        old_value: serde_json::json!(0),
                        new_value: serde_json::json!(5),
                        value_type: ValueType::Single,
                    }],
                },
            ],
            import_token: "test-import-token".into(),
        }
    }

    fn success_response() -> ImportSuccessResponse {
        ImportSuccessResponse {
            status: "success".into(),
            clients_created: 1,
            resources_imported: 3,
        }
    }

    async fn run_import(args: cli::ImportArgs, config: Config) -> Result<(), CliError> {
        tokio::task::spawn_blocking(move || cmd_import(&args, &config))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_import_dry_run_no_conflicts_prints_no_conflicts() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::to_value(success_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let import_path = write_temp_file("test_dry_run_no_conflicts.zip", b"dummy-zip");
        let args = make_import_args(import_path.to_str().unwrap(), true, None);
        let config = make_config(&uri, false, false);
        let result = run_import(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_import_dry_run_with_conflicts_writes_json_to_stdout() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(
                ResponseTemplate::new(409)
                    .set_body_json(serde_json::to_value(conflict_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let import_path = write_temp_file("test_dry_run_conflicts.zip", b"dummy-zip");
        let args = make_import_args(import_path.to_str().unwrap(), true, None);
        let config = make_config(&uri, false, false);
        let result = run_import(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_import_dry_run_quiet_suppresses_output() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(
                ResponseTemplate::new(409)
                    .set_body_json(serde_json::to_value(conflict_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let import_path = write_temp_file("test_dry_run_quiet.zip", b"dummy-zip");
        let args = make_import_args(import_path.to_str().unwrap(), true, None);
        let config = make_config(&uri, true, false);
        let result = run_import(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_import_resolve_file_success() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(
                ResponseTemplate::new(409)
                    .set_body_json(serde_json::to_value(conflict_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/api/import/resolve"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::to_value(success_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let resolution_json = serde_json::json!({
            "importToken": "test-import-token",
            "resolutions": {
                "00000000-0000-0000-0000-000000000001": {
                    "strategy": "keepOld"
                },
                "00000000-0000-0000-0000-000000000002": {
                    "strategy": "keepNew"
                }
            }
        });
        let resolve_path = write_temp_file(
            "test_resolve_file.json",
            serde_json::to_string_pretty(&resolution_json).unwrap().as_bytes(),
        );
        let import_path = write_temp_file("test_resolve_success.zip", b"dummy-zip");
        let args = make_import_args(
            import_path.to_str().unwrap(),
            false,
            Some(resolve_path.to_str().unwrap()),
        );
        let config = make_config(&uri, false, false);
        let result = run_import(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_import_resolve_file_with_per_attribute_strategy() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        let response = conflict_response();
        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(
                ResponseTemplate::new(409)
                    .set_body_json(serde_json::to_value(&response).unwrap()),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/api/import/resolve"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::to_value(success_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let resolution_json = serde_json::json!({
            "importToken": "test-import-token",
            "resolutions": {
                "00000000-0000-0000-0000-000000000001": {
                    "strategy": "perAttribute",
                    "attributes": {
                        "damage": "keepOld"
                    }
                },
                "00000000-0000-0000-0000-000000000002": {
                    "strategy": "keepNew"
                }
            }
        });
        let resolve_path = write_temp_file(
            "test_resolve_per_attr.json",
            serde_json::to_string_pretty(&resolution_json).unwrap().as_bytes(),
        );
        let import_path = write_temp_file("test_resolve_per_attr.zip", b"dummy-zip");
        let args = make_import_args(
            import_path.to_str().unwrap(),
            false,
            Some(resolve_path.to_str().unwrap()),
        );
        let config = make_config(&uri, false, false);
        let result = run_import(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_import_resolve_file_import_token_mismatch() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(
                ResponseTemplate::new(409)
                    .set_body_json(serde_json::to_value(conflict_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let resolution_json = serde_json::json!({
            "importToken": "wrong-token",
            "resolutions": {
                "00000000-0000-0000-0000-000000000001": {
                    "strategy": "keepOld"
                }
            }
        });
        let resolve_path = write_temp_file(
            "test_token_mismatch.json",
            serde_json::to_string_pretty(&resolution_json).unwrap().as_bytes(),
        );
        let import_path = write_temp_file("test_token_mismatch.zip", b"dummy-zip");
        let args = make_import_args(
            import_path.to_str().unwrap(),
            false,
            Some(resolve_path.to_str().unwrap()),
        );
        let config = make_config(&uri, false, false);
        let result = run_import(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(msg)) => {
                assert!(msg.contains("import_token"), "expected import_token mismatch, got: {msg}");
                assert_eq!(CliError::Args(msg.clone()).exit_code(), 2);
            }
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_import_resolve_file_invalid_resource_uuid() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(
                ResponseTemplate::new(409)
                    .set_body_json(serde_json::to_value(conflict_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let resolution_json = serde_json::json!({
            "importToken": "test-import-token",
            "resolutions": {
                "00000000-0000-0000-0000-000000000099": {
                    "strategy": "keepOld"
                }
            }
        });
        let resolve_path = write_temp_file(
            "test_invalid_uuid.json",
            serde_json::to_string_pretty(&resolution_json).unwrap().as_bytes(),
        );
        let import_path = write_temp_file("test_invalid_uuid.zip", b"dummy-zip");
        let args = make_import_args(
            import_path.to_str().unwrap(),
            false,
            Some(resolve_path.to_str().unwrap()),
        );
        let config = make_config(&uri, false, false);
        let result = run_import(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(msg)) => {
                assert!(msg.contains("not found in conflicts"), "expected UUID not found error, got: {msg}");
                assert_eq!(CliError::Args(msg.clone()).exit_code(), 2);
            }
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_import_resolve_file_per_attribute_missing_attrs_map() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(
                ResponseTemplate::new(409)
                    .set_body_json(serde_json::to_value(conflict_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let resolution_json = serde_json::json!({
            "importToken": "test-import-token",
            "resolutions": {
                "00000000-0000-0000-0000-000000000001": {
                    "strategy": "perAttribute"
                }
            }
        });
        let resolve_path = write_temp_file(
            "test_missing_attrs.json",
            serde_json::to_string_pretty(&resolution_json).unwrap().as_bytes(),
        );
        let import_path = write_temp_file("test_missing_attrs.zip", b"dummy-zip");
        let args = make_import_args(
            import_path.to_str().unwrap(),
            false,
            Some(resolve_path.to_str().unwrap()),
        );
        let config = make_config(&uri, false, false);
        let result = run_import(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(msg)) => {
                assert!(
                    msg.contains("must include an attributes map"),
                    "expected attributes map missing error, got: {msg}"
                );
                assert_eq!(CliError::Args(msg.clone()).exit_code(), 2);
            }
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_import_resolve_file_invalid_attribute_key() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(
                ResponseTemplate::new(409)
                    .set_body_json(serde_json::to_value(conflict_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let resolution_json = serde_json::json!({
            "importToken": "test-import-token",
            "resolutions": {
                "00000000-0000-0000-0000-000000000001": {
                    "strategy": "perAttribute",
                    "attributes": {
                        "nonexistent_attr": "keepOld"
                    }
                }
            }
        });
        let resolve_path = write_temp_file(
            "test_invalid_attr_key.json",
            serde_json::to_string_pretty(&resolution_json).unwrap().as_bytes(),
        );
        let import_path = write_temp_file("test_invalid_attr_key.zip", b"dummy-zip");
        let args = make_import_args(
            import_path.to_str().unwrap(),
            false,
            Some(resolve_path.to_str().unwrap()),
        );
        let config = make_config(&uri, false, false);
        let result = run_import(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(msg)) => {
                assert!(
                    msg.contains("not found in conflicts"),
                    "expected attribute key not found error, got: {msg}"
                );
                assert_eq!(CliError::Args(msg.clone()).exit_code(), 2);
            }
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_import_invalid_resolution_json_returns_args_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(
                ResponseTemplate::new(409)
                    .set_body_json(serde_json::to_value(conflict_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let resolve_path = write_temp_file("test_invalid_json.json", b"not valid json {{{");
        let import_path = write_temp_file("test_invalid_json.zip", b"dummy-zip");
        let args = make_import_args(
            import_path.to_str().unwrap(),
            false,
            Some(resolve_path.to_str().unwrap()),
        );
        let config = make_config(&uri, false, false);
        let result = run_import(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(msg)) => {
                assert!(
                    msg.contains("Invalid resolution file"),
                    "expected invalid JSON error, got: {msg}"
                );
                assert_eq!(CliError::Args(msg.clone()).exit_code(), 2);
            }
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_import_missing_file_returns_args_error() {
        let config = make_config("http://localhost:1", false, false);
        let args = make_import_args("/nonexistent/path/file.zip", false, None);
        let result = cmd_import(&args, &config);
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(msg)) => {
                assert!(
                    msg.contains("Failed to read import file"),
                    "expected file read error, got: {msg}"
                );
                assert_eq!(CliError::Args(msg.clone()).exit_code(), 2);
            }
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    #[test]
    fn test_import_connection_refused_returns_generic_error() {
        let import_path = write_temp_file("test_conn_refused.zip", b"dummy-zip");
        let config = make_config("http://127.0.0.1:1", false, false);
        let args = make_import_args(import_path.to_str().unwrap(), false, None);
        let result = cmd_import(&args, &config);
        assert!(result.is_err());
        match &result {
            Err(CliError::Generic(msg)) => {
                assert!(
                    msg.contains("Failed to connect"),
                    "expected connect error, got: {msg}"
                );
            }
            _ => panic!("expected Generic error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_import_api_error_returns_api_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(
                ResponseTemplate::new(500).set_body_json(serde_json::json!({
                    "type": "/errors/internal-error",
                    "title": "Internal Server Error",
                    "status": 500,
                    "detail": "Something broke"
                })),
            )
            .mount(&mock_server)
            .await;

        let import_path = write_temp_file("test_api_error.zip", b"dummy-zip");
        let args = make_import_args(import_path.to_str().unwrap(), false, None);
        let config = make_config(&uri, false, false);
        let result = run_import(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 500);
                assert_eq!(problem.detail, Some("Something broke".into()));
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_import_resolve_api_error_returns_api_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(
                ResponseTemplate::new(409)
                    .set_body_json(serde_json::to_value(conflict_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/api/import/resolve"))
            .respond_with(
                ResponseTemplate::new(400).set_body_json(serde_json::json!({
                    "type": "/errors/invalid-resolution",
                    "title": "Invalid Resolution",
                    "status": 400,
                    "detail": "Conflict resolution expired or invalid"
                })),
            )
            .mount(&mock_server)
            .await;

        let resolution_json = serde_json::json!({
            "importToken": "test-import-token",
            "resolutions": {
                "00000000-0000-0000-0000-000000000001": {
                    "strategy": "keepOld"
                },
                "00000000-0000-0000-0000-000000000002": {
                    "strategy": "keepNew"
                }
            }
        });
        let resolve_path = write_temp_file(
            "test_resolve_api_error.json",
            serde_json::to_string_pretty(&resolution_json).unwrap().as_bytes(),
        );
        let import_path = write_temp_file("test_resolve_api_error.zip", b"dummy-zip");
        let args = make_import_args(
            import_path.to_str().unwrap(),
            false,
            Some(resolve_path.to_str().unwrap()),
        );
        let config = make_config(&uri, false, false);
        let result = run_import(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 400);
                assert_eq!(
                    problem.detail,
                    Some("Conflict resolution expired or invalid".into())
                );
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_import_dry_run_then_resolve_integration() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        let response = serde_json::to_value(conflict_response()).unwrap();

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(ResponseTemplate::new(409).set_body_json(&response))
            .mount(&mock_server)
            .await;

        let import_path = write_temp_file("test_integration.zip", b"dummy-zip");
        let args_dry = make_import_args(import_path.to_str().unwrap(), true, None);
        let config = make_config(&uri, false, false);
        let result = run_import(args_dry, config.clone()).await;
        assert!(result.is_ok());

        let resolution_json = serde_json::json!({
            "importToken": "test-import-token",
            "resolutions": {
                "00000000-0000-0000-0000-000000000001": {
                    "strategy": "keepOld"
                },
                "00000000-0000-0000-0000-000000000002": {
                    "strategy": "keepNew"
                }
            }
        });
        let resolve_path = write_temp_file(
            "test_integration_resolve.json",
            serde_json::to_string_pretty(&resolution_json).unwrap().as_bytes(),
        );

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(ResponseTemplate::new(409).set_body_json(&response))
            .mount(&mock_server)
            .await;

        Mock::given(method("POST"))
            .and(path("/api/import/resolve"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::to_value(success_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let args_resolve = make_import_args(
            import_path.to_str().unwrap(),
            false,
            Some(resolve_path.to_str().unwrap()),
        );
        let result = run_import(args_resolve, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_import_resolve_file_nonexistent_returns_args_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import"))
            .respond_with(
                ResponseTemplate::new(409)
                    .set_body_json(serde_json::to_value(conflict_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let import_path = write_temp_file("test_nonexistent_resolve.zip", b"dummy-zip");
        let args = make_import_args(
            import_path.to_str().unwrap(),
            false,
            Some("/nonexistent/resolutions.json"),
        );
        let config = make_config(&uri, false, false);
        let result = run_import(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(msg)) => {
                assert!(
                    msg.contains("Failed to read resolution file"),
                    "expected file read error, got: {msg}"
                );
                assert_eq!(CliError::Args(msg.clone()).exit_code(), 2);
            }
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    struct MockPrompter {
        strategies: Vec<ResolutionStrategy>,
        attr_strategies: Vec<ResolutionStrategy>,
        strategy_index: usize,
        attr_index: usize,
    }

    impl MockPrompter {
        fn new(strategies: Vec<ResolutionStrategy>, attr_strategies: Vec<ResolutionStrategy>) -> Self {
            Self { strategies, attr_strategies, strategy_index: 0, attr_index: 0 }
        }
    }

    impl ImportPrompter for MockPrompter {
        fn prompt_strategy(
            &mut self,
            _detail: &ConflictDetail,
            _index: usize,
            _total: usize,
            _apply_all: &mut Option<ResolutionStrategy>,
        ) -> io::Result<ResolutionStrategy> {
            let strategy = self.strategies[self.strategy_index].clone();
            self.strategy_index += 1;
            Ok(strategy)
        }

        fn prompt_attribute(
            &mut self,
            _attr: &ConflictAttribute,
            _apply_all_attr: &mut Option<ResolutionStrategy>,
        ) -> io::Result<ResolutionStrategy> {
            let strategy = self.attr_strategies[self.attr_index].clone();
            self.attr_index += 1;
            Ok(strategy)
        }
    }

    #[test]
    fn test_resolve_interactively_all_keep_old() {
        let response = conflict_response();
        let mut prompter = MockPrompter::new(
            vec![ResolutionStrategy::KeepOld, ResolutionStrategy::KeepOld],
            vec![],
        );

        let result = resolve_interactively(&response, &mut prompter).unwrap();

        assert_eq!(result.import_token, "test-import-token");
        assert_eq!(result.resolutions.len(), 2);
        let res1 = result.resolutions.get(
            &Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
        ).unwrap();
        assert_eq!(res1.strategy, ResolutionStrategy::KeepOld);
        assert!(res1.attributes.is_none());
        let res2 = result.resolutions.get(
            &Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap()
        ).unwrap();
        assert_eq!(res2.strategy, ResolutionStrategy::KeepOld);
        assert!(res2.attributes.is_none());
    }

    #[test]
    fn test_resolve_interactively_all_keep_new() {
        let response = conflict_response();
        let mut prompter = MockPrompter::new(
            vec![ResolutionStrategy::KeepNew, ResolutionStrategy::KeepNew],
            vec![],
        );

        let result = resolve_interactively(&response, &mut prompter).unwrap();

        assert_eq!(result.resolutions.len(), 2);
        let res1 = result.resolutions.get(
            &Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
        ).unwrap();
        assert_eq!(res1.strategy, ResolutionStrategy::KeepNew);
    }

    #[test]
    fn test_resolve_interactively_per_attribute() {
        let response = conflict_response();
        let mut prompter = MockPrompter::new(
            vec![ResolutionStrategy::PerAttribute, ResolutionStrategy::KeepNew],
            vec![ResolutionStrategy::KeepNew],
        );

        let result = resolve_interactively(&response, &mut prompter).unwrap();

        let res1 = result.resolutions.get(
            &Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap()
        ).unwrap();
        assert_eq!(res1.strategy, ResolutionStrategy::PerAttribute);
        let attrs = res1.attributes.as_ref().unwrap();
        assert_eq!(attrs.get("damage").unwrap(), &ResolutionStrategy::KeepNew);
    }

    #[test]
    fn test_resolve_interactively_apply_all() {
        let response = conflict_response();

        struct ApplyAllPrompter {
            call_count: u32,
        }
        impl ImportPrompter for ApplyAllPrompter {
            fn prompt_strategy(
                &mut self,
                _detail: &ConflictDetail,
                _index: usize,
                _total: usize,
                apply_all: &mut Option<ResolutionStrategy>,
            ) -> io::Result<ResolutionStrategy> {
                self.call_count += 1;
                if self.call_count == 1 {
                    *apply_all = Some(ResolutionStrategy::KeepNew);
                    Ok(ResolutionStrategy::KeepNew)
                } else {
                    panic!("prompt_strategy called more than once");
                }
            }

            fn prompt_attribute(
                &mut self,
                _attr: &ConflictAttribute,
                _apply_all_attr: &mut Option<ResolutionStrategy>,
            ) -> io::Result<ResolutionStrategy> {
                panic!("should not call prompt_attribute for non-PerAttribute");
            }
        }

        let mut prompter = ApplyAllPrompter { call_count: 0 };
        let result = resolve_interactively(&response, &mut prompter).unwrap();

        assert_eq!(prompter.call_count, 1, "only first conflict should prompt, second uses apply_all");
        assert_eq!(result.resolutions.len(), 2);
    }

    #[tokio::test]
    async fn test_import_interactive_all_keep_old_success() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import/resolve"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::to_value(success_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);

        let result = tokio::task::spawn_blocking(move || {
            let client = client::build_client(&config);
            let rt = tokio::runtime::Runtime::new().unwrap();
            let strategies = vec![ResolutionStrategy::KeepOld, ResolutionStrategy::KeepOld];
            let mut prompter = MockPrompter::new(strategies, vec![]);
            handle_conflicts_interactively(
                &conflict_response(),
                &mut prompter,
                &config,
                &client,
                &rt,
            )
        })
        .await
        .unwrap();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_import_interactive_per_attribute_success() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import/resolve"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::to_value(success_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);

        let result = tokio::task::spawn_blocking(move || {
            let client = client::build_client(&config);
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut prompter = MockPrompter::new(
                vec![ResolutionStrategy::PerAttribute, ResolutionStrategy::KeepOld],
                vec![ResolutionStrategy::KeepNew],
            );
            handle_conflicts_interactively(
                &conflict_response(),
                &mut prompter,
                &config,
                &client,
                &rt,
            )
        })
        .await
        .unwrap();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_import_interactive_resolve_api_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/import/resolve"))
            .respond_with(
                ResponseTemplate::new(400).set_body_json(serde_json::json!({
                    "type": "/errors/invalid-resolution",
                    "title": "Invalid Resolution",
                    "status": 400,
                    "detail": "Import token expired"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);

        let result = tokio::task::spawn_blocking(move || {
            let client = client::build_client(&config);
            let rt = tokio::runtime::Runtime::new().unwrap();
            let mut prompter = MockPrompter::new(
                vec![ResolutionStrategy::KeepOld, ResolutionStrategy::KeepOld],
                vec![],
            );
            handle_conflicts_interactively(
                &conflict_response(),
                &mut prompter,
                &config,
                &client,
                &rt,
            )
        })
        .await
        .unwrap();
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 400);
                assert_eq!(problem.detail, Some("Import token expired".into()));
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }
}

#[cfg(test)]
mod export_tests {
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

    fn make_export_args(
        output: Option<&str>,
        clients: Option<&str>,
        include_api_keys: bool,
        include_audit_log: bool,
        inline_refs: bool,
    ) -> cli::ExportArgs {
        cli::ExportArgs {
            output: output.map(|s| s.to_string()),
            clients: clients.map(|s| s.to_string()),
            include_api_keys,
            include_audit_log,
            inline_refs,
        }
    }

    async fn run_export(args: cli::ExportArgs, config: Config) -> Result<(), CliError> {
        tokio::task::spawn_blocking(move || cmd_export(&args, &config))
            .await
            .unwrap()
    }

    fn write_temp_output(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(name)
    }

    #[tokio::test]
    async fn test_export_no_flags_writes_to_stdout() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"PK\x03\x04fake-zip-content", "application/zip"),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_export_args(None, None, false, false, false);
        let result = run_export(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_to_file_writes_zip() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"PK\x03\x04fake-zip-content", "application/zip"),
            )
            .mount(&mock_server)
            .await;

        let output_path = write_temp_output("test_export_to_file.zip");
        let path_str = output_path.to_str().unwrap().to_string();

        let config = make_config(&uri, false, false);
        let args = make_export_args(Some(&path_str), None, false, false, false);
        let result = run_export(args, config).await;
        assert!(result.is_ok());

        let written = std::fs::read(&output_path).unwrap();
        assert_eq!(written, b"PK\x03\x04fake-zip-content");

        std::fs::remove_file(&path_str).ok();
    }

    #[tokio::test]
    async fn test_export_with_clients_filter() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"client-filtered-zip", "application/zip"),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_export_args(
            None,
            Some("00000000-0000-0000-0000-000000000001,00000000-0000-0000-0000-000000000002"),
            false,
            false,
            false,
        );
        let result = run_export(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_invalid_client_uuid_returns_args_error() {
        let config = make_config("http://localhost:1", false, false);
        let args = make_export_args(None, Some("not-a-valid-uuid"), false, false, false);
        let result = run_export(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(msg)) => {
                assert!(msg.contains("Invalid client UUID"), "expected UUID parse error, got: {msg}");
                assert_eq!(CliError::Args(msg.clone()).exit_code(), 2);
            }
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_export_with_include_api_keys() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"zip-with-keys", "application/zip"),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_export_args(None, None, true, false, false);
        let result = run_export(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_with_include_audit_log() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"zip-with-audit-log", "application/zip"),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_export_args(None, None, false, true, false);
        let result = run_export(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_with_inline_refs() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"zip-with-inline-refs", "application/zip"),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_export_args(None, None, false, false, true);
        let result = run_export(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_all_flags_together() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"all-flags-zip", "application/zip"),
            )
            .mount(&mock_server)
            .await;

        let output_path = write_temp_output("test_all_flags.zip");
        let path_str = output_path.to_str().unwrap();

        let config = make_config(&uri, false, false);
        let args = make_export_args(
            Some(path_str),
            Some("00000000-0000-0000-0000-000000000001"),
            true,
            true,
            true,
        );
        let result = run_export(args, config).await;
        assert!(result.is_ok());

        let written = std::fs::read(&output_path).unwrap();
        assert_eq!(written, b"all-flags-zip");

        std::fs::remove_file(path_str).ok();
    }

    #[tokio::test]
    async fn test_export_quiet_mode_suppresses_output() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"quiet-zip", "application/zip"),
            )
            .mount(&mock_server)
            .await;

        let output_path = write_temp_output("test_quiet_export.zip");
        let path_str = output_path.to_str().unwrap();

        let config = make_config(&uri, true, false);
        let args = make_export_args(Some(path_str), None, false, false, false);
        let result = run_export(args, config).await;
        assert!(result.is_ok());

        std::fs::remove_file(path_str).ok();
    }

    #[tokio::test]
    async fn test_export_verbose_shows_request_details() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"verbose-zip", "application/zip"),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, true);
        let args = make_export_args(None, None, false, false, false);
        let result = run_export(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_server_error_returns_api_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(500).set_body_json(serde_json::json!({
                    "type": "/errors/internal-error",
                    "title": "Internal Server Error",
                    "status": 500,
                    "detail": "Something broke"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_export_args(None, None, false, false, false);
        let result = run_export(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 500);
                assert_eq!(problem.detail, Some("Something broke".into()));
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_export_permission_denied_returns_api_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(403).set_body_json(serde_json::json!({
                    "type": "/errors/forbidden",
                    "title": "Forbidden",
                    "status": 403,
                    "detail": "Super admin required"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_export_args(None, None, false, false, false);
        let result = run_export(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 403);
                assert_eq!(problem.detail, Some("Super admin required".into()));
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[test]
    fn test_export_connection_refused_returns_generic_error() {
        let config = make_config("http://127.0.0.1:1", false, false);
        let args = make_export_args(None, None, false, false, false);
        let result = cmd_export(&args, &config);
        assert!(result.is_err());
        match &result {
            Err(CliError::Generic(msg)) => {
                assert!(
                    msg.contains("Failed to connect"),
                    "expected connect error, got: {msg}"
                );
            }
            _ => panic!("expected Generic error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_export_write_to_invalid_path_returns_generic_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"zip-content", "application/zip"),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_export_args(
            Some("/nonexistent/dir/should/fail/export.zip"),
            None,
            false,
            false,
            false,
        );
        let result = run_export(args, config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Generic(msg)) => {
                assert!(
                    msg.contains("Failed to write export file"),
                    "expected write error, got: {msg}"
                );
            }
            _ => panic!("expected Generic error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_export_empty_clients_string_means_all() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"all-clients-zip", "application/zip"),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_export_args(None, Some(""), false, false, false);
        let result = run_export(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_single_client_uuid() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"single-client-zip", "application/zip"),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let args = make_export_args(
            None,
            Some("00000000-0000-0000-0000-000000000001"),
            false,
            false,
            false,
        );
        let result = run_export(args, config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_export_output_to_file_quiet_suppresses_success_message() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/export"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_raw(b"quiet-file-zip", "application/zip"),
            )
            .mount(&mock_server)
            .await;

        let output_path = write_temp_output("test_quiet_file_export.zip");
        let path_str = output_path.to_str().unwrap();

        let config = make_config(&uri, true, false);
        let args = make_export_args(Some(path_str), None, false, false, false);
        let result = run_export(args, config).await;
        assert!(result.is_ok());

        std::fs::remove_file(path_str).ok();
    }
}

#[cfg(test)]
mod client_tests {
    use super::*;
    use arche_types::crud::ApiKeySummary;
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

    fn sample_clients() -> Vec<ClientResponse> {
        vec![
            ClientResponse {
                id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                name: "My Game".into(),
                created_at: chrono::DateTime::<chrono::Utc>::from_timestamp_millis(0).unwrap(),
                api_keys: vec![],
            },
            ClientResponse {
                id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                name: "Other Client".into(),
                created_at: chrono::DateTime::<chrono::Utc>::from_timestamp_millis(0).unwrap(),
                api_keys: vec![ApiKeySummary {
                    id: uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
                    name: "test-key".into(),
                    permissions: vec![arche_types::Permission::Read],
                    created_at: chrono::DateTime::<chrono::Utc>::from_timestamp_millis(0)
                        .unwrap(),
                }],
            },
        ]
    }

    async fn run_client_list(config: Config) -> Result<(), CliError> {
        tokio::task::spawn_blocking(move || cmd_client_list(&config))
            .await
            .unwrap()
    }

    async fn run_client_create(name: String, config: Config) -> Result<(), CliError> {
        tokio::task::spawn_blocking(move || cmd_client_create(&name, &config))
            .await
            .unwrap()
    }

    async fn run_client_delete(client_id: String, config: Config) -> Result<(), CliError> {
        tokio::task::spawn_blocking(move || cmd_client_delete(&client_id, &config))
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_client_list_returns_clients() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/clients"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": sample_clients(),
                    "total": 2
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_client_list(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_client_list_quiet_suppresses_output() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/clients"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({
                    "data": sample_clients(),
                    "total": 2
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, true, false);
        let result = run_client_list(config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_client_list_returns_api_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/clients"))
            .respond_with(
                ResponseTemplate::new(403).set_body_json(serde_json::json!({
                    "type": "/errors/forbidden",
                    "title": "Forbidden",
                    "status": 403,
                    "detail": "Super admin required"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_client_list(config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 403);
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_client_create_returns_created_client() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        let created = ClientResponse {
            id: uuid::Uuid::parse_str("00000000-0000-0000-0000-00000000000a").unwrap(),
            name: "My Game".into(),
            created_at: chrono::DateTime::<chrono::Utc>::from_timestamp_millis(0).unwrap(),
            api_keys: vec![],
        };

        Mock::given(method("POST"))
            .and(path("/api/clients"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::to_value(&created).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_client_create("My Game".into(), config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_client_create_quiet_outputs_only_id() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        let created = ClientResponse {
            id: uuid::Uuid::parse_str("00000000-0000-0000-0000-00000000000b").unwrap(),
            name: "QuietGame".into(),
            created_at: chrono::DateTime::<chrono::Utc>::from_timestamp_millis(0).unwrap(),
            api_keys: vec![],
        };

        Mock::given(method("POST"))
            .and(path("/api/clients"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::to_value(&created).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, true, false);
        let result = run_client_create("QuietGame".into(), config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_client_create_duplicate_name_returns_409() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/clients"))
            .respond_with(
                ResponseTemplate::new(409).set_body_json(serde_json::json!({
                    "type": "/errors/conflict",
                    "title": "Conflict",
                    "status": 409,
                    "detail": "Client name 'My Game' already exists"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_client_create("My Game".into(), config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 409);
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_client_delete_succeeds() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();
        let client_id = "00000000-0000-0000-0000-000000000001";

        Mock::given(method("DELETE"))
            .and(path(format!("/api/clients/{client_id}")))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"deleted": true})),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_client_delete(client_id.to_string(), config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_client_delete_quiet_suppresses_output() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();
        let client_id = "00000000-0000-0000-0000-000000000002";

        Mock::given(method("DELETE"))
            .and(path(format!("/api/clients/{client_id}")))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"deleted": true})),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, true, false);
        let result = run_client_delete(client_id.to_string(), config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_client_delete_with_active_keys_returns_409() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();
        let client_id = "00000000-0000-0000-0000-000000000001";

        Mock::given(method("DELETE"))
            .and(path(format!("/api/clients/{client_id}")))
            .respond_with(
                ResponseTemplate::new(409).set_body_json(serde_json::json!({
                    "type": "/errors/delete-referenced-resource",
                    "title": "Delete Referenced Resource",
                    "status": 409,
                    "detail": "Client 00000000-0000-0000-0000-000000000001 has 3 active API key(s). Revoke all keys before deleting the client."
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_client_delete(client_id.to_string(), config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 409);
                assert_eq!(problem.type_, "/errors/delete-referenced-resource");
                let detail = problem.detail.as_deref().unwrap();
                assert!(detail.contains("3 active API key"));
                assert!(detail.contains("Revoke all keys"));
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_client_delete_not_found_returns_404() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();
        let client_id = "00000000-0000-0000-0000-000000000099";

        Mock::given(method("DELETE"))
            .and(path(format!("/api/clients/{client_id}")))
            .respond_with(
                ResponseTemplate::new(404).set_body_json(serde_json::json!({
                    "type": "/errors/not-found",
                    "title": "Not Found",
                    "status": 404,
                    "detail": "Client not found: 00000000-0000-0000-0000-000000000099"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_client_delete(client_id.to_string(), config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 404);
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_client_create_forbidden_returns_403() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/clients"))
            .respond_with(
                ResponseTemplate::new(403).set_body_json(serde_json::json!({
                    "type": "/errors/forbidden",
                    "title": "Forbidden",
                    "status": 403,
                    "detail": "Super admin required"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_client_create("ForbiddenGame".into(), config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 403);
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[test]
    fn test_client_list_connection_refused_returns_generic_error() {
        let config = make_config("http://127.0.0.1:1", false, false);
        let result = cmd_client_list(&config);
        assert!(result.is_err());
        match &result {
            Err(CliError::Generic(msg)) => {
                assert!(
                    msg.contains("Failed to connect"),
                    "expected connect error, got: {msg}"
                );
            }
            _ => panic!("expected Generic error, got {:?}", result),
        }
    }
}

#[cfg(test)]
mod key_tests {
    use super::*;
    use arche_types::crud::{ApiKeySummary, CreateApiKeyResponse};
    use arche_types::Permission;
    use chrono::DateTime;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};
    use uuid::Uuid;

    fn make_config(api_url: &str, quiet: bool, verbose: bool) -> Config {
        Config {
            api_url: api_url.to_string(),
            api_key: None,
            verbose,
            quiet,
        }
    }

    fn sample_key_summaries() -> Vec<ApiKeySummary> {
        vec![
            ApiKeySummary {
                id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                name: "ci-key".into(),
                permissions: vec![Permission::Read, Permission::Generate],
                created_at: DateTime::from_timestamp_millis(0).unwrap(),
            },
            ApiKeySummary {
                id: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                name: "admin-key".into(),
                permissions: vec![Permission::Admin],
                created_at: DateTime::from_timestamp_millis(1).unwrap(),
            },
        ]
    }

    fn sample_create_response() -> CreateApiKeyResponse {
        CreateApiKeyResponse {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000003").unwrap(),
            name: "ci-key".into(),
            permissions: vec![Permission::Read, Permission::Generate],
            key: "arche_k_abc123secret".into(),
        }
    }

    async fn run_key_list(
        client_id: String,
        config: Config,
    ) -> Result<(), CliError> {
        tokio::task::spawn_blocking(move || cmd_key_list(&client_id, &config))
            .await
            .unwrap()
    }

    async fn run_key_create(
        client_id: String,
        name: String,
        permissions: String,
        config: Config,
    ) -> Result<(), CliError> {
        tokio::task::spawn_blocking(move || {
            cmd_key_create(&client_id, &name, &permissions, &config)
        })
        .await
        .unwrap()
    }

    async fn run_key_revoke(
        client_id: String,
        key_id: String,
        config: Config,
    ) -> Result<(), CliError> {
        tokio::task::spawn_blocking(move || cmd_key_revoke(&client_id, &key_id, &config))
            .await
            .unwrap()
    }

    #[test]
    fn test_parse_permissions_valid() {
        let perms = parse_permissions("read,write,delete,generate,admin").unwrap();
        assert_eq!(
            perms,
            vec![
                Permission::Read,
                Permission::Write,
                Permission::Delete,
                Permission::Generate,
                Permission::Admin,
            ]
        );
    }

    #[test]
    fn test_parse_permissions_single() {
        let perms = parse_permissions("read").unwrap();
        assert_eq!(perms, vec![Permission::Read]);
    }

    #[test]
    fn test_parse_permissions_with_spaces() {
        let perms = parse_permissions("read, write , generate").unwrap();
        assert_eq!(
            perms,
            vec![Permission::Read, Permission::Write, Permission::Generate]
        );
    }

    #[test]
    fn test_parse_permissions_invalid() {
        let result = parse_permissions("read,invalid,generate");
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(msg)) => {
                assert!(msg.contains("Invalid permission: 'invalid'"));
                assert_eq!(CliError::Args(msg.clone()).exit_code(), 2);
            }
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_permissions_case_insensitive() {
        let perms = parse_permissions("Read,GENERATE,Admin").unwrap();
        assert_eq!(
            perms,
            vec![Permission::Read, Permission::Generate, Permission::Admin]
        );
    }

    #[test]
    fn test_parse_permissions_empty_string() {
        let perms = parse_permissions("").unwrap();
        assert!(perms.is_empty());
    }

    #[tokio::test]
    async fn test_key_list_success() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/clients/client-123/keys"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::to_value(sample_key_summaries()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_key_list("client-123".into(), config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_key_list_client_not_found_returns_api_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/clients/nonexistent/keys"))
            .respond_with(
                ResponseTemplate::new(404).set_body_json(serde_json::json!({
                    "type": "/errors/not-found",
                    "title": "Client not found",
                    "status": 404,
                    "detail": "Client 'nonexistent' not found"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_key_list("nonexistent".into(), config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 404);
                assert_eq!(problem.detail, Some("Client 'nonexistent' not found".into()));
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_key_list_permission_denied_returns_api_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/clients/client-123/keys"))
            .respond_with(
                ResponseTemplate::new(403).set_body_json(serde_json::json!({
                    "type": "/errors/forbidden",
                    "title": "Forbidden",
                    "status": 403,
                    "detail": "Insufficient permissions"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_key_list("client-123".into(), config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 403);
                assert_eq!(problem.detail, Some("Insufficient permissions".into()));
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_key_list_quiet_mode_suppresses_output() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/clients/client-123/keys"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::to_value(sample_key_summaries()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, true, false);
        let result = run_key_list("client-123".into(), config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_key_create_success() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/clients/client-123/keys"))
            .respond_with(
                ResponseTemplate::new(201)
                    .set_body_json(serde_json::to_value(sample_create_response()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_key_create(
            "client-123".into(),
            "ci-key".into(),
            "read,generate".into(),
            config,
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_key_create_invalid_permissions_returns_args_error() {
        let config = make_config("http://localhost:1", false, false);
        let result = run_key_create(
            "client-123".into(),
            "ci-key".into(),
            "read,invalid".into(),
            config,
        )
        .await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(msg)) => {
                assert!(msg.contains("Invalid permission: 'invalid'"));
                assert_eq!(CliError::Args(msg.clone()).exit_code(), 2);
            }
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_key_create_empty_permissions_returns_args_error() {
        let config = make_config("http://localhost:1", false, false);
        let result = run_key_create(
            "client-123".into(),
            "ci-key".into(),
            "".into(),
            config,
        )
        .await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Args(msg)) => {
                assert!(msg.contains("At least one permission is required"));
                assert_eq!(CliError::Args(msg.clone()).exit_code(), 2);
            }
            _ => panic!("expected Args error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_key_create_client_not_found_returns_api_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/clients/nonexistent/keys"))
            .respond_with(
                ResponseTemplate::new(404).set_body_json(serde_json::json!({
                    "type": "/errors/not-found",
                    "title": "Client not found",
                    "status": 404,
                    "detail": "Client 'nonexistent' not found"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_key_create(
            "nonexistent".into(),
            "ci-key".into(),
            "read,generate".into(),
            config,
        )
        .await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 404);
                assert_eq!(problem.detail, Some("Client 'nonexistent' not found".into()));
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_key_create_permission_denied_returns_api_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("POST"))
            .and(path("/api/clients/client-123/keys"))
            .respond_with(
                ResponseTemplate::new(403).set_body_json(serde_json::json!({
                    "type": "/errors/forbidden",
                    "title": "Forbidden",
                    "status": 403,
                    "detail": "Insufficient permissions"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_key_create(
            "client-123".into(),
            "ci-key".into(),
            "read,generate".into(),
            config,
        )
        .await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 403);
                assert_eq!(problem.detail, Some("Insufficient permissions".into()));
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_key_revoke_success() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("DELETE"))
            .and(path("/api/clients/client-123/keys/key-456"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"deleted": true})),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_key_revoke("client-123".into(), "key-456".into(), config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_key_revoke_not_found_returns_api_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("DELETE"))
            .and(path("/api/clients/client-123/keys/nonexistent-key"))
            .respond_with(
                ResponseTemplate::new(404).set_body_json(serde_json::json!({
                    "type": "/errors/not-found",
                    "title": "Key not found",
                    "status": 404,
                    "detail": "API key 'nonexistent-key' not found"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result =
            run_key_revoke("client-123".into(), "nonexistent-key".into(), config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 404);
                assert_eq!(problem.detail, Some("API key 'nonexistent-key' not found".into()));
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_key_revoke_permission_denied_returns_api_error() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("DELETE"))
            .and(path("/api/clients/client-123/keys/key-456"))
            .respond_with(
                ResponseTemplate::new(403).set_body_json(serde_json::json!({
                    "type": "/errors/forbidden",
                    "title": "Forbidden",
                    "status": 403,
                    "detail": "Insufficient permissions"
                })),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = run_key_revoke("client-123".into(), "key-456".into(), config).await;
        assert!(result.is_err());
        match &result {
            Err(CliError::Api(problem)) => {
                assert_eq!(problem.status, 403);
                assert_eq!(problem.detail, Some("Insufficient permissions".into()));
            }
            _ => panic!("expected Api error, got {:?}", result),
        }
    }

    #[tokio::test]
    async fn test_key_revoke_quiet_mode_suppresses_output() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("DELETE"))
            .and(path("/api/clients/client-123/keys/key-456"))
            .respond_with(
                ResponseTemplate::new(200).set_body_json(serde_json::json!({"deleted": true})),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, true, false);
        let result = run_key_revoke("client-123".into(), "key-456".into(), config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_key_list_verbose_shows_request_details() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/clients/client-123/keys"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::to_value(sample_key_summaries()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, true);
        let result = run_key_list("client-123".into(), config).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_key_cmd_dispatches_to_list() {
        let mock_server = MockServer::start().await;
        let uri = mock_server.uri();

        Mock::given(method("GET"))
            .and(path("/api/clients/client-123/keys"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::to_value(sample_key_summaries()).unwrap()),
            )
            .mount(&mock_server)
            .await;

        let config = make_config(&uri, false, false);
        let result = tokio::task::spawn_blocking(move || {
            cmd_key(
                &KeyCommand::List {
                    client_id: "client-123".into(),
                },
                &config,
            )
        })
        .await
        .unwrap();
        assert!(result.is_ok());
    }
}
