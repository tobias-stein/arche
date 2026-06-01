---
title: "Flee action does not show confirmation dialog"
status: completed
---

## Description

Pressing the "Flee" action in combat immediately executes the flee without any confirmation dialog. The PRD explicitly specifies a flee confirmation panel with "Yes, Flee" / "Stay & Fight" buttons.

## Current behaviour

In `CombatOverlay.tsx:130-133`, the flee handler:

```typescript
const handleFlee = useCallback(() => {
  const gs = getGameState()
  gs.fleeCombat()
}, [])
```

This immediately flees combat with no confirmation step.

The PRD (section "Combat Mechanics", item "Flee") specifies:

> **Flee** — confirmation dialog with "Yes, Flee" / "Stay & Fight".

And the UI spec describes a **Flee panel**:

> **Flee panel:** Confirmation card. Header ("Flee from combat?"), sub-text ("Cowardice has its rewards."), two buttons: "Yes, Flee" (red/danger) and "Stay & Fight" (green/confirm).

## Expected behaviour

- When the player selects "Flee" from the action menu, a confirmation dialog/card should appear.
- The dialog should show a header: "Flee from combat?"
- Sub-text: "Cowardice has its rewards."
- Two buttons: "Yes, Flee" (red/danger style) and "Stay & Fight" (green/confirm style).
- "Yes, Flee" executes the flee logic: stuns the creature, ends combat, moves player back.
- "Stay & Fight" returns to the action menu.
- Keyboard: Enter on "Yes, Flee" to confirm, Esc to cancel.

## Acceptance criteria

- [ ] Flee confirmation dialog appears when Flee is selected from action menu
- [ ] "Yes, Flee" button executes the flee (creature stun, combat end, player return)
- [ ] "Stay & Fight" button returns to the combat action menu
- [ ] Dialog uses correct styling (danger/red for confirm, green for cancel)
- [ ] Keyboard navigation works (Enter to confirm, Esc to cancel)
