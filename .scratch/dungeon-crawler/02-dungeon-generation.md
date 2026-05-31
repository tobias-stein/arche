---
title: "Dungeon generation + room rendering"
status: ready-for-agent
---

## What to build

Implement the dungeon generation algorithm and render rooms as Phaser tilemaps.

**Dungeon generation (Prim's growth):**
- Rooms placed on an N×N grid (default 64×64, configurable via `GAME_CONFIG.dungeon.gridSize`).
- Randomized Prim's spanning tree: start from random seed cell, maintain a frontier, randomly pick to add as a room. Target occupancy ~55% (`GAME_CONFIG.dungeon.targetRoomRatio`).
- Each room records its parent during growth forming the spanning tree.
- Adjacency built via spatial Map keyed by `"x,y"` (O(n), not O(n²)).
- Extra edges: ~20% of non-tree adjacencies (`GAME_CONFIG.dungeon.extraEdgeRatio`) randomly added for loops/shortcuts.
- Boss room: BFS from entrance over connection set, room with greatest graph distance wins.
- Connection set (tree edges + extra edges) determines which adjacent rooms have doors.

**Room tilemap:**
- Each room is 15×11 tiles (configurable via `GAME_CONFIG.room.width`/`height`).
- Outer ring = walls, interior = checkerboard floor (two alternating green tones).
- Doors at N/S/E/W midpoints — only if connection set has an edge to that neighbor.
- Door rendering: archway shape (dark void with arched cutout in wall direction), pulsing golden glow (sine wave intensity).
- Wall rendering: dark green/grey fill with lighter top/left edge highlights, darker bottom/right shadows. Brick line decoration at alternating rows. Shadow line every 3rd column.
- Vignette: radial gradient overlay (center transparent → edges `rgba(0,0,0,0.45)`) over entire room.

**Room transition:**
- 350ms fade-to-black overlay (`GAME_CONFIG.dungeon.transitionMs`) when player steps through a door.

Entity colors from the PROTOTYPE palette (see PRD Design System section):
- Wall: `#2d4d38` base, `#4a7a58` hi, `#1a3a2a` lo, `#1a2e22` dark, `#3a5a44` brick
- Floor: `#3a6a48` light, `#2a4a38` dark, `rgba(0,0,0,0.12)` lines
- Door tunnel: `#0a1a10`
- Vignette: `rgba(0,0,0,0.45)` at edges

## Prototype references

- `demos/dungeon-crawler/prototypes/room-generator-minimap/room-generator-minimap.html` — fully working prototype of the Prim's growth dungeon generation. READ this file carefully for the algorithm (Dungeon class, generate(), BFS boss placement, adjacency building, connection set). Also see the accompanying `room-generator-minimap.md` for detailed findings on grid size, performance, and decisions.
- `demos/dungeon-crawler/prototypes/room-layout-and-assets/room-layout-and-assets.html` — room tile rendering with walls, floor checkerboard, door archways, and vignette. The `drawWallTile()`, `drawRoom()`, and `drawArchway()` functions contain pixel-exact rendering decisions (brick line offsets, shadow edge widths, archway curves). Translate these from Canvas 2D to Phaser Graphics.
- `demos/dungeon-crawler/prototypes/HUD/hud-dungeon-view.html` — the room transition fade overlay is implemented as a CSS opacity transition on a fixed black div (see `#transition-overlay`). The scroll container for the large minimap (`.map-scroll`) shows the zoom/pan pattern.
- `demos/dungeon-crawler/docs/PRD.md` — section 5 (Design System) has the palette; section 6 (Asset Specification) has tile rendering specs.

## Acceptance criteria

- [ ] Clicking "Generate" (or on page load) produces a connected dungeon with rooms, walls, floor, doors
- [ ] Some rooms have N/S/E/W doors, some have fewer — matches the connection set
- [ ] Boss room exists and is the farthest from entrance
- [ ] Rendering uses Phaser Graphics API (not Canvas 2D or sprite sheets)
- [ ] All tile colors match the prototype palette
- [ ] Room transitions fade to black and back

## Blocked by

01-project-scaffold
