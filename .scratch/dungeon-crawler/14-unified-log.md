---
title: "Unified log"
status: completed
---

## What to build

Implement the collapsible activity log panel in the bottom-left corner.

**Position and sizing:**
- Bottom-left, fixed. Default collapsed: shows a small tab/icon with latest entry preview.
- Expanded: max-height `min(45vh, 500px)`, width `min(50vw, 500px)`. ~4–5 visible entries at default.
- Toggle via `[L]` key or click log header.
- Styling matches the HUD prototype: green card (`#2d4d38` bg, `#4a7a58` border) with pixel-art shadow.

**Entry format:**
```
[timestamp] [icon] message
```
Example:
```
[12:34] 🔫 You hit Goblin Champion for 12 damage.
[12:34] 💀 Goblin Champion slain! +35 XP
[12:35] 📦 Iron Sword (uncommon) picked up.
```

**Icons (FontAwesome) and colors per event type:**
| Event | Icon | Color |
|---|---|---|
| Player deals damage | `fa-crosshairs` | White |
| Player takes damage | `fa-shield-hacked` | Red |
| Enemy slain | `fa-skull` | Red |
| Spell cast | `fa-wand-sparkles` | Cyan |
| Potion consumed | `fa-flask` | Green |
| Item picked up | `fa-box-open` | Yellow |
| Item dropped | `fa-arrow-up-from-bracket` | Gray |
| Room entered | `fa-arrow-right-to-bracket` | White |
| Encounter started | `fa-triangle-exclamation` | Yellow |
| Level up / Victory | `fa-star` | Gold |
| Flee attempt | `fa-person-running` | Gray |

**Behavior:**
- Auto-scrolls to newest entry. If scrolled up, a "scroll to top" button appears.
- Oldest entries roll off as new ones arrive (no max limit — memory-bound, ~200-ish).
- Timestamps are relative to game start (MM:SS format).

**Log events emitted by GameState:** Player attacks, creature attacks, damage taken, creature slain, spells cast, items picked up/dropped/equipped, potions consumed, rooms entered, encounters started, flee attempts, level ups, victory/defeat.

## Prototype references

- `demos/dungeon-crawler/prototypes/HUD/hud-dungeon-view.html` — the activity log (`#log`) with collapsible header, auto-scrolling body, scroll-to-top button, entry styling with FontAwesome icons. The full CSS and structure are directly reusable.
- `demos/dungeon-crawler/prototypes/HUD/hud-battle-view.html` — the same log, shown in the combat context. Confirms it works identically across phases.

## Acceptance criteria

- [ ] Log panel visible in bottom-left, collapsed by default
- [ ] Click/tap header or `[L]` toggles expanded view
- [ ] Entries show timestamp, icon, and message with correct event colors
- [ ] Auto-scrolls to newest entry
- [ ] Scroll-to-top button appears when scrolled up
- [ ] Events from all game actions appear in the log

## Blocked by

04-title-screen-player-card
