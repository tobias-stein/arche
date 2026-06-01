---
title: Inventory drag-and-drop to equipment slots broken; equipable items missing headers; ghost element stuck on abandon dialog
status: ready-for-agent
---

## Summary

Three distinct but related bugs in the inventory dialog's drag-and-drop and display logic:

### Bug 1: Dragging items to empty equipment slots silently fails (items "pop back")

When dragging an equipment item from the inventory to an equipment slot, the item "pops back" (the drag is cancelled and nothing happens) **when the target equipment slot is empty**.

**Root cause**: In `InventoryDialog.tsx`, empty equipment slots have `item = equipment[equipSlot]` which is `null/undefined`. The `onMouseUp` handler on the equipment slot div is:

```tsx
onMouseUp={(e) => item && handleDrop(e, equipSlot, undefined, 'equipment')}
```

When `item` is `null`, `item && handleDrop(...)` evaluates to `null`, so `handleDrop` is **never called** on empty equipment slots. The only drop handler that runs in this case is `dropAtPosition()` (from the `window.mouseup` listener), which relies on `window.event` (non-standard, deprecated API) to get cursor coordinates. When `window.event` is `undefined`, coordinates are `(0,0)`, the target lookup fails, and `cleanupDrag()` resets everything without performing the swap.

Dragging to **occupied** equipment slots works because `item` is truthy and `handleDrop` fires before the window handler.

**Fix**: Remove the `item &&` guard from `onMouseUp` on equipment slots (and similarly on inventory slots that receive drops), so `handleDrop` is always called when the mouse is released, regardless of whether the target slot has an item. The `handleDrop` function already handles null/empty cases safely.

### Bug 2: Equipable items in inventory lack slot header labels

Equipment slots in the equipment section show labels like `WEAPON`, `HEAD`, `CHEST`, etc. (via CSS `::before` or the `data-equip-slot` attribute). However, **equipable items in the inventory grid** do not show these labels, even though they have an `equipSlot` property and can be equipped.

**Expected**: Items in the inventory that have an `equipSlot` should display the slot label (e.g., `WEAPON`, `HEAD`) in their inventory slot, so the player knows which equipment slot they belong to before dragging.

**Fix**: In `renderInventorySlots()` (lines 727–760), check if the item has an `equipSlot` property and render the slot label (e.g., `EQUIP_SLOT_LABELS[item.equipSlot]`) below the item name or as a small badge.

### Bug 3: Drag ghost element stuck on mouse after abandon dialog opens

When dragging an item to the DROP zone, the abandon confirmation dialog opens. The floating drag element (`drag-float`) is still visible and stuck to the mouse cursor behind/over the dialog. It should be hidden/removed immediately when the dialog appears.

**Current flow** (`dropAtPosition` → `showAbandonConfirm`):
1. `dropAtPosition()` detects target is trash zone
2. Calls `showAbandonConfirm(dragState)` which sets `pendingAbandon` and opens dialog
3. The `window.mouseup` handler's `onMouseUp` has a guard `!pendingAbandon`, so `dropAtPosition` won't be called again until `pendingAbandon` is cleared
4. But **`cleanupDrag()` is never called** when the abandon dialog is shown — only when the dialog is confirmed or cancelled

**Root cause**: The `dropAtPosition()` function calls `showAbandonConfirm()` but does **not** call `cleanupDrag()`. The drag float element (`dragFloat`) remains in the DOM and continues following the mouse via the `mousemove` listener (which is still active because `dragActive` is still `true`).

**Fix**: In `showAbandonConfirm()` (or in the trash-zone branch of `dropAtPosition`), call a partial cleanup that removes the drag float element and the `.dragging` class from the source, but keeps `dragState` and `pendingAbandon` alive (or restores them if `cleanupDrag` nullifies them). Alternatively, reset the float position off-screen or hide it with `display: none`.

## Files to modify

- `src/components/InventoryDialog.tsx`:
  - Remove `item &&` guard from `onMouseUp` on equipment/inventory/spell slot elements (lines 709, 743, 779)
  - Hide/remove the `dragFloat` element when the abandon dialog is shown
  - Add equip-slot header rendering in `renderInventorySlots()` for items with `equipSlot`
