---
title: arche export command
status: completed
---

## Parent

Arche CLI PRD (see `.scratch/arche-cli/prd.md`)

## What to build

Implement the `arche export` command:

- `arche export` — calls `POST /api/export` and downloads the ZIP archive
- Flags:
  - `--output` — output file path (default: write to stdout as binary stream)
  - `--clients` — comma-separated client UUIDs (default: all)
  - `--include-api-keys` — include API keys in export
  - `--include-audit-log` — include audit log in export
  - `--inline-refs` — resolve `$ref_id` to inline values

- If `--output` is specified, the ZIP bytes are written to the file
- If `--output` is not specified, the raw ZIP bytes are written to stdout (suitable for piping: `arche export > backup.zip`)
- Progress indicator when writing to file (optional, via `indicatif` or similar)

## Acceptance criteria

- [ ] `arche export --output backup.zip` writes a valid ZIP file
- [ ] `arche export` (no flags) writes ZIP bytes to stdout
- [ ] `--clients` filters which clients are exported
- [ ] `--include-api-keys` includes keys in export
- [ ] `--include-audit-log` includes audit log
- [ ] `--inline-refs` resolves references
- [ ] Error handling: API unreachable, permission denied
- [ ] Integration test: export and verify ZIP contents

## Blocked by

- arche-cli/cli-scaffolding.md
- arche-service/export-endpoint.md
