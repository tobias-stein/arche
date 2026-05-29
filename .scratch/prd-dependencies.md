## Dependency Graph

```
┌──────────────────────────────────────────────────┐
│               arche-types (shared)               │
│  Domain structs, enums, validation, API contracts │
└──────┬──────────────────────┬───────────────────-┘
       │ depends on           │ depends on
       ▼                      ▼
┌──────────────┐     ┌──────────────────┐
│ arche-service│     │   arche-cli      │
│ (REST API)   │◄────│  (CLI tool)      │
└──────┬───────┘     └──────────────────┘
       │ depends on (API contract)
       ▼
┌──────────────────┐
│  Admin UI        │
│ (React SPA)      │
└────────┬─────────┘
         │ depends on (API contract)
         ▼
┌──────────────────────────────────┐
│      arche-bench (benchmark)     │
│ Python script, aiohttp + pyyaml  │
│ Benchmarks POST /api/generate    │
└──────────────────────────────────┘
```

## Dependency Details

### 1. `arche-types` (shared library crate)

This crate is a **prerequisite** for both the service and the CLI. It contains:

- All domain structs and enums
- Request/response types for every API endpoint
- Validation logic used by both write-time validation (service) and client-side validation (CLI)
- Export/import format definitions (ZIP structure, conflict-resolution schema)
- The conflict-resolution JSON schema (shared between CLI and Admin UI import flows)

**Implementation order:** First. The service and CLI can be built in parallel once `arche-types` is stable.

### 2. `arche-service` → depends on `arche-types`

The service uses `arche-types` for:
- Request deserialization and response serialization
- Validation of incoming payloads
- Type safety across the HTTP boundary
- The export/import conflict-resolution data structures

The service also depends on the **PostgreSQL schema data model** defined in `docs/design/data-model.md` — migrations are a prerequisite to running the service.

**The service must be deployed and running before the Admin UI or CLI can use it.**

### 3. `arche-cli` → depends on `arche-types` + `arche-service`

The CLI depends on `arche-types` for:
- Shared domain types
- API request/response structures
- The export/import format
- The conflict-resolution schema (ensuring resolution files are portable between CLI and Admin UI)

The CLI depends on `arche-service` for:
- All REST API endpoints it calls (generate, CRUD, import/export, key management, client management)

**The service does NOT need to be complete for CLI development to start** — the CLI can be developed against a mock server or the generated JSON Schema, but end-to-end testing requires a running service.

### 4. `arche-bench` (benchmark suite) → depends on `arche-service`

The benchmark suite depends on the service for:

- `POST /api/generate` — the endpoint under test
- `POST /api/clients` — to create per-scenario clients for data isolation
- `POST /api/blueprints`, `POST /api/affixes`, `POST /api/blueprints/:id/affixes` — to seed test data
- The service must be running (the benchmark script starts Docker Compose or connects to `--target-url`)

The benchmark suite does **not** depend on `arche-types`, `arche-cli`, or the Admin UI.

### 5. Arche Admin UI → depends on `arche-service`

The Admin UI depends on the service for:
- All REST API endpoints (CRUD, generate, import/export, audit log, schema)
- The JSON Schema endpoints (`GET /api/schema/...`) for API client generation
- The RFC 9457 error format

**The Admin UI's API client is generated from the service's JSON Schema endpoints.** This means:
- The schema endpoints must be implemented before or in parallel with Admin UI development
- The Admin UI can be developed with mock data/scaffold schemas while the service is being built
- The generated TypeScript client should be regenerated whenever the API spec changes

## Implementation Ordering

### Recommended phases:

| Phase | Deliverables | Dependencies |
|-------|-------------|--------------|
| **Phase 0** | PostgreSQL schema + migrations | None |
| **Phase 1** | `arche-types` crate | None |
| **Phase 2a** | `arche-service` core: DB layer, auth, CRUD endpoints, generation engine | Phase 0, Phase 1 |
| **Phase 2b** | `arche-service` extended: audit log, export/import, batch ops, schema endpoints | Phase 2a |
| **Phase 3a** | Admin UI core: app shell, auth, blueprint CRUD, affix CRUD | Phase 2b (schema endpoints for client generation) |
| **Phase 3b** | Admin UI extended: global meta attributes, clients/keys, audit log, import/export wizard, global search, dashboard | Phase 3a |
| **Phase 3c** | `arche-cli`: all commands | Phase 1, Phase 2b |
| **Phase 4** | `arche-bench` benchmark suite | Phase 2b (needs generate endpoint) |

### Parallelism opportunities:
- `arche-types` (Phase 1) is a prerequisite for everything — build first.
- The service can be built in phases 2a/2b without any Admin UI work.
- The CLI (Phase 3c) can start as soon as `arche-types` is stable, mocking HTTP responses for testing.
- The Admin UI (Phase 3a/3b) can start with scaffold/mock data while the service is being built, but full integration requires the schema endpoints.

## Shared Interfaces

### Conflict-Resolution Schema (portable between CLI and Admin UI)

Both the CLI's `--resolve-file` and the Admin UI's import resolution phase produce/consume this exact schema:

```json
{
  "import_token": "uuid",
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

This schema is defined in `arche-types` and used by:
- The service (`POST /api/import/resolve`)
- The CLI (`arche import --resolve-file`)
- The Admin UI (import resolution modal)

### ZIP Export/Import Format

Defined in `arche-types`. Used by:
- The service (`POST /api/export`, `POST /api/import`)
- The CLI (`arche export`, `arche import`)
- The Admin UI (export download, import upload)

Structure:
```
client-<uuid>/
  global-meta-attributes.json
  blueprints.json
  affixes.json
  api-keys.json            (optional)
  audit-log.json           (optional)
```

### JSON Schema Endpoints

The three schema endpoints (`GET /api/schema/blueprints`, `/api/schema/affixes`, `/api/schema/generate`) are the **API contract between the service and Admin UI**. The Admin UI's TypeScript API client is generated from these.

Changes to the API spec must be reflected in these schemas. The schemas are derived from `arche-types` Rust types via `schemars` and served by the service.
