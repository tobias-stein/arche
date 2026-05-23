---
title: Arche CLI
status: ready-for-agent
---

## Problem Statement

Game designers and operators working with Arche need a way to automate generation, import/export data, and bootstrap the service without a browser. The Admin UI is great for interactive management, but it cannot be scripted — CI/CD pipelines, deployment automation, and power users need a command-line interface.

Additionally, the initial service bootstrap (auto-generating the super admin API key and printing it) requires a CLI-accessible mechanism. Without a CLI, every automation workflow must craft raw HTTP requests against the REST API, which is cumbersome and error-prone.

## Solution

A Rust CLI binary named `arche` that shares its type definitions with the Arche service through a common library crate (`arche-types` or similar). The CLI provides:

- **`arche init`** — Bootstrap a new Arche instance (print the generated super admin key, verify DB connectivity).
- **`arche generate`** — Request generation from the service with inline constraints, output as JSON or formatted text.
- **`arche export`** — Export client data as a ZIP archive to stdout or a file.
- **`arche import`** — Import a ZIP archive with two modes: interactive (terminal prompts for conflicts) and file-based (`--dry-run` generates a conflicts JSON, user edits it, feeds back with `--resolve-file`).
- **`arche key`** — Manage API keys (list, create, revoke).
- **`arche client`** — Manage clients (list, create, delete).

The CLI authenticates via the same `X-API-Key` header model, accepting the key via `--api-key` flag or `ARCHE_API_KEY` environment variable.

## User Stories

1. As a devops engineer, I want to bootstrap a fresh Arche instance with `arche init`, so that I get the super admin API key printed to stdout without manually reading server logs.
2. As a game developer, I want to run `arche generate --archetype sword --constraints '{"damage":{"gte":15}}'` from a shell script, so that I can integrate procedural generation into my build pipeline.
3. As a game developer, I want the generate output to be JSON by default (pipeable to `jq`), so that I can process results programmatically.
4. As a game developer, I want optional human-friendly formatted output (`--format pretty`) for the generate command, so that I can visually inspect results in the terminal.
5. As a game developer, I want to provide a seed (`--seed 12345`) for reproducible generation requests, so that I can debug specific output deterministically.
6. As a game developer, I want to export all clients' data to a ZIP file with `arche export --output backup.zip`, so that I can create snapshots for backup or migration.
7. As a game developer, I want to export specific clients only (`--clients uuid-1,uuid-2`), so that I can selectively back up data.
8. As a devops engineer, I want to include API keys and audit logs in exports (`--include-api-keys --include-audit-log`), so that complete service state can be captured.
9. As a devops engineer, I want to inline global `$ref_id` references in exports (`--inline-refs`), so that the export is self-contained and doesn't depend on the global attribute pool.
10. As a game developer, I want to import a previously exported ZIP archive with `arche import backup.zip`, so that I can restore or migrate data.
11. As a game developer, I want the import command to detect conflicts and prompt interactively for resolution when run without flags, so that I have full control over the import process.
12. As a devops engineer, I want `arche import --dry-run backup.zip` to write a conflicts JSON file, so that I can review and edit resolutions offline before applying.
13. As a devops engineer, I want `arche import --resolve-file resolutions.json backup.zip` to apply pre-defined resolutions non-interactively, so that imports can be automated in CI/CD.
14. As a game developer, I want to list API keys for a client with `arche key list --client-id uuid`, so that I can audit active keys from the terminal.
15. As a devops engineer, I want to create a new API key with `arche key create --client-id uuid --name ci-key --permissions read,generate`, so that I can provision keys in automated workflows.
16. As a devops engineer, I want to revoke an API key with `arche key revoke --key-id uuid`, so that I can rotate or decommission keys without the web UI.
17. As a game developer, I want to list all clients with `arche client list`, so that I can see what tenants exist.
18. As a devops engineer, I want to create a new client with `arche client create --name "My Game"`, so that I can onboard new projects programmatically.
19. As a devops engineer, I want to delete a client with `arche client delete --client-id uuid`, only if it has no active keys, so that I can clean up unused tenants safely.
20. As a game developer, I want all CLI commands to accept `--api-key` flag or `ARCHE_API_KEY` env var, so that I can choose the most convenient auth method for my context.
21. As a game developer, I want all CLI commands to accept `--api-url` flag or `ARCHE_API_URL` env var, so that I can target any Arche instance.
22. As a game developer, I want the CLI to return non-zero exit codes on errors with descriptive messages to stderr, so that shell scripts can detect failures.
23. As a devops engineer, I want `arche export` and `arche import` to use the same conflict-resolution schema as the Admin UI, so that resolution files written by one tool can be read by the other.

