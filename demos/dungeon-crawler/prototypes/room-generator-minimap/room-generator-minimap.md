# Room Generator & Minimap Prototype — Findings

## Dungeon Generation

### Prim's Growth Algorithm
- Rooms are selected via **Randomized Prim's growth**: start from a seed cell, maintain a frontier of adjacent empty cells, randomly pick one to add as a room. This guarantees a connected shape.
- **Target occupancy**: ~55% of grid cells become rooms (adjustable via `targetRoomRatio`).
- For a 64×64 grid, this yields ~2250 rooms.
- **Performance**: frontier array + Set for O(1) dedup is essential. Naive `frontier.some()` O(n) lookup becomes prohibitively slow at this scale.

### Spanning Tree + Extra Edges
- Each room added during Prim's growth records its parent, forming an implicit **spanning tree**.
- After growth, geographic adjacency between rooms is built via a spatial `Map` (keyed by `"x,y"`), which is much faster than O(n²) pair scanning.
- **Extra edges**: 20% of non-tree adjacencies are randomly added to create loops and shortcuts. This prevents the dungeon from being a straight-line traversal.

### Boss Room
- Determined by **BFS from entrance** over the connection set (tree + extra edges only). The room with the greatest graph distance from the entrance becomes the boss room.
- This guarantees the player must explore a significant portion of the dungeon before reaching the boss.

### Connection Model
- `connectionSet` (treeEdges + extraEdges) determines which adjacent rooms actually have doors between them.
- Not all geographically adjacent rooms are connected — only spanning tree edges + selected extra edges.
- This creates a more interesting dungeon where you must navigate around unconnected walls.

## Room Tilemap

### Layout
- Each room is a **15×11 tile grid** (configurable).
- Outer ring is walls, interior is checkerboard floor.
- Doors are wall tiles replaced with glowing door tiles at N/S/E/W midpoints.
- Only rooms connected via `connectionSet` get doors.

## Player Movement

### Hold-to-Move
- Keys track `heldDirs` Set — `keydown` adds, `keyup` removes.
- First move fires immediately on `keydown`; subsequent moves at 110ms interval via cooldown timer.
- Last pressed direction wins (Set iteration order).

### Click-to-Navigate
- BFS pathfinding within the current room's tile grid (15×11 = 165 tiles — trivial performance).
- Path is stored in `state.autoPath` and consumed one tile per tick.
- Holding a direction key cancels the auto-path and returns to manual control.

### Room Transitions
- Stepping on a door tile triggers a 350ms fade to black, then the player appears at the corresponding entrance tile in the adjacent room.
- The fade overlay is a fixed-position div with CSS opacity transition.

## Minimap System

### Two-Level Architecture
- **Small minimap** (always visible, top-right corner): 7×7 room grid centered on the player. Only visited rooms rendered. Player is centered at all times.
- **Large minimap** (toggleable overlay): full dungeon view with zoom slider, scroll-to-pan, and drag-to-pan.

### Small Minimap
- Fixed 18px cells with 2px gap, rendered to a dedicated canvas (#mcs).
- Shows visited rooms + the boss marker (even if undiscovered).
- Player indicator is a cyan dot inside the golden current-room cell.
- Clicking opens the large minimap overlay.

### Large Minimap
- **Fixed-size overlay**: 85vw × 85vh, never changes size.
- Scroll container inside uses `flex: 1; overflow: auto` with `display: flex` + `margin: auto` on the canvas for centering.
- **Zoom**: range slider (0.25×–5×, step 0.25) + scroll wheel. At 1×, the canvas exactly fills the scroll container.
- **Pan**: click-drag on the scroll area. Cursor changes to `grab`/`grabbing`. `user-select: none` during pan.
- **Scroll wheel** always controls zoom (prevented from scrolling the container).

### Fog of War
- Undiscovered rooms are **completely hidden** — not rendered at all.
- Edges between two undiscovered rooms are also hidden.
- The boss room is the only exception: always shown as a dark cell with red border and `?` marker before discovery, switching to red fill + `☠` after discovery.
- Entrance is marked with a green square only after discovery.

### Initial State
- On dungeon generation, the minimap opens at **fill zoom** — the zoom level where the canvas exactly fills the smaller dimension of the scroll container.
- The view scrolls to center on the player's room.

## Performance Observations

### Grid Size 64×64
- ~2250 rooms generated in <1s with optimized frontier (Set-based dedup).
- Spatial Map for adjacency building is O(n) instead of O(n²).
- Canvas at zoom 1× is approximately 800×800px (cell ≈ 9–10px), fitting in an 85vh container.
- Minimap rendering is fast enough to call every frame even at this scale.

### Canvas Sizing
- Cell size at 1× zoom computed from container dimensions: `min(containerW, containerH)` minus gaps and padding, then divided by grid size.
- Cell capped at 28px to prevent absurdly large cells on small grids.
- Minimum cell size is 2px (at extreme zoom-out).

## UI / UX Decisions

### Layout
- Controls (grid size input + Generate button) in top-right.
- Small minimap in top-right, below controls.
- Large minimap overlay centered, covering 85% of viewport.
- Room info (name, index, grid size) in top-left, always visible.
- HUD with keyboard hints at bottom-center.

### Open/Close Flow
- Small minimap is always visible (except when large overlay is open).
- Click small minimap → large overlay opens at fill zoom, centered on player.
- Press `M` or click × button → large overlay closes, small reappears.

### Edge Cases
- Player near dungeon boundary: 7×7 view has fewer rooms on those sides, but player stays centered.
- Room with no connections: no doors render, player is trapped (can regenerate).
- Very small grids (<4): fill zoom is capped at ZOOM_MAX (5×).

## What Would Change for the Real Game

1. **Tile-based rendering** should switch to a proper tilemap (Phaser tilemap / Tiled format) instead of Canvas 2D rectangles.
2. **Multi-room pathfinding** — currently click-to-navigate only works within the current room. A production version would pathfind across the dungeon graph.
3. **Minimap zoom slider** is fine for a prototype but the real game would use discrete zoom levels or a single "full map / zoom to player" toggle.
4. **Room variety** — rooms should have procedural furniture, obstacles, pits, chests, etc. Currently all rooms are identical boxes.
5. **Creature rendering** — the prototype has no enemies; the real game needs them visible on both the room view and the minimap.
6. **Edge connections** on the small minimap aren't drawn (just room cells). Might be useful to show at least the door directions.
