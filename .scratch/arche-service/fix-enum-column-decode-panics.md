---
title: Fix enum column decode panics in audit log and export
status: completed
---

## What to build

The service panics when decoding Postgres custom enum columns (`audit_action`, `affix_location`) as plain Rust `String` types via `sqlx::Row::get`. The panic occurs in two places:

1. **Audit log list endpoint** — `action` column is `audit_action` enum, fetched without `::text` cast
2. **Export endpoint** — `action` column is `audit_action` enum in `fetch_audit_log`, `location` column is `affix_location` enum in `fetch_blueprint_affixes`, both fetched without `::text` cast

Add `::text` SQL casts to the SELECT queries so sqlx can decode these columns as `String`.

## Acceptance criteria

- [ ] `audit_log.rs` SELECT query casts `action::text AS action`
- [ ] `export.rs` `fetch_audit_log` SELECT query casts `action::text AS action`
- [ ] `export.rs` `fetch_blueprint_affixes` SELECT query casts `location::text AS location`
- [ ] Existing tests pass
- [ ] Manual verification: audit log list endpoint returns data without panic; export with `includeAuditLog: true` succeeds

## Blocked by

None — can start immediately.
