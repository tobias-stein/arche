use std::io::{self, Write};
use std::cell::RefCell;
use std::rc::Rc;

use arche_types::crud::{ApiKeySummary, ClientResponse, CreateApiKeyResponse};
use arche_types::export_import::ImportConflictResponse;
use arche_types::generate::GenerateResponse;
use colored::Colorize;
use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, Color, Table};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Pretty,
    Quiet,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "pretty" => Ok(OutputFormat::Pretty),
            _ => Err(format!("unknown output format: {s} (expected 'json' or 'pretty')")),
        }
    }
}

#[derive(Clone)]
pub struct SharedBuffer {
    inner: Rc<RefCell<Vec<u8>>>,
}

impl Default for SharedBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedBuffer {
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn contents(&self) -> String {
        String::from_utf8_lossy(&self.inner.borrow()).to_string()
    }
}

impl io::Write for SharedBuffer {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.borrow_mut().flush()
    }
}

pub struct OutputWriter {
    format: OutputFormat,
    verbose: bool,
    stdout: SharedBuffer,
    stderr: SharedBuffer,
}

impl OutputWriter {
    pub fn new(format: OutputFormat, verbose: bool) -> Self {
        let stdout = SharedBuffer::new();
        let stderr = SharedBuffer::new();
        Self {
            format,
            verbose,
            stdout,
            stderr,
        }
    }

    pub fn stdout(&self) -> &SharedBuffer {
        &self.stdout
    }

    pub fn stderr(&self) -> &SharedBuffer {
        &self.stderr
    }

    pub fn format(&self) -> OutputFormat {
        self.format
    }

    pub fn log_verbose(&mut self, msg: &str) {
        if self.verbose {
            let _ = writeln!(self.stderr, "{}", msg.dimmed());
        }
    }

    pub fn write_json<T: Serialize>(&mut self, data: &T) -> io::Result<()> {
        match self.format {
            OutputFormat::Quiet => Ok(()),
            OutputFormat::Json | OutputFormat::Pretty => {
                let json = serde_json::to_string_pretty(data)?;
                writeln!(self.stdout, "{json}")
            }
        }
    }

    pub fn write_raw(&mut self, data: &str) -> io::Result<()> {
        match self.format {
            OutputFormat::Quiet => Ok(()),
            OutputFormat::Json | OutputFormat::Pretty => writeln!(self.stdout, "{data}"),
        }
    }

