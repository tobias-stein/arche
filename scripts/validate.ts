import { API_BASE, SEED_CONFIG, TARGET_LEVELS } from './shared'

interface GenerateResponse {
  seed: number
  name: string
  blueprint_id: string
  blueprint_attributes: Record<string, unknown>
  affix_attributes: unknown[]
}

const SAMPLE_SIZE = 200 // increase to 500 for tighter tolerance
const MAX_ATTEMPTS = 2000

function getLevel(attrs: Record<string, unknown>): number | null {
  const v = attrs['level']
  if (typeof v === 'number') return Math.round(v)
  if (typeof v === 'string') return Math.round(parseFloat(v))
  return null
}

function getAttr(attrs: Record<string, unknown>, key: string): string | null {
  const v = attrs[key]
  if (typeof v === 'string') return v
  return null
}

async function generate(
  apiKey: string,
  constraints: Record<string, unknown>,
  archetype?: string,
): Promise<GenerateResponse> {
  const body: Record<string, unknown> = { constraints }
  if (archetype) body.archetype = archetype
  const res = await fetch(`${API_BASE}/api/generate`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
      'X-API-Key': apiKey,
      'X-Cache-Refresh': 'true',
    },
    body: JSON.stringify(body),
  })
  if (!res.ok) {
    const text = await res.text().catch(() => '')
    throw new Error(`Generate failed (${res.status}): ${text.slice(0, 200)}`)
  }
  return res.json() as Promise<GenerateResponse>
}

async function collectSamples(
  apiKey: string,
  count: number,
  window: { gte: number; lte: number },
  archetype?: string,
  extraConstraints?: Record<string, unknown>,
): Promise<{ samples: GenerateResponse[]; totalAttempts: number; levelRejections: number }> {
  const samples: GenerateResponse[] = []
  let totalAttempts = 0
  let levelRejections = 0

  while (samples.length < count && totalAttempts < MAX_ATTEMPTS) {
    totalAttempts++
    try {
      const result = await generate(apiKey, {
        level: { gte: window.gte, lte: window.lte },
        ...extraConstraints,
      }, archetype)
      const level = getLevel(result.blueprint_attributes || {})
      if (level !== null && level >= window.gte && level <= window.lte) {
        samples.push(result)
      } else {
        levelRejections++
      }
    } catch {
      // skip failed generations
    }
  }

  return { samples, totalAttempts, levelRejections }
}

async function collectItems(
  apiKey: string,
  count: number,
  window: { gte: number; lte: number },
): Promise<{ samples: GenerateResponse[]; totalAttempts: number; levelRejections: number }> {
  const samples: GenerateResponse[] = []
  let totalAttempts = 0
  let levelRejections = 0

  while (samples.length < count && totalAttempts < MAX_ATTEMPTS) {
    totalAttempts++
    try {
      const result = await generate(apiKey, {
        level: { gte: window.gte, lte: window.lte },
      })
      const attrs = result.blueprint_attributes || {}
      const level = getLevel(attrs)
      const isItem = getAttr(attrs, 'rarity') !== null
      if (!isItem) continue
      if (level !== null && level >= window.gte && level <= window.lte) {
        samples.push(result)
      } else {
        levelRejections++
      }
    } catch {
      // skip
    }
  }

  return { samples, totalAttempts, levelRejections }
}

interface CheckResult {
  label: string
  passed: boolean
  failures: string[]
}

