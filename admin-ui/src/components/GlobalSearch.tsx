import { useEffect, useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { useQuery } from '@tanstack/react-query'
import { Blocks, Tags } from 'lucide-react'
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
import { Dialog, DialogContent } from '@/components/ui/dialog'
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
    queryFn: () => client.listBlueprints({ search: debouncedInput, per_page: 5 }),
    enabled: hasSearch,
    staleTime: 60_000,
  })

  const { data: affixData, isLoading: affixLoading } = useQuery<PaginatedResponse<Affix>>({
    queryKey: ['affixes', 'search', debouncedInput],
    queryFn: () => client.listAffixes({ search: debouncedInput, per_page: 5 }),
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
      <DialogContent className="overflow-hidden p-0" overlayClassName="bg-black/80">
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
      </DialogContent>
    </Dialog>
  )
}
