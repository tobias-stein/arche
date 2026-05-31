import type { GenerateResponse, GeneratedThing } from './types'

let nextSeed = Date.now()

function randInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

function pickRandom<T>(arr: T[]): T {
  return arr[Math.floor(Math.random() * arr.length)]
}

const CREATURE_SUBTYPES = ['goblin', 'skeleton', 'slime', 'bat', 'rat', 'spider', 'wolf', 'ghost', 'orc', 'troll', 'demon', 'dragon']

const DIFFICULTY_NAMES: Record<string, string[]> = {
  normal: ['Rat', 'Spider', 'Slime', 'Skeleton', 'Bat'],
  champion: ['Dire Rat', 'Shadow Spider', 'Acid Slime', 'Bone Warrior', 'Frost Wolf'],
  elite: ['Plague Rat', 'Night Weaver', 'Elder Slime', 'Death Knight', 'Demon Wolf'],
  boss: ['Vermin King', 'Arachna', 'The Blob', 'Bone Lord', 'Dark Dragon'],
}

const LEVEL_BANDS = [
  { min: 1, max: 10, subtypes: ['rat', 'bat', 'slime', 'spider'] },
  { min: 5, max: 20, subtypes: ['goblin', 'spider', 'rat', 'bat'] },
  { min: 15, max: 35, subtypes: ['skeleton', 'wolf', 'goblin', 'spider'] },
  { min: 30, max: 50, subtypes: ['ghost', 'orc', 'skeleton', 'wolf'] },
  { min: 45, max: 70, subtypes: ['troll', 'demon', 'orc', 'ghost'] },
  { min: 60, max: 85, subtypes: ['demon', 'troll', 'ghost', 'orc'] },
  { min: 80, max: 100, subtypes: ['dragon', 'demon', 'troll', 'ghost'] },
]

function findBand(level: number): typeof LEVEL_BANDS[0] {
  for (const band of LEVEL_BANDS) {
    if (level >= band.min && level <= band.max) return band
  }
  return LEVEL_BANDS[LEVEL_BANDS.length - 1]
}

function pickBandForLevel(level: number): { min: number; max: number } {
  const band = findBand(level)
  return { min: band.min, max: band.max }
}

const CREATURE_HP: Record<string, number> = { normal: 30, champion: 50, elite: 80, boss: 150 }
const CREATURE_ATK: Record<string, number> = { normal: 6, champion: 10, elite: 14, boss: 20 }
const CREATURE_DEF: Record<string, number> = { normal: 4, champion: 7, elite: 10, boss: 15 }
const CREATURE_XP: Record<string, number> = { normal: 10, champion: 25, elite: 50, boss: 100 }

export function generateMockCreatureThing(
  difficulty: string,
  level: number,
): GeneratedThing {
  const band = pickBandForLevel(level)
  const subtype = pickRandom(CREATURE_SUBTYPES)
  const baseHp = CREATURE_HP[difficulty] ?? 30
  const baseAtk = CREATURE_ATK[difficulty] ?? 6
  const baseDef = CREATURE_DEF[difficulty] ?? 4
  const baseXp = CREATURE_XP[difficulty] ?? 10

  const name = `${pickRandom(DIFFICULTY_NAMES[difficulty] ?? DIFFICULTY_NAMES.normal)} Lv${level}`

  return {
    name,
    archetype: 'creature',
    name_parts: { base: name, prefix: null, suffix: null },
    blueprint_attributes: [
      { attribute_id: 'mock_level', name: 'level', value_type: 'number', value: level },
      { attribute_id: 'mock_difficulty', name: 'difficulty', value_type: 'string', value: difficulty },
      { attribute_id: 'mock_subtype', name: 'subtype', value_type: 'string', value: subtype },
      { attribute_id: 'mock_health', name: 'health', value_type: 'number', value: baseHp + level * 5 },
      { attribute_id: 'mock_attack', name: 'attack', value_type: 'number', value: baseAtk + level * 2 },
      { attribute_id: 'mock_defense', name: 'defense', value_type: 'number', value: baseDef + level },
      { attribute_id: 'mock_xp', name: 'xp_reward', value_type: 'number', value: baseXp + level * 5 },
    ],
    affix_attributes: [],
  }
}

