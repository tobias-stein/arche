import {
  API_BASE, SEED_CONFIG, LEVEL_BANDS, BAND_SUBTYPES, DIFFICULTIES, NON_BOSS_DIFFICULTIES, RARITIES,
  AFFIX_COUNT_CONFIG, AFFIX_TIERS, CREATURE_PREFIX_DEFS, CREATURE_SUFFIX_DEFS,
  ITEM_PREFIX_DEFS, ITEM_SUFFIX_DEFS, SUBTYPE_NAMES, ALL_CREATURE_SUBTYPES, capitalize,
} from './shared'

interface NamedId {
  id: string
  name: string
}

async function api<T = unknown>(method: string, path: string, body?: unknown, apiKey?: string): Promise<T> {
  const url = `${API_BASE}${path}`
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  if (apiKey) headers['X-API-Key'] = apiKey
  const res = await fetch(url, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  })
  if (!res.ok) {
    const text = await res.text().catch(() => '')
    throw new Error(`HTTP ${res.status} ${res.statusText}: ${method} ${path} — ${text.slice(0, 200)}`)
  }
  return res.json() as Promise<T>
}

async function waitForService(maxRetries = 30): Promise<void> {
  for (let i = 0; i < maxRetries; i++) {
    try {
      const res = await fetch(`${API_BASE}/health`)
      if (res.ok) return
    } catch { /* not ready yet */ }
    await new Promise(r => setTimeout(r, 1000))
  }
  throw new Error('Arche service did not become healthy')
}

async function getSuperAdminKey(): Promise<string> {
  if (process.env.ARCHE_API_KEY) return process.env.ARCHE_API_KEY
  const resp = await api<{ bootstrapped: boolean; key?: string; message?: string }>('GET', '/api/bootstrap')
  if (resp.bootstrapped && resp.key) return resp.key
  throw new Error('Arche is already bootstrapped. Set ARCHE_API_KEY env var to the super admin key from server logs.')
}

async function createClient(name: string, apiKey: string): Promise<NamedId> {
  const resp = await api<{ id: string; name: string }>('POST', '/api/clients', { name }, apiKey)
  return { id: resp.id, name: resp.name }
}

async function createApiKey(clientId: string, name: string, permissions: string[], apiKey: string): Promise<string> {
  const resp = await api<{ key: string }>('POST', `/api/clients/${clientId}/keys`, { name, permissions }, apiKey)
  return resp.key
}

async function createGMA(name: string, description: string | null, valueType: string, payload: Record<string, unknown>, clientId: string, apiKey: string): Promise<NamedId> {
  const body = { name, description, value_type: valueType, ...payload } as Record<string, unknown>
  const resp = await api<{ id: string }>('POST', `/api/global-meta-attributes?client_id=${clientId}`, body, apiKey)
  return { id: resp.id, name }
}

async function createBlueprint(body: Record<string, unknown>, clientId: string, apiKey: string): Promise<NamedId> {
  const resp = await api<{ id: string }>('POST', `/api/blueprints?client_id=${clientId}`, body, apiKey)
  const name = body.name as string
  return { id: resp.id, name }
}

async function createAffix(body: Record<string, unknown>, clientId: string, apiKey: string): Promise<NamedId> {
  const resp = await api<{ id: string }>('POST', `/api/affixes?client_id=${clientId}`, body, apiKey)
  const name = body.name as string
  return { id: resp.id, name }
}

async function assignAffixes(blueprintIds: string[], affixIds: string[], weight: number, clientId: string, apiKey: string): Promise<void> {
  await api('POST', `/api/blueprints/batch/assign?client_id=${clientId}`, {
    blueprint_ids: blueprintIds,
    affix_ids: affixIds,
    weight,
  }, apiKey)
}

