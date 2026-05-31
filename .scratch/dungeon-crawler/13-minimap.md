---
title: "Mini-map"
status: completed
---

## What to build

Implement the two-level mini-map system: small corner minimap and large full-screen overlay.

**Small minimap (always visible):**
- Top-right corner. Canvas-rendered.
- 7×7 room grid centered on the player's current room.
- Only visited rooms rendered. Player stays centered at all times.
- Cell size: 18px with 2px gap.
- Colors: current room `#ffd700` with cyan player dot inside, visited rooms `#5a5a6a` (or `#4a7a58` on the HUD prototype), boss room (undiscovered) dark red `#2a1515` with "?" marker, boss room (discovered) red `#e53935` with "☠" marker.
- Entrance room marked with green square after discovery.
- Click opens large minimap overlay.

**Large minimap overlay (full-screen toggle):**
- Toggle with `[M]` key, or click on small minimap. Close with × button or `[M]`.
- Full view of entire dungeon grid.
- Zoom: slider (0.25×–5×, step 0.25) + scroll wheel. "Fill zoom" on open (canvas fills the scroll container).
- Pan: click-drag. Cursor changes to grab/grabbing.
- Shows all visited rooms + boss room (always visible even if undiscovered).
- Edges (connections) drawn between rooms — gold for current room edges, grey for others.
- Entrance marker (green), current room (gold highlight), boss room (red + skull).
- Legend: "■ Current   ☠ Boss   ❖ Entrance" at bottom.
- Padding 8px, gap between cells 3px.

**Fog of war:**
- Undiscovered rooms not rendered at all (except boss room which shows as dark cell with "?").
- Edges between two undiscovered rooms hidden.
- Within visited rooms: fog clears tile-by-tile as player walks (optional for initial implementation — at minimum, visited rooms are fully lit).

Mini-map rendering uses Canvas 2D (not Phaser) — drawn to dedicated canvases positioned via React. Both small and large minimap read from GameState.dungeon.

## Prototype references

- `demos/dungeon-crawler/prototypes/room-generator-minimap/room-generator-minimap.html` — THE prototype for this slice. Contains:
  - Full dungeon generation algorithm (Dungeon class)
  - Small minimap (`#mcs` canvas) with 7×7 grid, cell coloring, player centering
  - Large minimap overlay (`#minimap-wrap`) with zoom slider, scroll-to-zoom, drag-to-pan, fill-zoom on open, legend
  - All drawing functions (`drawMinimapSmall()`, `drawMinimap()`, `resizeMinimap()`, `setZoom()`)
  - The minimap is completely functional and directly translatable to the final game
  - Also see the accompanying `room-generator-minimap.md` for detailed findings and decisions about the minimap system
- `demos/dungeon-crawler/prototypes/HUD/hud-dungeon-view.html` — shows the minimap in the HUD layout context (small minimap top-right, controls overlay). The minimap here has the same API but is integrated with a React overlay.

## Acceptance criteria

- [ ] Small minimap visible in top-right corner showing 7×7 centered on player
- [ ] Visited rooms lit, unvisited hidden, boss room always shown
- [ ] Clicking small minimap opens full overlay
- [ ] `[M]` key toggles overlay
- [ ] Zoom slider and scroll wheel work
- [ ] Click-drag pan works
- [ ] Overlay shows legend, entrance/boss/current markers
- [ ] Closing overlay returns to small minimap

## Blocked by

03-player-movement
