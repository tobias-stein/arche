# Arche

A blueprint-based random name generation system for games. Arche lets you define **blueprints** (item templates with typed attributes), **affixes** (prefixes/suffixes that can randomly roll onto items), and **global meta-attributes** (shared attribute definitions referenced via `$ref_id`). A weighted random selection engine picks blueprints, rolls their attributes, selects affixes from configurable pools, and composes a final name with rolled affix attributes.

Architecture highlights:
- **REST API** built with [Axum](https://github.com/tokio-rs/axum) + [SQLx](https://github.com/launchbadge/sqlx) (PostgreSQL)
- **In-memory cache** of client resources (blueprints, affixes, GMAs) with background polling and optional Redis pub/sub for cross-instance invalidation
- **Admin UI** built with React 19 + shadcn/ui + Vite
- **CLI** for bootstrapping, generating, exporting/importing, and managing clients/keys
- **Benchmark suite** in Rust for measuring generation throughput and latency

## Project Structure

```
arche/
├── admin-ui/                  # React admin interface
├── arche-service/             # Database migrations
├── bench/                     # Rust benchmark suite
├── crates/
│   ├── arche-types/           # Shared domain types, validation, and API contracts
│   ├── arche-service/         # HTTP REST API server
│   └── arche-cli/             # Command-line interface
├── demos/
│   └── dungeon-crawler/       # Game client demo (React + TypeScript)
├── Cargo.toml                 # Workspace manifest
├── docker-compose.yml         # Full-stack Docker Compose (Postgres + service + admin-ui)
├── init.sh                    # Local development launcher
└── rust-toolchain.toml        # Rust toolchain pinning (1.88.0)
```

## Table of Contents

- [Quick Start](#quick-start)
  - [Dungeon Crawler Demo](#dungeon-crawler-demo)
- [Modules](#modules)
  - [arche-service (Rust API server)](#arche-service-rust-api-server)
  - [arche-types (Shared types)](#arche-types-shared-types)
  - [arche-cli (CLI tool)](#arche-cli-cli-tool)
  - [admin-ui (React frontend)](#admin-ui-react-frontend)
  - [dungeon-crawler (Game Demo)](#dungeon-crawler-game-demo)
  - [bench (Benchmark suite)](#bench-benchmark-suite)
- [API Overview](#api-overview)
- [Authentication & Permissions](#authentication--permissions)
- [Database Schema](#database-schema)
- [Configuration](#configuration)
- [Development](#development)
- [Testing](#testing)

## Quick Start

### Docker Compose (recommended for evaluation)

```bash
./init.sh --docker
```

This starts PostgreSQL, the service (port 8080), and the admin UI (port 3000). The super admin key is printed to service logs.

### Native (for development)

```bash
./init.sh
```

Prerequisites: Rust 1.88+, Node 22+, PostgreSQL running locally.

Or manually:

```bash
# Terminal 1 — start PostgreSQL (via Docker)
docker compose up postgres -d

# Terminal 2 — start the API server
cargo run -p arche-service

# Terminal 3 — start the admin UI
cd admin-ui && npm install && npm run dev
```

The super admin key is printed to service logs on first start. Use it to authenticate from the CLI or admin UI.

### Dungeon Crawler Demo

A turn-based dungeon crawler game built against the Arche generation API. Run it with:

```bash
cd demos/dungeon-crawler
./run-dungeon-crawler.sh
```

This builds all Docker images, starts PostgreSQL + Arche + a seed container + the game client, and prints the URLs.

| Service | Port |
|---------|------|
| Game client | `http://localhost:5173` |
| Admin UI | `http://localhost:8081` |
| Arche API | `http://localhost:8080` |

The client API key is printed to the terminal and injected into the game container automatically.

## Modules

### arche-service (Rust API server)

**Location:** `crates/arche-service/` | **Dependencies:** axum, sqlx, tokio, redis, bcrypt, zip

A REST HTTP API server that provides CRUD operations for blueprints, affixes, global meta attributes, clients, API keys, audit logging, and item generation.

**Key endpoints:**

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| GET | `/api/bootstrap` | Bootstrap first super admin key |
| GET | `/api/me` | Returns current API key info |
| GET/POST | `/api/blueprints` | List / create blueprints |
| GET/PUT/DELETE | `/api/blueprints/{id}` | Get / update / delete blueprint |
| POST | `/api/blueprints/batch/edit` | Batch edit blueprint attributes |
| POST | `/api/blueprints/batch/delete` | Batch delete blueprints |
| POST | `/api/blueprints/batch/assign` | Batch assign affixes to blueprints |
| GET/POST | `/api/affixes` | List / create affixes |
| GET/PUT/DELETE | `/api/affixes/{id}` | Get / update / delete affix |
| POST | `/api/affixes/batch/delete` | Batch delete affixes |
| POST | `/api/affixes/batch/assign` | Batch assign affixes |
| GET | `/api/affixes/references` | List affix references |
| GET/POST | `/api/global-meta-attributes` | List / create GMAs |
| GET/PUT/DELETE | `/api/global-meta-attributes/{id}` | Get / update / delete GMA |
| GET | `/api/audit-log` | List audit log entries |
| POST | `/api/generate` | Generate a single item name |
| POST | `/api/generate/batch` | Batch generate item names |
| GET/POST | `/api/clients` | List / create clients |
| GET/DELETE | `/api/clients/{id}` | Get / delete client |
| GET/POST | `/api/clients/{id}/keys` | List / create API keys for client |
| DELETE | `/api/clients/{id}/keys/{key_id}` | Revoke API key |
| POST | `/api/export` | Export client data as ZIP |
| POST | `/api/import` | Import client data from ZIP |
| POST | `/api/import/resolve` | Resolve import conflicts |
| GET | `/api/schema/blueprints` | JSON Schema for blueprints |
| GET | `/api/schema/affixes` | JSON Schema for affixes |
| GET | `/api/schema/generate` | JSON Schema for generate request/response |
| GET | `/api/schema/generate/batch` | JSON Schema for batch generate |

**Cache architecture:**
- On startup, all client data (blueprints, affixes, GMAs) is loaded into an in-memory `Cache` partitioned by client ID (`ClientCache`).
- A background poller checks `GREATEST(updated_at)` across all tables per client; when changes are detected, only that client's data is reloaded.
- API keys are cached in a separate `ApiKeyCache` (in-memory, TTL-aware) to avoid bcrypt on every request. The poller purges revoked/expired keys.
- Optional Redis pub/sub allows cache invalidation across multiple service instances. Publish to `arche:cache-invalidate` with `{"type": "invalidate", "client_id": "..."}`.

**Generation pipeline (POST /api/generate):**
1. Load `ClientCache` for the requesting client (from in-memory cache, or optionally from DB via `X-Cache-Refresh: true`)
2. Select a blueprint via weighted random (Efraimidis & Spirakis A-ES algorithm) filtered by archetype and constraints
3. Roll blueprint attributes — inline attributes are rolled directly; `$ref_id` attributes are resolved from the GMA pool with `value_type` injected into the payload
4. Select affixes from the blueprint's affix pool (weighted without replacement), respecting require/block constraints from the request
5. Compose the name as `[prefixes...] [base] [suffixes...]`
6. Roll affix attributes

**Build & run:**
```bash
cargo build -p arche-service
cargo run -p arche-service
cargo test -p arche-service
```

**Configuration (environment variables):**

| Variable | Default | Description |
|---|---|---|
| `ARCHE_DATABASE_URL` | (required) | PostgreSQL connection string |
| `ARCHE_PORT` | `8080` | HTTP listen port |
| `ARCHE_REDIS_URL` | (none) | Redis URL for pub/sub cache invalidation |
| `ARCHE_CACHE_POLL_INTERVAL_MS` | `60000` | Background cache refresh interval |
| `ARCHE_DB_POOL_SIZE` | `2*cores + 10` | SQLx connection pool size |

### arche-types (Shared types)

**Location:** `crates/arche-types/` | **Dependencies:** serde, uuid, chrono, schemars, thiserror

A library crate (no binary) containing all JSON-serializable types shared between archet-service and arche-cli. All JSON keys use **snake_case** (enforced by `#[serde(rename_all = "snake_case")]` on all request/response structs).

**Modules:**

| Module | Description |
|---|---|
| `attribute.rs` | `DistributionConfig` (Uniform/Normal/Exponential), `AttributePayload` (Single/Enum/Range/String/Boolean), `BlueprintAttribute` (+ `$ref_id`), `AffixAttribute`, pool entry types |
| `crud.rs` | Create/Update/Response types for blueprints, affixes, GMAs, clients, API keys, bootstrap |
| `generate.rs` | `GenerateRequest`, `GenerateResponse`, `BatchGenerateRequest/Response`, constraints |
| `common.rs` | `PaginatedResponse<T>`, `ProblemJson` (RFC 7807), list query types |
| `batch.rs` | Batch operation types (delete, assign affixes, edit attributes) |
| `export_import.rs` | Export/Import request/response types with conflict resolution |
| `validation.rs` | `ValidationError`, `validate_attribute_payload`, `validate_attribute_order`, `validate_blueprint` |

**Validation rules:**
- Weight must be > 0
- Range payload: min <= max, std_dev > 0, rate > 0
- Enum payload: at least 1 value
- Single payload: value must not be null
- `$ref_id` cannot coexist with inline fields
- Attribute order must have no duplicates, list all keys, and contain no extras
- Prefix/suffix counts must be non-negative and internally consistent

### arche-cli (CLI tool)

**Location:** `crates/arche-cli/` | **Dependencies:** clap, reqwest, serde, colored, comfy-table, dialoguer

A CLI client for interacting with the Arche API. Supports JSON and pretty-printed output modes.

**Subcommands:**

| Command | Description |
|---|---|
| `init` | Bootstrap a new Arche instance (prints super admin key) |
| `generate` | Generate item names with optional archetype filter, constraints, seed, and affix overrides |
| `export` | Export client data as ZIP to a file or stdout |
| `import` | Import client data from ZIP with dry-run mode and interactive conflict resolution |
| `key list` | List API keys for a client |
| `key create` | Create a new API key with specific permissions |
| `key revoke` | Revoke an API key |
| `client list` | List all clients (super admin only) |
| `client create` | Create a new client |
| `client delete` | Delete a client |

**Global flags:**
- `--api-url` (env: `ARCHE_API_URL`, default: `http://localhost:8080`)
- `--api-key` (env: `ARCHE_API_KEY`)
- `-v` / `--verbose`
- `-q` / `--quiet`

**Output modes:**
- `--format json` — pretty-printed JSON to stdout
- `--format pretty` — colored tables/text (default for tty)
- quiet mode suppresses all non-error output

**Error handling:** 3-tier error types with distinct exit codes: 1 (generic/connection), 2 (argument), 3 (API error with RFC 7807 Problem JSON).

```bash
cargo build -p arche-cli
cargo run -p arche-cli -- init
cargo test -p arche-cli
```

### admin-ui (React frontend)

**Location:** `admin-ui/` | **Stack:** React 19, TypeScript, Vite, Tailwind CSS 4, shadcn/ui, TanStack Query, Zustand, React Router 7

A full-featured admin interface for managing Arche data.

**Dependencies:** @radix-ui/*, @dnd-kit (sortable attribute order), react-hook-form + zod (form validation), lucide-react (icons), cmdk (command palette), react-diff-viewer (import conflict resolution)

**Key pages/views:**
- **Login:** API-key-based authentication, client-scoped login flow via `GET /api/me`
- **Dashboard:** Stat cards (total blueprints, affixes, GMAs, warnings), quick-generate playground
- **Blueprints:** List (table with search/filter/batch toolbar), detail (read-only tabbed view), create/edit overlay modals
- **Affixes:** Same list/detail/create/edit pattern as blueprints
- **Global Meta Attributes:** Same CRUD pattern
- **Clients:** Client management with API key management per client
- **Audit Log:** Paginated activity log with filters
- **Settings:** Dark/light mode toggle, sidebar configuration

**UI features:**
- shadcn/ui sidebar (collapsible icon mode)
- Client switcher in header + global search
- Animated page transitions, skeleton loaders
- Responsive (mobile/tablet/desktop)
- Dark mode

**Build & run:**
```bash
cd admin-ui
npm install
npm run dev        # Development on port 5173
npm run build      # Production build to dist/
npm run test       # Run vitest tests
npm run lint       # Run ESLint
```

**Environment variables:**
- `VITE_ARCHE_API_URL` — API base URL (default: `http://localhost:8080`)

**Docker build:** The `Dockerfile` produces an nginx-alpine image serving the built app.

### dungeon-crawler (Game Demo)

**Location:** `demos/dungeon-crawler/` | **Stack:** React 19, TypeScript, Vite, Arche API

A turn-based dungeon crawler that uses the Arche generation API to create creatures, items, potions, and spells. The player explores procedurally generated dungeons, fights enemies, collects loot, and levels up.

**Key mechanics:**
- **Dungeon generation:** Recursive room-carving algorithm with connected corridors
- **Creature spawning:** Arche API generates creatures scaled to the player's level, with fallback to local mock data
- **Loot system:** Items roll primary stats (damage, defense) and secondary stats (strength, intelligence, agility) via Arche's attribute rolling engine
- **Affix system:** Prefixes/suffixes apply bonus effects to generated items and creatures (e.g., "Smoldering Sword of Power")
- **Combat:** Turn-based with attack, spellcasting, and item consumption
- **Inventory:** Equipment slots (weapon, armor, shield, accessories), consumables, and spellbook with drag-and-drop

**Key files:**

| Path | Description |
|---|---|
| `scripts/seed.ts` | Seeds Arche with ~600 blueprints + ~40 affixes across 100 levels |
| `scripts/shared.ts` | Stat multipliers, affix/prefix definitions, subtype pools |
| `Dockerfile` | nginx-alpine production image for the Vite build |
| `docker-compose.yml` | Full demo stack (Postgres + Arche + seed + game + Admin UI) |
| `run-dungeon-crawler.sh` | One-shot launcher — builds, seeds, and runs everything |

**Run with Docker Compose (one command):**

```bash
cd demos/dungeon-crawler
./run-dungeon-crawler.sh
```

The script:
1. Builds all Docker images
2. Starts PostgreSQL, Arche service, Admin UI
3. Waits for Arche API health
4. Extracts the super admin key from service logs
5. Runs the seed container (creates client, API key, GMAs, affixes, and ~600 blueprints)
6. Starts the dungeon-crawler game container with the client API key injected
7. Verifies the game is reachable and prints all URLs

**Environment variables:**

| Variable | Default | Description |
|---|---|---|
| `DUNGEON_CRAWLER_PORT` | `5173` | Game client HTTP port |
| `ADMIN_UI_PORT` | `8081` | Admin UI HTTP port |
| `ARCHE_API_URL` | `http://localhost:8080` | Arche API base URL (used at runtime) |

**Manual development:**

```bash
# Start the Arche stack first
cd demos/dungeon-crawler
docker compose up -d postgres arche-service
# Wait for Arche, then seed
ARCHE_API_KEY=$(docker compose logs arche-service | grep -o 'arche_k_[a-zA-Z0-9]\{48\}')
docker compose run --rm -e ARCHE_API_KEY="$ARCHE_API_KEY" seed
# Start the game
npm install
npm run dev
```

**Testing:**
```bash
cd demos/dungeon-crawler
npm test            # 346+ tests (vitest)
npm run typecheck   # TypeScript strict check
```

### bench (Benchmark suite)

**Location:** `bench/` | **Stack:** Rust (arche-bench crate), bash orchestration

Measures throughput and latency of `POST /api/generate` under varying dataset sizes and concurrency levels.

**Components:**

| Path | Description |
|---|---|
| `rust-bench/` | Rust binary (`arche-bench`) with three subcommands |
| `run-bench.sh` | Bash orchestrator — starts Docker stack, seeds data, runs sweep |
| `scenarios.yaml` | Sweep dimension matrix (cross-product of blueprints × affixes × attributes × concurrency) |
| `report.html` | Self-contained Chart.js + Bootstrap report (generated) |
| `results.csv` | Machine-readable results (generated) |

**Subcommands:**
- `bench` — Fixed-concurrency benchmark loop (duration-based with per-request latency tracking, plateau detection, 5s stable sample after plateau)
- `seed` — Synthetic data generator using deterministic RNG (ChaCha8 + FarmHash) with configurable blueprint/affix/attribute counts and random value type/distribution assignment
- `sweep` — Multi-scenario sweep from YAML config: for each scenario combination, seeds a fresh client, runs the benchmark, records results, tears down. Outputs CSV + HTML report with throughput and latency percentiles (p50, p95, p99)

**Data generation:**
- Deterministic: identical dimensions produce identical datasets via FarmHash → ChaCha8Rng seed
- Randomly assigns value types (single/enum/range/string/boolean) to attributes
- Range attributes get random distributions (uniform/normal/exponential)
- Blueprints reference GMAs via `$ref_id` with ~20% probability
- Scoped API keys are created per client; benchmark runs against the scoped key

**Orchestration (`run-bench.sh`):**
1. Starts Docker Compose stack (or connects to existing)
2. Polls API until ready (up to 120s)
3. Extracts super admin key from container logs
4. Seeds 100 blueprints × 4 attributes × 0 affixes
5. Runs `sweep` from `scenarios.yaml` with 10s per scenario

```bash
cd bench
bash run-bench.sh

# Or manually:
cargo run --release --package arche-bench -- seed --target http://localhost:8080 --api-key "..."
cargo run --release --package arche-bench -- sweep --target http://localhost:8080 --api-key "..."
cargo run --release --package arche-bench -- bench --concurrency 10 --duration 30
```

## Authentication & Permissions

Arche uses **API-key-based authentication** via the `X-API-Key` header.

**Permission model:**

| Permission | Allowed Operations |
|---|---|
| `read` | GET endpoints (list, view details) |
| `write` | POST/PUT endpoints (create, update) |
| `delete` | DELETE endpoints |
| `generate` | POST `/api/generate`, `/api/generate/batch` |
| `admin` | Manage API keys for a client |
| _super admin_ | Full access to all clients, export/import, bootstrap |

- Super admin keys bypass all permission checks and client-scoping.
- Per-client keys are automatically scoped to their client's resources.
- Admin permission grants all other operation permissions.
- Export and import endpoints require super admin access.

**Public endpoints (no auth required):** `/health`, `/api/bootstrap`, `/api/schema/*`, `/api/me`

## Database Schema

Tables:
- `clients` — Multi-tenant game/project separations
- `api_keys` — bcrypt-hashed keys linked to clients (nullable = super admin)
- `blueprints` — Item templates with JSONB attributes, archetype classification, weight, affix min/max config
- `affixes` — Prefix/suffix definitions (PostgreSQL enum `affix_location`), each with a JSONB attribute definition
- `global_meta_attributes` — Reusable attribute definitions shared across blueprints and affixes via `$ref_id`
- `blueprint_affixes` — Join table mapping affixes to blueprints with per-entry weight, location, sort order
- `audit_log` — Immutable audit trail of all CRUD operations

Custom PostgreSQL enums: `value_type` (single/enum/range/string/boolean), `affix_location` (prefix/suffix), `audit_action` (created/updated/deleted/force_deleted/adjusted)

Migrations live in `arche-service/migrations/` and run automatically on service startup.

## Configuration

All modules read configuration from environment variables. A `.env` file is supported by `arche-service` (via dotenvy).

See the arche-service section above for service configuration.

## Development

**Prerequisites:**
- Rust 1.88.0 (see `rust-toolchain.toml`)
- Node 22+ (for admin-ui)
- PostgreSQL 16+ (or Docker for the containerized database)

**Build entire workspace:**
```bash
cargo build --workspace
```

**Run lints:**
```bash
cargo clippy --workspace
cargo fmt --check
```

**Run all unit tests:**
```bash
cargo test --workspace
```

**Run integration tests (requires PostgreSQL):**
```bash
ARCHE_DATABASE_URL=postgres://postgres:postgres@localhost:5432/arche cargo test -p arche-service -- --include-ignored
```

**Commit conventions:** This project has not historically enforced commit conventions, but the AI sandcastle workflow (`.sandcastle/`) uses descriptive prefixed commit messages (e.g., `feat:`, `refactor:`, `fix:`).

## Testing

| Module | Test Framework | Test Count | Notes |
|---|---|---|---|
| `arche-types` | built-in + proptest | ~90 serde round-trip + validation tests | Compile-time trait bounds check (~50 types) |
| `arche-service` | built-in + proptest | ~200+ unit + integration tests | DB tests guarded by `DATABASE_URL` env var |
| `arche-cli` | built-in + wiremock + insta | ~100 tests | Snapshot tests for output formatting |
| `admin-ui` | vitest + testing-library | ~100+ tests | JSDOM environment |
| `bench` | built-in | ~20 unit tests | Determinism and distribution tests |
