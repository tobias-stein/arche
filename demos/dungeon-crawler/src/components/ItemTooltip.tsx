import { getRarityColor } from '../game/loot'
import { getGameState } from '../GameState'
import type { ItemState } from '../types'
import { EQUIP_SLOT_LABELS } from '../types'
import { ceil } from '../utils/format'

export interface TooltipData {
  item: ItemState
  x: number
  y: number
  pinned: boolean
}

interface Props {
  tooltip: TooltipData | null
  onClose: () => void
}

function getCompareItem(item: ItemState): ItemState | null {
  if (!item.equipSlot) return null
  return getGameState().equipment[item.equipSlot] ?? null
}

function statDiff(current: number, compare: number): string {
  if (compare > current) return '(+)'
  if (compare < current) return '(-)'
  return ''
}

export default function ItemTooltip({ tooltip, onClose }: Props) {
  if (!tooltip) return null
  const { item, x, y } = tooltip
  const rarityColor = getRarityColor(item.rarity)
  const compareItem = getCompareItem(item)

  return (
    <div
      className="inv-tooltip"
      style={{
        left: Math.min(x, window.innerWidth - 230),
        top: Math.min(y, window.innerHeight - 200),
      }}
      onClick={onClose}
    >
      <div className="tt-name" style={{ color: rarityColor }}>
        {item.name}
      </div>
      <div className="tt-rarity" style={{ background: rarityColor }}>
        {item.rarity}
      </div>
      {item.subtype && <div className="tt-desc">A {item.subtype} item</div>}
      {item.equipSlot && (
        <div className="tt-slot">{EQUIP_SLOT_LABELS[item.equipSlot]}</div>
      )}
      <div className="tt-stats">
        {Object.entries(item.stats).map(([key, val]) => (
          <span key={key}>{key}: {ceil(val)}</span>
        ))}
        {item.level > 0 && <span>Level: {ceil(item.level)}</span>}
      </div>
      {compareItem && (
        <div className="tt-compare">
          <div className="tt-clbl">Equipped:</div>
          <div style={{ color: getRarityColor(compareItem.rarity) }}>
            {compareItem.name}
          </div>
          {Object.entries(compareItem.stats).map(([key, val]) => (
            <span key={key} className="tt-cstat">
              {key}: {ceil(val)} {item.stats[key] !== undefined && statDiff(item.stats[key], val)}
            </span>
          ))}
        </div>
      )}
      {!compareItem && item.equipSlot && (
        <div className="tt-compare">
          <div className="tt-clbl">Equipped:</div>
          <span className="tt-cstat empty">(empty)</span>
        </div>
      )}
    </div>
  )
}
