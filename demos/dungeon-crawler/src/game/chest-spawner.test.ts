import { describe, it, expect, beforeEach } from 'vitest'
import { generateChestsForRoom, resetChestIdCounter } from './chest-spawner'
import { TILE, type TileType } from './room-tiles'

beforeEach(() => {
  resetChestIdCounter()
})

function makeFloorTiles(rw: number, rh: number): TileType[][] {
  const tiles: TileType[][] = []
  for (let y = 0; y < rh; y++) {
    const row: TileType[] = []
    for (let x = 0; x < rw; x++) {
      if (x === 0 || x === rw - 1 || y === 0 || y === rh - 1) {
        row.push(TILE.WALL)
      } else {
        row.push(TILE.FLOOR)
      }
    }
    tiles.push(row)
  }
  return tiles
}

describe('generateChestsForRoom', () => {
  it('returns chests with unique ids', () => {
    const tiles = makeFloorTiles(15, 11)
    const chests = generateChestsForRoom(tiles, [{ x: 7, y: 0 }, { x: 7, y: 1 }])
    expect(chests.length).toBeGreaterThanOrEqual(0)
    expect(chests.length).toBeLessThanOrEqual(3)
    const ids = new Set(chests.map(c => c.id))
    expect(ids.size).toBe(chests.length)
  })

  it('places chests on floor tiles only', () => {
    const tiles = makeFloorTiles(15, 11)
    const chests = generateChestsForRoom(tiles, [{ x: 7, y: 0 }, { x: 7, y: 1 }])
    for (const c of chests) {
      expect(tiles[c.position.y][c.position.x]).toBe(TILE.FLOOR)
    }
  })

  it('excludes specified positions', () => {
    const tiles = makeFloorTiles(15, 11)
    const excluded = [{ x: 7, y: 0 }, { x: 7, y: 1 }, { x: 7, y: 2 }, { x: 7, y: 3 }]
    const chests = generateChestsForRoom(tiles, excluded)
    for (const c of chests) {
      expect(excluded.some(e => e.x === c.position.x && e.y === c.position.y)).toBe(false)
    }
  })

  it('excludes player position from chest spawns', () => {
    const tiles = makeFloorTiles(15, 11)
    const playerPos = { x: 7, y: 5 }
    const chests = generateChestsForRoom(tiles, [{ x: 7, y: 1 }, playerPos])
    for (const c of chests) {
      expect(c.position).not.toEqual(playerPos)
    }
  })

  it('excludes all door entry positions from chest spawns', () => {
    const tiles = makeFloorTiles(15, 11)
    const doorPositions = [
      { x: 7, y: 1 },
      { x: 7, y: 9 },
      { x: 1, y: 5 },
      { x: 13, y: 5 },
    ]
    const chests = generateChestsForRoom(tiles, doorPositions)
    for (const c of chests) {
      for (const doorPos of doorPositions) {
        const dist = Math.max(Math.abs(c.position.x - doorPos.x), Math.abs(c.position.y - doorPos.y))
        expect(dist).toBeGreaterThanOrEqual(4)
      }
    }
  })

  it('starts each chest as not opened', () => {
    const tiles = makeFloorTiles(15, 11)
    const chests = generateChestsForRoom(tiles, [{ x: 7, y: 0 }, { x: 7, y: 1 }])
    for (const c of chests) {
      expect(c.opened).toBe(false)
    }
  })
})

describe('generateChestLoot', () => {
  it('generates items without throwing', async () => {
    const { generateChestLoot } = await import('./loot')
    const items = generateChestLoot(5)
    expect(items.length).toBeGreaterThanOrEqual(1)
    expect(items.length).toBeLessThanOrEqual(2)
    for (const item of items) {
      expect(item.id).toBeTruthy()
      expect(item.name).toBeTruthy()
    }
  })
})
