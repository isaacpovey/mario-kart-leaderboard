import { Box, Button, Heading, HStack, Input, VStack } from '@chakra-ui/react'
import { useEffect, useMemo, useState } from 'react'
import { useClient, useQuery } from 'urql'
import { useLobby } from '../../hooks/features/useLobby'
import { usePlayerSelection } from '../../hooks/features/usePlayerSelection'
import { useLobbySubscription } from '../../hooks/useLobbySubscription'
import { lobbyQuery } from '../../queries/lobby.query'
import { LobbyPlayerList } from './LobbyPlayerList'
import { MeCheckInButton } from './MeCheckInButton'

export const Lobby = () => {
  const client = useClient()
  const [result] = useQuery({ query: lobbyQuery })
  const subscriptionResult = useLobbySubscription(true)

  // Refetch lobbyQuery whenever a subscription event arrives
  useEffect(() => {
    if (subscriptionResult.data || subscriptionResult.error) {
      client.query(lobbyQuery, {}, { requestPolicy: 'network-only' }).toPromise()
    }
  }, [subscriptionResult.data, subscriptionResult.error, client])

  const playerSelection = usePlayerSelection()

  const group = result.data?.currentGroup ?? null
  const allPlayers = useMemo(() => group?.players ?? [], [group])
  const checkedInPlayers = useMemo(() => group?.lobby ?? [], [group])
  const checkedInPlayerIds = useMemo(() => checkedInPlayers.map((p) => p.id), [checkedInPlayers])

  const { checkIn } = useLobby(checkedInPlayerIds)

  const [addPlayerOpen, setAddPlayerOpen] = useState(false)

  if (!group) return null

  return (
    <Box p={{ base: 3, md: 4 }} bg="bg.panel" borderRadius="card" borderWidth="1px" borderColor="gray.200">
      <VStack align="stretch" gap={4}>
        <Heading size={{ base: 'md', md: 'lg' }}>Lobby</Heading>

        <MeCheckInButton
          groupId={group.id}
          allPlayers={allPlayers}
          checkedInPlayerIds={checkedInPlayerIds}
          isCreatingPlayer={playerSelection.isCreatingPlayer}
          onCreatePlayerByName={async (name) => {
            const created = await playerSelection.createAndSelectPlayerByName(name)
            if (created) {
              await checkIn(created.id)
              client.query(lobbyQuery, {}, { requestPolicy: 'network-only' }).toPromise()
            }
            return created
          }}
        />

        <LobbyPlayerList allPlayers={allPlayers} checkedInPlayers={checkedInPlayers} onAddNewPlayer={() => setAddPlayerOpen(true)} />

        {addPlayerOpen && (
          <Box p={3} borderWidth="1px" borderRadius="md" borderColor="gray.200">
            <AddPlayerInline
              onCreated={async (player) => {
                await checkIn(player.id)
                client.query(lobbyQuery, {}, { requestPolicy: 'network-only' }).toPromise()
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
  onCreated: (player: { id: string; name: string }) => void | Promise<void>
  onCancel: () => void
}

const AddPlayerInline = ({ onCreated, onCancel }: AddPlayerInlineProps) => {
  const playerSelection = usePlayerSelection()
  const [name, setName] = useState('')

  const handleCreate = async () => {
    const created = await playerSelection.createAndSelectPlayerByName(name)
    if (created) {
      await onCreated(created)
      setName('')
    }
  }

  return (
    <VStack align="stretch" gap={2}>
      <Input value={name} onChange={(e) => setName(e.target.value)} placeholder="New player name" size="sm" />
      <HStack>
        <Button size="sm" onClick={handleCreate} disabled={!name.trim() || playerSelection.isCreatingPlayer}>
          Create & check in
        </Button>
        <Button size="sm" variant="outline" onClick={onCancel}>
          Cancel
        </Button>
      </HStack>
    </VStack>
  )
}
