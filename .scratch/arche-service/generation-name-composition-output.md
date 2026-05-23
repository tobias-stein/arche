---
title: Generation engine — name composition & output assembly
status: ready-for-agent
---

## Parent

Arche API Service PRD (see `.scratch/arche-service/prd.md`)

## What to build

Implement the final two phases of generation:

1. **Name composition:** Concatenate selected affix names with the blueprint's base name:
   - Format: `[prefix1] [prefix2] [base_name] [suffix1] [suffix2]`
   - Return both the full composed name and the individual parts (base, prefixes list, suffixes list)

2. **Output assembly:** Combine blueprint attributes and affix attributes into the final response:
   - `blueprint_attributes`: a JSON object keyed by attribute name, in `attribute_order` sequence
   - `affix_attributes`: an array of objects, each containing `affix_id`, `affix_name`, and the affix's attribute key-value pair. Ordered by `sort_order` on the `blueprint_affixes` join row.
   - The service does NOT combine or interpret attribute values — that is the consuming game's responsibility.

The output structure matches the API spec response format.

## Acceptance criteria

- [ ] Name composition: prefixes + base + suffixes, with proper spacing
- [ ] No affixes: just the base name returned
- [ ] Name parts: base, prefixes list, suffixes list all present
- [ ] Blueprint attributes ordered by attribute_order
- [ ] Affix attributes ordered by sort_order
- [ ] Each affix entry includes affix_id, affix_name, and the rolled attribute
- [ ] Unit tests for name formatting edge cases (empty prefix list, empty suffix list, etc.)

## Blocked by

- arche-service/generation-blueprint-selection.md
- arche-service/generation-attribute-rolling.md
- arche-service/generation-affix-selection.md
