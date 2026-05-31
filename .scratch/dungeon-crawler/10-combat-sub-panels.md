---
title: "Combat sub-panels (spell, item, flee)"
status: ready-for-agent
---

## What to build

Implement the three overlay panels used during combat: Cast Spell, Use Item, and Flee.

**Spell selection panel:**
- Small centered overlay panel (NOT the full inventory dialog).
- Header: "Select Spell" with close button (×).
- Scrollable list of known spells (from spellbook in GameState).
- Each entry: spell icon (FontAwesome by element), spell name, mana cost (right-aligned, blue `#6080ff`).
- Spells the player lacks mana for are greyed out with `opacity: .4` and `cursor: not-allowed`.
- Selecting a spell: deducts mana, deals damage/heal using spell's stats, closes panel, ends player turn.
- Empty state: "No spells known."
- Styling matches the prototype: green card (`#2d4d38` bg, `#4a7a58` border), scrollbar styled.

**Use Item panel:**
- Same layout/styling as spell panel.
- Header: "Use Item" with close button.
- Scrollable list of consumable items in inventory (potion type items only).
- Each entry: icon (FontAwesome by potion type), item name, effect value (right-aligned, green `#88b898`).
- Selecting an item: consumes it (heals HP/MP), removes from inventory, closes panel, ends player turn.
- Empty state: "No usable items."

**Flee confirmation panel:**
- Centered card, red-themed (`#3a2020` bg, `#6a3a3a` border).
- Header: "Flee from combat?" with close button.
- Sub-text: "Cowardice has its rewards."
- Two buttons: "Yes, Flee" (red danger style) and "Stay & Fight" (green confirm style).
- Flee: applies `GAME_CONFIG.fleeStunDuration` (1500ms) stun to creature, returns player to dungeon at the current room's entry door.

All three panels are React components. Keyboard navigation: Arrow keys to navigate list items, Enter to select, Esc to close. Mouse/touch: click/tap.

## Prototype references

- `demos/dungeon-crawler/prototypes/HUD/hud-battle-view.html` — all three panels are fully implemented in this prototype:
  - Spell panel (`#spell-panel`) — header, close button, scrollable list, spell items with icon/name/mana cost, disabled styling for insufficient mana, empty state.
  - Use Item panel (`#use-item-panel`) — same layout, consumable items with icon/name/effect value, empty state.
  - Flee panel (`#flee-panel`) — red-themed card, header, sub-text, "Yes, Flee" (red) / "Stay & Fight" (green) buttons.
  - All CSS for these panels is directly reusable. Translate to React components.

## Acceptance criteria

- [ ] Cast Spell opens spell list panel with known spells and mana costs
- [ ] Spells with insufficient mana are greyed out and unselectable
- [ ] Selecting a spell deducts mana and deals damage
- [ ] Use Item opens consumables list with effect values
- [ ] Selecting a consumable removes it from inventory and applies effect
- [ ] Flee panel shows confirmation with Yes/No buttons
- [ ] Fleeing applies 1.5s stun and returns player to entry door

## Blocked by

07-combat-core
