---
title: "Inventory dialog renders behind/over the loot popup instead of side-by-side"
status: completed
---

## Description

When looting a defeated enemy or a chest, only the loot popup is shown. Opening the inventory (via the "Inventory" button in the loot popup or the `[I]` key) opens the inventory dialog, but it renders either behind or on top of the loot popup instead of being displayed side-by-side as shown in the prototype.

## Current behaviour

- `LootPopup.tsx` renders at `z-index: 260` (line 3 in CSS)
- `InventoryDialog` renders at `z-index: 250` (line 2 in CSS)
- Both are full-screen overlays with `position: fixed; inset: 0`
- Pressing `[I]` during loot calls `gs.emitInventoryRequested()` which sets `showInventory = true` in `App.tsx`
- The inventory dialog appears as a full-screen modal on top of everything (or behind the loot popup, depending on z-index)

In the prototype (`hud-battle-view.html`), the loot dialog and inventory are shown **side-by-side**. The inventory/equipment dialog appears to the right (or is positioned such that both are visible simultaneously), allowing the player to manage inventory space while deciding which loot items to take.

## Expected behaviour

- When the inventory is opened during a loot session, the inventory dialog should appear alongside the loot popup (not as a full-screen modal that covers it).
- The loot popup should remain visible and interactive.
- The player should be able to take loot items, see them appear in inventory, and manage inventory space — all without closing the loot popup.
- Reference the prototype `hud-battle-view.html` for the exact side-by-side layout.

## Acceptance criteria

- [ ] Inventory dialog and loot popup are visible simultaneously during looting
- [ ] Taking a loot item immediately updates the visible inventory
- [ ] Player can drag items between inventory and equipment while loot is open
- [ ] Closing inventory via Esc returns focus to loot popup without dismissing loot