## Implementation Decisions

### Binary Structure

The CLI lives in its own Cargo workspace member (e.g., `crates/arche-cli/`). It depends on:
- A shared library crate (`arche-types` or `arche-core`) for domain types, API client types, and Zod-like validation.
- Clap for argument parsing.
- reqwest for HTTP calls to the Arche API.
- serde + serde_json for JSON serialization.

The CLI does **not** directly access the PostgreSQL database. All operations go through the Arche REST API. This keeps the CLI thin and ensures it always operates against the same consistency boundary as the Admin UI.

### Commands and Subcommands

```
arche
  init               Bootstrap instance, print super admin key
  generate            Request generation from the API
    --archetype       Filter by archetype
    --constraints     JSON string of attribute constraints
    --seed            Optional u64 seed
    --affixes         JSON string of affix constraints (min/max, require, block)
    --format          Output format: json (default) or pretty
  export              Export client data as ZIP
    --output          Output file path (default: stdout)
    --clients         Comma-separated client UUIDs (default: all)
    --include-api-keys
    --include-audit-log
    --inline-refs
  import              Import a ZIP archive
    <path>            Path to ZIP file
    --dry-run         Write conflicts JSON to stdout, do not import
    --resolve-file    Path to resolutions JSON file (non-interactive)
  key
    list --client-id  List keys for a client
    create            Create a new key
      --client-id     Client UUID
      --name          Key name
      --permissions   Comma-separated: read,write,delete,generate,admin
    revoke --key-id   Revoke a key
  client
    list              List all clients
    create --name     Create a new client
    delete            Delete a client
      --client-id     Client UUID
```

### Shared Library Crate

The CLI and service share a common crate (`arche-types`) containing:
- Domain structs: `Client`, `Blueprint`, `Affix`, `GlobalMetaAttribute`, `ApiKey`, `AuditLogEntry`
- Enums: `ValueType`, `AffixLocation`, `AuditAction`, `Permission`
- Request/response types for all API endpoints
- Validation logic (shared between API and CLI for consistency)
- The ZIP export/import format definitions
- The conflict-resolution schema (used by both Admin UI and CLI import flows)

This crate is published as a workspace dependency. The Admin UI does **not** use this crate (it uses the generated TypeScript client from JSON Schema), but the conflict-resolution schema format must be identical between CLI and Admin UI so that resolution files are portable.

### Authentication

- `--api-key` flag (highest priority)
- `ARCHE_API_KEY` environment variable (fallback)
- For `arche init`, no auth is needed (the endpoint may be unauthenticated or use a temporary bootstrap token).

### API Endpoint Dependencies

The CLI maps to these API endpoints:

| CLI Command | API Endpoint |
|---|---|
| `init` | Internal bootstrap (may not be a REST endpoint — could be a direct startup interaction) |
| `generate` | `POST /api/generate` |
| `export` | `POST /api/export` |
| `import` | `POST /api/import` + `POST /api/import/resolve` |
| `key list` | `GET /api/clients/:id/keys` |
| `key create` | `POST /api/clients/:id/keys` |
| `key revoke` | `DELETE /api/clients/:id/keys/:key_id` |
| `client list` | `GET /api/clients` |
| `client create` | `POST /api/clients` |
| `client delete` | `DELETE /api/clients/:id` |

