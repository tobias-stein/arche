---
title: "Unified inventory/equipment/spellbook dialog"
status: ready-for-agent
---

## What to build

Implement the unified modal dialog containing Equipment (10 slots), Inventory (18 slots), and Spells (7 slots). This is the central inventory management screen.

**Trigger:**
- `[I]` key or click/tap the player card.
- Can be opened during exploration and during loot popup.

**Layout (follows the HUD prototype exactly):**
- Centered modal dialog, ~480px wide, max 92vh height.
- Three labeled sections stacked vertically:

1. **Equipment** — grid: 5 columns × 2 rows (10 slots total). Each slot shows item icon + truncated name, plus a label bar across the top indicating the equipment slot name (WEAPON, HEAD, CHEST, LEGS, FEET, HANDS, BELT, RING, NECK, OFFHAND).
2. **Inventory** — grid: 6 columns × 3 rows (18 slots total). Empty slots shown as empty indicators. Items show icon + truncated name.
3. **Spells** — grid: 6 columns × 2 rows (7 slots, last cell empty). Spells show icon + name.

- Drop zone bar at the bottom (visually hidden until drag starts): "DROP" area with trash can icon.
- Close button (×) in header.

**Item display:**
- Items have a colored rarity corner triangle: common `#888`, uncommon `#4488ff`, rare `#44cc66`, legendary `#aa44ff`.
- Equipment items show their equip slot label in a top bar.

**Floating tooltip (item comparison):**
- When an item is hovered/selected in any section, a floating tooltip appears showing:
  - Item name, rarity badge (colored), description
  - Slot this item goes into
  - Item stats (damage, defense, stat bonus)
  - Comparison: currently equipped item in that slot with its stats (or "empty")
- Desktop: hover to show, click to pin. Tablet: tap to select → auto-dismiss after 3s. Mobile: inline comparison below the slot (no floating element).

**Drag and drop:**
- Items can be dragged between inventory slots, between inventory and equipment (matching slot type), and to the DROP zone.
- Drag creates a floating clone of the item with golden glow border.
- Valid drop targets highlighted green, invalid highlighted red.
- Dropping on occupied slot: swap items.
- Dropping on DROP zone: "Abandon [item]? This item will be destroyed." confirmation dialog with "Yes, Delete" / "Cancel".

**Actions (keyboard):**
- `[Enter]` on gear: equip/unequip. On potion: consume. On spell: add to spellbook.
- `[X]` on any item: drop with abandon confirmation.
- `[U]` on equipped slot: unequip to inventory.
- `[Del]` / `[Backspace]` on spell: erase with confirmation.

**Equipment slots (from config):** weapon, helmet, chest, legs, boots, gloves, belt, ring, amulet, shield.

**Spell replacement dialog:**
- When spellbook is full (7/7) and player tries to add a spell from inventory/loot:
  - Dialog shows 7 current spells + the new spell.
  - Player selects a slot to replace.
  - Confirmation: "Replace [old spell] with [new spell]?"
  - Confirmed: old spell discarded, new spell occupies slot.

**Capacity from config:**
- `GAME_CONFIG.capacity.inventorySlots` (18)
- `GAME_CONFIG.capacity.spellbookSlots` (7)

## Prototype references

- `demos/dungeon-crawler/prototypes/HUD/hud-dungeon-view.html` and `hud-battle-view.html` — BOTH contain the identical equipment dialog (`#equipment-dialog`) with:
  - Equipment grid (5 columns × 2 rows) with equip-slot labels
  - Inventory grid (6 columns × 3 rows, 18 slots)
  - Spells grid (6 columns × 2 rows, 6 filled + 1 empty)
  - Drag-and-drop with `drag-float` floating clone, green/red validity highlighting, DROP zone with abandon confirmation (`#abandon-dialog`)
  - Rarity corner triangles via CSS `::after`
  - Close button, section titles
  - The entire HTML structure and CSS are directly reusable for the React component

## Acceptance criteria

- [ ] `[I]` key or player card click opens the unified dialog
- [ ] Dialog shows Equipment (10), Inventory (18), Spells (7) sections
- [ ] Items show correct icons, names, rarity corners, and equip-slot labels
- [ ] Floating tooltip shows item comparison with equipped gear
- [ ] Drag-and-drop works between sections and to DROP zone
- [ ] Abandon confirmation appears on drop zone action
- [ ] Spell replacement dialog works when spellbook is full
- [ ] All keyboard actions work (Enter, X, U, Del)
- [ ] Dialog closes on Esc or × button
- [ ] Auto-equip works when picking up gear with empty slot

## Blocked by

10-loot-system
