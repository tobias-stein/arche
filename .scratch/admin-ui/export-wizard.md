---
title: Export wizard
status: ready-for-agent
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the Export wizard page:

Route: `/export`

1. **Client table**: list of all clients with per-row checkboxes (pre-selected)
2. **Per-client toggles**: "Export API keys", "Export audit log" (core tables always exported)
3. **Global toggle**: "Inline global references" — resolves `$ref_id` to inline values in the export
4. **Export button**: triggers `POST /api/export` with the selected options → browser downloads the resulting ZIP file

5. **Export progress**: show loading spinner during export preparation. For large exports, show a progress indicator.

6. **Download handling**: the API returns `application/zip`. The browser initiates a file download with a sensible filename (e.g., `arche-export-2026-05-22.zip`).

## Acceptance criteria

- [ ] Client table renders with checkboxes (all selected by default)
- [ ] Per-client toggles for API keys and audit log
- [ ] Global "inline global refs" toggle
- [ ] Export button calls API and downloads ZIP
- [ ] Loading state during export
- [ ] Error handling: show toast on failure
- [ ] Download filename includes date
- [ ] All options sent in API request correctly

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
- arche-service/export-endpoint.md
