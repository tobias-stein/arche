export interface AttributeValue {
  attribute_id: string
  name: string
  value_type: string
  value: string | number
}

export interface GenerateRequest {
  archetype?: string
  constraints?: Record<string, { gte?: number; lte?: number; in?: string[] }>
  affixes?: {
    min_prefixes?: number
    max_prefixes?: number
    min_suffixes?: number
    max_suffixes?: number
  }
  seed?: number
}

export interface NameParts {
  base: string
  prefix: string | null
  suffix: string | null
}

export interface GeneratedThing {
  name: string
  archetype: string
  name_parts: NameParts
  blueprint_attributes: AttributeValue[]
  affix_attributes: AttributeValue[][]
}

export interface GenerateResponse {
  things: GeneratedThing[]
  seed: number
}
