---
title: Pagination utilities
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement reusable pagination utilities used by all list endpoints.

Two pagination modes:
1. **Cursor-based** — Uses `?cursor=<uuid>&limit=N`. The cursor is the UUID of the last item in the current page. The query adds `WHERE id > cursor` ordering by UUID. Returns `next_cursor` which is the last item's UUID (or null if no more pages). Default limit 50, max 200.

2. **Offset-based** — Uses `?page=N&per_page=N`. Returns `total` count alongside data. Used exclusively by the Admin UI (which needs page numbers). Default per_page 50, max 200.

A common `PaginationParams` struct that can parse both modes from query parameters (trying cursor first if both present). A `PaginatedResponse<T>` wrapper struct with `data: Vec<T>`, `next_cursor: Option<Uuid>`, `total: Option<i64>`.

## Acceptance criteria

- [ ] `PaginationParams` deserializes from query string
- [ ] Cursor mode: correct SQL query, `next_cursor` returned
- [ ] Offset mode: correct SQL query with OFFSET/LIMIT, `total` returned
- [ ] If both cursor and page provided, cursor wins
- [ ] Default limit = 50, max = 200, invalid values clamped
- [ ] `PaginatedResponse` serializes correctly
- [ ] Unit tests for pagination edge cases (last page, empty result, etc.)

## Blocked by

- arche-service/service-scaffolding.md
