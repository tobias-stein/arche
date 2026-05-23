---
title: arche client subcommands
status: ready-for-agent
---

## Parent

Arche CLI PRD (see `.scratch/arche-cli/prd.md`)

## What to build

Implement client management subcommands:

- `arche client list` — calls `GET /api/clients`. Outputs JSON array or table of clients (id, name, created_at).

- `arche client create --name "My Game"` — calls `POST /api/clients`. Outputs the created client object in JSON.

- `arche client delete --client-id <uuid>` — calls `DELETE /api/clients/:id`. Confirms deletion.
  - If the client has active keys, the API returns 409. The CLI prints the error detail: "Cannot delete client: it has N active API keys. Revoke all keys first."

All commands require super admin API key (the API enforces this). Non-super-admin keys will get a 403 which the CLI displays.

Output follows CLI conventions: JSON default, pretty optional, quiet mode.

## Acceptance criteria

- [ ] `arche client list` lists all clients in JSON
- [ ] `arche client create --name "My Game"` creates client
- [ ] `arche client delete --client-id <uuid>` deletes client
- [ ] Delete client with active keys: 409 error displayed, instruction to revoke keys first
- [ ] Error handling: 403 for non-super-admin, 404 for non-existent
- [ ] Integration tests against running API

## Blocked by

- arche-cli/cli-scaffolding.md
- arche-service/client-crud.md
