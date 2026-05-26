use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "arche", about = "Arche CLI - procedural generation management")]
pub struct Cli {
    #[command(flatten)]
    pub global_opts: GlobalOpts,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Args, Clone, Debug)]
pub struct GlobalOpts {
    #[arg(
        long = "api-url",
        env = "ARCHE_API_URL",
        default_value = "http://localhost:8080",
        global = true,
        help = "Arche API base URL"
    )]
    pub api_url: String,

    #[arg(
        long = "api-key",
        env = "ARCHE_API_KEY",
        global = true,
        help = "API key for authentication"
    )]
    pub api_key: Option<String>,

    #[arg(
        long,
        short = 'v',
        global = true,
        help = "Enable verbose request/response logging"
    )]
    pub verbose: bool,

    #[arg(
        long,
        short = 'q',
        global = true,
        help = "Suppress all non-error output"
    )]
    pub quiet: bool,
}

#[derive(Subcommand)]
pub enum Command {
    /// Bootstrap a new Arche instance
    Init,
    /// Request generation from the API
    Generate(GenerateArgs),
    /// Export client data as ZIP
    Export(ExportArgs),
    /// Import a ZIP archive
    Import(ImportArgs),
    /// Manage API keys
    #[command(subcommand)]
    Key(KeyCommand),
    /// Manage clients
    #[command(subcommand)]
    Client(ClientCommand),
}

#[derive(Args, Clone)]
pub struct GenerateArgs {
    #[arg(long, help = "Filter by archetype")]
    pub archetype: Option<String>,

    #[arg(long, help = "JSON string of attribute constraints")]
    pub constraints: Option<String>,

    #[arg(long, help = "Optional u64 seed for reproducible generation")]
    pub seed: Option<u64>,

    #[arg(long, help = "JSON string of affix constraints")]
    pub affixes: Option<String>,

    #[arg(long, default_value = "json", help = "Output format: json or pretty")]
    pub format: String,
}

#[derive(Args, Clone)]
pub struct ExportArgs {
    #[arg(long, help = "Output file path (default: stdout)")]
    pub output: Option<String>,

    #[arg(long, help = "Comma-separated client UUIDs (default: all)")]
    pub clients: Option<String>,

    #[arg(long, help = "Include API keys in export")]
    pub include_api_keys: bool,

    #[arg(long, help = "Include audit log in export")]
    pub include_audit_log: bool,

    #[arg(long, help = "Inline global $ref_id references in export")]
    pub inline_refs: bool,
}

#[derive(Args, Clone)]
pub struct ImportArgs {
    #[arg(help = "Path to ZIP file")]
    pub path: String,

    #[arg(long, help = "Write conflicts JSON to stdout, do not import")]
    pub dry_run: bool,

    #[arg(long, help = "Path to resolutions JSON file (non-interactive)")]
    pub resolve_file: Option<String>,
}

#[derive(Subcommand)]
pub enum KeyCommand {
    /// List API keys for a client
    List {
        #[arg(long, help = "Client UUID")]
        client_id: String,
    },
    /// Create a new API key
    Create {
        #[arg(long, help = "Client UUID")]
        client_id: String,
        #[arg(long, help = "Key name")]
        name: String,
        #[arg(
            long,
            help = "Comma-separated permissions: read,write,delete,generate,admin"
        )]
        permissions: String,
    },
    /// Revoke an API key
    Revoke {
        #[arg(long, help = "Client UUID")]
        client_id: String,
        #[arg(long, help = "Key UUID")]
        key_id: String,
    },
}

