---
title: "Take All" and click-to-take still work when inventory is full, causing items to vanish
status: completed
---

## Summary

When the inventory is full (all 18 slots occupied and no empty equipment slots to auto-equip into), using "Take All" or clicking individual loot items in the loot dialog still "succeeds" — items are removed from the loot list but silently dropped because `addItemToInventoryOrEquip()` has no room. The items vanish from both the loot list and the inventory, effectively destroying them.

## Expected behavior

- Items should **not** be removable from the loot dialog if the inventory is full
- "Take All" should not take any items if there is insufficient space
- Clicking/taking individual items should not succeed if there is no space
- The player should receive some feedback that their inventory is full (log entry or visual indicator)

## Current implementation

### `GameState.takeLootItem()` (lines 347–358)
```typescript
takeLootItem(itemId: string): void {
    const idx = this.lootItems.findIndex(i => i.id === itemId)
    if (idx === -1) return
    const item = this.lootItems[idx]
    this.lootItems.splice(idx, 1)         // ← item removed from loot immediately
    this.addItemToInventoryOrEquip(item)  // ← silently fails if no space
    // ... log entry ...
    this.emit('loot:items-changed', [...this.lootItems])
}
```

### `GameState.takeAllLoot()` (lines 361–373)
```typescript
takeAllLoot(): void {
    const items = [...this.lootItems]
    this.lootItems = []                   // ← all items cleared from loot
    for (const item of items) {
      this.addItemToInventoryOrEquip(item)  // ← each silently fails if no space
    }
    // ... log entry ...
    this.emit('loot:items-changed', [])
}
```

### `GameState.addItemToInventoryOrEquip()` (lines 436–445)
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
    // If no space: item is silently dropped
}
```

## Fix

**Non-goal**: Creating a full-blown "inventory full" UI modal. The minimal fix is to **guard loot take operations** against a full inventory.

### Option A (minimal): Check before taking
In `takeLootItem()`, `takeAllLoot()`, `takeChestItem()`, and `takeAllChestLoot()`, check if the inventory is full before removing items from the loot list:

```typescript
private isInventoryFull(): boolean {
    return this.inventory.every(slot => slot !== null) 
        && Object.keys(this.equipment).length >= EQUIP_SLOTS.length;
}
```

If full, return early and optionally emit a log entry like "Inventory is full!".

### Option B (better): Count available space
Check if there are enough free slots for the batch operation. For `takeAllLoot`, count how many items could actually fit and only take that many, leaving the rest in the loot list.

## Files to modify

- `src/GameState.ts`:
  - `takeLootItem()` — add inventory-full guard
  - `takeAllLoot()` — add inventory-full guard
  - `takeChestItem()` — add inventory-full guard
  - `takeAllChestLoot()` — add inventory-full guard
  - Optionally: add `isInventoryFull()` helper method and a log entry emission when an item is rejected due to full inventory
