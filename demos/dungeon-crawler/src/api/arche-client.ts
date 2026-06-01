import type { GenerateRequest, GenerateResponse, RawGenerateResponse, AttributeValue } from './types'

let apiUrl = 'http://localhost:8080'
let apiKey = ''

export function configureArcheClient(url: string, key: string): void {
  apiUrl = url
  apiKey = key
}

export async function generate(request: GenerateRequest): Promise<GenerateResponse> {
  const headers: Record<string, string> = { 'Content-Type': 'application/json' }
  if (apiKey) headers['X-API-Key'] = apiKey

  const response = await fetch(`${apiUrl}/api/generate`, {
    method: 'POST',
    headers,
    body: JSON.stringify(request),
  })

  if (!response.ok) {
    const text = await response.text().catch(() => '')
    throw new Error(`Arche API error: ${response.status} ${response.statusText} — ${text.slice(0, 200)}`)
  }

  const raw = await response.json() as RawGenerateResponse

  const blueprint_attributes: AttributeValue[] = Object.entries(raw.blueprint_attributes).map(([name, value]) => ({
    attribute_id: '',
    name,
    value_type: typeof value === 'number' ? 'number' : String(typeof value),
    value: String(value),
  }))

  const affix_attributes: AttributeValue[][] = raw.affix_attributes.map(entry => {
    const { affix_id, affix_name, ...rest } = entry
    return Object.entries(rest).map(([name, value]) => ({
      attribute_id: affix_id,
      name,
      value_type: typeof value === 'number' ? 'number' : String(typeof value),
      value: String(value),
    }))
  })

  return {
    things: [{
      name: raw.name,
      archetype: request.archetype || '',
      name_parts: raw.name_parts,
      blueprint_attributes,
      affix_attributes,
    }],
    seed: raw.seed,
  }
}

