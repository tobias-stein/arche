---
title: "Responsive design + touch"
status: ready-for-agent
---

## What to build

Make the entire game responsive across mobile, tablet, and desktop. All interactions must work with touch, mouse, and keyboard.

**Breakpoints (matching UI.md):**
| Breakpoint | Width | Changes |
|---|---|---|
| Mobile | < 640px | Single-column. Inventory dialog = full-screen. Action menu = vertical list. Player card = collapsed top bar (tap to expand). Log = bottom tab. HUD buttons = bottom toolbar (icon-only, 56px height). |
| Tablet | 640–1024px | Adapted two-column. Inventory dialog = 60% viewport overlay. Action menu = 2×2 larger targets. Player card = compact (two-line). Log = resizable overlay. |
| Desktop | > 1024px | Full layout. Desktop defaults. |

**Mobile-specific:**
- Player card collapses to a thin top bar showing only HP bar + level. Tap to expand full stats.
- Action menu (combat): single column vertical list, full-width buttons, 56px min height.
- Inventory dialog: full-screen overlay (not centered modal).
- Floating tooltip not used on mobile. Selecting an item shows stats inline below the slot.
- Unified log: expandable bottom tab (tap to open full-screen overlay).
- HUD buttons (I, M, L, H): consolidated into bottom toolbar (persistent, 56px height) with icon-only buttons.
- Mini-map: full-screen overlay (unchanged from desktop).

**Touch interactions:**
- All interactive elements must have minimum touch target of 44×44px (WCAG 2.1).
- Tap: primary action (select, confirm, navigate).
- Long-press: destructive action (drop item) — shows confirmation dialog.
- Swipe: dismiss dialogs (right-to-left swipe to close inventory).
- No hover-dependent interactions. Any tooltip triggered by hover on desktop must also be triggerable by tap on touch devices.

**Input coverage:**
Every action available via keyboard must also be available via touch and mouse. See `demos/dungeon-crawler/docs/UI.md` for the full 75-entry action table. Key additions for mobile:
- Move player: Tap destination tile.
- Navigation: Tap buttons/controls (no hover).
- Item comparison: Tap slot shows inline comparison below (mobile) or floating tooltip (tablet+).

**Implementation approach:**
CSS media queries for breakpoints. React components use a custom `useBreakpoint()` hook or `window.matchMedia` to adjust layout. The Phaser canvas resizes via `window.addEventListener('resize')` — tile size recalculated.

## Prototype references

- `demos/dungeon-crawler/docs/UI.md` — the complete responsive design specification with mobile/tablet/desktop breakpoints, touch interaction patterns (tap, long-press, swipe), touch target sizing (44×44px), and the full 75-entry input coverage table. This is THE reference for this slice.
- The prototypes are desktop-only HTML files but can be used to understand the base layout that should be adapted.

## Acceptance criteria

- [ ] Game is playable on mobile viewport (< 640px) with all actions accessible
- [ ] Game is playable on tablet viewport (640–1024px)
- [ ] Desktop layout unchanged (> 1024px)
- [ ] All touch targets ≥ 44×44px
- [ ] Tap, long-press, and swipe interactions work
- [ ] No hover-only interactions exist
- [ ] Bottom toolbar appears on mobile with icon-only HUD buttons
- [ ] Player card collapses on mobile, expands on tap
- [ ] Inventory dialog is full-screen on mobile

## Blocked by

12-unified-log
