---
title: arche generate command
status: completed
---

## Parent

Arche CLI PRD (see `.scratch/arche-cli/prd.md`)

## What to build

Implement the `arche generate` command:

- `arche generate` — calls `POST /api/generate` with optional constraints
- Flags:
  - `--archetype` — filter by archetype string
  - `--constraints` — JSON string of attribute constraints (e.g., `'{"damage":{"gte":15}}'`)
  - `--seed` — optional u64 seed
  - `--affixes` — JSON string of affix constraints (min/max, require, block)
  - `--format` — output format: `json` (default, pipeable to `jq`) or `pretty` (colored terminal output)

- JSON output: the raw API response written to stdout
- Pretty output: formatted display with colors using `colored` or `termcolor` crate. Shows name prominently, blueprint attributes in a table, affix attributes in a table.
- The `--seed` flag enables reproducible generation
- The `--api-key` / `ARCHE_API_KEY` auth applies to this command like all others

## Acceptance criteria

- [ ] `arche generate` (no flags) returns a random result
- [ ] `--archetype sword` filters by archetype
- [ ] `--constraints` accepts valid JSON string, sends correct API request
- [ ] `--seed 12345` produces deterministic output
- [ ] `--affixes` sends affix constraints correctly
- [ ] `--format json` outputs raw JSON to stdout
- [ ] `--format pretty` outputs colored, formatted terminal output
- [ ] 404 no-match: error message to stderr, exit code 3
- [ ] Integration test: run generate against real API, verify output

## Blocked by

- arche-cli/cli-scaffolding.md
- arche-service/generate-endpoint.md
