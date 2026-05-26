---
title: arche key subcommands
status: completed
---

## Parent

Arche CLI PRD (see `.scratch/arche-cli/prd.md`)

## What to build

Implement API key management subcommands:

- `arche key list --client-id <uuid>` — calls `GET /api/clients/:id/keys`. Outputs a table or JSON list of keys (id, name, permissions, created_at).

- `arche key create --client-id <uuid> --name <name> --permissions <perms>` — calls `POST /api/clients/:id/keys`. Permissions is a comma-separated list: `read,write,delete,generate,admin`. Outputs the raw API key with a prominent warning: "This key will not be shown again."

- `arche key revoke --key-id <uuid>` — calls `DELETE /api/clients/:id/keys/:key_id`. Confirms with "Key revoked" message.

Output format follows the CLI-wide conventions: JSON by default (pipeable), optional pretty format.

## Acceptance criteria

- [ ] `arche key list --client-id <uuid>` lists keys in JSON
- [ ] `arche key create --client-id <uuid> --name ci-key --permissions read,generate` creates key, displays raw key once
- [ ] Warning printed: "This key will not be shown again. Copy it now."
- [ ] Invalid permissions string: validation error, exit code 2
- [ ] `arche key revoke --key-id <uuid>` revokes key
- [ ] Error handling: client not found (404), permission denied (403)
- [ ] Integration tests against running API

## Blocked by

- arche-cli/cli-scaffolding.md
- arche-service/api-key-management.md
