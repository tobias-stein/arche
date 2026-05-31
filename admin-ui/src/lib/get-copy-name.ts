export function getCopyName(name: string, existing: { name: string }[]): string {
  const base = `${name} (Copy)`
  const existingNames = new Set(existing.map((b) => b.name))
  if (!existingNames.has(base)) return base
  let n = 2
  while (existingNames.has(`${name} (Copy ${n})`)) n++
  return `${name} (Copy ${n})`
}
