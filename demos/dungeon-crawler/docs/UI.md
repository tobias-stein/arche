# Dungeon Crawler Demo — UI Design

## Overview

The game UI is rendered as React overlays on top of a Phaser.js canvas. All HUD elements are HTML/CSS (React), not Phaser sprites. The Phaser canvas handles dungeon tilemap, player movement, creature sprites, and scene transitions. React handles the player card, enemy card, action menus, inventory/spellbook side panels, unified log, mini-map, and dialogs.

## Responsive Design

The UI must function on mobile, tablet, and desktop. All controls and actions must work with touch-only, mouse-only, and keyboard-only input. No interaction may rely on hover alone.

### Breakpoints

| Breakpoint | Width | Layout changes |
|---|---|---|
| Mobile | < 640px | Single-column, stacked. Side panels become full-screen overlays. Action menu becomes vertical list. Player card shrinks. Log becomes a bottom tab. Map is full-screen (unchanged). HUD buttons move to a bottom toolbar. |
| Tablet | 640–1024px | Adapted two-column. Side panels are wider overlays (not full-screen). Action menu stays 2×2 but larger touch targets. Player card is compact. Log is a resizable overlay. |
| Desktop | > 1024px | Full layout as specified. Side panels slide from right. Everything at default sizing. |

### Layout adaptation by breakpoint

**Mobile:**
- Player card: collapsed to a thin top bar showing only HP bar + level. Tap to expand full stats.
- Player card (combat): same collapsed bar. Enemy card is a dismissible panel at top-right.
- Action menu (combat): single column vertical list (full-width buttons, 56px min height).
- Side panels (inventory/spellbook): full-screen overlay (not sliding partial panel).
- Floating tooltip: not possible on mobile. Instead, selecting an item shows stats inline below the slot (the comparison text occupies screen space below the grid).
- Unified log: a small expandable bottom tab (tap to open full-screen overlay).
- Map: full-screen (unchanged).
- Help: full-screen overlay (unchanged).
- HUD buttons (I, S, M, L, H): consolidated into a bottom toolbar (persistent, 56px height) with icon-only buttons.

**Tablet:**
- Player card: compact (two-line: HP/MP bar row + stats row).
- Side panels: overlay at 60% viewport width (not full-screen).
- Floating tooltip: enabled (touch → tap to select → tooltip appears above/below item, auto-dismiss after 3s or tap elsewhere).
- All other layouts as desktop.

**Desktop:**
- Full layout as specified.
- Floating tooltip: hover to show, or tap to pin (remains open until tap elsewhere).

### Touch/mouse input coverage

Every action available via keyboard must also be available via touch and mouse:

| Action | Mouse | Touch | Keyboard |
|---|---|---|---|
| Move player | Click destination tile | Tap destination tile | WASD / Arrow keys |
| Engage encounter | Click "Engage" button | Tap "Engage" button | Enter |
| Back away | Click "Back away" | Tap "Back away" | Esc |
| Attack (combat) | Click Attack button | Tap Attack button | Arrow + Enter |
| Cast spell | Click Cast → select spell | Tap Cast → tap spell | Arrow + Enter |
| Use item (combat) | Click Use → select item | Tap Use → tap item | Arrow + Enter |
| Flee (combat) | Click Flee button | Tap Flee button | Arrow + Enter |
| Open inventory | Click [I] button | Tap [I] button | I |
| Open spellbook | Click [S] button | Tap [S] button | S |
| Navigate inventory | Click slot | Tap slot | Arrow keys |
| Select inventory item | Click/hover slot | Tap slot (tooltip appears) | Arrow keys |
| Equip item | Click equip action in tooltip | Tap equip button in tooltip | Enter (on selected item) |
| Consume potion | Click consume action | Tap consume button | Enter |
| Drop item | Click drop action | Long-press slot → "Drop" confirm | X |
| Unequip item | Click unequip action | Tap unequip button | U |
| Add spell to book | Click spellbook action | Tap spellbook button | S |
| Erase spell | Click erase action | Tap erase button | Del / Backspace |
| Toggle mini-map | Click [M] button | Tap [M] button | M |
| Toggle log | Click log tab/button | Tap log tab/button | L |
| Toggle help | Click [H] or [?] button | Tap [H] or [?] button | H / ? |
| Close any panel | Click close/X button | Tap close/X button | Esc |
| Quit | Click Quit button | Tap Quit button | Q |
| Restart (game over) | Click "Play Again" | Tap "Play Again" | Enter |
| Take loot item | Click item in list | Tap item in list | Enter |
| Take all loot | Click "Take All" | Tap "Take All" | Space |
| Leave loot | Click "Leave All" | Tap "Leave All" | Esc |
| Open loot while in inventory | Click loot tab/back | Tap loot tab/back | Esc (close inventory) |

