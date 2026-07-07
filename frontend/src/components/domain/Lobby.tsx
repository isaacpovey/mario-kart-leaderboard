import { Box, Button, HStack, Heading, Input, Text, VStack } from '@chakra-ui/react'
import { useAtomValue } from 'jotai'
import { useMemo, useState } from 'react'
import { useLobby } from '../../hooks/features/useLobby'
import { usePlayerSelection } from '../../hooks/features/usePlayerSelection'
import { useLobbySubscription } from '../../hooks/useLobbySubscription'
import { lobbyQueryAtom } from '../../store/queries'
import { LobbyPlayerList } from './LobbyPlayerList'
import { MeCheckInButton } from './MeCheckInButton'

export const Lobby = () => {
  const result = useAtomValue(lobbyQueryAtom)
  // Subscription must be active so urql's cacheExchange sees `lobbyUpdated`
  // Events and invalidates `Query.currentGroup` (configured in lib/urql.ts).
  // No manual refetch needed.
  useLobbySubscription()

  const playerSelection = usePlayerSelection()

  const group = result?.data?.currentGroup ?? null
  const allPlayers = useMemo(() => group?.players ?? [], [group])
  const checkedInPlayers = useMemo(() => group?.lobby ?? [], [group])
  const checkedInPlayerIds = useMemo(() => checkedInPlayers.map((p) => p.id), [checkedInPlayers])

  const { checkIn } = useLobby(checkedInPlayerIds)

  const [addPlayerOpen, setAddPlayerOpen] = useState(false)

  if (!group) {
    return null
  }

  return (
    <Box p={{ base: 3, md: 4 }} bg="bg.panel" borderRadius="card" borderWidth="1px" borderColor="gray.200">
      <VStack align="stretch" gap={4}>
        <Box>
          <Heading size={{ base: 'md', md: 'lg' }}>Lobby</Heading>
          <Text fontSize="sm" color="gray.600" mt={1}>
            Players in the lobby are auto-selected when starting a new race.
          </Text>
        </Box>

        <MeCheckInButton
          groupId={group.id}
          allPlayers={allPlayers}
          checkedInPlayerIds={checkedInPlayerIds}
          isCreatingPlayer={playerSelection.isCreatingPlayer}
          onCreatePlayerByName={async (name) => {
            const created = await playerSelection.createAndSelectPlayerByName(name)
            if (created) {
              await checkIn(created.id)
            }
            return created
          }}
        />

        <LobbyPlayerList allPlayers={allPlayers} checkedInPlayers={checkedInPlayers} onAddNewPlayer={() => setAddPlayerOpen(true)} />

        {addPlayerOpen && (
          <Box p={3} borderWidth="1px" borderRadius="md" borderColor="gray.200">
            <AddPlayerInline
              isCreatingPlayer={playerSelection.isCreatingPlayer}
              onCreatePlayerByName={playerSelection.createAndSelectPlayerByName}
              onCreated={async (player) => {
                await checkIn(player.id)
                setAddPlayerOpen(false)
              }}
              onCancel={() => setAddPlayerOpen(false)}
            />
          </Box>
        )}
      </VStack>
    </Box>
  )
}

type AddPlayerInlineProps = {
  isCreatingPlayer: boolean
  onCreatePlayerByName: (name: string) => Promise<{ id: string; name: string } | null>
  onCreated: (player: { id: string; name: string }) => void | Promise<void>
  onCancel: () => void
}

const AddPlayerInline = ({ isCreatingPlayer, onCreatePlayerByName, onCreated, onCancel }: AddPlayerInlineProps) => {
  const [name, setName] = useState('')

  const handleCreate = async () => {
    const created = await onCreatePlayerByName(name)
    if (created) {
      await onCreated(created)
      setName('')
    }
  }

  return (
    <VStack align="stretch" gap={2}>
      <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="New player name" size="sm" />
      <HStack>
        <Button size="sm" onClick={handleCreate} disabled={!name.trim() || isCreatingPlayer}>
          Create & check in
        </Button>
        <Button size="sm" variant="outline" onClick={onCancel}>
          Cancel
        </Button>
      </HStack>
    </VStack>
  )
}
