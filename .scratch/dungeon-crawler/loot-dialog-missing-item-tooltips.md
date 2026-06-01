---
title: Item tooltips not rendered in loot dialog
status: completed
---

## Summary

Hovering or clicking items in the loot dialog (`LootPopup`) does not show item tooltips. Tooltips only appear for items in the inventory dialog (`InventoryDialog`). Items in the loot dialog should also show tooltips when hovered or clicked/pinned.

## Expected behavior

- Hovering an item in the loot dialog should show a tooltip with the item's name, rarity, subtype, stats, and equipment slot info
- Clicking an item in the loot dialog should pin the tooltip (same behavior as inventory items)
- The tooltip should be visually consistent with the inventory tooltips

## Current implementation

The `LootPopup.tsx` renders each loot item as (lines 215–227):

```tsx
<div
  key={item.id}
  className={`ld-item ${idx === selectedIdx ? 'selected' : ''}`}
  data-rarity={item.rarity}
  onClick={() => handleTakeItem(item.id)}
>
  <span className="li-icon" style={{ color: rarityColor }}>
    <i className={icon} />
  </span>
  <span className="li-name">{item.name}</span>
  <span className="li-rarity-corner" ... />
</div>
```

There are no `onMouseEnter`/`onMouseOver`/`onMouseLeave` handlers that could trigger tooltips, and no tooltip state/render logic in the component.

## Fix

Add tooltip support to `LootPopup.tsx` matching the pattern used in `InventoryDialog.tsx`:
- Add `tooltip` state (item, position, pinned flag)
- Add `handleMouseEnter`/`handleMouseLeave` handlers on each loot item
- Add `handleTooltipClick` to pin on click
- Render the tooltip overlay in the same style as `InventoryDialog`'s `renderTooltip()`
- The tooltip rendering logic can be a shared function/component to avoid duplication

## Files to modify

- `src/components/LootPopup.tsx` — add tooltip state, handlers, and render

## Additional context

The tooltip render logic in `InventoryDialog.tsx:560-612` uses:
- `getRarityColor(item.rarity)` for color
- `item.name`, `item.rarity`, `item.subtype`, `item.equipSlot`, `item.stats`, `item.level`
- Compare tooltip (`getCompareItem`) showing currently equipped item if applicable
- Pin/unpin on click

Consider extracting the tooltip component to a shared file (`ItemTooltip.tsx`) to avoid duplication between `LootPopup` and `InventoryDialog`.
