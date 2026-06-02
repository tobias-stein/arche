import { describe, it, expect } from 'vitest'
import { buildCreatureRequest, buildBossRequest, buildLootRequest } from '../api/request-builders'

describe('request-builders', () => {
  describe('buildCreatureRequest', () => {
    it('returns creature request with flat band and difficulty constraints', () => {
      const req = buildCreatureRequest(10)
      expect(req.archetype).toBe('creature')
      expect(req.constraints).toBeDefined()
      expect(req.constraints!.level).toBeDefined()
      expect(req.constraints!.level!.gte).toBe(8)
      expect(req.constraints!.level!.lte).toBe(12)
      expect(req.constraints!.difficulty).toEqual({ in: ['normal', 'champion', 'elite'] })
      expect(req.affixes).toBeUndefined()
    })

    it('clamps level to 1-100 range at low end', () => {
      const req = buildCreatureRequest(1)
      expect(req.constraints!.level!.gte).toBe(1)
      expect(req.constraints!.level!.lte).toBe(3)
    })

    it('clamps level to 1-100 range at high end', () => {
      const req = buildCreatureRequest(100)
      expect(req.constraints!.level!.gte).toBe(98)
      expect(req.constraints!.level!.lte).toBe(100)
    })

    it('works for mid-range levels', () => {
      const req = buildCreatureRequest(50)
      expect(req.constraints!.level!.gte).toBe(48)
      expect(req.constraints!.level!.lte).toBe(52)
    })
  })

  describe('buildBossRequest', () => {
    it('returns boss request with flat band and boss difficulty', () => {
      const req = buildBossRequest(15)
      expect(req.archetype).toBe('creature')
      expect(req.constraints!.level!.gte).toBe(13)
      expect(req.constraints!.level!.lte).toBe(17)
      expect(req.constraints!.difficulty).toEqual({ in: ['boss'] })
      expect(req.affixes).toBeUndefined()
    })

    it('clamps boss level to 1-100', () => {
      const req = buildBossRequest(1)
      expect(req.constraints!.level!.gte).toBe(1)
      expect(req.constraints!.level!.lte).toBe(3)
    })
  })

  describe('buildLootRequest', () => {
    it('returns loot request with flat band and no archetype', () => {
      const req = buildLootRequest(20)
      expect(req.archetype).toBeUndefined()
      expect(req.constraints!.level!.gte).toBe(18)
      expect(req.constraints!.level!.lte).toBe(22)
      expect(req.affixes).toBeUndefined()
    })

    it('clamps loot level to 1-100', () => {
      const req = buildLootRequest(100)
      expect(req.constraints!.level!.gte).toBe(98)
      expect(req.constraints!.level!.lte).toBe(100)
    })

    it('handles creature level for loot generation', () => {
      const req = buildLootRequest(35)
      expect(req.constraints!.level!.gte).toBe(33)
      expect(req.constraints!.level!.lte).toBe(37)
    })
  })
})