#[derive(Subcommand)]
pub enum ClientCommand {
    /// List all clients
    List,
    /// Create a new client
    Create {
        #[arg(long, help = "Client name")]
        name: String,
    },
    /// Delete a client
    Delete {
        #[arg(long, help = "Client UUID")]
        client_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_init() {
        let cli = Cli::try_parse_from(&["arche", "init"]).unwrap();
        assert!(matches!(cli.command, Command::Init));
    }

    #[test]
    fn test_parse_generate_defaults() {
        let cli = Cli::try_parse_from(&["arche", "generate"]).unwrap();
        match cli.command {
            Command::Generate(ref args) => {
                assert_eq!(args.format, "json");
                assert!(args.archetype.is_none());
                assert!(args.constraints.is_none());
                assert!(args.seed.is_none());
                assert!(args.affixes.is_none());
            }
            _ => panic!("expected Generate"),
        }
    }

    #[test]
    fn test_parse_generate_all_flags() {
        let cli = Cli::try_parse_from(&[
            "arche",
            "generate",
            "--archetype",
            "sword",
            "--constraints",
            r#"{"damage":{"gte":15}}"#,
            "--seed",
            "12345",
            "--affixes",
            r#"{"min":1,"max":2}"#,
            "--format",
            "pretty",
        ])
        .unwrap();
        match cli.command {
            Command::Generate(ref args) => {
                assert_eq!(args.archetype.as_deref(), Some("sword"));
                assert_eq!(
                    args.constraints.as_deref(),
                    Some(r#"{"damage":{"gte":15}}"#)
                );
                assert_eq!(args.seed, Some(12345));
                assert_eq!(args.affixes.as_deref(), Some(r#"{"min":1,"max":2}"#));
                assert_eq!(args.format, "pretty");
            }
            _ => panic!("expected Generate"),
        }
    }

    #[test]
    fn test_parse_export_defaults() {
        let cli = Cli::try_parse_from(&["arche", "export"]).unwrap();
        match cli.command {
            Command::Export(ref args) => {
                assert!(args.output.is_none());
                assert!(args.clients.is_none());
                assert!(!args.include_api_keys);
                assert!(!args.include_audit_log);
                assert!(!args.inline_refs);
            }
            _ => panic!("expected Export"),
        }
    }

    #[test]
    fn test_parse_export_all_flags() {
        let cli = Cli::try_parse_from(&[
            "arche",
            "export",
            "--output",
            "backup.zip",
            "--clients",
            "uuid-1,uuid-2",
            "--include-api-keys",
            "--include-audit-log",
            "--inline-refs",
        ])
        .unwrap();
        match cli.command {
            Command::Export(ref args) => {
                assert_eq!(args.output.as_deref(), Some("backup.zip"));
                assert_eq!(args.clients.as_deref(), Some("uuid-1,uuid-2"));
                assert!(args.include_api_keys);
                assert!(args.include_audit_log);
                assert!(args.inline_refs);
            }
            _ => panic!("expected Export"),
        }
    }

    #[test]
    fn test_parse_import() {
        let cli = Cli::try_parse_from(&["arche", "import", "backup.zip"]).unwrap();
        match cli.command {
            Command::Import(ref args) => {
                assert_eq!(args.path, "backup.zip");
                assert!(!args.dry_run);
                assert!(args.resolve_file.is_none());
            }
            _ => panic!("expected Import"),
        }
    }

    #[test]
    fn test_parse_import_dry_run() {
        let cli = Cli::try_parse_from(&["arche", "import", "backup.zip", "--dry-run"]).unwrap();
        match cli.command {
            Command::Import(ref args) => {
                assert!(args.dry_run);
            }
            _ => panic!("expected Import"),
        }
    }

    #[test]
    fn test_parse_import_resolve_file() {
        let cli = Cli::try_parse_from(&[
            "arche",
            "import",
            "backup.zip",
            "--resolve-file",
            "resolutions.json",
        ])
        .unwrap();
        match cli.command {
            Command::Import(ref args) => {
                assert_eq!(args.resolve_file.as_deref(), Some("resolutions.json"));
            }
            _ => panic!("expected Import"),
        }
    }

    #[test]
    fn test_parse_key_list() {
        let cli = Cli::try_parse_from(&["arche", "key", "list", "--client-id", "abc-123"]).unwrap();
        match cli.command {
            Command::Key(KeyCommand::List { client_id }) => {
                assert_eq!(client_id, "abc-123");
            }
            _ => panic!("expected Key List"),
        }
    }

    #[test]
    fn test_parse_key_create() {
        let cli = Cli::try_parse_from(&[
            "arche",
            "key",
            "create",
            "--client-id",
            "abc-123",
            "--name",
            "ci-key",
            "--permissions",
            "read,generate",
        ])
        .unwrap();
        match cli.command {
            Command::Key(KeyCommand::Create {
                client_id,
                name,
                permissions,
            }) => {
                assert_eq!(client_id, "abc-123");
                assert_eq!(name, "ci-key");
                assert_eq!(permissions, "read,generate");
            }
            _ => panic!("expected Key Create"),
        }
    }

    #[test]
    fn test_parse_key_revoke() {
        let cli = Cli::try_parse_from(&[
            "arche",
            "key",
            "revoke",
            "--client-id",
            "abc-123",
            "--key-id",
            "key-456",
        ])
        .unwrap();
        match cli.command {
            Command::Key(KeyCommand::Revoke {
                client_id,
                key_id,
            }) => {
                assert_eq!(client_id, "abc-123");
                assert_eq!(key_id, "key-456");
            }
            _ => panic!("expected Key Revoke"),
        }
    }

    #[test]
    fn test_parse_client_list() {
        let cli = Cli::try_parse_from(&["arche", "client", "list"]).unwrap();
        assert!(matches!(cli.command, Command::Client(ClientCommand::List)));
    }

    #[test]
    fn test_parse_client_create() {
        let cli = Cli::try_parse_from(&["arche", "client", "create", "--name", "My Game"]).unwrap();
        match cli.command {
            Command::Client(ClientCommand::Create { ref name }) => {
                assert_eq!(name, "My Game");
            }
            _ => panic!("expected Client Create"),
        }
    }

    #[test]
    fn test_parse_client_delete() {
        let cli =
            Cli::try_parse_from(&["arche", "client", "delete", "--client-id", "abc-123"]).unwrap();
        match cli.command {
            Command::Client(ClientCommand::Delete { ref client_id }) => {
                assert_eq!(client_id, "abc-123");
            }
            _ => panic!("expected Client Delete"),
        }
    }

    #[test]
    fn test_parse_global_flags() {
        let cli = Cli::try_parse_from(&[
            "arche",
            "--api-url",
            "http://example.com",
            "--api-key",
            "sk-abc",
            "-v",
            "-q",
            "init",
        ])
        .unwrap();
        assert_eq!(cli.global_opts.api_url, "http://example.com");
        assert_eq!(cli.global_opts.api_key.as_deref(), Some("sk-abc"));
        assert!(cli.global_opts.verbose);
        assert!(cli.global_opts.quiet);
    }

    #[test]
    fn test_parse_global_flags_short() {
        let cli = Cli::try_parse_from(&["arche", "-v", "init"]).unwrap();
        assert!(cli.global_opts.verbose);
    }

    #[test]
    fn test_parse_quiet_short() {
        let cli = Cli::try_parse_from(&["arche", "-q", "init"]).unwrap();
        assert!(cli.global_opts.quiet);
    }

    #[test]
    fn test_parse_help_does_not_panic() {
        let _ = Cli::try_parse_from(&["arche", "--help"]);
    }

    #[test]
    fn test_parse_subcommand_help_does_not_panic() {
        let _ = Cli::try_parse_from(&["arche", "generate", "--help"]);
        let _ = Cli::try_parse_from(&["arche", "export", "--help"]);
        let _ = Cli::try_parse_from(&["arche", "import", "--help"]);
        let _ = Cli::try_parse_from(&["arche", "key", "--help"]);
        let _ = Cli::try_parse_from(&["arche", "key", "list", "--help"]);
        let _ = Cli::try_parse_from(&["arche", "client", "--help"]);
        let _ = Cli::try_parse_from(&["arche", "client", "list", "--help"]);
    }
}