### Touch interaction patterns

- **Tap**: primary action (select, confirm, navigate).
- **Long-press**: destructive action (drop item) — shows confirmation dialog.
- **Swipe**: dismiss side panels (swipe right-to-left to close inventory/spellbook).
- **No hover-dependent interactions**: any tooltip or menu triggered by hover on desktop must also be triggerable by tap on touch devices.

### Touch target sizing

All interactive elements must have a minimum touch target of 44×44px (WCAG 2.1 guideline). Buttons, grid cells, inventory slots, log entries, map rooms, tooltip actions all follow this rule.

### Floating tooltip (touch behavior)

- Desktop: hover to show, click to pin (persists until clicking elsewhere or pressing Esc).
- Tablet: tap an inventory slot to select → tooltip appears above or below the slot. Auto-dismisses after 3s idle or tap elsewhere.
- Mobile: tooltip is not used. Selecting an inventory slot shows stats inline below the slot instead (text occupies space within the side panel, not as a floating element).

---

## Layout
```

### Element positions

| Element | Position | Phase | Behavior |
|---|---|---|---|
| Player card | Top-left, fixed | All | Always visible. Shows HP bar, MP bar, ATK, DEF, level, XP bar. |
| Enemy card | Bottom-right | Combat only | Creature name (with difficulty badge), HP bar, ATK, DEF. Appears on combat fade-in, disappears on combat end. |
| Action menu | Center | Combat only | 2×2 grid layout. Attack, Cast Spell, Use Item, Flee. Only visible during `combat_player_turn` phase. |
| Unified log | Bottom-left | All | Collapsible overlay panel. Detailed list with timestamps and vector icons. FontAwesome for icon set. Auto-scrolls to newest entry. ~4 visible entries at default height. Resizable (future). |
| Mini-map | Top-Right, Full-screen | All | Toggle with `[M]` key or map button. Shows dungeon grid layout. Visited rooms lit, unvisited dark. Fog-of-war within visited rooms (cleared step by step). Breadcrumb trail. Current position highlighted. |
| Inventory | Side panel (right) | Exploration + Loot | Slides in from right. 18 grid slots. Floating tooltip for item comparison with equipped gear. Accessible via `[I]` key. |
| Spellbook | Side panel (right) | Exploration + Loot | Slides in from right (same panel area, different tab/view). 7 slots. Accessible via `[S]` key. |
| Controls overlay | Full-screen | All | Triggered by `[H]` or `[?]` key, or clickable help icon. Lists all keyboard/mouse/touch bindings. |
| Title screen | Full-screen | Startup | Simple title screen. "Press Enter to start" or tap to begin. |
| Game over | Full-screen overlay | Death | Centered "GAME OVER". Stats summary. `[Enter]` or button to restart. `[Q]` to quit. |
| Victory | Full-screen overlay | Boss slain | Centered "YOU WIN". Stats summary. `[Enter]` or button to restart. `[Q]` to quit. |
| Encounter prompt | Centered card | Encounter | Creature info + difficulty badge. "Engage [Enter]" / "Back away [Esc]" choice. Dungeon dims behind. |

## Player Card (persistent)

Always visible top-left. Compact card with:

- **HP bar**: Colored bar (green → yellow → red based on %), numeric value to the right. Label: `<i class="fa-solid fa-heart" style="color:#ff4444"></i>` or red heart icon.
- **MP bar**: Blue bar, numeric value to the right. Label: `<i class="fa-solid fa-star"></i>` or blue mana icon.
- **Stats row**: ATK: value — DEF: value
- **Level row**: Lv. N — XP bar (small) — XP: current/next

Transitions smoothly between exploration and combat (same position, same visual).

## Enemy Card (combat only)

Appears in bottom-right on combat fade-in. Shows:

- **Creature name**: Bold, with difficulty badge (colored tag: Normal, Champion, Elite, BOSS)
- **HP bar**: Red bar with numeric value (current/max)
- **Stats**: ATK: value — DEF: value

Disappears when combat ends.

## Action Menu (combat only)

2×2 grid centered on screen. Each cell is a button with icon + label:

| `<i class="fa-solid fa-crosshairs"></i>` Attack | `<i class="fa-solid fa-wand-sparkles"></i>` Cast Spell |
|---|---|---|
| `<i class="fa-solid fa-flask"></i>` Use Item | `<i class="fa-solid fa-person-running"></i>` Flee |

Keyboard: Arrow keys to navigate grid, Enter to confirm. Mouse/touch: click/tap button.

Navigating into Cast Spell or Use Item opens the spellbook or inventory side panel respectively (overlaid on the combat scene).

## Unified Log

Collapsible panel in bottom-left. Toggle: click a log icon in the HUD or press `[L]` key (tbd). Default state: collapsed (shows only a small tab/icon with latest entry preview).

### Entry format

```
[timestamp] [icon] message
```

Example:
```
[12:34]  `<i class="fa-solid fa-crosshairs"></i>`  You hit Goblin Champion for 12 damage.
[12:34]  `<i class="fa-solid fa-skull"></i>`  Goblin Champion slain! +35 XP
[12:35]  `<i class="fa-solid fa-box-open"></i>`  Iron Sword (uncommon) picked up.
[12:36]  `<i class="fa-solid fa-flask"></i>`  Minor Health Potion consumed. +15 HP.
[12:36]  `<i class="fa-solid fa-door-open"></i>`  Entered Ancient Chamber.
```

### Icon set (FontAwesome)

| Event | FA icon | Color |
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

### Max visible entries

~4–5 entries at default height. Older entries scroll up. Panel can be expanded/collapsed.

## Mini-Map

Full-screen overlay triggered by `[M]` key or map button.

### Mini-map render

- Each room shown as a grid cell on a simplified representation of the dungeon layout
- **Unexplored rooms**: Dark/black (not shown at all)
- **Visited rooms**: Lit cell with floor color
- **Current room**: Highlighted (pulsing border or bright indicator)
- **Fog-of-war**: Within visited rooms, fog clears tile-by-tile as player walks
- **Breadcrumb trail**: Faint dotted line showing path walked through visited rooms
- **Doors**: Small gaps or lines between connected rooms (only shown if both ends visited)
- **Boss room**: Special marker (skull icon) when discovered
- **Entrance**: Special marker (stairs/arrow icon)

## Inventory Side Panel

Slides in from the right edge. Dungeon/combat dims behind.

- **18 slots** in a grid layout
- Empty slots shown as "empty" indicators
- Item icons + names in each slot
- Arrow keys (or mouse) to navigate
- When an item is selected: **floating tooltip** appears next to it showing:
  - Item name, rarity badge, description
  - Slot this item goes into
  - The item's stats (damage, defense, stat bonus)
  - Comparison: currently equipped item in that slot with its stats (or "empty")
- Actions per item (shown in tooltip or bottom bar):
  - `[Enter]` — equip (gear) or consume (potions)
  - `[S]` — add to spellbook (spell items only)
  - `[X]` — drop item
  - `[U]` on an equipped slot — unequip (shows in inventory as well)

### Inventory during loot

When the loot popup is active, inventory remains accessible. Player can open it to drop/equip/consume items to free space, then return to looting.

## Spellbook Side Panel

Same area as inventory (right slide panel). Shows:

- **7 slots** in a column or grid
- Each slot shows spell name, element, mana cost, damage/heal
- Empty slots clearly indicated
- Actions:
  - `[Del]` / `[Backspace]` — erase spell (with confirmation or instant for the spellbook-is-full case)
- During loot: accessible alongside the loot popup

### Spell replacement flow (loot)

1. Player tries to take a spell from loot
2. If spellbook is full (7/7 slots occupied):
   - A replacement dialog appears over the loot popup
   - Shows the 7 current spells and the new spell
   - Player selects a slot to replace
   - Confirmation: "Replace [old spell] with [new spell]?"
   - If confirmed: old spell is discarded, new spell occupies the slot
   - Dialog closes, back to loot popup
3. If spellbook has space: spell goes directly into the first empty slot

## Loot Popup

Centered card after victory. Shows:

- "VICTORY!" title with XP gain and level-up notification
- Vertical list of dropped items
- Each item: icon + name + type/rarity badge
- Actions:
  - `[↑/↓]` — navigate items
  - `[Enter]` — take selected item (auto-equips or goes to inventory/spellbook)
  - `[Space]` — take all items
  - `[Esc]` — leave all loot behind
  - `[I]` — open inventory (side panel)
  - `[S]` — open spellbook (side panel)

Player can freely toggle inventory/spellbook while the loot popup is open to manage space.

## Encounter Prompt

Centered card on a dimmed dungeon background when player walks into a creature tile.

- Creature name (bold) with difficulty badge
- Brief stats (HP: value, ATK: value, DEF: value)
- Options:
  - `[Enter]` — Engage (fades into combat scene)
  - `[Esc]` — Back away (player steps back one tile, creature remains)

## Game Over Screen

Full-screen overlay when player HP reaches 0.

- Large "GAME OVER" text
- Stats: Level reached, rooms explored, enemies slain, time survived
- `[Enter]` or "Play Again" button — restart from beginning
- `[Q]` or "Quit" button — return to title screen

## Victory Screen

Full-screen overlay when boss is slain.

- Large "YOU WIN!" text
- Stats: Final level, rooms explored, enemies slain, items collected
- `[Enter]` or "Play Again" button — restart
- `[Q]` or "Quit" button — return to title screen

## Title Screen

Full-screen on initial load.

- Game title (large, centered)
- Subtitle / tagline
- "Press Enter to Start" or tap-to-play
- Background: static dungeon-themed graphic or animated pattern

## Controls Overlay

Full-screen overlay. Toggle with `[H]` or `[?]` key, or clickable help button in HUD. Dismiss with any key or tap.

```
MOVEMENT          COMBAT
  [W/A/S/D] move    [↑/↓/←/→] navigate menu
  [Arrow keys]      [Enter] confirm action
  [Click] move to   [Click] tap button