function scaleStats(bandMin: number, bandMax: number, level: number): { health: number[]; attack: number[]; defense: number[]; xpReward: number[] } {
  const t = (level - bandMin) / Math.max(bandMax - bandMin, 1)
  return {
    health: [Math.round(15 + t * 85), Math.round(30 + t * 170)],
    attack: [Math.round(3 + t * 12), Math.round(6 + t * 25)],
    defense: [Math.round(1 + t * 5), Math.round(3 + t * 12)],
    xpReward: [Math.round(5 + t * 45), Math.round(10 + t * 90)],
  }
}

function buildCreatureBlueprint(name: string, subtype: string, difficulty: string, weight: number, band: typeof LEVEL_BANDS[0]): Record<string, unknown> {
  const bandCenter = Math.round((band.min + band.max) / 2)
  const stats = scaleStats(band.min, band.max, bandCenter)
  const affixCfg = AFFIX_COUNT_CONFIG[difficulty]
  return {
    name,
    archetype: 'creature',
    weight,
    description: `A ${difficulty} ${subtype} (level ${band.label})`,
    attributes: {
      level: { value_type: 'range', min: band.min, max: band.max, distribution: { type: 'uniform' } },
      difficulty: { value_type: 'enum', values: [difficulty] },
      subtype: { value_type: 'enum', values: [subtype] },
      health: { value_type: 'range', min: stats.health[0], max: stats.health[1], distribution: { type: 'uniform' } },
      attack: { value_type: 'range', min: stats.attack[0], max: stats.attack[1], distribution: { type: 'uniform' } },
      defense: { value_type: 'range', min: stats.defense[0], max: stats.defense[1], distribution: { type: 'uniform' } },
      xp_reward: { value_type: 'range', min: stats.xpReward[0], max: stats.xpReward[1], distribution: { type: 'uniform' } },
    },
    attribute_order: ['level', 'difficulty', 'subtype', 'health', 'attack', 'defense', 'xp_reward'],
    affixes: {
      min_prefixes: affixCfg.prefixes,
      max_prefixes: affixCfg.prefixes,
      min_suffixes: affixCfg.suffixes,
      max_suffixes: affixCfg.suffixes,
      prefixes: [],
      suffixes: [],
    },
  }
}

function buildItemBlueprint(name: string, archetype: string, rarity: string, subtype: string, weight: number, band: typeof LEVEL_BANDS[0]): Record<string, unknown> {
  const baseAttrs: Record<string, unknown> = {
    level: { value_type: 'range', min: band.min, max: band.max, distribution: { type: 'uniform' } },
    rarity: { value_type: 'enum', values: [rarity] },
  }
  const attrOrder = ['level', 'rarity']
  const affixCfg = AFFIX_COUNT_CONFIG[rarity]

  switch (archetype) {
    case 'weapon': {
      const bandCenter = Math.round((band.min + band.max) / 2)
      const dmg = Math.round(3 + (bandCenter / 100) * 47)
      baseAttrs.subtype = { value_type: 'enum', values: [subtype] }
      baseAttrs.damage = { value_type: 'range', min: Math.round(dmg * 0.6), max: Math.round(dmg * 1.4), distribution: { type: 'uniform' } }
      attrOrder.push('subtype', 'damage')
      break
    }
    case 'armor': {
      const bandCenter = Math.round((band.min + band.max) / 2)
      const def = Math.round(2 + (bandCenter / 100) * 28)
      baseAttrs.subtype = { value_type: 'enum', values: [subtype] }
      baseAttrs.defense_bonus = { value_type: 'range', min: Math.round(def * 0.6), max: Math.round(def * 1.4), distribution: { type: 'uniform' } }
      attrOrder.push('subtype', 'defense_bonus')
      break
    }
    case 'shield': {
      const bandCenter = Math.round((band.min + band.max) / 2)
      const def = Math.round(3 + (bandCenter / 100) * 37)
      baseAttrs.defense_bonus = { value_type: 'range', min: Math.round(def * 0.6), max: Math.round(def * 1.4), distribution: { type: 'uniform' } }
      baseAttrs.block_chance = { value_type: 'range', min: 5, max: 25, distribution: { type: 'uniform' } }
      attrOrder.push('defense_bonus', 'block_chance')
      break
    }
    case 'accessory': {
      const bandCenter = Math.round((band.min + band.max) / 2)
      const bonus = Math.round(2 + (bandCenter / 100) * 18)
      baseAttrs.subtype = { value_type: 'enum', values: [subtype] }
      baseAttrs.stat_bonus = { value_type: 'range', min: Math.round(bonus * 0.6), max: Math.round(bonus * 1.4), distribution: { type: 'uniform' } }
      attrOrder.push('subtype', 'stat_bonus')
      break
    }
  }

  return {
    name,
    archetype,
    weight,
    description: `A ${rarity} ${subtype} (level ${band.label})`,
    attributes: baseAttrs,
    attribute_order: attrOrder,
    affixes: {
      min_prefixes: affixCfg.prefixes,
      max_prefixes: affixCfg.prefixes,
      min_suffixes: affixCfg.suffixes,
      max_suffixes: affixCfg.suffixes,
      prefixes: [],
      suffixes: [],
    },
  }
}

