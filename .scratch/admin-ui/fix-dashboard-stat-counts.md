---
title: Fix dashboard stat counts always showing zero
status: ready-for-agent
---

## What to build

The Dashboard stat cards (Total Blueprints, Total Affixes, Warnings) always show zero even when data exists. The root cause is that the list queries (`useBlueprintsList`, `useAffixesList`, `useGlobalMetaAttributesList`) in `Dashboard.tsx` only pass `{ perPage: 500 }` without a `page` parameter. The backend pagination logic routes to **cursor-based pagination** when `page` is absent, which returns `total: null` in the response. Adding `page: 1` forces **offset-based pagination**, which includes the actual `total` count.

Also note the backend clamps `perPage` to a max of 200, so passing `500` is effectively 200. The `perPage` value should also be reduced to the effective max to avoid misleading readers.

## Acceptance criteria

- [ ] Total Blueprints card shows the actual count from the API
- [ ] Total Affixes card shows the actual count from the API
- [ ] Warnings count is computed correctly from loaded data
- [ ] Existing tests pass (stat card tests assert correct totals)
- [ ] Stat numbers are not impacted by pagination limits when data set exceeds `perPage` (total comes from COUNT query, not data array length)

## Blocked by

None - can start immediately
