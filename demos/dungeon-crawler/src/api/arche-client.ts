import type { GenerateRequest, GenerateResponse } from './types'

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

  return response.json() as Promise<GenerateResponse>
}

