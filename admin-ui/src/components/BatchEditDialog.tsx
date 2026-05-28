import { useMemo, useState } from 'react'

import { useBatchEditBlueprints } from '@/api/generated'
import type { Blueprint, InlineAttributeDef, ValueType } from '@/api/generated'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import {
  Dialog,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { useToast } from '@/hooks/use-toast'

interface MergedAttr {
  name: string
  value_type: ValueType
  allSame: boolean
  commonValue: InlineAttributeDef | null
}

interface FormValues {
  [attrName: string]: InlineAttributeDef | null
}

function inlineAttrsEqual(a: InlineAttributeDef, b: InlineAttributeDef): boolean {
  return JSON.stringify(a) === JSON.stringify(b)
}

function computeMergedAttrs(blueprints: Blueprint[]): MergedAttr[] {
  if (blueprints.length === 0) return []

  const allKeys = new Set<string>()
  for (const bp of blueprints) {
    for (const key of bp.attributeOrder) {
      const attr = bp.attributes[key]
      if (attr && !('$ref_id' in attr)) {
        allKeys.add(key)
      }
    }
  }

  const result: MergedAttr[] = []

  for (const key of allKeys) {
    const firstAttr = blueprints[0].attributes[key]
    if (!firstAttr || '$ref_id' in firstAttr) continue
    const firstInline = firstAttr as InlineAttributeDef
    const firstType = firstInline.value_type

    let existsInAll = true
    let allSame = true

    for (const bp of blueprints) {
      const attr = bp.attributes[key]
      if (!attr || '$ref_id' in attr) {
        existsInAll = false
        break
      }
      const inline = attr as InlineAttributeDef
      if (inline.value_type !== firstType) {
        existsInAll = false
        break
      }
      if (!inlineAttrsEqual(firstInline, inline)) {
        allSame = false
      }
    }

    if (!existsInAll) continue

    result.push({
      name: key,
      value_type: firstType,
      allSame,
      commonValue: allSame ? { ...firstInline } : null,
    })
  }

  return result
}

function blankAttr(valueType: ValueType): InlineAttributeDef {
  switch (valueType) {
    case 'single':
      return { value_type: 'single', value: 0 }
    case 'enum':
      return { value_type: 'enum', values: [] }
    case 'range':
      return { value_type: 'range', min: 0, max: 0 }
    case 'string':
      return { value_type: 'string' }
    case 'boolean':
      return { value_type: 'boolean', value: false }
  }
}

interface BatchEditDialogProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  blueprints: Blueprint[]
  onSuccess: () => void
}

