---
title: Fix dialog overlay staying dark after sub-dialog closes
status: ready-for-agent
---

## What to build

When a sub-dialog (`GlobalAttributePickerDialog` or `AffixPickerDialog`) closes inside `BlueprintFormModal`, the parent dialog's dark overlay either doesn't return or the sub-dialog's overlay persists, leaving the screen unnavigably dark. Clicking the dark background also closes the parent dialog.

The current fix uses `hideOverlay` which conditionally mounts/unmounts the parent `DialogOverlay`. This conflicts with Radix's animation state machine — when the sub-dialog closes animation runs at the same z-50 level as the parent overlay mounts, causing flickering, persistent darkness, or intercepting clicks meant for the parent.

**Fix:** Instead of conditionally rendering the overlay (`{!hideOverlay && <DialogOverlay />}`), always keep it mounted but toggle its opacity and pointer-events:

```tsx
<DialogOverlay
  className={cn(
    hideOverlay && 'opacity-0 pointer-events-none',
    className,
  )}
/>
```

This eliminates the mount/unmount animation conflict entirely. The parent overlay stays in the DOM and just fades out/in smoothly.

## Acceptance criteria

- [ ] Opening `GlobalAttributePickerDialog` from BlueprintFormModal: parent overlay fades out, sub-dialog overlay appears
- [ ] Selecting a global attribute: sub-dialog closes, parent overlay fades back in cleanly
- [ ] Opening `AffixPickerDialog` (prefix/suffix): same smooth behavior
- [ ] Opening `ConfirmDialog` (delete attribute / remove pool entry): same smooth behavior
- [ ] Escape key and click-outside still work correctly for all dialogs
- [ ] Existing tests pass

## Blocked by

None - can start immediately
