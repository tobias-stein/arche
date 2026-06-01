---
title: "Spellbook has 7 slots instead of the designed 6"
status: needs-triage
---

## Description

The spellbook shows 7 spell slots, but the design (prototype and PRD) specifies 6 spell slots — 5 filled + 1 empty, arranged in a 6-column grid. The config value `capacity.spellbookSlots` is set to `7` instead of `6`.

## Current behaviour

- `GAME_CONFIG.capacity.spellbookSlots` is `7` in both `config.ts:84` and `GameState.ts:57`
- `InventoryDialog.tsx:764` iterates `GAME_CONFIG.capacity.spellbookSlots` to render spell slots — rendering 7 slots
- The spells grid uses `grid-template-columns: repeat(6, 1fr)` (6 columns), so 7 slots creates an orphan in the last row

The prototype shows the spell grid as "6 columns × 2 rows, 6 filled + 1 empty" which appears to describe 7 visual slots. The user explicitly states there should be 6 spell slots total.

## Expected behaviour

- `GAME_CONFIG.capacity.spellbookSlots` should be `6`
- Exactly 6 spell slots rendered in the spellbook grid
- The grid layout should be adjusted accordingly (e.g., `grid-template-columns: repeat(3, 1fr)` for 2 rows × 3 columns, or `repeat(6, 1fr)` for 1 row × 6 columns)

## Acceptance criteria

- [ ] `spellbookSlots` changed from `7` to `6`
- [ ] Only 6 spell slots are rendered in the inventory dialog
- [ ] Grid layout looks correct with 6 slots (no orphan cells)