    pub fn write_generate(&mut self, response: &GenerateResponse) -> io::Result<()> {
        match self.format {
            OutputFormat::Quiet => Ok(()),
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(response)?;
                writeln!(self.stdout, "{json}")
            }
            OutputFormat::Pretty => {
                writeln!(self.stdout)?;
                writeln!(
                    self.stdout,
                    "{}",
                    response.name.bold().underline()
                )?;
                writeln!(self.stdout)?;

                let mut attrs = build_table(vec![
                        Cell::new("Attribute").fg(Color::Cyan),
                        Cell::new("Value").fg(Color::Cyan),
                    ]);
                for (key, value) in &response.blueprint_attributes {
                    attrs.add_row(vec![Cell::new(key), Cell::new(value)]);
                }
                writeln!(self.stdout, "{}", attrs)?;

                if !response.affix_attributes.is_empty() {
                    let mut affixes = build_table(vec![
                            Cell::new("Affix").fg(Color::Cyan),
                            Cell::new("ID").fg(Color::Cyan),
                            Cell::new("Attributes").fg(Color::Cyan),
                        ]);
                    for affix in &response.affix_attributes {
                        affixes.add_row(vec![
                            Cell::new(&affix.affix_name),
                            Cell::new(affix.affix_id.to_string()),
                            Cell::new(&affix.attributes),
                        ]);
                    }
                    writeln!(self.stdout, "{}", affixes)?;
                }

                writeln!(
                    self.stdout,
                    "  Seed: {}",
                    response.seed.to_string().yellow()
                )?;
                Ok(())
            }
        }
    }

    pub fn write_key_list(&mut self, keys: &[ApiKeySummary]) -> io::Result<()> {
        match self.format {
            OutputFormat::Quiet => Ok(()),
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&keys)?;
                writeln!(self.stdout, "{json}")
            }
            OutputFormat::Pretty => {
                let mut table = build_table(vec![
                        Cell::new("ID").fg(Color::Cyan),
                        Cell::new("Name").fg(Color::Cyan),
                        Cell::new("Permissions").fg(Color::Cyan),
                        Cell::new("Created").fg(Color::Cyan),
                    ]);
                for key in keys {
                    let perms: Vec<String> =
                        key.permissions.iter().map(|p| format!("{p:?}")).collect();
                    table.add_row(vec![
                        Cell::new(key.id.to_string()),
                        Cell::new(&key.name),
                        Cell::new(perms.join(", ")),
                        Cell::new(key.created_at.to_rfc3339()),
                    ]);
                }
                writeln!(self.stdout, "{}", table)
            }
        }
    }

    pub fn write_key_created(&mut self, response: &CreateApiKeyResponse) -> io::Result<()> {
        match self.format {
            OutputFormat::Quiet => {
                writeln!(self.stdout, "{}", response.key)
            }
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(response)?;
                writeln!(self.stdout, "{json}")
            }
            OutputFormat::Pretty => {
                writeln!(self.stdout, "API key created:")?;
                writeln!(self.stdout, "  ID:          {}", response.id)?;
                writeln!(self.stdout, "  Name:        {}", response.name)?;
                let perms: Vec<String> =
                    response.permissions.iter().map(|p| format!("{p:?}")).collect();
                writeln!(self.stdout, "  Permissions: {}", perms.join(", "))?;
                writeln!(
                    self.stdout,
                    "  Key:         {}",
                    response.key.bold()
                )?;
                Ok(())
            }
        }
    }

    pub fn write_key_revoked(&mut self) -> io::Result<()> {
        match self.format {
            OutputFormat::Quiet => Ok(()),
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&serde_json::json!({"message": "Key revoked."}))?;
                writeln!(self.stdout, "{json}")
            }
            OutputFormat::Pretty => {
                writeln!(self.stdout, "{}", "Key revoked.".green())
            }
        }
    }

    pub fn write_client_list(&mut self, clients: &[ClientResponse]) -> io::Result<()> {
        match self.format {
            OutputFormat::Quiet => Ok(()),
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(&clients)?;
                writeln!(self.stdout, "{json}")
            }
            OutputFormat::Pretty => {
                let mut table = build_table(vec![
                        Cell::new("ID").fg(Color::Cyan),
                        Cell::new("Name").fg(Color::Cyan),
                        Cell::new("Created").fg(Color::Cyan),
                        Cell::new("Keys").fg(Color::Cyan),
                    ]);
                for client in clients {
                    table.add_row(vec![
                        Cell::new(client.id.to_string()),
                        Cell::new(&client.name),
                        Cell::new(client.created_at.to_rfc3339()),
                        Cell::new(client.api_keys.len()),
                    ]);
                }
                writeln!(self.stdout, "{}", table)
            }
        }
    }

    pub fn write_import_conflicts(
        &mut self,
        conflict_response: &ImportConflictResponse,
    ) -> io::Result<()> {
        match self.format {
            OutputFormat::Quiet => Ok(()),
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(conflict_response)?;
                writeln!(self.stdout, "{json}")
            }
            OutputFormat::Pretty => {
                writeln!(
                    self.stdout,
                    "{}",
                    "Import conflicts detected:".red().bold()
                )?;
                writeln!(self.stdout)?;
                for conflict in &conflict_response.conflicts {
                    writeln!(
                        self.stdout,
                        "  {} {} ({})",
                        "→".red(),
                        conflict.resource_name.yellow(),
                        conflict.resource_type,
                    )?;
                    writeln!(
                        self.stdout,
                        "    ID: {}",
                        conflict.resource_id.to_string().dimmed()
                    )?;
                    for attr in &conflict.attributes {
                        writeln!(self.stdout, "    {} {}", "-".yellow(), attr.key)?;
                        writeln!(
                            self.stdout,
                            "      old: {}",
                            format_attribute_value(&attr.old_value).red()
                        )?;
                        writeln!(
                            self.stdout,
                            "      new: {}",
                            format_attribute_value(&attr.new_value).green()
                        )?;
                    }
                    writeln!(self.stdout)?;
                }
                Ok(())
            }
        }
    }

    pub fn flush_to_stdio(&self) {
        let stdout = self.stdout.contents();
        if !stdout.is_empty() {
            print!("{stdout}");
        }
        let stderr = self.stderr.contents();
        if !stderr.is_empty() {
            eprint!("{stderr}");
        }
    }

    pub fn write_error(&mut self, error: &crate::error::CliError) -> io::Result<()> {
        writeln!(self.stderr, "{}", error.display_message())
    }
}

