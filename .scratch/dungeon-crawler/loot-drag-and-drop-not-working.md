---
title: Loot dialog items cannot be dragged into inventory + inventory not updated after taking loot
status: ready-for-agent
---

## Summary

Two related bugs in the loot-to-inventory flow:

1. **No drag-and-drop on loot items**: The `LootPopup` component renders loot items as `.ld-item` divs that only support `onClick` (take one item) and a "Take All" button. There is no drag initiation (`onMouseDown`, `onDragStart`, etc.) on loot items, so dragging from the loot dialog into the inventory is impossible.

2. **Inventory state not updated after taking loot**: When a loot item is taken via `takeLootItem()` or `takeAllLoot()`, the `GameState` calls `addItemToInventoryOrEquip()` which mutates `this.inventory` / `this.equipment` but does **not** emit `inventory:changed`. The `InventoryDialog` subscribes to `inventory:changed` to sync its local state — so when loot is taken, the inventory dialog's local copy of `inventory`/`equipment` never reflects the new items.

## Expected behavior

- Items in the loot dialog should be draggable into inventory/equipment slots in the inventory dialog (when both are open simultaneously).
- After taking loot (via click, drag, or Take All), the inventory dialog should immediately reflect the new items and their positions.

## Root causes

### 1. No drag support on loot items (`LootPopup.tsx`)
The `.ld-item` elements in `LootPopup.tsx` (lines 215–227) only have `onClick` and keyboard handlers. There are no `onMouseDown`, `onMouseMove`, `onMouseUp` handlers to initiate or participate in the drag-and-drop flow defined in `InventoryDialog.tsx`.

The drag system in `InventoryDialog.tsx` uses module-level mutable state (`dragState`, `dragFloat`, etc.) and listens for `mousemove`/`mouseup` on `window`. This design *could* support cross-component drag, but the loot items need to initiate the drag by setting `dragState` and creating the drag float element.

### 2. Missing `inventory:changed` emission (`GameState.ts`)
Taking loot calls `addItemToInventoryOrEquip()` (line 352) which modifies `this.inventory`/`this.equipment` but emits no event:
```typescript
private addItemToInventoryOrEquip(item: ItemState): void {
  if (item.equipSlot && !this.equipment[item.equipSlot]) {
    this.equipment[item.equipSlot] = item
    return
  }
  const emptyIdx = this.inventory.findIndex(slot => slot === null)
  if (emptyIdx !== -1) {
    this.inventory[emptyIdx] = item
  }
}
```

The method should emit `inventory:changed` after mutating, so that any open `InventoryDialog` reacts to the change.

## Files to modify

- `src/components/LootPopup.tsx` — add drag initiation handlers (`onMouseDown`) to `.ld-item` elements
- `src/GameState.ts` — emit `inventory:changed` in `addItemToInventoryOrEquip()`

## Additional context

When adding drag initiation to loot items, ensure the drag state format matches what `InventoryDialog.tsx` expects in `handleDragStart()` (lines 284–363):
- `sourceType: 'inventory'` (loot items go to inventory first)
- `sourceIdx` needs to map to the loot item index
- `itemType`, `equipSlot`, `itemId`, `html`, `itemName`, `itemIcon`, `rarity` must be populated
- The drag float element must be created and appended to `document.body`

Consider extracting the drag initiation logic into a shared function/dedicated hook so both components reuse the same drag-start code.
