---
title: arche-types — domain structs & enums
status: ready-for-agent
---

## Parent

arche-types shared library (see `.scratch/arche-types/prd.md`)

## What to build

Define all domain structs and enums in the `arche-types` crate. These are the canonical types shared by the service and CLI.

Structs: `Client`, `Blueprint`, `Affix`, `GlobalMetaAttribute`, `ApiKey`, `AuditLogEntry`, `BlueprintAffix`.

Enums: `ValueType` (Single, Enum, Range, String, Boolean), `AffixLocation` (Prefix, Suffix), `AuditAction` (Created, Updated, Deleted, ForceDeleted, Adjusted), `Permission` (Read, Write, Delete, Generate, Admin).

Each struct derives `Serialize`, `Deserialize`, `Debug`, `Clone`, `PartialEq`. Use `serde::rename_all = "camelCase"` for JSON serialization matching the API spec. UUIDs use `uuid::Uuid`. Timestamps use `chrono::DateTime<Utc>`.

## Acceptance criteria

- [ ] All structs defined in `arche-types/src/lib.rs` or submodules
- [ ] All enums defined with `Serialize`/`Deserialize` derives
- [ ] JSON serialization uses camelCase field names
- [ ] `cargo test` passes for the crate
- [ ] Types are re-exported from `arche-types/src/lib.rs`

## Blocked by

- arche-types/workspace-scaffolding.md
