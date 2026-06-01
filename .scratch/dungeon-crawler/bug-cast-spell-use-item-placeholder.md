---
title: "Cast Spell and Use Item actions still show placeholder alerts"
status: completed
---

## Description

In combat, the "Cast Spell" and "Use Item" action buttons show a browser `alert()` saying they will be wired in slice 9, despite the corresponding issue (`.scratch/dungeon-crawler/09-encounter-and-screens.md`) being marked as completed.

## Current behaviour

In `CombatOverlay.tsx:122-128`, both callbacks call `alert()`:

```typescript
const handleCastSpell = useCallback(() => {
  alert('Cast Spell — will be wired in slice 9')
}, [])

const handleUseItem = useCallback(() => {
  alert('Use Item — will be wired in slice 9')
}, [])
```

These actions were supposed to be wired in slice 9 (encounter screens / combat UI), but the implementation was never completed.

## Expected behaviour

- **Use Item:** Opens a sub-panel/modal showing the player's inventory items. The player selects an item to use/consume (e.g., health potion). The item is consumed from inventory and its effect is applied (HP heal, MP restore, etc.).
- **Cast Spell:** Opens a sub-panel/modal showing the player's spellbook. The player selects a spell to cast. The spell is cast on the enemy (damage, debuff) or on the player (heal, buff). MP cost is deducted.
- Both sub-panels should be keyboard-navigable (arrow keys to select items/spells, Enter to confirm, Esc to cancel back to action menu).
- The existing PRD and issue 11 (combat sub-panels) should be referenced for the exact UI/UX design.

## Acceptance criteria

- [ ] "Use Item" opens an inventory item picker during combat
- [ ] "Cast Spell" opens a spellbook picker during combat
- [ ] Consumables can be used from inventory during combat
- [ ] Spells can be cast during combat with MP cost applied
- [ ] Esc returns to the main combat action menu
- [ ] No `alert()` calls remain for these actions
