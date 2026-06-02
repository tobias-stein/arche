import { GAME_CONFIG } from '../config'
import type { ItemState, Rarity, EquipSlot, Difficulty } from '../types'
import { generate, buildLootRequest, parseItem } from '../api'

let nextItemId = 1

function randInt(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

function pickRandom<T>(arr: T[]): T {
  return arr[Math.floor(Math.random() * arr.length)]
}

interface ArchetypeDef {
  archeType: string
  equipSlot?: EquipSlot
  namePrefixes: string[]
  icon: string
}

const ARCHETYPES: ArchetypeDef[] = [
  { archeType: 'weapon', equipSlot: 'weapon', namePrefixes: ['Iron', 'Steel', 'Bronze', 'Silver', 'Runic'], icon: 'fa-solid fa-crosshairs' },
  { archeType: 'helmet', equipSlot: 'helmet', namePrefixes: ['Iron', 'Steel', 'Bronze', 'Silver', 'Runic'], icon: 'fa-solid fa-helmet-safety' },
  { archeType: 'chest', equipSlot: 'chest', namePrefixes: ['Chain', 'Scale', 'Plate', 'Leather', 'Runic'], icon: 'fa-solid fa-shirt' },
  { archeType: 'legs', equipSlot: 'legs', namePrefixes: ['Iron', 'Steel', 'Plate', 'Leather', 'Runic'], icon: 'fa-solid fa-chevron-down' },
  { archeType: 'boots', equipSlot: 'boots', namePrefixes: ['Leather', 'Iron', 'Steel', 'Shadow', 'Runic'], icon: 'fa-solid fa-boot' },
  { archeType: 'gloves', equipSlot: 'gloves', namePrefixes: ['Leather', 'Iron', 'Steel', 'Shadow', 'Runic'], icon: 'fa-solid fa-hand-back-fist' },
  { archeType: 'belt', equipSlot: 'belt', namePrefixes: ['Iron', 'Steel', 'Leather', 'Shadow', 'Runic'], icon: 'fa-solid fa-link' },
  { archeType: 'ring', equipSlot: 'ring', namePrefixes: ['Copper', 'Silver', 'Ruby', 'Sapphire', 'Diamond'], icon: 'fa-regular fa-gem' },
  { archeType: 'amulet', equipSlot: 'amulet', namePrefixes: ['Silver', 'Gold', 'Ruby', 'Sapphire', 'Diamond'], icon: 'fa-regular fa-star' },
  { archeType: 'shield', equipSlot: 'shield', namePrefixes: ['Wood', 'Iron', 'Steel', 'Tower', 'Runic'], icon: 'fa-solid fa-shield-halved' },
]

const CONSUMABLE_PREFIXES = ['Health', 'Mana', 'Stamina', 'Antidote', 'Elixir']
const CONSUMABLE_TYPES = ['Potion', 'Potion', 'Tonic', 'Antidote', 'Elixir']
const CONSUMABLE_ICON = 'fa-solid fa-flask'

const SPELL_NAMES = ['Fire Bolt', 'Ice Shard', 'Lightning', 'Heal', 'Poison Cloud', 'Arcane Blast']
const SPELL_ICON = 'fa-solid fa-wand-sparkles'

const RARITY_COLORS: Record<Rarity, string> = {
  common: '#888',
  uncommon: '#4488ff',
  rare: '#44cc66',
  legendary: '#aa44ff',
}

const SUBTYPE_NAMES: Record<string, string[]> = {
  weapon: ['Sword', 'Axe', 'Mace', 'Dagger', 'Spear'],
  helmet: ['Helm', 'Crown', 'Hood', 'Cap', 'Sallet'],
  chest: ['Mail', 'Armor', 'Plate', 'Vest', 'Robe'],
  legs: ['Leggings', 'Greaves', 'Pants', 'Chaps', 'Skirt'],
  boots: ['Boots', 'Shoes', 'Sabatons', 'Treads', 'Sandals'],
  gloves: ['Gauntlets', 'Gloves', 'Bracers', 'Handwraps', 'Mittens'],
  belt: ['Belt', 'Sash', 'Girdle', 'Chain', 'Cord'],
  ring: ['Ring', 'Band', 'Loop', 'Seal', 'Circle'],
  amulet: ['Amulet', 'Necklace', 'Pendant', 'Chain', 'Charm'],
  shield: ['Shield', 'Aegis', 'Guard', 'Barrier', 'Buckler'],
}

function generateItemId(): string {
  return `item_${nextItemId++}`
}

export function getRarityColor(rarity: Rarity): string {
  return RARITY_COLORS[rarity]
}

export function getItemIcon(item: { archeType: string }): string {
  const arch = ARCHETYPES.find(a => a.archeType === item.archeType)
  if (arch) return arch.icon
  if (item.archeType === 'consumable') return CONSUMABLE_ICON
  if (item.archeType === 'spell') return SPELL_ICON
  return 'fa-solid fa-circle-question'
}

export function generateMockItem(creatureLevel: number, levelVariance: number): ItemState {
  const rarityWeights: Rarity[] = ['common', 'common', 'common', 'uncommon', 'uncommon', 'rare', 'legendary']
  const rarity = pickRandom(rarityWeights)
  const level = Math.max(1, creatureLevel + randInt(-levelVariance, levelVariance))

  if (rarity === 'legendary' && Math.random() < 0.3) {
    // Sometimes generate a spell for legendary
    const spellName = pickRandom(SPELL_NAMES)
    const isHeal = spellName.toLowerCase().includes('heal')
    return {
      id: generateItemId(),
      name: spellName,
      archeType: 'spell',
      rarity,
      level,
      stats: isHeal
        ? { heal: Math.round(10 + level * 3.5), mana_cost: Math.round(5 + level * 0.35) }
        : { damage: Math.round(5 + level * 2.5), mana_cost: Math.round(5 + level * 0.35) },
      affixes: [],
    }
  }

  if (Math.random() < 0.2) {
    // Generate a consumable
    const idx = randInt(0, CONSUMABLE_PREFIXES.length - 1)
    const name = `${CONSUMABLE_PREFIXES[idx]} ${CONSUMABLE_TYPES[idx]}`
    const isMana = CONSUMABLE_PREFIXES[idx] === 'Mana'
    return {
      id: generateItemId(),
      name,
      archeType: 'consumable',
      subtype: CONSUMABLE_TYPES[idx].toLowerCase(),
      rarity,
      level,
      stats: isMana
        ? { mana: Math.round(10 + level * 3) }
        : { heal: Math.round(10 + level * 3) },
      affixes: [],
    }
  }

  const archetype = pickRandom(ARCHETYPES)
  const prefix = pickRandom(archetype.namePrefixes)
  const subtypeNames = SUBTYPE_NAMES[archetype.equipSlot ?? archetype.archeType] ?? [archetype.archeType]
  const suffix = pickRandom(subtypeNames)
  const name = `${prefix} ${suffix}`

  const statMultiplier: Record<Rarity, number> = { common: 1, uncommon: 1.5, rare: 2.5, legendary: 4 }
  const mult = statMultiplier[rarity]
  const baseStat = Math.round(2 + level * 1.5)

  const stats: Record<string, number> = {}
  if (archetype.archeType === 'weapon') {
    stats.attack = Math.round(baseStat * mult)
  } else if (archetype.archeType === 'shield') {
    stats.defense = Math.round(baseStat * 0.8 * mult)
  } else {
    stats.defense = Math.round(baseStat * 0.5 * mult)
    stats.attack = Math.round(baseStat * 0.2 * mult)
  }

  return {
    id: generateItemId(),
    name,
    archeType: archetype.archeType,
    rarity,
    level,
    equipSlot: archetype.equipSlot,
    stats,
    affixes: [],
  }
}

export async function generateMockItems(
  difficulty: Difficulty,
  creatureLevel: number,
  levelVariance: number,
): Promise<ItemState[]> {
  const cfg = GAME_CONFIG.lootDrops[difficulty]
  const count = randInt(cfg.min, cfg.max)

  try {
    const request = buildLootRequest(creatureLevel)
    const response = await generate(request)
    const parsed: ItemState[] = []
    const usedThings = response.things.slice(0, count)
    for (const thing of usedThings) {
      parsed.push(parseItem(thing))
    }
    if (parsed.length > 0) return parsed
  } catch {
    // Fallback to mock data
  }

  const fallback: ItemState[] = []
  for (let i = 0; i < count; i++) {
    fallback.push(generateMockItem(creatureLevel, levelVariance))
  }
  return fallback
}

export function generateChestLoot(playerLevel: number): ItemState[] {
  const count = randInt(1, 2);
  const items: ItemState[] = [];
  for (let i = 0; i < count; i++) {
    items.push(generateMockItem(playerLevel, GAME_CONFIG.generationWindow.itemLevelVariance));
  }
  return items;
}

export function resetItemIdCounter(): void {
  nextItemId = 1
}