function buildPotionBlueprint(name: string, potionType: string, band: typeof LEVEL_BANDS[0]): Record<string, unknown> {
  const bandCenter = Math.round((band.min + band.max) / 2)
  const effectVal = Math.round(10 + (bandCenter / 100) * 90)
  return {
    name,
    archetype: 'potion',
    weight: 1.0,
    description: `A ${potionType} potion (level ${band.label})`,
    attributes: {
      level: { value_type: 'range', min: band.min, max: band.max, distribution: { type: 'uniform' } },
      potion_type: { value_type: 'enum', values: [potionType] },
      effect_value: { value_type: 'range', min: Math.round(effectVal * 0.7), max: Math.round(effectVal * 1.3), distribution: { type: 'uniform' } },
    },
    attribute_order: ['level', 'potion_type', 'effect_value'],
    affixes: {
      min_prefixes: 0, max_prefixes: 0, min_suffixes: 0, max_suffixes: 0,
      prefixes: [], suffixes: [],
    },
  }
}

function buildSpellBlueprint(name: string, spellType: string): Record<string, unknown> {
  const isHeal = spellType === 'heal' || name.toLowerCase().includes('heal')
  return {
    name,
    archetype: 'spell',
    weight: 1.0,
    description: `A ${spellType} spell`,
    attributes: {
      level: { value_type: 'range', min: 1, max: 100, distribution: { type: 'uniform' } },
      spell_type: { value_type: 'enum', values: [spellType] },
      mana_cost: { value_type: 'range', min: 5, max: 40, distribution: { type: 'uniform' } },
      ...(isHeal
        ? { heal_amount: { value_type: 'range', min: 15, max: 200, distribution: { type: 'uniform' } } }
        : { damage: { value_type: 'range', min: 10, max: 150, distribution: { type: 'uniform' } } }),
    },
    attribute_order: isHeal ? ['level', 'spell_type', 'mana_cost', 'heal_amount'] : ['level', 'spell_type', 'mana_cost', 'damage'],
    affixes: {
      min_prefixes: 0, max_prefixes: 0, min_suffixes: 0, max_suffixes: 0,
      prefixes: [], suffixes: [],
    },
  }
}

function getTierName(baseName: string, tierIndex: number): string {
  return `${AFFIX_TIERS[tierIndex].name} ${baseName}`
}

function scaleRange(baseRange: number[], tierIndex: number): [number, number] {
  const multipliers = [1, 3, 6, 10]
  const mult = multipliers[tierIndex] || 1
  return [baseRange[0] * mult, baseRange[1] * mult]
}

function buildAffixBody(name: string, type: 'prefix' | 'suffix', attrName: string, minVal: number, maxVal: number): Record<string, unknown> {
  return {
    name,
    type,
    attribute: {
      name: attrName,
      value_type: 'range',
      min: minVal,
      max: maxVal,
      distribution: { type: 'uniform' },
    },
  }
}

