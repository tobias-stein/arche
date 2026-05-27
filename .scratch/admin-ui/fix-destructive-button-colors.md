---
title: Fix destructive button text and icon readability
status: completed
---

## What to build

In `admin-ui/src/index.css`, `--destructive` and `--destructive-foreground` are set to the same color value (`oklch(0.577 0.245 27.325)`) in both light and dark modes. This makes red/destructive buttons render with invisible text and icons (red on red).

Fix by setting `--destructive-foreground` to a high-contrast light color in both modes, matching the pattern of `--primary-foreground`.

## Acceptance criteria

- [ ] `--destructive-foreground` is a light/white color that contrasts with `--destructive`
- [ ] Red danger buttons show visible text and icons in both light and dark modes
- [ ] No regressions in other button styles

## Blocked by

None — can start immediately.
