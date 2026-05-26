---
title: Export endpoint
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement `POST /api/export` for exporting client data as a ZIP archive. Super admin only.

Request:
```json
{
  "client_ids": ["uuid-1", "uuid-2"],
  "include_api_keys": false,
  "include_audit_log": false,
  "inline_global_refs": false
}
```

Response: Binary ZIP stream (`application/zip`).

ZIP structure:
```
client-<uuid>/
  global-meta-attributes.json
  blueprints.json
  affixes.json
  api-keys.json            (optional)
  audit-log.json           (optional)
```

If `inline_global_refs` is true, resolve all `$ref_id` references to inline attribute definitions in the export. If false, keep `$ref_id` as-is.

If `client_ids` is empty or omitted, export ALL clients (all data the super admin can see).

## Acceptance criteria

- [ ] Export single client: ZIP with correct folder structure
- [ ] Export multiple clients: each has its own folder
- [ ] `include_api_keys=true`: api-keys.json present in ZIP
- [ ] `include_audit_log=true`: audit-log.json present
- [ ] `inline_global_refs=true`: no `$ref_id` in exported JSON, all resolved to inline
- [ ] `inline_global_refs=false`: `$ref_id` preserved
- [ ] Empty client_ids exports all clients
- [ ] Response Content-Type is `application/zip`
- [ ] Integration test: export, verify ZIP contents, re-import, verify round-trip

## Blocked by

- arche-types/export-import-format.md
- arche-types/domain-structs-enums.md
- arche-service/auth-api-key-extractor.md
- arche-service/permission-matrix.md
