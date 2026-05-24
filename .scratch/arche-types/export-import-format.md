---
title: arche-types — export/import format & conflict schema
status: completed
---

## Parent

arche-types shared library (see `.scratch/arche-types/prd.md`)

## What to build

Define the ZIP export/import format types and the conflict-resolution JSON schema in `arche-types`. These are shared between the service, CLI, and Admin UI (via the generated TypeScript client).

Export format:
- ZIP archive structure with folder-per-client: `client-<uuid>/` containing `blueprints.json`, `affixes.json`, `global-meta-attributes.json`, `api-keys.json` (optional), `audit-log.json` (optional)
- Each JSON file is an array of the corresponding resource type
- An `ExportManifest` type (version, timestamp, client count)

Conflict resolution schema (portable between CLI and Admin UI):
```json
{
  "import_token": "uuid-from-service",
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

Define `ResolutionStrategy` enum, `ResourceResolution` struct, and `ConflictResolution` container struct.

## Acceptance criteria

- [ ] `ExportManifest` struct defined with version field
- [ ] `ConflictResolution` struct with import_token and resolutions map
- [ ] `ResolutionStrategy` enum with KeepOld, KeepNew, PerAttribute variants
- [ ] `ResourceResolution` struct with strategy and optional per-attribute map
- [ ] Types serialize/deserialize to the expected JSON schema
- [ ] `cargo test` passes
- [ ] Schema is documented in `docs/design/api-spec.md` export/import section

## Blocked by

- arche-types/domain-structs-enums.md
