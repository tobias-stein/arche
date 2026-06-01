---
title: "Dialog styling does not match the established colour palette and prototype"
status: completed
---

## Description

Various dialogs (loot popup, encounter prompt, combat overlays, etc.) use hardcoded colours that deviate from the project's established design system. The user reports that the styling doesn't correspond to the styles and colour palette defined in the PRD and prototypes.

## Current behaviour

The project has an established colour palette defined in the PRD and demonstrated in the HTML prototypes under `prototypes/HUD/`. The colour tokens include:

- Card background: `#2d4d38`
- Card border: `#4a7a58`
- Card shadow: `#1a3a2a`
- Dark overlay background: `rgba(10, 25, 15, 0.85)` (encounter) or `rgba(10, 25, 15, 0.95)` (inventory)
- Danger elements: `#4a1a1a` background, `#6a2a2a` border
- Text colours: white for headings, `#88b898` for labels, `#d8f0e0` for body text

Specific inconsistencies noted:
- The loot dialog has a `victory-bg` with `rgba(10, 25, 15, 0.5)` — should it match the encounter overlay at `0.85`?
- The encounter overlay uses `rgba(10, 25, 15, 0.85)` — consistent but may need review against inventory dialog's `rgba(10, 25, 15, 0.95)`
- The combat overlay background CSS may use different colours than the prototypes
- Various dialog shadows, borders, and text colours should be audited against the prototype CSS

## Expected behaviour

- All dialog components should use a consistent colour palette matching the design tokens.
- A full audit of CSS across all dialog components against the prototype HTML/CSS and PRD design tokens.
- Extract common CSS variables or tokens to a shared location to prevent drift.

## Acceptance criteria

- [ ] All dialogs use consistent background, border, and shadow colours
- [ ] Dialog styles match the prototype HTML/CSS references
- [ ] Common colour tokens are extracted to CSS variables or a shared file
