---
title: PlayerCard and ActivityLog are not visible during combat/encounter
status: ready-for-agent
---

## Summary

During an encounter or combat with an enemy, the `PlayerCard` (top-left player stats panel) and the `ActivityLog` (bottom-left event log) are not rendered on screen. They should remain visible throughout combat so the player can see their HP/MP and read combat log entries.

## Expected behavior

- `PlayerCard` should remain visible at all times once the game has started, including during encounters and combat
- `ActivityLog` should remain visible at all times once the game has started, including during encounters and combat
- These are HUD elements that provide critical information to the player

## Investigation needed

Both components are rendered unconditionally in `App.tsx` (lines 145–148):

```tsx
<PlayerCard />
<GameComponent />
<ActivityLog expanded={...} onToggle={...} />
```

Neither component should be hidden during combat. Check for:

1. **CSS z-index layering**: `CombatOverlay` or `EncounterPrompt` may cover the entire viewport with a higher `z-index`, obscuring the `PlayerCard` and `ActivityLog` elements. Inspect the z-index values:
   - `PlayerCard` CSS: `z-index: 50` (needs verification against combat overlay z-index)
   - `ActivityLog` CSS: needs z-index verification
   - `CombatOverlay` / `EncounterPrompt` CSS: may use `z-index: 100+` with full-screen backgrounds

2. **CSS `pointer-events` or `visibility`**: Check if the combat overlay has `pointer-events: all` with a fully opaque background that covers the HUD.

## Likely root cause

The `EncounterPrompt` and/or `CombatOverlay` likely render a full-screen semi-transparent/opaque overlay with a higher z-index than the HUD components, visually covering them.

## Fix

Adjust the CSS z-index layering so that `PlayerCard` and `ActivityLog` are rendered above any combat/encounter overlays, or ensure those overlays do not use full-screen backgrounds that obscure the HUD.

## Files to investigate

- `src/components/PlayerCard.css` — check `z-index`
- `src/components/ActivityLog.css` — check `z-index`
- `src/components/EncounterPrompt.css` — check overlay dimensions and `z-index`
- `src/components/CombatOverlay.css` — check overlay dimensions and `z-index`
