---
title: Import wizard
status: completed
---

## Parent

Arche Admin UI PRD (see `.scratch/admin-ui/prd.md`)

## What to build

Implement the Import wizard page:

Route: `/import`

**Step 1 — File picker**: drag-and-drop zone for ZIP file upload. Also has a "Browse files" button. Validates file type on drop. Shows file name and size after selection.

**Step 2 — Preview**: after uploading, the API parses the ZIP and returns a preview: clients found, resource counts per client, any client name collisions. User reviews and confirms.

**Step 3 — Client collision handling**: for each client name collision, a prompt: "Import into existing client [name]" or "Create new client".

**Step 4 — Begin Import**: calls `POST /api/import`. If no conflicts, shows success summary. If conflicts detected, transitions to:

**Step 5 — Conflict Resolution**: 
- Table of conflicting resources (type, name, per-attribute diff)
- Per-attribute resolution dropdown (keep_old / keep_new)
- Per-resource "Apply to all attributes" dropdown
- Top bar: "Apply to all remaining" button (keep_old or keep_new)
- "Apply Resolutions" button → calls `POST /api/import/resolve` → success/error summary toast

Progress bar or stepper throughout the wizard. Back/forward navigation between steps.

## Acceptance criteria

- [ ] Drag-and-drop ZIP upload with validation
- [ ] Preview step shows client and resource counts
- [ ] Client name collision handling (import into existing vs create new)
- [ ] Conflict resolution step shows per-attribute diff and dropdowns
- [ ] Per-resource and global "apply to all" shortcuts
- [ ] "Apply Resolutions" calls resolve endpoint
- [ ] Success/error toast with import summary
- [ ] Wizard navigation (back/forward)
- [ ] Loading states for each step
- [ ] Error handling for invalid ZIP or import failures

## Blocked by

- admin-ui/api-client-generation.md
- admin-ui/app-shell-layout.md
- arche-service/import-parse-conflict-detection.md
- arche-service/import-resolve-commit.md