function checkDistribution(
  samples: GenerateResponse[],
  label: string,
  window: { gte: number; lte: number },
  totalAttempts: number,
  levelRejections: number,
  isCreature: boolean,
): CheckResult {
  const result: CheckResult = { label, passed: true, failures: [] }
  const weightConfig = isCreature ? SEED_CONFIG.creatureWeights : SEED_CONFIG.rarityWeights
  const totalWeight = Object.values(weightConfig).reduce((s, w) => s + w.weight, 0)
  const targetPcts: Record<string, number> = {}
  for (const [key, cfg] of Object.entries(weightConfig)) {
    targetPcts[key] = (cfg.weight / totalWeight) * 100
  }

  const distribution: Record<string, number> = {}
  const subtypes = new Set<string>()

  for (const r of samples) {
    const attrs = r.blueprint_attributes || {}
    if (isCreature) {
      const diff = getAttr(attrs, 'difficulty') || 'unknown'
      distribution[diff] = (distribution[diff] || 0) + 1
      const subtype = getAttr(attrs, 'subtype') || 'unknown'
      subtypes.add(subtype)
    } else {
      const rarity = getAttr(attrs, 'rarity') || 'unknown'
      distribution[rarity] = (distribution[rarity] || 0) + 1
      const subtype = getAttr(attrs, 'subtype') || getAttr(attrs, 'potion_type') || getAttr(attrs, 'spell_type') || 'unknown'
      subtypes.add(subtype)
    }
  }

  console.log(`\n--- ${label} ---`)
  const efficiency = totalAttempts > samples.length
    ? ` (${totalAttempts} attempts, ${levelRejections} level rejections, ${((samples.length / totalAttempts) * 100).toFixed(0)}% efficiency)`
    : ''
  console.log(`  ${samples.length} samples${efficiency}`)

  const distKey = isCreature ? 'difficulty' : 'rarity'
  console.log(`  ${distKey} distribution:`)
  for (const [key, count] of Object.entries(distribution).sort()) {
    const pct = (count / samples.length) * 100
    const target = targetPcts[key] || 0
    const deviation = Math.abs(pct - target)
    if (key === 'unknown') {
      console.log(`    ? ${key}: ${count}/${samples.length} = ${pct.toFixed(1)}%`)
      continue
    }
    if (deviation > 10) {
      result.failures.push(`${key}: ${pct.toFixed(1)}% vs target ${target.toFixed(1)}%, Δ=${deviation.toFixed(1)}%`)
      console.log(`    ✗ ${key}: ${count}/${samples.length} = ${pct.toFixed(1)}% (target ${target.toFixed(1)}%, Δ=${deviation.toFixed(1)}%)`)
    } else {
      console.log(`    ✓ ${key}: ${count}/${samples.length} = ${pct.toFixed(1)}% (target ${target.toFixed(1)}%, Δ=${deviation.toFixed(1)}%)`)
    }
  }

  console.log(`  Subtypes (${subtypes.size}): ${[...subtypes].sort().join(', ')}`)
  if (subtypes.size < 2) {
    result.failures.push(`Only ${subtypes.size} subtypes seen (need ≥2)`)
  }

  if (result.failures.length > 0) {
    result.passed = false
  }

  return result
}

async function main() {
  const apiKey = process.env.ARCHE_API_KEY
  if (!apiKey) {
    console.error('ARCHE_API_KEY env var is required')
    process.exit(1)
  }

  console.log('=== Distribution Validation ===')
  console.log('Connecting to', API_BASE)

  const healthRes = await fetch(`${API_BASE}/health`).catch(() => null)
  if (!healthRes?.ok) {
    console.error('Arche service is not healthy')
    process.exit(1)
  }
  console.log('Arche service healthy')

  const variance = SEED_CONFIG.levelVariance
  const failures: string[] = []

  for (const playerLevel of TARGET_LEVELS) {
    const window = { gte: playerLevel - variance, lte: playerLevel + variance }

    console.log(`\n========== Creatures at level ${playerLevel} (window ${window.gte}-${window.lte}) ==========`)
    const { samples: creatureSamples, totalAttempts: creatureAttempts, levelRejections: creatureRejections } =
      await collectSamples(apiKey, SAMPLE_SIZE, window, 'creature', {
        difficulty: { in: ['normal', 'champion', 'elite'] },
      })

    if (creatureSamples.length < SAMPLE_SIZE) {
      failures.push(`Creatures Lv${playerLevel}: only ${creatureSamples.length}/${SAMPLE_SIZE} collected after ${creatureAttempts} attempts`)
      continue
    }

    const creatureResult = checkDistribution(creatureSamples, `Creatures Lv${playerLevel}`, window, creatureAttempts, creatureRejections, true)
    if (!creatureResult.passed) {
      failures.push(...creatureResult.failures.map(f => `Creatures Lv${playerLevel}: ${f}`))
    }
  }

  for (const targetLevel of TARGET_LEVELS) {
    const window = { gte: targetLevel - variance, lte: targetLevel + variance }

    console.log(`\n========== Items at level ${targetLevel} (window ${window.gte}-${window.lte}) ==========`)
    const { samples: itemSamples, totalAttempts: itemAttempts, levelRejections: itemRejections } =
      await collectItems(apiKey, SAMPLE_SIZE, window)

    if (itemSamples.length < SAMPLE_SIZE) {
      failures.push(`Items Lv${targetLevel}: only ${itemSamples.length}/${SAMPLE_SIZE} collected after ${itemAttempts} attempts`)
      continue
    }

    const itemResult = checkDistribution(itemSamples, `Items Lv${targetLevel}`, window, itemAttempts, itemRejections, false)
    if (!itemResult.passed) {
      failures.push(...itemResult.failures.map(f => `Items Lv${targetLevel}: ${f}`))
    }
  }

  console.log(`\n========================================`)
  if (failures.length > 0) {
    console.error(`FAILED (${failures.length} checks)`)
    for (const f of failures) {
      console.error(`  ✗ ${f}`)
    }
    process.exit(1)
  } else {
    console.log('ALL CHECKS PASSED ✓')
  }
}

main().catch(err => {
  console.error('Validate failed:', err)
  process.exit(1)
})
