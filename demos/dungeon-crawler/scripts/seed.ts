import {
  API_BASE, SEED_CONFIG, DIFFICULTIES,
  AFFIX_COUNT_CONFIG, AFFIX_TIERS, CREATURE_PREFIX_DEFS, CREATURE_SUFFIX_DEFS,
  ITEM_PREFIX_DEFS, ITEM_SUFFIX_DEFS, SPELL_PREFIX_DEFS, SPELL_SUFFIX_DEFS,
  SUBTYPE_NAMES, ALL_CREATURE_SUBTYPES,
  getSubtypesForLevel, pickFromPool, capitalize,
  computeStats,
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

// ── Builders ────────────────────────────────────────────────────────

function buildCreatureBlueprint(name: string, subtype: string, difficulty: string, weight: number, level: number): Record<string, unknown> {
  const stats = computeStats(level, { archetype: 'creature', difficulty: difficulty as typeof DIFFICULTIES[number] })
  const affixCfg = AFFIX_COUNT_CONFIG[difficulty]
  return {
    name,
    archetype: 'creature',
    weight,
    description: `A ${difficulty} ${subtype} (level ${level})`,
    attributes: {
      level: { value_type: 'single', value: level },
      difficulty: { value_type: 'enum', values: [difficulty] },
      subtype: { value_type: 'enum', values: [subtype] },
      health: { value_type: 'single', value: Math.round(stats.health) },
      attack: { value_type: 'single', value: Math.round(stats.attack) },
      defense: { value_type: 'single', value: Math.round(stats.defense) },
      xp_reward: { value_type: 'single', value: Math.round(stats.xpReward) },
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

function buildItemBlueprint(name: string, archetype: string, rarity: string, subtype: string, weight: number, level: number): Record<string, unknown> {
  const baseAttrs: Record<string, unknown> = {
    level: { value_type: 'single', value: level },
    rarity: { value_type: 'enum', values: [rarity] },
  }
  const attrOrder = ['level', 'rarity']
  const affixCfg = AFFIX_COUNT_CONFIG[rarity]

  switch (archetype) {
    case 'weapon': {
      const stats = computeStats(level, { archetype: 'weapon' })
      baseAttrs.subtype = { value_type: 'enum', values: [subtype] }
      baseAttrs.damage = { value_type: 'single', value: Math.round(stats.damage) }
      attrOrder.push('subtype', 'damage')
      break
    }
    case 'armor': {
      const stats = computeStats(level, { archetype: 'armor' })
      baseAttrs.subtype = { value_type: 'enum', values: [subtype] }
      baseAttrs.defense_bonus = { value_type: 'single', value: Math.round(stats.defense_bonus) }
      attrOrder.push('subtype', 'defense_bonus')
      break
    }
    case 'shield': {
      const stats = computeStats(level, { archetype: 'shield' })
      baseAttrs.defense_bonus = { value_type: 'single', value: Math.round(stats.defense_bonus) }
      baseAttrs.block_chance = { value_type: 'single', value: Math.round(5 + level * 0.2) }
      attrOrder.push('defense_bonus', 'block_chance')
      break
    }
    case 'accessory': {
      const stats = computeStats(level, { archetype: 'accessory' })
      baseAttrs.subtype = { value_type: 'enum', values: [subtype] }
      baseAttrs.stat_bonus = { value_type: 'single', value: Math.round(stats.stat_bonus) }
      attrOrder.push('subtype', 'stat_bonus')
      break
    }
  }

  return {
    name,
    archetype,
    weight,
    description: `A ${rarity} ${subtype} (level ${level})`,
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

function buildPotionBlueprint(name: string, potionType: string, level: number): Record<string, unknown> {
  const stats = computeStats(level, { archetype: 'potion' })
  return {
    name,
    archetype: 'potion',
    weight: 1.0,
    description: `A ${potionType} potion (level ${level})`,
    attributes: {
      level: { value_type: 'single', value: level },
      potion_type: { value_type: 'enum', values: [potionType] },
      effect_value: { value_type: 'single', value: Math.round(stats.effect_value) },
    },
    attribute_order: ['level', 'potion_type', 'effect_value'],
    affixes: {
      min_prefixes: 0, max_prefixes: 0, min_suffixes: 0, max_suffixes: 0,
      prefixes: [], suffixes: [],
    },
  }
}

function buildSpellBlueprint(name: string, spellType: string, rarity: string, level: number): Record<string, unknown> {
  const isHeal = spellType === 'heal' || name.toLowerCase().includes('heal')
  const stats = computeStats(level, { archetype: 'spell', spellType: isHeal ? 'heal' : 'damage' })
  const affixCfg = AFFIX_COUNT_CONFIG[rarity]
  return {
    name,
    archetype: 'spell',
    weight: 1.0,
    description: `A ${rarity} ${spellType} spell (level ${level})`,
    attributes: {
      level: { value_type: 'single', value: level },
      rarity: { value_type: 'enum', values: [rarity] },
      spell_type: { value_type: 'enum', values: [spellType] },
      mana_cost: { value_type: 'single', value: Math.round(5 + level * 0.35) },
      ...(isHeal
        ? { heal_amount: { value_type: 'single', value: Math.round(stats.heal_amount) } }
        : { damage: { value_type: 'single', value: Math.round(stats.damage) } }),
    },
    attribute_order: isHeal
      ? ['level', 'rarity', 'spell_type', 'mana_cost', 'heal_amount']
      : ['level', 'rarity', 'spell_type', 'mana_cost', 'damage'],
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

// ── Affix helpers ───────────────────────────────────────────────────

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

function buildSpellPrefixAffixBody(name: string, element: string): Record<string, unknown> {
  return {
    name,
    type: 'prefix',
    attribute: {
      name: 'element',
      value_type: 'enum',
      values: [element],
    },
  }
}

async function createTieredAffixes(
  defs: { baseName: string; attr: string; tierRanges: number[][] }[],
  type: 'prefix' | 'suffix',
  ids: string[],
  clientId: string,
  apiKey: string,
): Promise<void> {
  for (const def of defs) {
    for (let t = 0; t < AFFIX_TIERS.length; t++) {
      const [minVal, maxVal] = scaleRange(def.tierRanges[t], t)
      const tierName = getTierName(def.baseName, t)
      const affix = await createAffix(buildAffixBody(tierName, type, def.attr, minVal, maxVal), clientId, apiKey)
      ids.push(affix.id)
    }
  }
}

async function assignBlueprintAffixes(
  bpIds: string[],
  prefixIds: string[],
  suffixIds: string[],
  label: string,
  clientId: string,
  apiKey: string,
): Promise<void> {
  console.log(`Assigning affixes to ${label}...`)
  if (prefixIds.length > 0) {
    await assignAffixes(bpIds, prefixIds, 1.0, clientId, apiKey)
  }
  if (suffixIds.length > 0) {
    await assignAffixes(bpIds, suffixIds, 1.0, clientId, apiKey)
  }
}

// ── Generation helpers ──────────────────────────────────────────────

function generateCreaturesForLevel(level: number): { subtype: string; difficulty: string }[] {
  const pool = getSubtypesForLevel(level)
  const result: { subtype: string; difficulty: string }[] = []

  // 1st: always normal
  result.push({ subtype: pickFromPool(pool, level), difficulty: 'normal' })

  // 2nd: normal on 25% of levels (every 4th starting at 1), otherwise champion/elite
  if (level % 4 === 1) {
    result.push({ subtype: pickFromPool(pool, level + 7), difficulty: 'normal' })
  } else {
    result.push({
      subtype: pickFromPool(pool, level + 3),
      difficulty: level % 6 === 0 ? 'elite' : 'champion',
    })
  }

  // 3rd: even levels get an extra creature (50 levels)
  if (level % 2 === 0) {
    if (level % 4 === 0) {
      // 25 boss levels
      result.push({ subtype: pickFromPool(pool, level + 11), difficulty: 'boss' })
    } else {
      // 25 elite 3rd slots
      result.push({ subtype: pickFromPool(pool, level + 5), difficulty: 'elite' })
    }
  }

  return result
}

function generateItemsForLevel(level: number): { archetype: string; subtype: string; rarity: string }[] {
  // 75 levels get 2 items, 25 levels get 1 (total 175)
  const count = level % 4 === 0 ? 1 : 2

  const rarities = ['common', 'common', 'uncommon', 'common', 'rare', 'common', 'uncommon', 'common', 'legendary', 'uncommon']
  const archetypes = ['weapon', 'armor', 'weapon', 'accessory', 'weapon', 'shield', 'weapon', 'armor', 'weapon', 'accessory']

  const items: { archetype: string; subtype: string; rarity: string }[] = []
  for (let i = 0; i < count; i++) {
    const idx = ((level - 1) * 2 + i * 7) % 10
    const archetype = archetypes[idx]
    const rarity = rarities[idx]
    let subtype: string
    switch (archetype) {
      case 'weapon':
        subtype = pickFromPool(SUBTYPE_NAMES.weapon, level + i * 5)
        break
      case 'armor':
        subtype = pickFromPool(SUBTYPE_NAMES.armor, level + i * 7)
        break
      case 'shield':
        subtype = 'shield'
        break
      case 'accessory':
        subtype = pickFromPool(SUBTYPE_NAMES.accessory, level + i * 11)
        break
      default:
        subtype = 'ring'
    }
    items.push({ archetype, subtype, rarity })
  }
  return items
}

function generateSpellsForLevel(level: number): { name: string; spellType: string; rarity: string } | null {
  // 75 levels out of 100 get spells (all levels where %4 !== 0)
  if (level % 4 === 0) return null

  // Continuous index among spell-having levels: count of non-multiples-of-4 up to L
  const spellIdx = level - Math.floor(level / 4) - 1

  const rarityCycle = ['uncommon', 'uncommon', 'uncommon', 'rare', 'uncommon', 'uncommon', 'uncommon', 'legendary']
  const typeCycle = ['projectile', 'burst', 'beam', 'heal', 'shield', 'projectile', 'burst', 'projectile']

  const idx = spellIdx % 8
  const rarity = rarityCycle[idx]
  const spellType = typeCycle[idx]

  const elementNames = ['Fire', 'Ice', 'Arcane', 'Poison', 'Lightning', 'Holy']
  const elementIdx = (level - 1) % elementNames.length
  const prefix = `${elementNames[elementIdx]}`
  const typeLabel = spellType === 'heal' ? 'Healing' : spellType === 'shield' ? 'Ward' : capitalize(spellType)

  return {
    name: `${prefix} ${typeLabel}`,
    spellType,
    rarity,
  }
}

function generatePotionLevels(): number[] {
  // 12 potions (6 health, 6 mana) spread across levels
  return [1, 10, 20, 30, 40, 50, 60, 70, 80, 90, 95, 100]
}

// ── Main ────────────────────────────────────────────────────────────

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
  const spellPrefixIds: string[] = []
  const spellSuffixIds: string[] = []

  await createTieredAffixes(CREATURE_PREFIX_DEFS, 'prefix', creaturePrefixIds, clientId, apiKey)
  await createTieredAffixes(CREATURE_SUFFIX_DEFS, 'suffix', creatureSuffixIds, clientId, apiKey)
  await createTieredAffixes(ITEM_PREFIX_DEFS, 'prefix', itemPrefixIds, clientId, apiKey)
  await createTieredAffixes(ITEM_SUFFIX_DEFS, 'suffix', itemSuffixIds, clientId, apiKey)
  await createTieredAffixes(SPELL_SUFFIX_DEFS, 'suffix', spellSuffixIds, clientId, apiKey)

  for (const def of SPELL_PREFIX_DEFS) {
    const affix = await createAffix(buildSpellPrefixAffixBody(def.baseName, def.element), clientId, apiKey)
    spellPrefixIds.push(affix.id)
  }

  const totalAffixes = creaturePrefixIds.length + creatureSuffixIds.length +
    itemPrefixIds.length + itemSuffixIds.length +
    spellPrefixIds.length + spellSuffixIds.length
  console.log(`  Created ${totalAffixes} affixes`)
  console.log(`    Creature prefixes: ${creaturePrefixIds.length}, suffixes: ${creatureSuffixIds.length}`)
  console.log(`    Item prefixes: ${itemPrefixIds.length}, suffixes: ${itemSuffixIds.length}`)
  console.log(`    Spell prefixes: ${spellPrefixIds.length}, suffixes: ${spellSuffixIds.length}`)

  // ── Creature blueprints (per-level, ~250 total) ────────────────

  console.log('Creating creature blueprints...')
  const creatureBlueprintIds: NamedId[] = []

  for (let level = 1; level <= 100; level++) {
    const creatures = generateCreaturesForLevel(level)
    for (const c of creatures) {
      const weight = SEED_CONFIG.creatureWeights[c.difficulty as keyof typeof SEED_CONFIG.creatureWeights].weight
      const name = `${capitalize(c.subtype)} ${capitalize(c.difficulty)} Lv${level}`
      const bp = buildCreatureBlueprint(name, c.subtype, c.difficulty, weight, level)
      const result = await createBlueprint(bp, clientId, apiKey)
      creatureBlueprintIds.push(result)
    }
  }

  console.log(`  Created ${creatureBlueprintIds.length} creature blueprints`)

  // ── Item blueprints (per-level, ~175 total) ────────────────────

  console.log('Creating item blueprints...')
  const itemBlueprintIds: NamedId[] = []

  for (let level = 1; level <= 100; level++) {
    const items = generateItemsForLevel(level)
    for (const item of items) {
      const rarityWeight = SEED_CONFIG.rarityWeights[item.rarity as keyof typeof SEED_CONFIG.rarityWeights].weight
      const name = `${capitalize(item.subtype)} ${capitalize(item.rarity)} Lv${level}`
      const bp = buildItemBlueprint(name, item.archetype, item.rarity, item.subtype, rarityWeight, level)
      const result = await createBlueprint(bp, clientId, apiKey)
      itemBlueprintIds.push(result)
    }
  }

  console.log(`  Created ${itemBlueprintIds.length} item blueprints`)

  // ── Potion blueprints (~12 total) ──────────────────────────────

  console.log('Creating potion blueprints...')
  const potionBlueprintIds: NamedId[] = []

  const potionLevels = generatePotionLevels()
  for (let i = 0; i < potionLevels.length; i++) {
    const pLevel = potionLevels[i]
    const pType = i % 2 === 0 ? 'health' : 'mana'
    const name = `${capitalize(pType)} Potion Lv${pLevel}`
    const bp = buildPotionBlueprint(name, pType, pLevel)
    const result = await createBlueprint(bp, clientId, apiKey)
    potionBlueprintIds.push(result)
  }

  console.log(`  Created ${potionBlueprintIds.length} potion blueprints`)

  // ── Spell blueprints (~75 total, sparse across levels) ─────────

  console.log('Creating spell blueprints...')
  const spellBlueprintIds: NamedId[] = []

  for (let level = 1; level <= 100; level++) {
    const spell = generateSpellsForLevel(level)
    if (!spell) continue

    const bp = buildSpellBlueprint(spell.name, spell.spellType, spell.rarity, level)
    const result = await createBlueprint(bp, clientId, apiKey)
    spellBlueprintIds.push(result)
  }

  console.log(`  Created ${spellBlueprintIds.length} spell blueprints`)

  const totalBlueprints = creatureBlueprintIds.length + itemBlueprintIds.length +
    potionBlueprintIds.length + spellBlueprintIds.length
  console.log(`  Total: ${totalBlueprints} blueprints`)

  // ── Affix assignments ──────────────────────────────────────────

  await assignBlueprintAffixes(
    creatureBlueprintIds.map(b => b.id), creaturePrefixIds, creatureSuffixIds,
    'creature blueprints', clientId, apiKey,
  )
  await assignBlueprintAffixes(
    itemBlueprintIds.map(b => b.id), itemPrefixIds, itemSuffixIds,
    'item blueprints', clientId, apiKey,
  )
  await assignBlueprintAffixes(
    spellBlueprintIds.map(b => b.id), spellPrefixIds, spellSuffixIds,
    'spell blueprints', clientId, apiKey,
  )

  const boxW = 64
  const pad = (s: string) => s.padEnd(boxW)
  console.log(`\n${'═'.repeat(72)}`)
  console.log(`  Seed complete`)
  console.log(`  API Key: ${clientKey}`)
  console.log(`  Client ID:       ${clientId}`)
  console.log(`  GMAs:            ${Object.keys(gmas).length}`)
  console.log(`  Affixes:         ${totalAffixes}`)
  console.log(`  Creature BPs:    ${creatureBlueprintIds.length}`)
  console.log(`  Item BPs:        ${itemBlueprintIds.length}`)
  console.log(`  Potion BPs:      ${potionBlueprintIds.length}`)
  console.log(`  Spell BPs:       ${spellBlueprintIds.length}`)
  console.log(`\n╔${'═'.repeat(boxW)}╗`)
  console.log(`║${pad('=== DUNGEON CRAWLER CLIENT API KEY ===')}║`)
  console.log(`║${pad('')}║`)
  console.log(`║  ${clientKey.padEnd(boxW - 2)}║`)
  console.log(`║${pad('')}║`)
  console.log(`║${pad('Use this key to lock in a web frontend.')}║`)
  console.log(`║${pad("Set ARCHE_API_KEY=<key> in your .env")}║`)
  console.log(`╚${'═'.repeat(boxW)}╝`)
}

main().catch(err => {
  console.error('Seed failed:', err)
  process.exit(1)
})
