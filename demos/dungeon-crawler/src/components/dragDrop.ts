import type { EquipSlot } from '../types'

export interface DragState {
  source: HTMLElement
  sourceType: 'equipment' | 'inventory' | 'spell' | 'loot'
  sourceEquipSlot?: EquipSlot
  sourceIdx?: number
  itemType: string
  equipSlot?: EquipSlot
  itemId: string
  html: string
  itemName: string
  itemIcon: string
  rarity: string
}

export let dragState: DragState | null = null
export let dragFloat: HTMLElement | null = null
export let dragOffX = 0
export let dragOffY = 0
export let pendingAbandon: DragState | null = null

export function setPendingAbandon(v: DragState | null): void {
  pendingAbandon = v
}

export function clearDragUI(): void {
  document.querySelectorAll('.dragging, .drag-valid, .drag-invalid, .drag-over-trash')
    .forEach(el => el.classList.remove('dragging', 'drag-valid', 'drag-invalid', 'drag-over-trash'))
  const et = document.querySelector('.et')
  if (et) et.classList.remove('et-active')
  if (dragFloat) {
    dragFloat.remove()
    dragFloat = null
  }
}

export function cleanupDragState(): void {
  clearDragUI()
  dragState = null
  pendingAbandon = null
}

export function initDragFromItem(
  e: MouseEvent,
  sourceEl: HTMLElement,
  sourceType: 'equipment' | 'inventory' | 'spell' | 'loot',
  itemName: string,
  rarity: string,
  itemId: string,
  itemType: string,
  itemEquipSlot: EquipSlot | undefined,
  itemIcon: string,
  sourceIdx?: number,
  equipSlot?: EquipSlot,
): void {
  e.preventDefault()

  const html = `<span class="ei"><i class="${itemIcon}"></i></span><span class="en">${itemName}</span>`

  dragState = {
    source: sourceEl,
    sourceType,
    sourceEquipSlot: equipSlot,
    sourceIdx,
    itemType,
    equipSlot: itemEquipSlot,
    itemId,
    html,
    itemName,
    itemIcon,
    rarity,
  }

  sourceEl.classList.add('dragging')

  const fl = document.createElement('div')
  fl.className = 'drag-float'
  if (itemEquipSlot) fl.dataset.equipSlot = itemEquipSlot
  fl.innerHTML = html
  const sr = sourceEl.getBoundingClientRect()
  dragOffX = e.clientX - sr.left
  dragOffY = e.clientY - sr.top
  fl.style.left = (e.clientX - dragOffX) + 'px'
  fl.style.top = (e.clientY - dragOffY) + 'px'
  fl.style.width = sr.width + 'px'
  fl.style.height = sr.height + 'px'
  document.body.appendChild(fl)
  dragFloat = fl

  const dropZone = document.querySelector('.et')
  if (dropZone) dropZone.classList.add('et-active')
}
