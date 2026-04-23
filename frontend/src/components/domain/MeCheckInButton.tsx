import { Button, HStack, Text, VStack } from '@chakra-ui/react'
import { useState } from 'react'
import { useLobby } from '../../hooks/features/useLobby'
import { useMe } from '../../hooks/features/useMe'
import { PlayerSelectorDrawer } from '../common/PlayerSelectorDrawer'

type Player = {
  id: string
  name: string
  avatarFilename?: string | null
  currentTournamentElo: number | null
}

type MeCheckInButtonProps = {
  groupId: string
  allPlayers: Player[]
  checkedInPlayerIds: string[]
  isCreatingPlayer: boolean
  onCreatePlayerByName: (name: string) => Promise<Player | null>
}

export const MeCheckInButton = (props: MeCheckInButtonProps) => {
  const { groupId, allPlayers, checkedInPlayerIds, isCreatingPlayer, onCreatePlayerByName } = props
  const { playerId: meId, setMe, clearMe } = useMe(groupId)
  const { toggle, isLoading } = useLobby(checkedInPlayerIds)
  const [pickerOpen, setPickerOpen] = useState(false)

  const me = meId ? (allPlayers.find((p) => p.id === meId) ?? null) : null
  const isCheckedIn = me !== null && checkedInPlayerIds.includes(me.id)

  const pickerDrawer = (
    <PlayerSelectorDrawer
      open={pickerOpen}
      onOpenChange={setPickerOpen}
      allPlayers={allPlayers}
      selectedPlayerIds={meId ? [meId] : []}
      onTogglePlayer={(playerId) => {
        setMe(playerId)
        setPickerOpen(false)
      }}
      isCreatingPlayer={isCreatingPlayer}
      onCreatePlayer={async (name) => {
        const created = await onCreatePlayerByName(name)
        if (created) {
          setMe(created.id)
          setPickerOpen(false)
        }
      }}
    />
  )

  if (!me) {
    return (
      <>
        <Button onClick={() => setPickerOpen(true)} colorScheme="blue" size="md" width="full" borderRadius="button">
          Choose your racer
        </Button>
        {pickerDrawer}
      </>
    )
  }

  return (
    <VStack align="stretch" gap={1}>
      <Button onClick={() => toggle(me.id)} colorScheme={isCheckedIn ? 'red' : 'green'} size="md" width="full" borderRadius="button" loading={isLoading}>
        {isCheckedIn ? `Check me out (${me.name})` : `Check me in as ${me.name}`}
      </Button>
      <HStack justify="flex-end">
        <Text fontSize="xs" color="gray.500">
          not {me.name}?{' '}
        </Text>
        <Text as="button" fontSize="xs" color="blue.600" onClick={clearMe} _hover={{ textDecoration: 'underline' }}>
          change
        </Text>
      </HStack>
      {pickerDrawer}
    </VStack>
  )
}