fn build_table(headers: Vec<Cell>) -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_header(headers);
    table
}

fn format_attribute_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

pub fn print_success(msg: &str, quiet: bool) {
    if !quiet {
        println!("{msg}");
    }
}

pub fn print_error(msg: &str) {
    eprintln!("Error: {msg}");
}

pub fn print_verbose(msg: &str, verbose: bool) {
    if verbose {
        eprintln!("[verbose] {msg}");
    }
}

pub fn write_binary(data: &[u8], path: Option<&str>) -> Result<(), std::io::Error> {
    if let Some(path) = path {
        std::fs::write(path, data)
    } else {
        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        handle.write_all(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arche_types::common::ProblemJson;
    use arche_types::crud::CreateApiKeyResponse;
    use arche_types::export_import::{ConflictAttribute, ConflictDetail};
    use arche_types::generate::{AffixAttributeEntry, NameParts};
    use arche_types::{Permission, ValueType};
    use chrono::{DateTime, Utc};
    use std::collections::BTreeMap;
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn test_output_format_from_str() {
        assert_eq!(OutputFormat::from_str("json").unwrap(), OutputFormat::Json);
        assert_eq!(
            OutputFormat::from_str("pretty").unwrap(),
            OutputFormat::Pretty
        );
        assert!(OutputFormat::from_str("other").is_err());
    }

    #[test]
    fn test_write_json_mode_serializes_to_pretty_json() {
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        let data = serde_json::json!({"key": "value", "num": 42});
        writer.write_json(&data).unwrap();
        let output = writer.stdout().contents();
        insta::assert_yaml_snapshot!("json_output", output);
    }

    #[test]
    fn test_write_json_mode_quiet_suppresses_output() {
        let mut writer = OutputWriter::new(OutputFormat::Quiet, false);
        let data = serde_json::json!({"key": "value"});
        writer.write_json(&data).unwrap();
        assert!(writer.stdout().contents().is_empty());
    }

    #[test]
    fn test_write_generate_json_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        let response = GenerateResponse {
            seed: 12345,
            name: "Fire Longsword of the Bear".into(),
            name_parts: NameParts {
                base: "Longsword".into(),
                prefixes: vec!["Fire".into()],
                suffixes: vec!["of the Bear".into()],
            },
            blueprint_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            blueprint_attributes: {
                let mut m = BTreeMap::new();
                m.insert("damage".into(), serde_json::json!(27.3));
                m.insert("weight".into(), serde_json::json!(3.5));
                m
            },
            affix_attributes: vec![AffixAttributeEntry {
                affix_id: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                affix_name: "Fire".into(),
                attributes: serde_json::json!({"fireDamage": 12.7}),
            }],
        };
        writer.write_generate(&response).unwrap();
        let output = writer.stdout().contents();
        insta::assert_yaml_snapshot!("generate_json_output", output);
    }

    #[test]
    fn test_write_generate_pretty_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Pretty, false);
        let response = GenerateResponse {
            seed: 12345,
            name: "Fire Longsword of the Bear".into(),
            name_parts: NameParts {
                base: "Longsword".into(),
                prefixes: vec!["Fire".into()],
                suffixes: vec!["of the Bear".into()],
            },
            blueprint_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            blueprint_attributes: {
                let mut m = BTreeMap::new();
                m.insert("damage".into(), serde_json::json!(27.3));
                m.insert("weight".into(), serde_json::json!(3.5));
                m
            },
            affix_attributes: vec![AffixAttributeEntry {
                affix_id: Uuid::parse_str("00000000-0000-0000-0000-000000000002").unwrap(),
                affix_name: "Fire".into(),
                attributes: serde_json::json!({"fireDamage": 12.7}),
            }],
        };
        writer.write_generate(&response).unwrap();
        let output = writer.stdout().contents();
        assert!(output.contains("Fire Longsword of the Bear"));
        assert!(output.contains("Seed:"));
    }

    #[test]
    fn test_write_generate_quiet_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Quiet, false);
        let response = GenerateResponse {
            seed: 0,
            name: "Sword".into(),
            name_parts: NameParts {
                base: "Sword".into(),
                prefixes: vec![],
                suffixes: vec![],
            },
            blueprint_id: Uuid::nil(),
            blueprint_attributes: BTreeMap::new(),
            affix_attributes: vec![],
        };
        writer.write_generate(&response).unwrap();
        assert!(writer.stdout().contents().is_empty());
    }

    #[test]
    fn test_write_key_list_json_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        let keys = vec![ApiKeySummary {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            name: "ci-key".into(),
            permissions: vec![Permission::Read, Permission::Generate],
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
        }];
        writer.write_key_list(&keys).unwrap();
        let output = writer.stdout().contents();
        insta::assert_yaml_snapshot!("key_list_json_output", output);
    }

    #[test]
    fn test_write_key_list_pretty_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Pretty, false);
        let keys = vec![ApiKeySummary {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            name: "ci-key".into(),
            permissions: vec![Permission::Read, Permission::Generate],
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
        }];
        writer.write_key_list(&keys).unwrap();
        let output = writer.stdout().contents();
        assert!(output.contains("ci-key"));
    }

    #[test]
    fn test_write_key_list_quiet_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Quiet, false);
        let keys = vec![ApiKeySummary {
            id: Uuid::nil(),
            name: "ci-key".into(),
            permissions: vec![],
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
        }];
        writer.write_key_list(&keys).unwrap();
        assert!(writer.stdout().contents().is_empty());
    }

    #[test]
    fn test_write_client_list_json_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        let clients = vec![ClientResponse {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            name: "My Game".into(),
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            api_keys: vec![],
        }];
        writer.write_client_list(&clients).unwrap();
        let output = writer.stdout().contents();
        insta::assert_yaml_snapshot!("client_list_json_output", output);
    }

    #[test]
    fn test_write_client_list_pretty_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Pretty, false);
        let clients = vec![ClientResponse {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            name: "My Game".into(),
            created_at: DateTime::<Utc>::from_timestamp_millis(0).unwrap(),
            api_keys: vec![],
        }];
        writer.write_client_list(&clients).unwrap();
        let output = writer.stdout().contents();
        assert!(output.contains("My Game"));
    }

    #[test]
    fn test_write_import_conflicts_json_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        let conflict = ImportConflictResponse {
            problem: ProblemJson {
                type_: "/errors/import-conflict".into(),
                title: "Import conflicts require resolution".into(),
                status: 409,
                detail: Some("2 conflicts found".into()),
            },
            conflicts: vec![ConflictDetail {
                resource_type: "blueprint".into(),
                resource_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                resource_name: "Longsword".into(),
                attributes: vec![ConflictAttribute {
                    key: "damage".into(),
                    old_value: serde_json::json!({"min": 10}),
                    new_value: serde_json::json!({"min": 15}),
                    value_type: ValueType::Range,
                }],
            }],
            import_token: "test-token-1".into(),
        };
        writer.write_import_conflicts(&conflict).unwrap();
        let output = writer.stdout().contents();
        insta::assert_yaml_snapshot!("import_conflicts_json_output", output);
    }

    #[test]
    fn test_write_key_created_pretty_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Pretty, false);
        let response = CreateApiKeyResponse {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            name: "ci-key".into(),
            permissions: vec![Permission::Read, Permission::Generate],
            key: "arche_k_abc123".into(),
        };
        writer.write_key_created(&response).unwrap();
        let output = writer.stdout().contents();
        assert!(output.contains("arche_k_abc123"));
        assert!(output.contains("API key created"));
    }

    #[test]
    fn test_write_key_created_json_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        let response = CreateApiKeyResponse {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            name: "ci-key".into(),
            permissions: vec![Permission::Read],
            key: "arche_k_abc123".into(),
        };
        writer.write_key_created(&response).unwrap();
        let output = writer.stdout().contents();
        assert!(output.contains("arche_k_abc123"));
        assert!(output.contains("ci-key"));
    }

    #[test]
    fn test_write_key_created_quiet_mode_outputs_raw_key() {
        let mut writer = OutputWriter::new(OutputFormat::Quiet, false);
        let response = CreateApiKeyResponse {
            id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
            name: "ci-key".into(),
            permissions: vec![Permission::Read],
            key: "arche_k_abc123".into(),
        };
        writer.write_key_created(&response).unwrap();
        let output = writer.stdout().contents();
        assert_eq!(output.trim(), "arche_k_abc123");
    }

    #[test]
    fn test_verbose_logging() {
        let mut writer = OutputWriter::new(OutputFormat::Json, true);
        writer.log_verbose("Request: GET /api/generate");
        writer.log_verbose("Response: 200 OK");
        let stderr = writer.stderr().contents();
        assert!(stderr.contains("Request: GET /api/generate"));
        assert!(stderr.contains("Response: 200 OK"));
    }

    #[test]
    fn test_verbose_logging_suppressed_when_not_verbose() {
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        writer.log_verbose("should not appear");
        assert!(writer.stderr().contents().is_empty());
    }

    #[test]
    fn test_error_output_goes_to_stderr() {
        use crate::error::CliError;
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        let error = CliError::Api(ProblemJson {
            type_: "/errors/not-found".into(),
            title: "Resource not found".into(),
            status: 404,
            detail: Some("Blueprint not found".into()),
        });
        writer.write_error(&error).unwrap();
        let stderr = writer.stderr().contents();
        let stdout = writer.stdout().contents();
        assert!(stderr.contains("Blueprint not found"));
        assert!(stdout.is_empty());
    }

    #[test]
    fn test_error_output_args_stderr() {
        use crate::error::CliError;
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        let error = CliError::Args("missing required argument".into());
        writer.write_error(&error).unwrap();
        let stderr = writer.stderr().contents();
        let stdout = writer.stdout().contents();
        assert!(stderr.contains("missing required argument"));
        assert!(stdout.is_empty());
    }

    #[test]
    fn test_error_output_generic_stderr() {
        use crate::error::CliError;
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        let error = CliError::Generic("connection refused".into());
        writer.write_error(&error).unwrap();
        let stderr = writer.stderr().contents();
        let stdout = writer.stdout().contents();
        assert!(stderr.contains("connection refused"));
        assert!(stdout.is_empty());
    }

    #[test]
    fn test_error_output_uses_detail_field_from_problem_json() {
        use crate::error::CliError;
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        let error = CliError::Api(ProblemJson {
            type_: "/errors/conflict".into(),
            title: "Conflict".into(),
            status: 409,
            detail: Some("Blueprint name already exists".into()),
        });
        writer.write_error(&error).unwrap();
        let stderr = writer.stderr().contents();
        assert!(stderr.contains("Blueprint name already exists"));
    }

    #[test]
    fn test_error_output_falls_back_to_title_when_no_detail() {
        use crate::error::CliError;
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        let error = CliError::Api(ProblemJson {
            type_: "/errors/conflict".into(),
            title: "Conflict".into(),
            status: 409,
            detail: None,
        });
        writer.write_error(&error).unwrap();
        let stderr = writer.stderr().contents();
        assert!(stderr.contains("Conflict"));
    }

    #[test]
    fn test_exit_codes() {
        use crate::error::CliError;
        assert_eq!(CliError::Args("x".into()).exit_code(), 2);
        assert_eq!(
            CliError::Api(ProblemJson {
                type_: "/x".into(),
                title: "x".into(),
                status: 400,
                detail: None,
            })
            .exit_code(),
            3
        );
        assert_eq!(CliError::Generic("x".into()).exit_code(), 1);
    }

    #[test]
    fn test_write_import_conflicts_pretty_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Pretty, false);
        let conflict = ImportConflictResponse {
            problem: ProblemJson {
                type_: "/errors/import-conflict".into(),
                title: "Import conflicts require resolution".into(),
                status: 409,
                detail: Some("1 conflict found".into()),
            },
            conflicts: vec![ConflictDetail {
                resource_type: "blueprint".into(),
                resource_id: Uuid::parse_str("00000000-0000-0000-0000-000000000001").unwrap(),
                resource_name: "Longsword".into(),
                attributes: vec![ConflictAttribute {
                    key: "damage".into(),
                    old_value: serde_json::json!(10),
                    new_value: serde_json::json!(15),
                    value_type: ValueType::Range,
                }],
            }],
            import_token: "test-token-2".into(),
        };
        writer.write_import_conflicts(&conflict).unwrap();
        let output = writer.stdout().contents();
        assert!(output.contains("Longsword"));
        assert!(output.contains("damage"));
    }

    #[test]
    fn test_write_import_conflicts_quiet_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Quiet, false);
        let conflict = ImportConflictResponse {
            problem: ProblemJson {
                type_: "/errors/import-conflict".into(),
                title: "Import conflicts".into(),
                status: 409,
                detail: None,
            },
            conflicts: vec![],
            import_token: "test-token-3".into(),
        };
        writer.write_import_conflicts(&conflict).unwrap();
        assert!(writer.stdout().contents().is_empty());
    }

    #[test]
    fn test_write_key_revoked_pretty() {
        let mut writer = OutputWriter::new(OutputFormat::Pretty, false);
        writer.write_key_revoked().unwrap();
        let output = writer.stdout().contents();
        assert!(output.contains("Key revoked"));
    }

    #[test]
    fn test_write_key_revoked_json() {
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        writer.write_key_revoked().unwrap();
        let output = writer.stdout().contents();
        assert!(output.contains("Key revoked"));
    }

    #[test]
    fn test_write_key_revoked_quiet() {
        let mut writer = OutputWriter::new(OutputFormat::Quiet, false);
        writer.write_key_revoked().unwrap();
        assert!(writer.stdout().contents().is_empty());
    }

    #[test]
    fn test_write_raw_json_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        writer.write_raw("hello world").unwrap();
        let output = writer.stdout().contents();
        assert_eq!(output, "hello world\n");
    }

    #[test]
    fn test_write_raw_quiet_mode() {
        let mut writer = OutputWriter::new(OutputFormat::Quiet, false);
        writer.write_raw("hello world").unwrap();
        assert!(writer.stdout().contents().is_empty());
    }

    #[test]
    fn test_json_output_is_valid_and_pretty_printed() {
        let mut writer = OutputWriter::new(OutputFormat::Json, false);
        let data = serde_json::json!({
            "name": "test",
            "nested": {"key": "value"},
            "array": [1, 2, 3]
        });
        writer.write_json(&data).unwrap();
        let output = writer.stdout().contents();
        let parsed: serde_json::Value = serde_json::from_str(output.trim()).unwrap();
        assert_eq!(parsed["name"], "test");
        assert!(output.contains('\n'));
    }
}
