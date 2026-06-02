import type { AttributeValue, GeneratedThing } from './types'
import type { CreatureState, ItemState, Difficulty, Rarity, EquipSlot } from '../types'

let nextCreatureId = 1
let nextItemId = 1

function findAttr(attrs: AttributeValue[], name: string): AttributeValue | undefined {
  return attrs.find(a => a.name === name)
}

function getNumber(attrs: AttributeValue[], name: string, fallback: number): number {
  const attr = findAttr(attrs, name)
  return attr ? Number(attr.value) : fallback
}

function getString(attrs: AttributeValue[], name: string, fallback: string): string {
  const attr = findAttr(attrs, name)
  return attr ? String(attr.value) : fallback
}

const DIFFICULTY_MAP: Record<string, Difficulty> = {
  normal: 'normal',
  champion: 'champion',
  elite: 'elite',
  boss: 'boss',
}

const RARITY_MAP: Record<string, Rarity> = {
  common: 'common',
  uncommon: 'uncommon',
  rare: 'rare',
  legendary: 'legendary',
}

const SUBTYPE_TO_EQUIP_SLOT: Record<string, EquipSlot> = {
  sword: 'weapon',
  axe: 'weapon',
  dagger: 'weapon',
  bow: 'weapon',
  staff: 'weapon',
  mace: 'weapon',
  spear: 'weapon',
  crossbow: 'weapon',
  wand: 'weapon',
  halberd: 'weapon',
  helmet: 'helmet',
  chest: 'chest',
  legs: 'legs',
  boots: 'boots',
  gloves: 'gloves',
  belt: 'belt',
  ring: 'ring',
  amulet: 'amulet',
  shield: 'shield',
}

const AGGRO_RANGES: Record<string, number> = {
  normal: 1,
  champion: 2,
  elite: 3,
  boss: 3,
}

export function parseCreature(
  thing: GeneratedThing,
  x: number,
  y: number,
): CreatureState {
  const { blueprint_attributes, affix_attributes } = thing
  const difficulty = DIFFICULTY_MAP[getString(blueprint_attributes, 'difficulty', 'normal')] ?? 'normal'
  const level = getNumber(blueprint_attributes, 'level', 1)
  const subtype = getString(blueprint_attributes, 'subtype', 'monster')
  const baseHp = getNumber(blueprint_attributes, 'health', 30)
  const baseAtk = getNumber(blueprint_attributes, 'attack', 6)
  const baseDef = getNumber(blueprint_attributes, 'defense', 4)
  const baseXp = getNumber(blueprint_attributes, 'xp_reward', 10)

  let bonusHp = 0, bonusAtk = 0, bonusDef = 0
  for (const affixAttrs of affix_attributes) {
    for (const attr of affixAttrs) {
      const val = Number(attr.value) || 0
      if (attr.name === 'bonus_health') bonusHp += val
      else if (attr.name === 'rage_damage' || attr.name === 'bonus_damage') bonusAtk += val
      else if (attr.name === 'bonus_defense') bonusDef += val
    }
  }

  const hpMax = Math.max(1, baseHp + bonusHp)

  return {
    id: `creature_${nextCreatureId++}`,
    name: thing.name,
    archeType: 'creature',
    subtype,
    difficulty,
    level,
    hp: { current: hpMax, max: hpMax },
    attack: Math.max(1, baseAtk + bonusAtk),
    defense: Math.max(0, baseDef + bonusDef),
    xpReward: Math.max(1, baseXp),
    position: { x, y },
    aggroRange: AGGRO_RANGES[difficulty] ?? 1,
    aggro: false,
    stunned: false,
    stunTimer: 0,
  }
}

export function parseItem(thing: GeneratedThing): ItemState {
  const { blueprint_attributes, affix_attributes } = thing
  const rarity = RARITY_MAP[getString(blueprint_attributes, 'rarity', 'common')] ?? 'common'
  const level = getNumber(blueprint_attributes, 'level', 1)
  const subtype = getString(blueprint_attributes, 'subtype', '')
  const equipSlot = SUBTYPE_TO_EQUIP_SLOT[subtype] ?? undefined
  const archetype = thing.archetype

  const stats: Record<string, number> = {}
  const damage = getNumber(blueprint_attributes, 'damage', 0)
  const defenseBonus = getNumber(blueprint_attributes, 'defense_bonus', 0)
  const statBonus = getNumber(blueprint_attributes, 'stat_bonus', 0)
  const blockChance = getNumber(blueprint_attributes, 'block_chance', 0)
  const effectValue = getNumber(blueprint_attributes, 'effect_value', 0)
  const manaCost = getNumber(blueprint_attributes, 'mana_cost', 0)
  const healAmount = getNumber(blueprint_attributes, 'heal_amount', 0)

  if (damage) {
    stats.attack = damage
    stats.damage = damage
  }
  if (defenseBonus) stats.defense = defenseBonus
  if (statBonus) stats.stat_bonus = statBonus
  if (blockChance) stats.block_chance = blockChance
  if (effectValue) stats.heal = effectValue
  if (manaCost) stats.mana_cost = manaCost
  if (healAmount) stats.heal = healAmount

  const affixNames: string[] = []
  for (const affixAttrs of affix_attributes) {
    for (const attr of affixAttrs) {
      affixNames.push(attr.name)
      const val = Number(attr.value) || 0
      if (attr.name === 'bonus_damage' || attr.name === 'fire_damage' || attr.name === 'ice_damage') {
        stats.attack = (stats.attack || 0) + val
      } else if (attr.name === 'bonus_defense') {
        stats.defense = (stats.defense || 0) + val
      }
    }
  }

  return {
    id: `item_${nextItemId++}`,
    name: thing.name,
    archeType: archetype,
    ...(subtype ? { subtype } : {}),
    rarity,
    level,
    ...(equipSlot ? { equipSlot } : {}),
    stats: Object.keys(stats).length > 0 ? stats : { value: level },
    affixes: affixNames,
  }
}

export function resetCreatureIdCounter(): void {
  nextCreatureId = 1
}

export function resetItemIdCounter(): void {
  nextItemId = 1
}