INVENTORY         MAP
  [I] toggle inv    [M] toggle map
  [S] toggle book   [Esc] close

GENERAL           LOOT
  [L] toggle log    [Enter] take selected
  [H][?] help       [Space] take all
  [Esc] back        [Esc] leave all
  [Q] quit
```

Mouse/touch: all interactive elements (buttons, inventory slots, map tiles, log entries) are clickable/tappable.

## Transitions

| Transition | Effect |
|---|---|
| Dungeon → Encounter prompt | Screen dims, card fades in |
| Encounter → Combat | Fade to combat scene (solid dark background, no dungeon) |
| Combat → Victory/Loot | Brief flash, loot popup fades in over dimmed dungeon |
| Combat → Game Over | Screen dims to red, game over fade-in |
| Room transition (door) | Phaser scene slide/fade to adjacent room |
| Inventory/Spellbook toggle | Slide from right edge, ~200ms ease-out |
| Log toggle | Slide/fade from bottom-left edge |
| Mini-map toggle | Quick fade-in of full-screen overlay |
| Title → Dungeon | Cross-fade, dungeon generation happens during transition |

## Controls

| Key | Context | Action |
|---|---|---|
| W / Arrow Up | Exploration | Move player up |
| S / Arrow Down | Exploration | Move player down |
| A / Arrow Left | Exploration | Move player left |
| D / Arrow Right | Exploration | Move player right |
| I | Exploration, Loot | Toggle inventory side panel |
| S | Exploration, Loot | Toggle spellbook side panel |
| M | All | Toggle mini-map overlay |
| L | All | Toggle unified log |
| H / ? | All | Toggle controls overlay |
| Enter | Detection, Combat, Loot | Confirm action / take item |
| Space | Loot | Take all items |
| Esc | Various | Back / cancel / leave loot |
| Q | All | Quit to title |
| Del / Backspace | Spellbook | Erase selected spell |
| X | Inventory | Drop selected item |
| U | Inventory (equipped) | Unequip item |

All keyboard actions also available via mouse/touch clicks on UI elements.
