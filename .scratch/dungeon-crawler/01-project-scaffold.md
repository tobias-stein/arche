---
title: "Project scaffold + core loop"
status: completed
---

## What to build

Set up the full project skeleton for the dungeon-crawler demo:

- **Vite + React + TypeScript** project with Phaser.js 3.x installed
- **Docker Compose** file that spins up postgres:16, arche-service (Rust API server), and the dungeon-crawler Vite dev server. The game SPA calls `POST /api/generate` directly from the browser (Arche has `CorsLayer::permissive()` for local dev).
- **Phaser embedded in React** — a Phaser game instance created inside a `useEffect`, rendered into a `<div>` ref. React DOM overlays sit above the canvas via CSS absolute positioning.
- **GameState class** — a plain TypeScript class extending EventEmitter. Single source of truth for all game state. Phaser reads state sync per frame, React subscribes reactively via `useEffect` + `useState`. The class emits events like `player:stats-changed`, `room:entered`, `combat:started`, etc.
- **Game config file** — a single TypeScript object at a location of the agent's choosing that defines all tweakable game mechanics (player base stats, XP thresholds, recovery %, combat formulas, creature spawn params, aggro ranges, loot drop ranges, rarity weights, capacities, room/dungeon dimensions, movement speeds). Loaded once at boot before GameState is initialized. All types must be exported so game systems `import { GAME_CONFIG } from './config'`.
- **Pixel Game font** — include the `PixelGame-Regular.otf` font file (download from 1001fonts.com) in the project and load via `@font-face` in CSS. Apply globally: `font-family: 'Pixel Game', monospace`.
- **Folder structure** left to the agent's discretion — see PRD Technical Specification section for the high-level org (game engine code, game state, React UI, integration, infrastructure). Do not prescribe exact file paths.
- **Empty Phaser scene** renders (a blank canvas) so the page loads without errors.

## Prototype references

- `demos/dungeon-crawler/prototypes/HUD/hud-dungeon-view.html` — shows the full Dungeon view (exploration) with HUD. Use this as reference for styles, colors, and component layout.
- `demos/dungeon-crawler/prototypes/HUD/hud-battle-view.html` — shows the Battle view with combat HUD.
- `demos/dungeon-crawler/docs/PRD.md` — Design System section (section 5) has the full color palette, typography, spacing, and shadow tokens extracted from the prototypes.
- `demos/dungeon-crawler/docs/PRD.md` — Game Configuration section (section 9) has the full config schema.

## Acceptance criteria

- [ ] `docker compose up` starts all three services
- [ ] Page loads in browser, Phaser canvas renders, no console errors
- [ ] GameState class exists, can be imported, emits events
- [ ] Game config file is loadable with all the fields from the PRD config section
- [ ] Pixel Game font is loaded and applied globally
- [ ] Arche service responds to `POST /api/generate` from the browser

## Blocked by

None — can start immediately
