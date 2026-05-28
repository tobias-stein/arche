---
title: Client CRUD
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement the three client management endpoints, all super-admin only:

- `POST /api/clients` — Create a client. Request: `{ "name": "My Game" }`. Response: the created client object with id, name, timestamps, empty api_keys array.
- `GET /api/clients` — List all clients. Returns array of client objects.
- `DELETE /api/clients/:id` — Delete a client. Rejected with 409 if the client still has active API keys. On success, cascade-deletes all client data (blueprints, affixes, global meta attributes, keys).

Client names must be unique (enforced by DB unique constraint). Name validation: non-empty, `[a-zA-Z0-9_]` only.

## Acceptance criteria

- [ ] Create client with valid name returns 201 with client object
- [ ] Create client with duplicate name returns 409
- [ ] Create client with invalid name characters returns 400
- [ ] List clients returns all clients (super admin only)
- [ ] Delete client with no active keys succeeds (cascade deletes all data)
- [ ] Delete client with active keys returns 409 with key count in error detail
- [ ] Non-super-admin key gets 403 on all client endpoints
- [ ] Integration tests cover all scenarios

## Blocked by

- arche-types/database-migrations.md
- arche-types/domain-structs-enums.md
- arche-service/auth-api-key-extractor.md
- arche-service/permission-matrix.md
