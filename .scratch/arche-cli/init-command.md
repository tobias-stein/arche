---
title: arche init command
status: completed
---

## Parent

Arche CLI PRD (see `.scratch/arche-cli/prd.md`)

## What to build

Implement the `arche init` command that bootstraps a new Arche instance:

- `arche init` — connects to the Arche service, triggers bootstrap, receives the super admin API key
- The command calls the service's bootstrap endpoint (e.g., `POST /api/bootstrap` or `GET /api/bootstrap`) 
- If the instance has already been bootstrapped (key exists), the CLI prints a message indicating the key already exists and can be found in the server logs
- Outputs the super admin key with a clear banner and instructions to save it
- Optionally verifies DB connectivity by checking the health endpoint first
- Returns exit code 0 on success, non-zero on failure

The interaction model: the CLI makes an HTTP request to the Arche API, which triggers the bootstrap check on the server side. If no super admin key exists, the server generates one and returns it.

## Acceptance criteria

- [ ] `arche init` returns the super admin API key
- [ ] If already bootstrapped, prints informative message (key not available)
- [ ] Error handling: API unreachable, server error
- [ ] Verbose mode shows request details
- [ ] Quiet mode suppresses everything except the key
- [ ] Exit codes correct (0 success, 3 API error)
- [ ] Integration test against running Arche instance

## Blocked by

- arche-cli/cli-scaffolding.md
- arche-service/super-admin-bootstrap.md