async function main() {
  const apiKey = process.env.ARCHE_API_KEY || await getSuperAdminKey()
  console.log('Connecting to Arche at', API_BASE)
  await waitForService()
  console.log('Arche service healthy')

  console.log('Creating client...')
  const client = await createClient('dungeon_crawler', apiKey)
  const clientId = client.id
  console.log('  Client ID:', clientId)

  console.log('Creating API key...')
  const clientKey = await createApiKey(clientId, 'game-server-key', ['read', 'generate', 'write', 'delete', 'admin'], apiKey)
  console.log('  Client API Key:', clientKey)

  console.log('Creating Global Meta Attributes...')
  const gmas: Record<string, NamedId> = {}

  const gmaDefs: { name: string; description: string | null; valueType: string; payload: Record<string, unknown> }[] = [
    { name: 'rarity', description: 'Item quality tier', valueType: 'enum', payload: { values: ['common', 'uncommon', 'rare', 'legendary'] } },
    { name: 'difficulty', description: 'Creature difficulty tier', valueType: 'enum', payload: { values: ['normal', 'champion', 'elite', 'boss'] } },
    { name: 'creature_subtype', description: 'Creature variant type', valueType: 'enum', payload: { values: ALL_CREATURE_SUBTYPES } },
    { name: 'weapon_subtype', description: 'Weapon type', valueType: 'enum', payload: { values: SUBTYPE_NAMES.weapon } },
    { name: 'armor_subtype', description: 'Armor slot type', valueType: 'enum', payload: { values: SUBTYPE_NAMES.armor } },
    { name: 'shield_subtype', description: 'Shield type', valueType: 'enum', payload: { values: SUBTYPE_NAMES.shield } },
    { name: 'accessory_subtype', description: 'Accessory type', valueType: 'enum', payload: { values: SUBTYPE_NAMES.accessory } },
    { name: 'potion_type', description: 'Potion effect type', valueType: 'enum', payload: { values: ['health', 'mana'] } },
    { name: 'spell_type', description: 'Spell casting type', valueType: 'enum', payload: { values: ['projectile', 'beam', 'burst', 'heal', 'shield'] } },
    { name: 'element', description: 'Elemental affinity', valueType: 'enum', payload: { values: ['fire', 'ice', 'lightning', 'arcane', 'poison', 'holy'] } },
  ]

  for (const def of gmaDefs) {
    const gma = await createGMA(def.name, def.description, def.valueType, def.payload, clientId, apiKey)
    gmas[def.name] = gma
    console.log(`  GMA ${def.name} → ${gma.id}`)
  }

  console.log('Creating affixes...')
  const creaturePrefixIds: string[] = []
  const creatureSuffixIds: string[] = []
  const itemPrefixIds: string[] = []
  const itemSuffixIds: string[] = []

  for (const def of CREATURE_PREFIX_DEFS) {
    for (let t = 0; t < AFFIX_TIERS.length; t++) {
      const [minVal, maxVal] = scaleRange(def.tierRanges[t], t)
      const tierName = getTierName(def.baseName, t)
      const affix = await createAffix(buildAffixBody(tierName, 'prefix', def.attr, minVal, maxVal), clientId, apiKey)
      creaturePrefixIds.push(affix.id)
    }
  }

  for (const def of CREATURE_SUFFIX_DEFS) {
    for (let t = 0; t < AFFIX_TIERS.length; t++) {
      const [minVal, maxVal] = scaleRange(def.tierRanges[t], t)
      const tierName = getTierName(def.baseName, t)
      const affix = await createAffix(buildAffixBody(tierName, 'suffix', def.attr, minVal, maxVal), clientId, apiKey)
      creatureSuffixIds.push(affix.id)
    }
  }

  for (const def of ITEM_PREFIX_DEFS) {
    for (let t = 0; t < AFFIX_TIERS.length; t++) {
      const [minVal, maxVal] = scaleRange(def.tierRanges[t], t)
      const tierName = getTierName(def.baseName, t)
      const affix = await createAffix(buildAffixBody(tierName, 'prefix', def.attr, minVal, maxVal), clientId, apiKey)
      itemPrefixIds.push(affix.id)
    }
  }

  for (const def of ITEM_SUFFIX_DEFS) {
    for (let t = 0; t < AFFIX_TIERS.length; t++) {
      const [minVal, maxVal] = scaleRange(def.tierRanges[t], t)
      const tierName = getTierName(def.baseName, t)
      const affix = await createAffix(buildAffixBody(tierName, 'suffix', def.attr, minVal, maxVal), clientId, apiKey)
      itemSuffixIds.push(affix.id)
    }
  }

  const totalAffixes = creaturePrefixIds.length + creatureSuffixIds.length + itemPrefixIds.length + itemSuffixIds.length
  console.log(`  Created ${totalAffixes} affixes`)
  console.log(`    Creature prefixes: ${creaturePrefixIds.length}, suffixes: ${creatureSuffixIds.length}`)
  console.log(`    Item prefixes: ${itemPrefixIds.length}, suffixes: ${itemSuffixIds.length}`)

  const VALIDATION_LEVELS = [
    { level: 5, min: 3, max: 7, suffix: 'V3-7' },
    { level: 25, min: 23, max: 27, suffix: 'V23-27' },
    { level: 50, min: 48, max: 52, suffix: 'V48-52' },
    { level: 75, min: 73, max: 77, suffix: 'V73-77' },
  ]

  console.log('Creating creature blueprints...')
  const creatureBlueprintIds: NamedId[] = []
  const creatureNames = new Set<string>()

  for (let b = 0; b < LEVEL_BANDS.length; b++) {
    const band = LEVEL_BANDS[b]
    const subtypes = BAND_SUBTYPES[b] || []

    for (const subtype of subtypes) {
      for (const difficulty of NON_BOSS_DIFFICULTIES) {
        const baseName = `${capitalize(subtype)} ${capitalize(difficulty)} Lv${band.label}`
        creatureNames.add(baseName)
        const weight = SEED_CONFIG.creatureWeights[difficulty as keyof typeof SEED_CONFIG.creatureWeights].weight
        const bp = buildCreatureBlueprint(baseName, subtype, difficulty, weight, band)
        const result = await createBlueprint(bp, clientId, apiKey)
        creatureBlueprintIds.push(result)
      }
    }
  }

  for (let b = 0; b < LEVEL_BANDS.length; b++) {
    const band = LEVEL_BANDS[b]
    const subtypes = BAND_SUBTYPES[b] || []
    const firstSubtype = subtypes[0] || 'dragon'
    const bossName = `${capitalize(firstSubtype)} Boss Lv${band.label}`
    creatureNames.add(bossName)
    const bp = buildCreatureBlueprint(bossName, firstSubtype, 'boss', SEED_CONFIG.creatureWeights.boss.weight, band)
    const result = await createBlueprint(bp, clientId, apiKey)
    creatureBlueprintIds.push(result)
  }

  console.log(`  Created ${creatureBlueprintIds.length} creature blueprints`)

  // Add validation-level creature blueprints with narrow bands
  for (const vl of VALIDATION_LEVELS) {
    const narrowBand = { min: vl.min, max: vl.max, label: vl.suffix }
    const overlappingBands = LEVEL_BANDS
      .map((b, idx) => ({ ...b, idx }))
      .filter(b => b.min <= vl.max && b.max >= vl.min)
    const subtypes = [...new Set(overlappingBands.flatMap(b => BAND_SUBTYPES[b.idx] || []))].slice(0, 2)

    for (const subtype of subtypes) {
      for (const difficulty of NON_BOSS_DIFFICULTIES) {
        const baseName = `${capitalize(subtype)} ${capitalize(difficulty)} ${vl.suffix}`
        creatureNames.add(baseName)
        const weight = SEED_CONFIG.creatureWeights[difficulty as keyof typeof SEED_CONFIG.creatureWeights].weight
        const bp = buildCreatureBlueprint(baseName, subtype, difficulty, weight, narrowBand)
        const result = await createBlueprint(bp, clientId, apiKey)
        creatureBlueprintIds.push(result)
      }
    }

    const firstSubtype = subtypes[0] || 'dragon'
    const bossName = `${capitalize(firstSubtype)} Boss ${vl.suffix}`
    creatureNames.add(bossName)
    const bossBp = buildCreatureBlueprint(bossName, firstSubtype, 'boss', SEED_CONFIG.creatureWeights.boss.weight, narrowBand)
    const bossResult = await createBlueprint(bossBp, clientId, apiKey)
    creatureBlueprintIds.push(bossResult)
  }

  console.log(`  Created ${creatureBlueprintIds.length} creature blueprints (incl. validation bands)`)

  console.log('Creating item blueprints...')
  const itemBlueprintIds: NamedId[] = []

  for (const rarity of RARITIES) {
    const rarityWeight = SEED_CONFIG.rarityWeights[rarity].weight
    const itemBands = rarity === 'legendary' ? [LEVEL_BANDS[3], LEVEL_BANDS[5]] : [LEVEL_BANDS[1], LEVEL_BANDS[3], LEVEL_BANDS[5]]

    for (const subtype of SUBTYPE_NAMES.weapon) {
      for (const band of itemBands) {
        const name = `${capitalize(subtype)} ${rarity} Lv${band.label}`
        const bp = buildItemBlueprint(name, 'weapon', rarity, subtype, rarityWeight, band)
        const result = await createBlueprint(bp, clientId, apiKey)
        itemBlueprintIds.push(result)
      }
    }
  }

  for (const rarity of RARITIES) {
    const rarityWeight = SEED_CONFIG.rarityWeights[rarity].weight
    const itemBands = rarity === 'legendary' ? [LEVEL_BANDS[2], LEVEL_BANDS[4]] : [LEVEL_BANDS[1], LEVEL_BANDS[3], LEVEL_BANDS[5]]

    for (const subtype of SUBTYPE_NAMES.armor) {
      for (const band of itemBands) {
        const name = `${capitalize(subtype)} ${rarity} Lv${band.label}`
        const bp = buildItemBlueprint(name, 'armor', rarity, subtype, rarityWeight, band)
        const result = await createBlueprint(bp, clientId, apiKey)
        itemBlueprintIds.push(result)
      }
    }
  }

  for (const rarity of RARITIES) {
    const rarityWeight = SEED_CONFIG.rarityWeights[rarity].weight
    const itemBands = rarity === 'legendary' ? [LEVEL_BANDS[2]] : [LEVEL_BANDS[1], LEVEL_BANDS[3]]

    for (const subtype of SUBTYPE_NAMES.shield) {
      for (const band of itemBands) {
        const name = `Shield ${rarity} Lv${band.label}`
        const bp = buildItemBlueprint(name, 'shield', rarity, subtype, rarityWeight, band)
        const result = await createBlueprint(bp, clientId, apiKey)
        itemBlueprintIds.push(result)
      }
    }
  }

  for (const rarity of RARITIES) {
    const rarityWeight = SEED_CONFIG.rarityWeights[rarity].weight
    const itemBands = rarity === 'legendary' ? [LEVEL_BANDS[2]] : [LEVEL_BANDS[1], LEVEL_BANDS[3]]

    for (const subtype of SUBTYPE_NAMES.accessory) {
      for (const band of itemBands) {
        const name = `${capitalize(subtype)} ${rarity} Lv${band.label}`
        const bp = buildItemBlueprint(name, 'accessory', rarity, subtype, rarityWeight, band)
        const result = await createBlueprint(bp, clientId, apiKey)
        itemBlueprintIds.push(result)
      }
    }
  }

  const potionBands = [LEVEL_BANDS[0], LEVEL_BANDS[2], LEVEL_BANDS[4], LEVEL_BANDS[6]]
  for (const potionType of ['health', 'mana']) {
    for (const band of potionBands) {
      const name = `${capitalize(potionType)} Potion Lv${band.label}`
      const bp = buildPotionBlueprint(name, potionType, band)
      const result = await createBlueprint(bp, clientId, apiKey)
      itemBlueprintIds.push(result)
    }
  }

  const spellDefs: { name: string; type: string }[] = [
    { name: 'Fire Bolt', type: 'projectile' },
    { name: 'Ice Shard', type: 'projectile' },
    { name: 'Arcane Blast', type: 'burst' },
    { name: 'Heal', type: 'heal' },
    { name: 'Poison Cloud', type: 'burst' },
    { name: 'Lightning Strike', type: 'beam' },
    { name: 'Mana Shield', type: 'shield' },
    { name: 'Holy Light', type: 'heal' },
  ]
  for (const spell of spellDefs) {
    const bp = buildSpellBlueprint(spell.name, spell.type)
    const result = await createBlueprint(bp, clientId, apiKey)
    itemBlueprintIds.push(result)
  }

  // Add validation-level item blueprints with narrow bands
  for (const vl of VALIDATION_LEVELS) {
    const narrowBand = { min: vl.min, max: vl.max, label: vl.suffix }
    for (const rarity of RARITIES) {
      const rarityWeight = SEED_CONFIG.rarityWeights[rarity].weight
      for (const subtype of SUBTYPE_NAMES.weapon.slice(0, 4)) {
        const name = `${capitalize(subtype)} ${rarity} ${vl.suffix}`
        const bp = buildItemBlueprint(name, 'weapon', rarity, subtype, rarityWeight, narrowBand)
        const result = await createBlueprint(bp, clientId, apiKey)
        itemBlueprintIds.push(result)
      }
      for (const subtype of SUBTYPE_NAMES.armor.slice(0, 2)) {
        const name = `${capitalize(subtype)} ${rarity} ${vl.suffix}`
        const bp = buildItemBlueprint(name, 'armor', rarity, subtype, rarityWeight, narrowBand)
        const result2 = await createBlueprint(bp, clientId, apiKey)
        itemBlueprintIds.push(result2)
      }
    }
  }

  console.log(`  Created ${itemBlueprintIds.length} item blueprints (incl. validation bands)`)
  console.log(`  Total: ${creatureBlueprintIds.length + itemBlueprintIds.length} blueprints`)

  console.log('Assigning affixes to creature blueprints...')
  const creatureBpIds = creatureBlueprintIds.map(b => b.id)
  if (creaturePrefixIds.length > 0) {
    await assignAffixes(creatureBpIds, creaturePrefixIds, 1.0, clientId, apiKey)
  }
  if (creatureSuffixIds.length > 0) {
    await assignAffixes(creatureBpIds, creatureSuffixIds, 1.0, clientId, apiKey)
  }

  console.log('Assigning affixes to item blueprints...')
  const itemBpIds = itemBlueprintIds.map(b => b.id)
  if (itemPrefixIds.length > 0) {
    await assignAffixes(itemBpIds, itemPrefixIds, 1.0, clientId, apiKey)
  }
  if (itemSuffixIds.length > 0) {
    await assignAffixes(itemBpIds, itemSuffixIds, 1.0, clientId, apiKey)
  }

  console.log('\n=== Seed complete ===')
  console.log(`Client ID: ${clientId}`)
  console.log(`API Key: ${clientKey}`)
  console.log(`GMAs: ${Object.keys(gmas).length}`)
  console.log(`Affixes: ${totalAffixes}`)
  console.log(`Creature blueprints: ${creatureBlueprintIds.length}`)
  console.log(`Item blueprints: ${itemBlueprintIds.length}`)
}

main().catch(err => {
  console.error('Seed failed:', err)
  process.exit(1)
})
