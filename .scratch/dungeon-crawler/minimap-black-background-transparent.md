---
title: Minimap full-screen overlay has opaque black background instead of transparent
status: completed
---

## Summary

When the large minimap overlay is opened (press `M` or click the small minimap), the `<canvas>` within the overlay is filled with an opaque black background (`ctx.fillStyle = '#000'; ctx.fillRect(0, 0, cw, ch)`). This fills the entire canvas black, making it impossible to see any elements behind it — the minimap wrapper's own background and frame are completely hidden.

## Expected behavior

The minimap canvas background should be fully transparent so that the minimap wrapper's CSS background (`#minimap-wrap`) and its border/frame show through as the backdrop. The portal/tunnel effect of the overlay wrapper should be visible behind the minimap content.

## Current implementation

### Small minimap (`drawSmallMinimap`, `MiniMap.tsx:118-119`)
```typescript
ctx.fillStyle = '#000';
ctx.fillRect(0, 0, w, h);
```

### Large minimap (`drawLargeMinimap`, `MiniMap.tsx:184-185`)
```typescript
ctx.fillStyle = '#000';
ctx.fillRect(0, 0, cw, ch);
```

## Fix

Replace the opaque black fill on both canvases with transparent fill:
- `ctx.fillStyle = 'transparent'` (or simply omit the fill — the canvas is cleared via `canvas.width = w` which resets to transparent)

## Files to modify

- `src/components/MiniMap.tsx` — lines 118–119 and 184–185
