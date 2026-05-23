# ADR 0001: Auth Model — Super Admin + Per-Client RBAC Keys

**Date:** 2026-05-22
**Status:** Accepted

## Context

Arche is a multi-tenant service. It needs to support:
- A human operator managing multiple games/clients from one admin UI
- Game clients calling the API with minimal friction but scoped to their own data
- Multiple keys per client for different use cases (game server, monitoring, CI/CD)

## Decision

Two-tier API key model:

1. **Super admin key** — auto-generated at first startup, printed to stdout. Has access to all clients. Used by the admin UI and for client management.

2. **Per-client RBAC keys** — created via the admin API/UI. Scoped to a single client. Permission set: `read`, `write`, `delete`, `generate`, `admin` (manage keys for that client).

All keys are hashed with bcrypt. The raw key is returned exactly once on creation.

## Consequences

**Positive:**
- No user account system needed (simpler operational model)
- Multiple keys per client for different trust levels
- Super admin key can be rotated by the operator

**Negative:**
- No per-user auditing (audit log shows key name, not a person)
- Key management is manual — no SSO, no OAuth
- Admin UI tie to a single super admin key makes team collaboration harder (share the key)

## Alternatives Considered

- **Single shared key per client** — simpler but no permission granularity
- **JWT / OAuth2 with user accounts** — more familiar for web apps, but overkill for this use case and adds identity provider dependency
- **No auth / trust network** — simplest but insecure for multi-tenant
