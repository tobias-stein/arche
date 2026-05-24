import { z } from 'zod'

const distributionConfigSchema = z.discriminatedUnion('type', [
  z.object({ type: z.literal('uniform') }),
  z.object({ type: z.literal('normal'), stdDev: z.number() }),
  z.object({
    type: z.literal('exponential'),
    rate: z.number(),
  }),
])

export const refAttributeSchema = z.object({
  $ref_id: z.string().min(1, 'Reference ID is required'),
})

export const inlineAttributeDefSchema = z.object({
  valueType: z.enum(['single', 'enum', 'range', 'string', 'boolean']),
  value: z.number().optional(),
  values: z.array(z.string()).optional(),
  min: z.number().optional(),
  max: z.number().optional(),
  minLength: z.number().optional(),
  maxLength: z.number().optional(),
  distribution: distributionConfigSchema.optional(),
  description: z.string().optional(),
})

export const blueprintAttributeSchema = z.union([
  refAttributeSchema,
  inlineAttributeDefSchema,
])

export const affixPoolEntrySchema = z.object({
  affixId: z.string().min(1),
  weight: z.number().min(0),
})

export const blueprintAffixConfigSchema = z
  .object({
    minPrefixes: z.number().min(0),
    maxPrefixes: z.number().min(0),
    minSuffixes: z.number().min(0),
    maxSuffixes: z.number().min(0),
    prefixes: z.array(affixPoolEntrySchema),
    suffixes: z.array(affixPoolEntrySchema),
  })
  .refine((data) => data.maxPrefixes >= data.minPrefixes, {
    message: 'Max prefixes must be >= min prefixes',
    path: ['maxPrefixes'],
  })
  .refine((data) => data.maxSuffixes >= data.minSuffixes, {
    message: 'Max suffixes must be >= min suffixes',
    path: ['maxSuffixes'],
  })

export const createBlueprintSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  archetype: z.string().min(1, 'Archetype is required'),
  weight: z.number().positive('Weight must be greater than 0'),
  description: z.string().optional().nullable(),
  attributes: z.record(z.string(), blueprintAttributeSchema),
  attributeOrder: z.array(z.string()),
  affixes: blueprintAffixConfigSchema,
})

export type CreateBlueprintFormValues = z.infer<typeof createBlueprintSchema>
export type DistributionConfigForm = z.infer<typeof distributionConfigSchema>
