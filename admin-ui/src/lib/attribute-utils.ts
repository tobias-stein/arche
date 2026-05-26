import type { GlobalMetaAttribute } from '@/api/generated/types'

export function getPreview(gma: GlobalMetaAttribute): string {
  const p = gma.payload
  switch (gma.valueType) {
    case 'single':
      return String(p.value ?? '\u2014')
    case 'enum': {
      const values = p.values as string[] | undefined
      return values?.join(', ') ?? '\u2014'
    }
    case 'range':
      return `${p.min ?? '?'} \u2013 ${p.max ?? '?'}`
    case 'string': {
      const min = p.minLength
      const max = p.maxLength
      if (min != null && max != null) return `${min} \u2013 ${max} characters`
      if (min != null) return `\u2265 ${min} characters`
      if (max != null) return `\u2264 ${max} characters`
      return '\u2014'
    }
    case 'boolean':
      return String(p.value ?? '\u2014')
    default:
      return '\u2014'
  }
}

export interface AttributeFormState {
  singleValue: string
  enumValues: string
  rangeMin: string
  rangeMax: string
  strMinLen: string
  strMaxLen: string
  boolVal: boolean
  useDist: boolean
  distType: 'uniform' | 'normal' | 'exponential'
  distStdDev: string
  distRate: string
}

export function extractPayloadState(gma: GlobalMetaAttribute): AttributeFormState {
  const p = gma.payload
  let singleValue = ''
  let enumValues = ''
  let rangeMin = ''
  let rangeMax = ''
  let strMinLen = ''
  let strMaxLen = ''
  let boolVal = false
  let useDist = false
  let distType: 'uniform' | 'normal' | 'exponential' = 'uniform'
  let distStdDev = ''
  let distRate = ''

  switch (gma.valueType) {
    case 'single':
      singleValue = p.value != null ? String(p.value) : ''
      break
    case 'enum':
      enumValues = ((p.values as string[] | undefined) ?? []).join(', ')
      break
    case 'range':
      rangeMin = p.min != null ? String(p.min) : ''
      rangeMax = p.max != null ? String(p.max) : ''
      break
    case 'string':
      strMinLen = p.minLength != null ? String(p.minLength) : ''
      strMaxLen = p.maxLength != null ? String(p.maxLength) : ''
      break
    case 'boolean':
      boolVal = p.value === true
      break
  }

  const dist = p.distribution as { type?: string; stdDev?: number; rate?: number } | undefined
  if (dist && (gma.valueType === 'single' || gma.valueType === 'range')) {
    useDist = true
    if (dist.type === 'normal' || dist.type === 'exponential') {
      distType = dist.type as 'normal' | 'exponential'
    }
    if (dist.type === 'normal' && dist.stdDev != null) {
      distStdDev = String(dist.stdDev)
    }
    if (dist.type === 'exponential' && dist.rate != null) {
      distRate = String(dist.rate)
    }
  }

  return { singleValue, enumValues, rangeMin, rangeMax, strMinLen, strMaxLen, boolVal, useDist, distType, distStdDev, distRate }
}

export function buildDistribution(
  distType: 'uniform' | 'normal' | 'exponential',
  distStdDev: string,
  distRate: string,
): { type: string; stdDev?: number; rate?: number } | null {
  if (distType === 'uniform') return { type: 'uniform' }
  if (distType === 'normal') {
    const sd = parseFloat(distStdDev)
    if (isNaN(sd) || sd <= 0) return null
    return { type: 'normal', stdDev: sd }
  }
  const rate = parseFloat(distRate)
  if (isNaN(rate) || rate <= 0) return null
  return { type: 'exponential', rate }
}
