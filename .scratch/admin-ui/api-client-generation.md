---
title: API client generation
status: ready-for-agent
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Set up the generated TypeScript API client from the service's JSON Schema endpoints:

1. Create a script (`npm run generate-api-client`) that fetches schemas from `GET /api/schema/blueprints`, `GET /api/schema/affixes`, `GET /api/schema/generate` and generates TypeScript types and API client methods
2. The generated client wraps all REST endpoints with typed request/response interfaces
3. Handles `X-API-Key` header injection via a configurable `apiKey` parameter
4. Normalizes RFC 9457 Problem JSON errors into typed error objects
5. Parses pagination cursor/offset from responses
6. Provides React hooks via TanStack Query wrappers (useQuery, useMutation for each endpoint)
7. Generated client is checked into the repo for reproducible builds
8. Falls back to local static schema files during development (when the API isn't running)

The client covers all endpoints from `docs/design/api-spec.md`.

## Acceptance criteria

- [ ] `npm run generate-api-client` produces TypeScript files in `src/api/generated/`
- [ ] All endpoints from the API spec have corresponding typed methods
- [ ] TanStack Query hooks: `useBlueprintsList()`, `useBlueprint()`, `useCreateBlueprint()`, etc.
- [ ] Auth: `apiKey` configurable, injected as `X-API-Key` header
- [ ] Error normalization: Problem JSON parsed into typed error
- [ ] Pagination: cursor and offset correctly parsed
- [ ] Fallback schemas available for development without running API
- [ ] Generated client compiles with zero TypeScript errors

## Blocked by

- admin-ui/scaffolding.md
- arche-service/json-schema-endpoints.md
