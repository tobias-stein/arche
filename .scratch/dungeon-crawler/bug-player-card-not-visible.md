---
title: "Player card is never rendered on screen"
status: needs-triage
---

## Description

The player card (top-left HUD element showing avatar, name, HP/MP bars, ATK/DEF, XP) never appears during gameplay. The card was implemented in slice 04 and should be visible at all times after the game starts.

## Current behaviour

`PlayerCard.tsx:9` initialises `visible` state as `false`. It becomes `true` only when:
1. The `game:started` event fires (`onGameStarted` handler sets `visible = true`)
2. Or on mount, if `gs.gameStarted` is already `true`

The component renders `if (!visible) return null` at line 42.

Possible causes:
- The `game:started` event may not be emitted correctly by GameState
- The game might start but the event subscription doesn't trigger before the initial render
- CSS stacking/z-index issue: the Phaser canvas or other overlays might be covering the player card
- The card renders but with zero dimensions due to missing CSS or broken layout
- The card renders with `position: absolute` and `top: 16px; left: 16px; z-index: 50` which should place it above the canvas, but the parent stacking context might be different

## Expected behaviour

- Player card is visible at the top-left of the screen as soon as the game starts (title screen dismissed)
- Card shows: avatar icon, player name, level, HP bar (red gradient) with numeric values, MP bar (blue gradient) with numeric values, ATK/DEF stats row, XP bar
- Card matches the prototype styling (`#2d4d38` background, `#4a7a58` border, pixel-art 3D shadow)
- Clicking the card opens the inventory dialog

## Acceptance criteria

- [ ] Player card renders immediately after game starts
- [ ] All stats display correctly (HP, MP, ATK, DEF, level, XP)
- [ ] HP/MP bars animate on change
- [ ] Card is positioned top-left and visible above the Phaser canvas
