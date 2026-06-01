---
title: Doors rendered with pulsing golden glow instead of tunnel-like curved archway
status: ready-for-agent
---

## Summary

Doors in the dungeon room are rendered with a pulsing golden/orange glow effect (`drawDoorGlow`) on top of a pointed (straight-line) archway cutout. The intended visual from the `room-layout-and-assets` prototype is a **curved tunnel archway** without the golden glow — just the dark tunnel background carved into the wall with quadratic bezier curves.

## Current rendering (`src/game/room-renderer.ts`)

For each door tile (`DOOR_N/S/E/W`), the `drawRoom()` function calls:

```typescript
drawArchway(graphics, px, py, tileSize, DOOR_DIR[t]);
drawDoorGlow(graphics, px, py, tileSize, time, DOOR_DIR[t]);
```

### `drawArchway()` (lines 44–87)
Draws a pointed arch with **straight lines** (`lineTo`), not curved:

```typescript
if (dir === 'N') {
    graphics.moveTo(px + pad, py + ts - pad);
    graphics.lineTo(px + pad, s);
    graphics.lineTo(cx, py + pad);     // ← pointed tip, not curved
    graphics.lineTo(px + ts - pad, s);
    graphics.lineTo(px + ts - pad, py + ts - pad);
}
```

### `drawDoorGlow()` (lines 96–111)
Overlays a pulsing golden rectangle with random sparkles:

```typescript
const glow = 0.5 + 0.5 * Math.sin(time * 0.003);
// ... golden rects and sparkles
```

## Intended visual (from `prototypes/room-layout-and-assets/room-layout-and-assets.html`)

The archway uses `quadraticCurveTo` for smooth curved entries, and **no glow effect**:

```javascript
if (dir === 'N') {
    gcx.moveTo(px + pad, py + ts - pad);
    gcx.lineTo(px + pad, s);
    gcx.quadraticCurveTo(px + pad, py + pad, cx, py + pad);    // ← curved arch
    gcx.quadraticCurveTo(px + ts - pad, py + pad, px + ts - pad, s);
    gcx.lineTo(px + ts - pad, py + ts - pad);
}
```

## Required changes

1. **Replace `lineTo` with `quadraticCurveTo`** in `drawArchway()` to match the curved tunnel shape from the prototype, for all four directions (N/S/E/W).

2. **Remove or disable `drawDoorGlow()`** from the door rendering path. Doors should be carved archways into the wall, not glowing tiles.

3. Keep the `PALETTE.tunnelBg` (`#0a1a10`) as the fill color for the archway cutout.

## Files to modify

- `src/game/room-renderer.ts` — rewrite `drawArchway()` to use `quadraticCurveTo`, remove `drawDoorGlow()` call from `drawRoom()`
