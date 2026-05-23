---
title: arche import command — interactive
status: ready-for-agent
---

## Parent

Arche CLI PRD (see `.scratch/arche-cli/prd.md`)

## What to build

Implement the `arche import` command with interactive conflict resolution:

- `arche import <path>` — imports a ZIP archive. `<path>` is required.
- The command calls `POST /api/import` with the ZIP file content
- If no conflicts: prints success summary to stdout
- If conflicts detected (409): enters interactive mode
  1. Prints a summary of conflicts found
  2. For each conflicting resource, prompts the user for resolution strategy:
     - `keep_old` — keep existing
     - `keep_new` — use imported version
     - `per_attribute` — per-key resolution
  3. For `per_attribute` strategy, prompts for each conflicting attribute key
  4. After all resolutions collected, calls `POST /api/import/resolve`
  5. Prints final success/error summary

- Terminal prompts: colored, formatted, with default suggestions and validation
- "Apply to all remaining" shortcut available at each prompt

## Acceptance criteria

- [ ] `arche import backup.zip` sends ZIP to API and handles response
- [ ] No conflicts: success message to stdout
- [ ] Conflicts detected: enters interactive prompt loop
- [ ] Each conflict: shows resource name, type, and per-attribute diff
- [ ] Resolution strategy selection (keep_old, keep_new, per_attribute)
- [ ] Per-attribute resolution when per_attribute selected
- [ ] "Apply to all remaining" shortcut
- [ ] After resolution, calls resolve endpoint
- [ ] Success/error summary displayed
- [ ] Integration test: export → modify → import with conflicts → resolve

## Blocked by

- arche-cli/cli-scaffolding.md
- arche-service/import-parse-conflict-detection.md
- arche-service/import-resolve-commit.md
