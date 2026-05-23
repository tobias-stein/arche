---
title: CLI output formatting
status: ready-for-agent
---

## Parent

Arche CLI PRD (see `.scratch/arche-cli/prd.md`)

## What to build

Implement the CLI-wide output formatting layer used by all commands:

1. **JSON mode** (default for all commands except explicit binary):
   - Serialize response data to pretty-printed JSON
   - Write to stdout
   - Pipeable with `jq` and other tools

2. **Pretty mode** (`--format pretty`):
   - Colored, formatted terminal output using `colored` or `termcolor`
   - Different format per command:
     - `generate`: name in bold + underlined, attributes in a table, affixes in a table
     - `export`: progress bar (optional)
     - `key list`: table with columns
     - `client list`: table with columns
     - `import`: colored diff of conflicts, progress indicator

3. **Quiet mode** (`--quiet`):
   - Suppress all stdout output (except raw key for `key create`)
   - Exit code only
   - Errors still go to stderr

4. **Error output**: all errors go to stderr. RFC 9457 `detail` field is the primary error message.

## Acceptance criteria

- [ ] JSON output: valid, pretty-printed JSON to stdout
- [ ] Pretty output: colored, formatted, readable terminal output
- [ ] Quiet mode: no stdout, exit code only
- [ ] Error output goes to stderr, never stdout
- [ ] `--verbose` adds request/response details to stderr
- [ ] Exit codes: 0 success, 1 generic, 2 args, 3 API error
- [ ] Unit tests for output formatting with snapshots

## Blocked by

- arche-cli/cli-scaffolding.md