export function BatchEditDialog({
  open,
  onOpenChange,
  blueprints,
  onSuccess,
}: BatchEditDialogProps) {
  const { toast } = useToast()
  const batchEditMutation = useBatchEditBlueprints()

  const mergedAttrs = useMemo(() => computeMergedAttrs(blueprints), [blueprints])

  const [formValues, setFormValues] = useState<FormValues>(() => {
    const initial: FormValues = {}
    for (const attr of mergedAttrs) {
      initial[attr.name] = attr.commonValue
    }
    return initial
  })

  function setAttrValue(attrName: string, updater: (prev: InlineAttributeDef) => InlineAttributeDef) {
    setFormValues((prev) => {
      const current = prev[attrName]
      if (!current) {
        const mt = mergedAttrs.find((a) => a.name === attrName)
        if (!mt) return prev
        return { ...prev, [attrName]: updater(blankAttr(mt.value_type)) }
      }
      return { ...prev, [attrName]: updater(current) }
    })
  }

  function setSingleValue(attrName: string, _field: 'value', val: number) {
    setAttrValue(attrName, (prev) => {
      if ('value' in prev) return { ...prev, value: val } as InlineAttributeDef
      return prev
    })
  }

  function setEnumValue(attrName: string, text: string) {
    setAttrValue(attrName, (prev) => {
      if ('values' in prev) {
        const values = text.split(',').map((s) => s.trim()).filter(Boolean)
        return { ...prev, values } as InlineAttributeDef
      }
      return prev
    })
  }

  function setRangeValue(attrName: string, field: 'min' | 'max', val: number) {
    setAttrValue(attrName, (prev) => {
      if ('min' in prev || 'max' in prev) {
        return { ...prev, [field]: val } as InlineAttributeDef
      }
      return prev
    })
  }

  function setStringValue(attrName: string, field: 'minLength' | 'maxLength', val: string) {
    setAttrValue(attrName, (prev) => {
      const numVal = val === '' ? undefined : Number(val)
      return { ...prev, [field]: numVal } as InlineAttributeDef
    })
  }

  function setBoolValue(attrName: string, val: boolean) {
    setAttrValue(attrName, (prev) => {
      if ('value' in prev && prev.value_type === 'boolean') {
        return { ...prev, value: val } as InlineAttributeDef
      }
      return prev
    })
  }

  function getDisplayValue(attrName: string, field: string): string | number {
    const val = formValues[attrName]
    if (!val) return ''
    const v = (val as Record<string, unknown>)[field]
    if (v === undefined || v === null) return ''
    if (typeof v === 'boolean') return ''
    return String(v)
  }

  function getBoolChecked(attrName: string): boolean {
    const val = formValues[attrName]
    if (!val || !('value' in val) || val.value_type !== 'boolean') return false
    return val.value as boolean
  }

  function hasChanges(): boolean {
    return mergedAttrs.some((attr) => {
      const current = formValues[attr.name]
      if (current === null && attr.commonValue === null) return false
      if (current === null || attr.commonValue === null) return true
      return !inlineAttrsEqual(current, attr.commonValue)
    })
  }

  async function handleSubmit() {
    const attributes: Record<string, unknown> = {}
    for (const attr of mergedAttrs) {
      const val = formValues[attr.name]
      if (val) {
        attributes[attr.name] = val
      }
    }
    if (Object.keys(attributes).length === 0) return

    try {
      await batchEditMutation.mutateAsync({
        blueprintIds: blueprints.map((bp) => bp.id),
        attributes,
      })
      toast({
        title: `Updated attributes on ${blueprints.length} ${blueprints.length === 1 ? 'blueprint' : 'blueprints'}`,
      })
      onOpenChange(false)
      onSuccess()
    } catch {
      toast({ title: 'Failed to batch edit blueprints', variant: 'destructive' })
    }
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-lg max-h-[85vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Batch Edit Blueprints</DialogTitle>
        </DialogHeader>

        <div className="space-y-4">
          <div className="text-sm text-muted-foreground">
            Editing {blueprints.length} {blueprints.length === 1 ? 'blueprint' : 'blueprints'}:{' '}
            {blueprints.map((bp) => bp.name).join(', ')}
          </div>

          {mergedAttrs.length === 0 ? (
            <div className="text-center py-6 border bg-background rounded-md">
              <p className="text-muted-foreground text-sm">
                No common attributes found across selected blueprints
              </p>
            </div>
          ) : (
            <div className="space-y-4">
              {mergedAttrs.map((attr) => (
                <fieldset
                  key={attr.name}
                  className="border bg-background rounded-md p-3 space-y-2"
                >
                  <legend className="text-sm font-medium px-1 flex items-center gap-2">
                    {attr.name}
                    <Badge variant="outline" className="text-xs">
                      {attr.value_type}
                    </Badge>
                    {!attr.allSame && (
                      <span className="text-xs text-muted-foreground">
                        (values differ — blank)
                      </span>
                    )}
                  </legend>

                  {attr.value_type === 'single' && (
                    <div className="space-y-1">
                      <Label className="text-xs">Value</Label>
                      <Input
                        type="number"
                        value={getDisplayValue(attr.name, 'value')}
                        onChange={(e) =>
                          setSingleValue(attr.name, 'value', Number(e.target.value))
                        }
                        placeholder="Enter a number"
                      />
                    </div>
                  )}

                  {attr.value_type === 'enum' && (
                    <div className="space-y-1">
                      <Label className="text-xs">Values (comma-separated)</Label>
                      <Input
                        type="text"
                        value={
                          formValues[attr.name] && 'values' in (formValues[attr.name] || {})
                            ? (formValues[attr.name] as { values: string[] }).values.join(', ')
                            : ''
                        }
                        onChange={(e) => setEnumValue(attr.name, e.target.value)}
                        placeholder="e.g. fire, ice, lightning"
                      />
                    </div>
                  )}

                  {attr.value_type === 'range' && (
                    <div className="grid grid-cols-2 gap-2">
                      <div className="space-y-1">
                        <Label className="text-xs">Min</Label>
                        <Input
                          type="number"
                          value={getDisplayValue(attr.name, 'min')}
                          onChange={(e) =>
                            setRangeValue(attr.name, 'min', Number(e.target.value))
                          }
                          placeholder="Min"
                        />
                      </div>
                      <div className="space-y-1">
                        <Label className="text-xs">Max</Label>
                        <Input
                          type="number"
                          value={getDisplayValue(attr.name, 'max')}
                          onChange={(e) =>
                            setRangeValue(attr.name, 'max', Number(e.target.value))
                          }
                          placeholder="Max"
                        />
                      </div>
                    </div>
                  )}

                  {attr.value_type === 'string' && (
                    <div className="grid grid-cols-2 gap-2">
                      <div className="space-y-1">
                        <Label className="text-xs">Min Length</Label>
                        <Input
                          type="number"
                          value={getDisplayValue(attr.name, 'minLength')}
                          onChange={(e) =>
                            setStringValue(attr.name, 'minLength', e.target.value)
                          }
                          placeholder="Optional"
                        />
                      </div>
                      <div className="space-y-1">
                        <Label className="text-xs">Max Length</Label>
                        <Input
                          type="number"
                          value={getDisplayValue(attr.name, 'maxLength')}
                          onChange={(e) =>
                            setStringValue(attr.name, 'maxLength', e.target.value)
                          }
                          placeholder="Optional"
                        />
                      </div>
                    </div>
                  )}

                  {attr.value_type === 'boolean' && (
                    <label className="flex items-center gap-2 text-sm cursor-pointer">
                      <input
                        type="checkbox"
                        checked={getBoolChecked(attr.name)}
                        onChange={(e) =>
                          setBoolValue(attr.name, e.target.checked)
                        }
                        className="rounded"
                      />
                      Value
                    </label>
                  )}
                </fieldset>
              ))}
            </div>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button
            onClick={handleSubmit}
            disabled={mergedAttrs.length === 0 || !hasChanges() || batchEditMutation.isPending}
          >
            {batchEditMutation.isPending
              ? 'Saving...'
              : `Update (${blueprints.length} ${blueprints.length === 1 ? 'blueprint' : 'blueprints'})`}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