### Import Conflict Resolution (File-Based)

The `--dry-run` output and `--resolve-file` input use the same schema:

```json
{
  "import_token": "uuid-from-service",
  "resolutions": {
    "<resource-uuid>": {
      "strategy": "keep_old" | "keep_new" | "per_attribute",
      "attributes": {
        "<attribute-key>": "keep_old" | "keep_new"
      }
    }
  }
}
```

This is the identical schema used by the Admin UI's import resolution phase (`POST /api/import/resolve`).

### Error Handling

- HTTP errors are printed to stderr with the RFC 9457 `detail` field.
- Non-zero exit codes: 1 for generic errors, 2 for invalid arguments, 3 for API errors.
- `--verbose` flag enables request/response logging to stderr.

### Output Format

- Default: JSON to stdout. If the response contains binary data (export ZIP), write to the specified file path or pipe raw bytes to stdout.
- Pretty mode (`--format pretty`): colored, formatted terminal output using `colored` or `termcolor` crate.
- All commands support `--quiet` to suppress non-error output (exit code only).

## Testing Decisions

### Testing Philosophy

The CLI is a thin wrapper around the REST API. Tests should focus on:
1. Argument parsing and validation (unit tests per subcommand)
2. Correct API call construction (mock HTTP server)
3. Output formatting (snapshot tests for JSON and pretty modes)
4. End-to-end with a real Arche service instance (integration, separate test suite)

### Modules to Test

| Module | Test Type | What to Test |
|--------|-----------|--------------|
| **Argument parsing** | Unit (clap tests) | Each subcommand parses flags correctly, errors on missing required args |
| **API client** | Integration (mock server) | Each command sends the correct HTTP request and handles responses |
| **Output formatter** | Unit | JSON output matches expected schema; pretty output is valid; `--quiet` suppresses output |
| **Import resolution (file-based)** | Unit | Resolution file parsing, merging with dry-run output, validation |
| **End-to-end** | Integration (real API) | Full flow: init → login → create client → create resources → export → import |

### Prior Art

No existing tests in this codebase. The Rust tests follow standard Cargo conventions:
- Unit tests: `#[cfg(test)] mod tests { ... }` co-located with the code
- Integration tests: `tests/` directory at the crate root
- Mock HTTP server: `wiremock` or `httpmock` crate
- Snapshot testing: `insta` crate for output format snapshots
- End-to-end: separate test binary that spins up a temporary Arche service (or connects to a configured test instance)

## Out of Scope

- **Direct database access** — the CLI never reads or writes the database directly. All operations go through the REST API.
- **Interactive configuration wizards** — beyond the interactive import conflict prompts, the CLI does not offer `arche configure` or setup wizards.
- **File watcher / watch mode** — no `arche generate --watch` or similar.
- **Webhook triggers** — the CLI does not subscribe to cache invalidation events or webhooks.
- **Admin UI rendering** — the CLI is text-based only.

## Further Notes

- The `arche init` command may require the Arche service to expose a special bootstrap endpoint (e.g., `GET /api/bootstrap` that returns the super admin key if one exists, or triggers creation if none exists). If the service auto-prints the key on stdout at startup, `arche init` might instead parse server logs or check a known file path — this interaction model is an open question that should be resolved during implementation.
- The CLI binary name `arche` must not conflict with any system binaries. Install via `cargo install arche` or download a prebuilt release from GitHub.
- The shared types crate enables a "Rust-first" API contract — the service's OpenAPI/JSON Schema is derived from the types, not the other way around.
- Import resolution files produced by the Admin UI's export should be consumable by the CLI's `--resolve-file` and vice versa, guaranteeing cross-tool compatibility.