const ITEM_ARCHETYPES = [
  { archetype: 'weapon', equipType: 'weapon', prefixes: ['Iron', 'Steel', 'Bronze', 'Silver', 'Runic'], subtypes: ['Sword', 'Axe', 'Dagger', 'Bow', 'Staff'] },
  { archetype: 'helmet', equipType: 'helmet', prefixes: ['Iron', 'Steel', 'Bronze', 'Silver', 'Runic'], subtypes: ['Helm', 'Crown', 'Hood', 'Cap', 'Sallet'] },
  { archetype: 'chest', equipType: 'chest', prefixes: ['Chain', 'Scale', 'Plate', 'Leather', 'Runic'], subtypes: ['Mail', 'Armor', 'Plate', 'Vest', 'Robe'] },
  { archetype: 'boots', equipType: 'boots', prefixes: ['Leather', 'Iron', 'Steel', 'Shadow', 'Runic'], subtypes: ['Boots', 'Shoes', 'Sabatons', 'Treads', 'Sandals'] },
  { archetype: 'ring', equipType: 'ring', prefixes: ['Copper', 'Silver', 'Ruby', 'Sapphire', 'Diamond'], subtypes: ['Ring', 'Band', 'Loop', 'Seal', 'Circle'] },
  { archetype: 'shield', equipType: 'shield', prefixes: ['Wood', 'Iron', 'Steel', 'Tower', 'Runic'], subtypes: ['Shield', 'Aegis', 'Guard', 'Barrier', 'Buckler'] },
]

const RARITY_WEIGHTS: string[] = ['common', 'common', 'common', 'uncommon', 'uncommon', 'rare', 'legendary']

export function generateMockItemThing(creatureLevel: number): GeneratedThing {
  const rarity = pickRandom(RARITY_WEIGHTS)
  const level = Math.max(1, creatureLevel + randInt(-2, 2))
  const archetypeDef = pickRandom(ITEM_ARCHETYPES)
  const prefix = pickRandom(archetypeDef.prefixes)
  const suffix = pickRandom(archetypeDef.subtypes)
  const name = `${prefix} ${suffix}`

  const statMultiplier: Record<string, number> = { common: 1, uncommon: 1.5, rare: 2.5, legendary: 4 }
  const mult = statMultiplier[rarity] ?? 1
  const baseStat = Math.round(2 + level * 1.5)

  const attrs: GeneratedThing['blueprint_attributes'] = [
    { attribute_id: 'mock_level', name: 'level', value_type: 'number', value: level },
    { attribute_id: 'mock_rarity', name: 'rarity', value_type: 'string', value: rarity },
    { attribute_id: 'mock_subtype', name: 'subtype', value_type: 'string', value: suffix.toLowerCase() },
  ]

  if (archetypeDef.archetype === 'weapon') {
    attrs.push({ attribute_id: 'mock_damage', name: 'damage', value_type: 'number', value: Math.round(baseStat * mult) })
  } else if (archetypeDef.archetype === 'shield') {
    attrs.push({ attribute_id: 'mock_defense', name: 'defense_bonus', value_type: 'number', value: Math.round(baseStat * 0.8 * mult) })
  } else {
    attrs.push(
      { attribute_id: 'mock_defense', name: 'defense_bonus', value_type: 'number', value: Math.round(baseStat * 0.5 * mult) },
      { attribute_id: 'mock_attack', name: 'damage', value_type: 'number', value: Math.round(baseStat * 0.2 * mult) },
    )
  }

  return {
    name,
    archetype: archetypeDef.archetype,
    name_parts: { base: suffix, prefix, suffix: null },
    blueprint_attributes: attrs,
    affix_attributes: [],
  }
}

export function generateMockResponse(
  archetype: 'creature' | 'item',
  level: number,
  difficulty?: string,
  count?: number,
): GenerateResponse {
  const things: GeneratedThing[] = []

  if (archetype === 'creature') {
    const diff = difficulty ?? 'normal'
    const creatureLevel = Math.max(1, level + randInt(-2, 2))
    for (let i = 0; i < (count ?? 1); i++) {
      things.push(generateMockCreatureThing(diff, creatureLevel))
    }
  } else {
    for (let i = 0; i < (count ?? 1); i++) {
      things.push(generateMockItemThing(level))
    }
  }

  return { things, seed: nextSeed++ }
}
