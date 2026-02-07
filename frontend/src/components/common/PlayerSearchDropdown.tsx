import { Badge, Box, HStack, Tag, Text, VStack } from '@chakra-ui/react'
import { useState } from 'react'
import { PlayerSelectorDrawer } from './PlayerSelectorDrawer'

type Player = {
  id: string
  name: string
  currentTournamentElo: number | null
  avatarFilename?: string | null
}

type PlayerSearchDropdownProps = {
  selectedPlayers: Player[]
  selectedPlayerIds: string[]
  onTogglePlayer: (playerId: string) => void
  isCreatingPlayer: boolean
  onCreatePlayerByName: (name: string) => Promise<Player | null>
  placeholder?: string
  allPlayers?: Player[]
}

export const PlayerSearchDropdown = (props: PlayerSearchDropdownProps) => {
  const { selectedPlayers, selectedPlayerIds, onTogglePlayer, isCreatingPlayer, onCreatePlayerByName, placeholder = 'Tap to select players...', allPlayers = [] } = props

  const [drawerOpen, setDrawerOpen] = useState(false)

  const handleContainerClick = () => {
    setDrawerOpen(true)
  }

  const handleRemovePlayer = (e: React.MouseEvent, playerId: string) => {
    e.stopPropagation()
    onTogglePlayer(playerId)
  }

  return (
    <VStack w="full" align="stretch" gap={2}>
      <Box
        w="full"
        borderWidth="1px"
        borderRadius="lg"
        borderColor="border.emphasized"
        cursor="pointer"
        onClick={handleContainerClick}
        _hover={{ borderColor: 'border.active' }}
        transition="border-color 0.2s"
      >
        <HStack justify="space-between" px={4} py={2} borderBottomWidth={selectedPlayers.length > 0 ? '1px' : '0'} borderColor="border.muted">
          <Text fontWeight="medium" color="fg.muted">
            Players
          </Text>
          {selectedPlayerIds.length > 0 && (
            <Badge colorPalette="blue" variant="subtle">
              {selectedPlayerIds.length} selected
            </Badge>
          )}
        </HStack>

        <Box px={4} py={3}>
          {selectedPlayers.length === 0 ? (
            <Text color="fg.subtle" fontSize="sm">
              {placeholder}
            </Text>
          ) : (
            <HStack flexWrap="wrap" gap={2}>
              {selectedPlayers.map((player) => (
                <Tag.Root key={player.id} size="md" colorPalette="gray" variant="subtle">
                  <Tag.Label>{player.name}</Tag.Label>
                  <Tag.CloseTrigger onClick={(e) => handleRemovePlayer(e, player.id)} />
                </Tag.Root>
              ))}
            </HStack>
          )}
        </Box>
      </Box>

      <PlayerSelectorDrawer
        open={drawerOpen}
        onOpenChange={setDrawerOpen}
        allPlayers={allPlayers}
        selectedPlayerIds={selectedPlayerIds}
        onTogglePlayer={onTogglePlayer}
        isCreatingPlayer={isCreatingPlayer}
        onCreatePlayer={onCreatePlayerByName}
      />
    </VStack>
  )
}
