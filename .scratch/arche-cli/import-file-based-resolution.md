---
title: arche import — file-based resolution
status: ready-for-agent
---

## Parent

Arche CLI PRD (see `.scratch/arche-cli/prd.md`)

## What to build

Implement the file-based import resolution modes:

- `arche import --dry-run <path>` — calls `POST /api/import`, receives conflicts, writes the conflicts JSON to stdout (not stderr). Does NOT apply the import. The output is a conflicts JSON that the user can edit and feed back.

- `arche import --resolve-file resolutions.json <path>` — calls `POST /api/import`, receives conflicts, then reads the specified resolution file, validates it, and calls `POST /api/import/resolve` with the resolutions.

The resolution file format matches the shared schema defined in `arche-types`:
```json
{
  "import_token": "uuid-from-service",
  "resolutions": {
    "<resource-uuid>": {
      "strategy": "keep_old" | "keep_new" | "per_attribute",
      "attributes": {
        "<attribute-key>": "keep_old" | "keep_new"
      }
    }
  }
}
```

The CLI validates the resolution file against the conflict response (detects invalid resource UUIDs, invalid strategies, etc.) before sending to the API.

## Acceptance criteria

- [ ] `--dry-run` writes conflicts JSON to stdout (import NOT applied)
- [ ] `--dry-run` when no conflicts: prints "No conflicts" message
- [ ] `--resolve-file resolutions.json`: reads resolutions, validates, sends to API
- [ ] Validation error in resolution file: error to stderr with details, exit code 2
- [ ] Resolution file matches the shared arche-types schema
- [ ] Integration test: dry-run → edit resolution file → resolve-file → verify
- [ ] Cross-tool compatible: resolution file from Admin UI works with CLI and vice versa

## Blocked by

- arche-cli/import-interactive.md
- arche-types/export-import-format.md
