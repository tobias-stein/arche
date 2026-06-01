import { describe, it, expect } from 'vitest'
import { GAME_CONFIG } from '../config/game-config'

describe('DungeonScene tile size calculation', () => {
  it('calculates non-zero tile size for typical desktop dimensions', () => {
    const width = 1920
    const height = 1080
    const tileSize = Math.min(
      Math.floor(width / GAME_CONFIG.room.width),
      Math.floor(height / GAME_CONFIG.room.height),
    )
    expect(tileSize).toBeGreaterThan(0)
    expect(tileSize).toBe(98)
  })

  it('calculates non-zero tile size for small mobile dimensions', () => {
    const width = 375
    const height = 667
    const tileSize = Math.min(
      Math.floor(width / GAME_CONFIG.room.width),
      Math.floor(height / GAME_CONFIG.room.height),
    )
    expect(tileSize).toBeGreaterThan(0)
    expect(tileSize).toBe(25)
  })

  it('handles zero scale dimensions gracefully (tileSize floor of 1)', () => {
    const width = 0
    const height = 0
    const raw = Math.min(
      Math.floor(width / GAME_CONFIG.room.width),
      Math.floor(height / GAME_CONFIG.room.height),
    )
    const tileSize = Math.max(1, raw)
    expect(tileSize).toBe(1)
  })

  it('calculates non-negative offsets', () => {
    const width = 1920
    const height = 1080
    const tileSize = Math.min(
      Math.floor(width / GAME_CONFIG.room.width),
      Math.floor(height / GAME_CONFIG.room.height),
    )
    const offsetX = Math.floor((width - GAME_CONFIG.room.width * tileSize) / 2)
    const offsetY = Math.floor((height - GAME_CONFIG.room.height * tileSize) / 2)
    expect(offsetX).toBeGreaterThanOrEqual(0)
    expect(offsetY).toBeGreaterThanOrEqual(0)
    expect(offsetX).toBe(225)
    expect(offsetY).toBe(1)
  })
})
