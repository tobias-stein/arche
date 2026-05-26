import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { Blocks, Tags, X } from 'lucide-react'
import { useUi } from '@/stores/ui'
import { getClient } from '@/api/generated/hooks'
import type { Affix, Blueprint, PaginatedResponse } from '@/api/generated/types'
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from '@/components/ui/command'
import * as DialogPrimitive from '@radix-ui/react-dialog'
import { Dialog, DialogOverlay, DialogPortal, DialogClose } from '@/components/ui/dialog'
import { Badge } from '@/components/ui/badge'

export function GlobalSearch() {
  const { searchOpen, openSearch, closeSearch } = useUi()
  const [input, setInput] = useState('')
  const [debouncedInput, setDebouncedInput] = useState('')
  const navigate = useNavigate()
  const client = getClient()

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedInput(input), 300)
    return () => clearTimeout(timer)
  }, [input])

  useEffect(() => {
    if (!searchOpen) {
      // eslint-disable-next-line react-hooks/set-state-in-effect
      setInput('')
      setDebouncedInput('')
    }
  }, [searchOpen])

  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === 'k') {
        e.preventDefault()
        openSearch()
      }
    }
    document.addEventListener('keydown', handler)
    return () => document.removeEventListener('keydown', handler)
  }, [openSearch])

  const hasSearch = debouncedInput.length > 0

  const { data: bpData, isLoading: bpLoading } = useQuery<PaginatedResponse<Blueprint>>({
    queryKey: ['blueprints', 'search', debouncedInput],
    queryFn: () => client.listBlueprints({ search: debouncedInput, perPage: 5 }),
    enabled: hasSearch,
    staleTime: 60_000,
  })

  const { data: affixData, isLoading: affixLoading } = useQuery<PaginatedResponse<Affix>>({
    queryKey: ['affixes', 'search', debouncedInput],
    queryFn: () => client.listAffixes({ search: debouncedInput, perPage: 5 }),
    enabled: hasSearch,
    staleTime: 60_000,
  })

  const blueprints = bpData?.data ?? []
  const affixes = affixData?.data ?? []
  const isLoading = bpLoading || affixLoading
  const hasResults = blueprints.length > 0 || affixes.length > 0

  const handleSelect = (type: 'blueprint' | 'affix', id: string) => {
    closeSearch()
    if (type === 'blueprint') {
      navigate(`/blueprints/${id}`)
    } else {
      navigate(`/affixes/${id}`)
    }
  }

  return (
    <Dialog open={searchOpen} onOpenChange={(open) => !open && closeSearch()}>
      <DialogPortal>
        <DialogOverlay className="bg-black/30" />
        <DialogPrimitive.Content className="fixed left-[50%] top-[50%] z-50 grid w-full max-w-lg translate-x-[-50%] translate-y-[-50%] gap-4 border bg-background shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] sm:rounded-lg overflow-hidden p-0">
          <Command shouldFilter={false}>
            <CommandInput
              placeholder="Search blueprints and affixes..."
              value={input}
              onValueChange={setInput}
            />
            <CommandList>
              {hasSearch && !isLoading && !hasResults && (
                <CommandEmpty>No results found.</CommandEmpty>
              )}
              {!hasSearch && (
                <CommandEmpty>Type to search blueprints and affixes by name.</CommandEmpty>
              )}
              {hasSearch && !isLoading && blueprints.length > 0 && (
                <CommandGroup heading="Blueprints">
                  {blueprints.map((bp) => (
                    <CommandItem
                      key={bp.id}
                      value={`bp-${bp.name}`}
                      onSelect={() => handleSelect('blueprint', bp.id)}
                    >
                      <Blocks className="h-4 w-4" />
                      <span>{bp.name}</span>
                      <Badge variant="secondary" className="ml-auto text-xs">
                        {bp.archetype}
                      </Badge>
                    </CommandItem>
                  ))}
                </CommandGroup>
              )}
              {hasSearch && !isLoading && affixes.length > 0 && (
                <CommandGroup heading="Affixes">
                  {affixes.map((affix) => (
                    <CommandItem
                      key={affix.id}
                      value={`affix-${affix.name}`}
                      onSelect={() => handleSelect('affix', affix.id)}
                    >
                      <Tags className="h-4 w-4" />
                      <span>{affix.name}</span>
                      <Badge variant="secondary" className="ml-auto text-xs">
                        {affix.location}
                      </Badge>
                    </CommandItem>
                  ))}
                </CommandGroup>
              )}
            </CommandList>
          </Command>
          <DialogClose className="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-accent data-[state=open]:text-muted-foreground">
            <X className="h-4 w-4" />
            <span className="sr-only">Close</span>
          </DialogClose>
        </DialogPrimitive.Content>
      </DialogPortal>
    </Dialog>
  )
}
