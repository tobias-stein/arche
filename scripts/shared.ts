export const API_BASE = process.env.ARCHE_API_URL || 'http://localhost:8080'

export const SEED_CONFIG = {
  creatureWeights: {
    normal: { weight: 1.0 },
    champion: { weight: 0.6 },
    elite: { weight: 0.35 },
    boss: { weight: 0.15 },
  },
  rarityWeights: {
    common: { weight: 1.0 },
    uncommon: { weight: 0.6 },
    rare: { weight: 0.35 },
    legendary: { weight: 0.15 },
  },
  levelVariance: 2,
}

export const DIFFICULTIES = ['normal', 'champion', 'elite', 'boss'] as const
export const RARITIES = ['common', 'uncommon', 'rare', 'legendary'] as const

export const LEVEL_BANDS = [
  { min: 1, max: 10, label: '1-10' },
  { min: 5, max: 20, label: '5-20' },
  { min: 15, max: 35, label: '15-35' },
  { min: 30, max: 50, label: '30-50' },
  { min: 45, max: 70, label: '45-70' },
  { min: 60, max: 85, label: '60-85' },
  { min: 80, max: 100, label: '80-100' },
]

export const BAND_SUBTYPES: Record<number, string[]> = {
  0: ['rat', 'bat', 'slime', 'spider'],
  1: ['goblin', 'spider', 'rat', 'bat'],
  2: ['skeleton', 'wolf', 'goblin', 'spider'],
  3: ['ghost', 'orc', 'skeleton', 'wolf'],
  4: ['troll', 'demon', 'orc', 'ghost'],
  5: ['demon', 'troll', 'ghost', 'orc'],
  6: ['dragon', 'demon', 'troll', 'ghost'],
}

export const ALL_CREATURE_SUBTYPES = ['goblin', 'skeleton', 'slime', 'bat', 'rat', 'spider', 'wolf', 'ghost', 'orc', 'troll', 'demon', 'dragon']

export const TARGET_LEVELS = [5, 25, 50, 75]

export const DIFFICULTY_AFFIX_MAP: Record<string, { prefixes: number; suffixes: number }> = {
  normal: { prefixes: 0, suffixes: 0 },
  champion: { prefixes: 1, suffixes: 0 },
  elite: { prefixes: 1, suffixes: 1 },
  boss: { prefixes: 2, suffixes: 1 },
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

export const SUBTYPE_NAMES: Record<string, string[]> = {
  weapon: ['sword', 'axe', 'dagger', 'bow', 'staff', 'mace', 'spear', 'crossbow', 'wand', 'halberd'],
  armor: ['helmet', 'chest', 'legs', 'boots', 'gloves', 'belt'],
  shield: ['shield'],
  accessory: ['ring', 'amulet'],
}

export interface ApiResponse<T = unknown> {
  id?: string
  data?: T
  [key: string]: unknown
}
