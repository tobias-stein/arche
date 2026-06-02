export const API_BASE = process.env.ARCHE_API_URL || 'http://localhost:8080'

export const SEED_CONFIG = {
  creatureWeights: {
    normal: { weight: 1.0 },
    champion: { weight: 0.5 },
    elite: { weight: 0.3 },
    boss: { weight: 0.2 },
  },
  rarityWeights: {
    common: { weight: 1.0 },
    uncommon: { weight: 0.5 },
    rare: { weight: 0.3 },
    legendary: { weight: 0.2 },
  },
  levelVariance: 2,
}

export const DIFFICULTIES = ['normal', 'champion', 'elite', 'boss'] as const
export const RARITIES = ['common', 'uncommon', 'rare', 'legendary'] as const

export const SUBTYPE_POOLS = [
  { min: 1, max: 10, pool: ['rat', 'bat', 'slime', 'spider', 'goblin'] },
  { min: 11, max: 20, pool: ['goblin', 'spider', 'skeleton', 'wolf', 'rat'] },
  { min: 21, max: 35, pool: ['skeleton', 'wolf', 'goblin', 'ghost'] },
  { min: 36, max: 50, pool: ['ghost', 'orc', 'skeleton', 'wolf'] },
  { min: 51, max: 65, pool: ['troll', 'demon', 'orc', 'ghost'] },
  { min: 66, max: 80, pool: ['demon', 'troll', 'orc', 'ghost'] },
  { min: 81, max: 100, pool: ['dragon', 'demon', 'troll'] },
]

export const ALL_CREATURE_SUBTYPES = ['goblin', 'skeleton', 'slime', 'bat', 'rat', 'spider', 'wolf', 'ghost', 'orc', 'troll', 'demon', 'dragon']

export function getSubtypesForLevel(level: number): string[] {
  const found = SUBTYPE_POOLS.find(p => level >= p.min && level <= p.max)
  return found ? found.pool : ['rat']
}

export function pickFromPool<T>(pool: T[], index: number): T {
  return pool[((index * 7 + 13) % pool.length + pool.length) % pool.length]
}

export const TARGET_LEVELS = [5, 25, 50, 75]

export function capitalize(s: string): string {
  return s.charAt(0).toUpperCase() + s.slice(1)
}

// ── Stat reference curve ────────────────────────────────────────────

export function referenceAttack(level: number): number {
  return 5 + (level - 1) * 0.5
}

export function referenceDefense(level: number): number {
  return 2 + (level - 1) * 0.3
}

export function referenceHealth(level: number): number {
  return 20 + (level - 1) * 2.0
}

export const STAT_MULTIPLIERS = {
  creature: {
    normal: { attack: 1.0, defense: 1.0, health: 1.0, xpReward: 1.0 },
    champion: { attack: 1.2, defense: 1.2, health: 1.5, xpReward: 1.5 },
    elite: { attack: 1.5, defense: 1.5, health: 2.0, xpReward: 2.5 },
    boss: { attack: 2.0, defense: 2.0, health: 4.0, xpReward: 5.0 },
  },
  weapon: { damage: 0.8 },
  armor: { defense_bonus: 0.7 },
  shield: { defense_bonus: 0.5 },
  accessory: { stat_bonus: 0.3 },
  potion: { health: 2.0 },
  spell: {
    damage: { attack: 1.2 },
    heal: { health: 1.5 },
  },
}

export type Archetype = 'creature' | 'weapon' | 'armor' | 'shield' | 'accessory' | 'potion' | 'spell'

export interface ArchetypeTemplate {
  archetype: Archetype
  difficulty?: typeof DIFFICULTIES[number]
  spellType?: 'damage' | 'heal'
}

export function computeStats(level: number, template: ArchetypeTemplate): Record<string, number> {
  const refAtk = referenceAttack(level)
  const refDef = referenceDefense(level)
  const refHp = referenceHealth(level)

  switch (template.archetype) {
    case 'creature': {
      const diff = template.difficulty || 'normal'
      const mult = STAT_MULTIPLIERS.creature[diff]
      return {
        health: Math.round(refHp * mult.health),
        attack: Math.round(refAtk * mult.attack),
        defense: Math.round(refDef * mult.defense),
        xpReward: Math.round(refHp * 0.5 * mult.xpReward),
      }
    }
    case 'weapon': {
      return {
        damage: Math.round(refAtk * STAT_MULTIPLIERS.weapon.damage),
      }
    }
    case 'armor': {
      return {
        defense_bonus: Math.round(refDef * STAT_MULTIPLIERS.armor.defense_bonus),
      }
    }
    case 'shield': {
      return {
        defense_bonus: Math.round(refDef * STAT_MULTIPLIERS.shield.defense_bonus),
      }
    }
    case 'accessory': {
      return {
        stat_bonus: Math.round(refAtk * STAT_MULTIPLIERS.accessory.stat_bonus),
      }
    }
    case 'potion': {
      return {
        effect_value: Math.round(refHp * STAT_MULTIPLIERS.potion.health),
      }
    }
    case 'spell': {
      if (template.spellType === 'heal') {
        return {
          heal_amount: Math.round(refHp * STAT_MULTIPLIERS.spell.heal.health),
        }
      }
      return {
        damage: Math.round(refAtk * STAT_MULTIPLIERS.spell.damage.attack),
      }
    }
    default:
      return {}
  }
}

