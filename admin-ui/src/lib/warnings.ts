import type {
  Affix,
  AffixPoolEntry,
  Blueprint,
  GlobalMetaAttribute,
  InlineAttributeDef,
  RefAttribute,
} from '@/api/generated'

export interface Warning {
  id: string
  title: string
  description: string
  severity: 'error' | 'warning' | 'info'
  resourceType: 'blueprint' | 'affix' | 'global_meta_attribute'
  resourceId: string
  resourceName: string
}

type ExtendedBlueprint = Blueprint & {
  prefixes?: AffixPoolEntry[]
  suffixes?: AffixPoolEntry[]
}

function checkBlueprintWeight(
  bp: Blueprint,
  warnings: Warning[],
) {
  if (bp.weight === 0) {
    warnings.push({
      id: `zero-weight-bp-${bp.id}`,
      title: 'Zero-weight blueprint',
      description: `Blueprint "${bp.name}" has a weight of 0 and will never be selected.`,
      severity: 'warning',
      resourceType: 'blueprint',
      resourceId: bp.id,
      resourceName: bp.name,
    })
  }
}

function checkBlueprintPool(
  bp: ExtendedBlueprint,
  warnings: Warning[],
) {
  if (bp.minPrefixes > 0 && (bp.prefixes?.length ?? 0) === 0) {
    warnings.push({
      id: `empty-prefix-pool-${bp.id}`,
      title: 'Empty prefix pool',
      description: `Blueprint "${bp.name}" requires at least ${bp.minPrefixes} prefix(es) but has no prefixes assigned.`,
      severity: 'error',
      resourceType: 'blueprint',
      resourceId: bp.id,
      resourceName: bp.name,
    })
    return
  }
  if (bp.minPrefixes > (bp.prefixes?.length ?? 0)) {
    warnings.push({
      id: `insufficient-prefix-pool-${bp.id}`,
      title: 'Insufficient prefix pool',
      description: `Blueprint "${bp.name}" requires at least ${bp.minPrefixes} prefix(es) but has only ${bp.prefixes?.length ?? 0} assigned.`,
      severity: 'warning',
      resourceType: 'blueprint',
      resourceId: bp.id,
      resourceName: bp.name,
    })
  }
  if (bp.minSuffixes > 0 && (bp.suffixes?.length ?? 0) === 0) {
    warnings.push({
      id: `empty-suffix-pool-${bp.id}`,
      title: 'Empty suffix pool',
      description: `Blueprint "${bp.name}" requires at least ${bp.minSuffixes} suffix(es) but has no suffixes assigned.`,
      severity: 'error',
      resourceType: 'blueprint',
      resourceId: bp.id,
      resourceName: bp.name,
    })
    return
  }
  if (bp.minSuffixes > (bp.suffixes?.length ?? 0)) {
    warnings.push({
      id: `insufficient-suffix-pool-${bp.id}`,
      title: 'Insufficient suffix pool',
      description: `Blueprint "${bp.name}" requires at least ${bp.minSuffixes} suffix(es) but has only ${bp.suffixes?.length ?? 0} assigned.`,
      severity: 'warning',
      resourceType: 'blueprint',
      resourceId: bp.id,
      resourceName: bp.name,
    })
  }
}

function checkBlueprintPoolWeights(
  bp: ExtendedBlueprint,
  warnings: Warning[],
) {
  const pools = [
    ...(bp.prefixes ?? []).map((e) => ({ ...e, type: 'prefix' as const })),
    ...(bp.suffixes ?? []).map((e) => ({ ...e, type: 'suffix' as const })),
  ]
  for (const entry of pools) {
    if (entry.weight === 0) {
      warnings.push({
        id: `zero-weight-pool-${bp.id}-${entry.affixId}`,
        title: 'Zero-weight affix assignment',
        description: `Blueprint "${bp.name}" has a ${entry.type} affix "${entry.affixId}" with weight 0 — it will never roll.`,
        severity: 'warning',
        resourceType: 'blueprint',
        resourceId: bp.id,
        resourceName: bp.name,
      })
    }
  }
}

