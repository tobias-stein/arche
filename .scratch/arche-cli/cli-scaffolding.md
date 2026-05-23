---
title: CLI scaffolding
status: ready-for-agent
---

## Parent

Arche CLI PRD (see `.scratch/arche-cli/prd.md`)

## What to build

Set up the `arche-cli` binary crate with the CLI framework:

1. **Clap argument parser**: define the CLI structure with all subcommands and flags as per the PRD. Use clap's derive API for type-safe argument parsing.

2. **Config resolution**: load API URL and API key from:
   - `--api-url` flag / `ARCHE_API_URL` env var (default: `http://localhost:8080`)
   - `--api-key` flag / `ARCHE_API_KEY` env var
   - Flags take precedence over env vars

3. **HTTP client**: `reqwest` client configured with the base URL and default headers (including `X-API-Key` when provided).

4. **Error handling**: custom error enum mapping to exit codes:
   - 0: success
   - 1: generic error
   - 2: invalid arguments
   - 3: API error (RFC 9457)

5. **Logging**: `--verbose` flag enables request/response logging to stderr. `--quiet` suppresses all non-error output.

6. **Output helpers**: functions for writing JSON to stdout, writing binary data to file/stdout, and pretty printing.

The CLI builds, parses all subcommands, and prints help text. API calls not yet implemented.

## Acceptance criteria

- [ ] `arche --help` shows all subcommands and global flags
- [ ] Each subcommand has its own `--help` with all flags
- [ ] Config resolves from flags > env vars > defaults
- [ ] `--verbose` logs request details to stderr
- [ ] `--quiet` suppresses non-error output
- [ ] Error exit codes: 1, 2, 3 as specified
- [ ] `cargo build` and `cargo test` pass

## Blocked by

- arche-types/workspace-scaffolding.md
- arche-types/domain-structs-enums.md
