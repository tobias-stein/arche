import { z } from 'zod'

const distributionConfigSchema = z.discriminatedUnion('type', [
  z.object({ type: z.literal('uniform') }),
  z.object({ type: z.literal('normal'), std_dev: z.number() }),
  z.object({
    type: z.literal('exponential'),
    rate: z.number(),
  }),
])

export const refAttributeSchema = z.object({
  $ref_id: z.string().min(1, 'Reference ID is required'),
})

export const inlineAttributeDefSchema = z.object({
  value_type: z.enum(['single', 'enum', 'range', 'string', 'boolean']),
  value: z.number().optional(),
  values: z.array(z.string()).optional(),
  min: z.number().optional(),
  max: z.number().optional(),
  min_length: z.number().optional(),
  max_length: z.number().optional(),
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
    min_prefixes: z.number().min(0),
    max_prefixes: z.number().min(0),
    min_suffixes: z.number().min(0),
    max_suffixes: z.number().min(0),
    prefixes: z.array(affixPoolEntrySchema),
    suffixes: z.array(affixPoolEntrySchema),
  })
  .refine((data) => data.max_prefixes >= data.min_prefixes, {
    message: 'Max prefixes must be >= min prefixes',
    path: ['max_prefixes'],
  })
  .refine((data) => data.max_suffixes >= data.min_suffixes, {
    message: 'Max suffixes must be >= min suffixes',
    path: ['max_suffixes'],
  })

export const createBlueprintSchema = z.object({
  name: z.string().min(1, 'Name is required'),
  archetype: z.string().min(1, 'Archetype is required'),
  weight: z.number().positive('Weight must be greater than 0'),
  description: z.string().optional().nullable(),
  attributes: z.record(z.string(), blueprintAttributeSchema),
  attribute_order: z.array(z.string()),
  affixes: blueprintAffixConfigSchema,
})

export const editBlueprintSchema = createBlueprintSchema.partial()

export type CreateBlueprintFormValues = z.infer<typeof createBlueprintSchema>
export type EditBlueprintFormValues = z.infer<typeof editBlueprintSchema>
export type DistributionConfigForm = z.infer<typeof distributionConfigSchema>
