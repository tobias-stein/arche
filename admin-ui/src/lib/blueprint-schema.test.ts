import { describe, expect, it } from 'vitest'
import { createBlueprintSchema } from '@/lib/blueprint-schema'

describe('createBlueprintSchema', () => {
  const validData = {
    name: 'Test Blueprint',
    archetype: 'weapon',
    weight: 5,
    description: null,
    attributes: {
      damage: { value_type: 'single', value: 10 },
    },
    attributeOrder: ['damage'],
    affixes: {
      minPrefixes: 0,
      maxPrefixes: 2,
      minSuffixes: 0,
      maxSuffixes: 0,
      prefixes: [],
      suffixes: [],
    },
  }

  it('validates a complete valid blueprint', () => {
    const result = createBlueprintSchema.safeParse(validData)
    expect(result.success).toBe(true)
  })

  it('fails when name is empty', () => {
    const result = createBlueprintSchema.safeParse({
      ...validData,
      name: '',
    })
    expect(result.success).toBe(false)
    expect(result.error?.issues[0].message).toBe('Name is required')
  })

  it('fails when archetype is empty', () => {
    const result = createBlueprintSchema.safeParse({
      ...validData,
      archetype: '',
    })
    expect(result.success).toBe(false)
    expect(result.error?.issues[0].message).toBe('Archetype is required')
  })

  it('fails when weight is not positive', () => {
    const result = createBlueprintSchema.safeParse({
      ...validData,
      weight: 0,
    })
    expect(result.success).toBe(false)
    expect(result.error?.issues[0].message).toBe(
      'Weight must be greater than 0',
    )
  })

  it('fails when max prefixes is less than min prefixes', () => {
    const result = createBlueprintSchema.safeParse({
      ...validData,
      affixes: {
        ...validData.affixes,
        minPrefixes: 5,
        maxPrefixes: 2,
      },
    })
    expect(result.success).toBe(false)
    expect(result.error?.issues[0].message).toBe(
      'Max prefixes must be >= min prefixes',
    )
  })

  it('fails when max suffixes is less than min suffixes', () => {
    const result = createBlueprintSchema.safeParse({
      ...validData,
      affixes: {
        ...validData.affixes,
        minSuffixes: 5,
        maxSuffixes: 2,
      },
    })
    expect(result.success).toBe(false)
    expect(result.error?.issues[0].message).toBe(
      'Max suffixes must be >= min suffixes',
    )
  })

  it('validates ref attribute with $ref_id', () => {
    const result = createBlueprintSchema.safeParse({
      ...validData,
      attributes: {
        dmg: { $ref_id: 'gma-1' },
      },
      attributeOrder: ['dmg'],
    })
    expect(result.success).toBe(true)
  })

  it('fails when ref attribute has empty $ref_id', () => {
    const result = createBlueprintSchema.safeParse({
      ...validData,
      attributes: {
        dmg: { $ref_id: '' },
      },
      attributeOrder: ['dmg'],
    })
    expect(result.success).toBe(false)
  })
})