export const AFFIX_COUNT_CONFIG: Record<string, { prefixes: number; suffixes: number }> = {
  normal: { prefixes: 0, suffixes: 0 },
  champion: { prefixes: 1, suffixes: 0 },
  elite: { prefixes: 1, suffixes: 1 },
  boss: { prefixes: 2, suffixes: 1 },
  common: { prefixes: 0, suffixes: 0 },
  uncommon: { prefixes: 1, suffixes: 0 },
  rare: { prefixes: 1, suffixes: 1 },
  legendary: { prefixes: 2, suffixes: 1 },
}

export const AFFIX_TIERS = [
  { name: 'Weak', levelMin: 1, levelMax: 30 },
  { name: 'Strong', levelMin: 20, levelMax: 60 },
  { name: 'Greater', levelMin: 50, levelMax: 85 },
  { name: 'Mythic', levelMin: 75, levelMax: 100 },
]

export const CREATURE_PREFIX_DEFS = [
  { baseName: 'Angry', attr: 'rage_damage', tierRanges: [[1, 4], [3, 10], [8, 25], [20, 50]] },
  { baseName: 'Armored', attr: 'bonus_defense', tierRanges: [[1, 3], [3, 8], [6, 18], [15, 35]] },
  { baseName: 'Vampiric', attr: 'life_steal', tierRanges: [[1, 2], [2, 5], [4, 12], [10, 25]] },
]

export const CREATURE_SUFFIX_DEFS = [
  { baseName: 'of Rage', attr: 'bonus_damage', tierRanges: [[2, 5], [5, 15], [12, 35], [30, 80]] },
  { baseName: 'of the Void', attr: 'bonus_defense', tierRanges: [[2, 4], [4, 10], [10, 25], [25, 50]] },
  { baseName: 'of Fortitude', attr: 'bonus_health', tierRanges: [[5, 15], [15, 40], [40, 100], [100, 250]] },
]

export const ITEM_PREFIX_DEFS = [
  { baseName: 'Smoldering', attr: 'fire_damage', tierRanges: [[1, 4], [3, 12], [10, 30], [30, 80]] },
  { baseName: 'Chilled', attr: 'ice_damage', tierRanges: [[1, 4], [3, 12], [10, 30], [30, 80]] },
  { baseName: 'Keen', attr: 'bonus_damage', tierRanges: [[1, 3], [3, 8], [8, 20], [20, 60]] },
]

export const ITEM_SUFFIX_DEFS = [
  { baseName: 'of Power', attr: 'bonus_damage', tierRanges: [[2, 5], [5, 12], [12, 30], [30, 70]] },
  { baseName: 'of Protection', attr: 'bonus_defense', tierRanges: [[1, 3], [3, 8], [8, 20], [20, 45]] },
  { baseName: 'of the Leech', attr: 'life_steal', tierRanges: [[1, 2], [2, 5], [5, 12], [12, 25]] },
]

export const SPELL_PREFIX_DEFS = [
  { baseName: 'Smoldering', element: 'fire' },
  { baseName: 'Frost', element: 'ice' },
  { baseName: 'Crackling', element: 'lightning' },
  { baseName: 'Arcane', element: 'arcane' },
  { baseName: 'Toxic', element: 'poison' },
  { baseName: 'Sacred', element: 'holy' },
]

export const SPELL_SUFFIX_DEFS = [
  { baseName: 'of Potency', attr: 'bonus_damage', tierRanges: [[2, 5], [5, 15], [15, 40], [40, 100]] },
  { baseName: 'of Destruction', attr: 'bonus_damage', tierRanges: [[3, 8], [8, 25], [25, 60], [60, 150]] },
  { baseName: 'of Siphoning', attr: 'life_steal', tierRanges: [[1, 2], [2, 5], [5, 12], [12, 30]] },
  { baseName: 'of Vitality', attr: 'bonus_heal', tierRanges: [[3, 8], [8, 20], [20, 50], [50, 120]] },
  { baseName: 'of Brilliance', attr: 'mana_return', tierRanges: [[1, 3], [3, 8], [8, 20], [20, 50]] },
  { baseName: 'of the Phoenix', attr: 'revival', tierRanges: [[1, 1], [1, 2], [2, 3], [3, 5]] },
]

export const SUBTYPE_NAMES: Record<string, string[]> = {
  weapon: ['sword', 'axe', 'dagger', 'bow', 'staff', 'mace', 'spear', 'crossbow', 'wand', 'halberd'],
  armor: ['helmet', 'chest', 'legs', 'boots', 'gloves', 'belt'],
  shield: ['shield'],
  accessory: ['ring', 'amulet'],
}


