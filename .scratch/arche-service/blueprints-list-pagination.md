---
title: Blueprints — list with pagination & filters
status: completed
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement `GET /api/blueprints` with:
- Cursor-based pagination (`?cursor=<uuid>&limit=50`)
- Offset-based pagination (`?page=1&per_page=50`) for Admin UI convenience
- `archetype` filter — exact match on archetype label
- `search` param — substring/ILIKE match on blueprint name
- Response: `{ "data": [...], "next_cursor": "uuid", "total": 42 }`

Both pagination modes work independently. If both are provided, cursor mode wins. Default limit is 50. Max limit is 200.

Results are scoped to the authenticated client (super admin can see all clients, or filter by client_id).

## Acceptance criteria

- [ ] List returns paginated results, scoped to current client
- [ ] Cursor pagination: `?limit=5` returns 5 results + next_cursor
- [ ] Offset pagination: `?page=2&per_page=10` returns correct page
- [ ] `archetype` filter: only blueprints with matching archetype returned
- [ ] `search` filter: only blueprints with name containing search string returned
- [ ] Combined filters work together
- [ ] Default limit is 50, max is 200
- [ ] Integration tests

## Blocked by

- arche-service/blueprints-crud-core.md
