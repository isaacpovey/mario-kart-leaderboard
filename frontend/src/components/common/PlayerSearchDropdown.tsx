import { Box, Checkbox, HStack, Input, Tag, Text, VStack } from '@chakra-ui/react'
import { useEffect, useRef } from 'react'

type Player = {
  id: string
  name: string
  currentTournamentElo: number | null
}

type PlayerSearchDropdownProps = {
  searchTerm: string
  onSearchTermChange: (term: string) => void
  showDropdown: boolean
  onShowDropdownChange: (show: boolean) => void
  filteredPlayers: Player[]
  selectedPlayers: Player[]
  selectedPlayerIds: string[]
  onTogglePlayer: (playerId: string) => void
  canCreateNewPlayer: boolean
  isCreatingPlayer: boolean
  onCreateAndSelectPlayer: () => Promise<Player | null>
  placeholder?: string
}

export const PlayerSearchDropdown = (props: PlayerSearchDropdownProps) => {
  const {
    searchTerm,
    onSearchTermChange,
    showDropdown,
    onShowDropdownChange,
    filteredPlayers,
    selectedPlayers,
    selectedPlayerIds,
    onTogglePlayer,
    canCreateNewPlayer,
    isCreatingPlayer,
    onCreateAndSelectPlayer,
    placeholder = 'Search or type to create player...',
  } = props

  const blurTimeoutRef = useRef<number | null>(null)
  const dropdownRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (dropdownRef.current && !dropdownRef.current.contains(event.target as Node)) {
        onShowDropdownChange(false)
      }
    }

    if (showDropdown) {
      document.addEventListener('mousedown', handleClickOutside)
    }

    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [showDropdown, onShowDropdownChange])

  return (
    <VStack align="stretch" gap={2}>
      <Box position="relative" ref={dropdownRef}>
        <Input
          placeholder={placeholder}
          value={searchTerm}
          onChange={(e) => {
            onSearchTermChange(e.target.value)
            onShowDropdownChange(true)
          }}
          onFocus={() => onShowDropdownChange(true)}
          onBlur={() => {
            blurTimeoutRef.current = window.setTimeout(() => onShowDropdownChange(false), 300)
          }}
          onKeyDown={(e) => {
            if (e.key === 'Enter') {
              e.preventDefault()
              if (filteredPlayers.length > 0) {
                onTogglePlayer(filteredPlayers[0].id)
                onSearchTermChange('')
                onShowDropdownChange(false)
              } else if (canCreateNewPlayer) {
                onCreateAndSelectPlayer()
              }
            }
          }}
        />
        {showDropdown && searchTerm.trim() && (filteredPlayers.length > 0 || canCreateNewPlayer) && (
          <Box position="absolute" zIndex={10} bg="bg.panel" borderWidth="1px" borderRadius="md" mt={1} maxH="200px" overflowY="auto" width="100%">
            {filteredPlayers.map((player) => (
              <Box
                key={player.id}
                p={2}
                cursor="pointer"
                _hover={{ bg: 'bg.subtle' }}
                onClick={() => {
                  if (blurTimeoutRef.current) {
                    clearTimeout(blurTimeoutRef.current)
                  }
                  onTogglePlayer(player.id)
                  onSearchTermChange('')
                  onShowDropdownChange(false)
                }}
              >
                <Checkbox.Root checked={selectedPlayerIds.includes(player.id)}>
                  <Checkbox.HiddenInput />
                  <Checkbox.Control />
                  <Checkbox.Label>
                    {player.name} (ELO: {player.currentTournamentElo ?? 1200})
                  </Checkbox.Label>
                </Checkbox.Root>
              </Box>
            ))}
            {canCreateNewPlayer && (
              <Box
                p={2}
                cursor={isCreatingPlayer ? 'not-allowed' : 'pointer'}
                _hover={{ bg: isCreatingPlayer ? undefined : 'bg.subtle' }}
                onClick={() => {
                  if (blurTimeoutRef.current) {
                    clearTimeout(blurTimeoutRef.current)
                  }
                  onCreateAndSelectPlayer()
                }}
                borderTopWidth={filteredPlayers.length > 0 ? '1px' : '0'}
              >
                <Text color="blue.500" fontWeight="semibold">
                  {isCreatingPlayer ? 'Creating...' : `Create "${searchTerm}"`}
                </Text>
              </Box>
            )}
            {filteredPlayers.length === 0 && !canCreateNewPlayer && (
              <Box p={2}>
                <Text color="fg.subtle">No players found</Text>
              </Box>
            )}
          </Box>
        )}
      </Box>
      {selectedPlayers.length > 0 && (
        <HStack flexWrap="wrap" gap={2}>
          {selectedPlayers.map((player) => (
            <Tag.Root key={player.id} size="sm">
              <Tag.Label>{player.name}</Tag.Label>
              <Tag.CloseTrigger onClick={() => onTogglePlayer(player.id)} />
            </Tag.Root>
          ))}
        </HStack>
      )}
    </VStack>
  )
}
