---
title: "Room floor tiles and walls not rendered — only dark Phaser background visible"
status: completed
---

## Description

The Phaser canvas shows only the dark greenish-black background (`#0a0a0a`). The room tiles (walls, floor checkerboard, doors) are not drawn. The player character, creatures, and chests are also not visible.

## Current behaviour

- The Phaser game config in `GameComponent.tsx:18` sets `backgroundColor: '#0a0a0a'`.
- `DungeonScene.create()` calls `generateDungeon()` which calls `drawCurrentRoom()`.
- `drawCurrentRoom()` calls `drawRoom()` (from `room-renderer.ts`) which iterates over all tiles and draws walls, floor, and doors using Phaser `Graphics` objects.
- Despite this, only the background colour is visible on screen.

## Suspected root causes (any or all may apply)

1. **Tile size calculation** — `tileSize = Math.min(floor(innerWidth/15), floor(innerHeight/11))` may be zero or too small if the container dimensions are not yet ready when the Phaser scene creates.
2. **Phaser RESIZE scale mode** — The game uses `Phaser.Scale.RESIZE` which may not properly size the canvas relative to the parent div. The canvas might have 0 dimensions.
3. **Offscreen offset** — `offsetX`/`offsetY` could place tiles outside the visible canvas if `tileSize` is miscalculated.
4. **Graphics depth/Z-order** — The `roomGraphics` object has default depth (0). The `vignetteGraphics` draws on top but with `fillStyle(0x000000, 0)` which should be transparent, but the loop might paint opaque black over everything.
5. **Drawing coordinates** — The Phaser Graphics fill/stroke calls might use coordinates that land outside the visible area.
6. **Scene doesn't start properly** — The BootScene immediately transitions to DungeonScene, but there could be an error during scene creation that silently fails.

## Expected behaviour

- Room walls (dark green bricks), floor (checkerboard pattern with two green tones), and door archways with golden glow are visible in the Phaser canvas.
- The player character (circle) is visible.
- Creature triangles and chest sprites are visible.

## Acceptance criteria

- [ ] Room tiles are rendered in the Phaser canvas with correct wall, floor, and door visuals
- [ ] Player character is visible
- [ ] Creatures and chests are visible
- [ ] No console errors related to Phaser rendering
