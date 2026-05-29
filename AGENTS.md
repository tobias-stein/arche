## Agent skills

### Issue tracker

Issues are tracked as local markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default label vocabulary: needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context layout: one `CONTEXT.md` + `docs/adr/` at repo root. See `docs/agents/domain.md`.

## Session context (2026-05-22)

### UI design decisions (Arche Admin UI)

The following were resolved in the last design session. See `docs/design/ui-ux.md` for full details.

**Layout:**
- Left-side **Navigation Drawer**: client switcher, API key management, login/logout, settings
- Right-side **Activity Panel**: audit/activity log, toggleable (float by default, optional dock)
- Dark/light mode toggle, responsive to mobile/tablet/desktop
- Default shadcn/ui color palette, animations throughout (page transitions, micro-interactions, skeletons, panel transitions)

**Views pattern (applies to blueprints, affixes, global meta attributes, clients):**
- **List page**: table with search/filter, row actions, batch toolbar
- **Detail page**: read-only, shareable URL, tabbed layout, "Edit" button
- **Create/Edit**: overlay modals, not separate pages

**Dashboard:**
- Stat cards: total blueprints, affixes, warnings (misconfigurations)
- Quick Generate playground (existing design)

**Warnings (misconfiguration detection):**
- Blueprint min prefix/suffix > 0 with empty pool
- Zero-weight blueprint or affix assignment
- Dangling `$ref_id` (references non-existent global)
- Invalid attribute payload (range min > max, empty enum, etc.)
- Invalid distribution config (std_dev = 0, rate = 0)

**Auth:** API-key-only model kept (ADR 0001). Multi-user deferred.

### JSON serialization convention

All JSON in the Arche API uses **snake_case** for keys. This applies to:
- `crates/` — serde attributes use `#[serde(rename_all = "snake_case")]` on all request/response structs and enums
- `admin-ui/` — TypeScript interfaces mirror the API with snake_case property names
- `bench/` — `json!()` literals use snake_case keys for API payloads

Exceptions (explicit serde overrides):
- `#[serde(rename = "type")]` — Rust reserved-word workaround (`type_` fields)
- `#[serde(rename = "enum")]` — Rust reserved-word workaround (`Enum` variant)
- `#[serde(rename = "$ref_id")]` — JSON Schema `$ref` syntax
- `#[serde(tag = "type")]` — internally-tagged `DistributionConfig` (single word)
- `#[serde(tag = "value_type")]` — internally-tagged `AttributePayload`