function checkAttributePayload(
  attr: InlineAttributeDef,
  resourceId: string,
  resourceName: string,
  resourceType: 'blueprint' | 'affix',
  attrKey: string,
  warnings: Warning[],
) {
  switch (attr.valueType) {
    case 'range': {
      if (attr.min > attr.max) {
        warnings.push({
          id: `invalid-range-${resourceId}-${attrKey}`,
          title: 'Invalid range attribute',
          description: `${resourceType === 'blueprint' ? 'Blueprint' : 'Affix'} "${resourceName}" has attribute "${attrKey}" with min (${attr.min}) greater than max (${attr.max}).`,
          severity: 'error',
          resourceType,
          resourceId,
          resourceName,
        })
      }
      if (attr.distribution) {
        checkDistribution(attr.distribution, resourceId, resourceName, resourceType, attrKey, warnings)
      }
      break
    }
    case 'enum': {
      if (attr.values.length === 0) {
        warnings.push({
          id: `empty-enum-${resourceId}-${attrKey}`,
          title: 'Empty enum attribute',
          description: `${resourceType === 'blueprint' ? 'Blueprint' : 'Affix'} "${resourceName}" has attribute "${attrKey}" with an empty enum list.`,
          severity: 'error',
          resourceType,
          resourceId,
          resourceName,
        })
      }
      break
    }
    case 'single': {
      if (attr.distribution) {
        checkDistribution(attr.distribution, resourceId, resourceName, resourceType, attrKey, warnings)
      }
      break
    }
    default:
      break
  }
}

function checkDistribution(
  dist: { type: string; stdDev?: number; rate?: number },
  resourceId: string,
  resourceName: string,
  resourceType: 'blueprint' | 'affix',
  attrKey: string,
  warnings: Warning[],
) {
  if (dist.type === 'normal' && (dist.stdDev === 0 || dist.stdDev === undefined)) {
    warnings.push({
      id: `invalid-dist-${resourceId}-${attrKey}`,
      title: 'Invalid distribution config',
      description: `${resourceType === 'blueprint' ? 'Blueprint' : 'Affix'} "${resourceName}" has attribute "${attrKey}" with a normal distribution where stdDev is 0.`,
      severity: 'error',
      resourceType,
      resourceId,
      resourceName,
    })
  }
  if (dist.type === 'exponential' && (dist.rate === 0 || dist.rate === undefined)) {
    warnings.push({
      id: `invalid-dist-${resourceId}-${attrKey}`,
      title: 'Invalid distribution config',
      description: `${resourceType === 'blueprint' ? 'Blueprint' : 'Affix'} "${resourceName}" has attribute "${attrKey}" with an exponential distribution where rate is 0.`,
      severity: 'error',
      resourceType,
      resourceId,
      resourceName,
    })
  }
}

function isRefAttribute(attr: unknown): attr is RefAttribute {
  return typeof attr === 'object' && attr !== null && '$ref_id' in (attr as Record<string, unknown>)
}

function isInlineAttribute(attr: unknown): attr is InlineAttributeDef {
  return typeof attr === 'object' && attr !== null && 'valueType' in (attr as Record<string, unknown>)
}

function checkBlueprintAttributes(
  bp: Blueprint,
  globalIds: Set<string>,
  warnings: Warning[],
) {
  for (const key of bp.attributeOrder ?? []) {
    const attr = bp.attributes?.[key]
    if (!attr) continue

    if (isRefAttribute(attr)) {
      if (!globalIds.has(attr.$ref_id)) {
        warnings.push({
          id: `dangling-ref-bp-${bp.id}-${key}`,
          title: 'Dangling `$ref_id`',
          description: `Blueprint "${bp.name}" references global meta attribute "${attr.$ref_id}" via key "${key}", but that global attribute does not exist.`,
          severity: 'error',
          resourceType: 'blueprint',
          resourceId: bp.id,
          resourceName: bp.name,
        })
      }
    } else if (isInlineAttribute(attr)) {
      checkAttributePayload(attr, bp.id, bp.name, 'blueprint', key, warnings)
    }
  }
}

function checkAffixAttributes(
  affix: Affix,
  globalIds: Set<string>,
  warnings: Warning[],
) {
  const attr = affix.attribute
  if (!attr) return

  if (isRefAttribute(attr)) {
    if (!globalIds.has(attr.$ref_id)) {
      warnings.push({
        id: `dangling-ref-aff-${affix.id}`,
        title: 'Dangling `$ref_id`',
        description: `Affix "${affix.name}" references global meta attribute "${attr.$ref_id}", but that global attribute does not exist.`,
        severity: 'error',
        resourceType: 'affix',
        resourceId: affix.id,
        resourceName: affix.name,
      })
    }
  } else if (isInlineAttribute(attr)) {
    checkAttributePayload(attr, affix.id, affix.name, 'affix', 'attribute', warnings)
  }
}

export function computeWarnings(
  blueprints: Blueprint[],
  affixes: Affix[],
  globalMetaAttributes: GlobalMetaAttribute[],
): Warning[] {
  const warnings: Warning[] = []
  const globalIds = new Set(globalMetaAttributes.map((g) => g.id))

  for (const bp of blueprints) {
    const ext = bp as ExtendedBlueprint
    checkBlueprintWeight(bp, warnings)
    checkBlueprintPool(ext, warnings)
    checkBlueprintPoolWeights(ext, warnings)
    checkBlueprintAttributes(bp, globalIds, warnings)
  }

  for (const affix of affixes) {
    checkAffixAttributes(affix, globalIds, warnings)
  }

  return warnings
}
