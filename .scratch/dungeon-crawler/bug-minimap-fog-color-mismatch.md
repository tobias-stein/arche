---
title: "Minimap fog/background colour has hard cut against dialog frame"
status: needs-triage
---

## Description

The minimap's unvisited/unexplored areas and the overall canvas background use a different colour (`#0a0a0a` for minimap canvas, `#111` for large minimap overlay) than the overlay frame background (`rgba(0,0,0,0.92)`). This creates a visible hard cut between the minimap content area and the surrounding dialog frame.

## Current behaviour

- **Small minimap** (`MiniMap.tsx:118`): canvas background filled with `#0a0a0a`
- **Large minimap overlay** (`MiniMap.tsx:184`): canvas background filled with `#111`
- **Overlay frame** (`MiniMap.css:26`): `background: rgba(0, 0, 0, 0.92)`

Unvisited cells on the large minimap use `#1a1a1e` (line 213). Visited cells use `#5a5a6a`. The player's small minimap `#5a5a6a` colour for visited rooms creates another visual discontinuity against the `rgba(0,0,0,0.92)` frame.

The user expects the minimap to blend in with the entire dialog frame — the fog (unvisited areas) and canvas background should use the same colour as the dialog background so there is no hard cut.

## Expected behaviour

- The large minimap canvas background should match the overlay background colour used by the dialog frame.
- The "fog" (undiscovered rooms shown as dark cells) should use the same colour as the canvas background.
- No hard visual edge should exist between the minimap content and the dialog frame.
- The small minimap background should also blend with its container.

## Acceptance criteria

- [ ] Large minimap canvas background matches the overlay frame background
- [ ] Undiscovered rooms in the minimap are invisible against the canvas background (no visible cells for undiscovered content)
- [ ] No hard cut between minimap content and dialog frame border
- [ ] Small minimap blends similarly with its container
