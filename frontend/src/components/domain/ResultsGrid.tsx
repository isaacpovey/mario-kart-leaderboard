import { Box, Button, Heading, HStack, Text, VStack } from '@chakra-ui/react'
import { useMemo, useState } from 'react'
import { LuArrowLeftRight } from 'react-icons/lu'
import { Avatar } from '../common/Avatar'
import { GridSlotPicker } from './GridSlotPicker'

const TOTAL_SLOTS = 24

type Player = {
  id: string
  name: string
  avatarFilename?: string | null
  teamId?: string | unknown
}

type Round = {
  roundNumber: number
  track?: {
    id: string
    name: string
  } | null
  players: Player[]
}

type ResultsGridProps = {
  round: Round
  error: string
  submitting: boolean
  onSubmit: (results: Array<{ playerId: string; position: number }>) => void | Promise<void>
  onSwapPlayer?: (player: Player) => void
}

const ordinal = (n: number): string => {
  const mod100 = n % 100
  if (mod100 >= 11 && mod100 <= 13) return `${n}th`
  const mod10 = n % 10
  if (mod10 === 1) return `${n}st`
  if (mod10 === 2) return `${n}nd`
  if (mod10 === 3) return `${n}rd`
  return `${n}th`
}

export const ResultsGrid = ({ round, error, submitting, onSubmit, onSwapPlayer }: ResultsGridProps) => {
  const [slots, setSlots] = useState<Record<number, string | null>>({})
  const [pickerSlot, setPickerSlot] = useState<number | null>(null)

  const assignedPlayerIds = useMemo(() => new Set(Object.values(slots).filter((v): v is string => v !== null)), [slots])
  const unassignedPlayers = useMemo(() => round.players.filter((p) => !assignedPlayerIds.has(p.id)), [round.players, assignedPlayerIds])
  const playersById = useMemo(() => new Map(round.players.map((p) => [p.id, p])), [round.players])

  const currentSlotAssignment = pickerSlot !== null && slots[pickerSlot] ? (playersById.get(slots[pickerSlot] as string) ?? null) : null

  const allAssigned = unassignedPlayers.length === 0

  const assignedCount = round.players.length - unassignedPlayers.length
  const validationHint = !allAssigned ? `Assign every player to a position before submitting. (${assignedCount}/${round.players.length} assigned)` : ''

  const handleAssign = (playerId: string | null) => {
    if (pickerSlot === null) return
    setSlots((prev) => {
      const next = { ...prev }
      // Clear any other slot that previously held this player — a player can only be in one slot at a time.
      if (playerId !== null) {
        for (const [slotKey, existingId] of Object.entries(next)) {
          if (existingId === playerId && Number(slotKey) !== pickerSlot) {
            next[Number(slotKey)] = null
          }
        }
      }
      next[pickerSlot] = playerId
      return next
    })
  }

  const handleSubmit = () => {
    const results = Object.entries(slots)
      .filter(([, playerId]) => playerId !== null)
      .map(([position, playerId]) => ({
        playerId: playerId as string,
        position: Number(position),
      }))
    onSubmit(results)
  }

  return (
    <Box p={{ base: 5, md: 6 }} bg="bg.panel" borderRadius="card" borderWidth="1px" borderColor="brand.400" boxShadow="card-hover">
      <VStack gap={{ base: 4, md: 5 }} align="stretch">
        <VStack gap={1} align="start">
          <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
            Record Results
          </Heading>
          <Text fontSize={{ base: 'sm', md: 'md' }} color="gray.600">
            Race {round.roundNumber}
            {round.track ? ` - ${round.track.name}` : ''}
          </Text>
        </VStack>

        {onSwapPlayer && round.players.length > 0 && (
          <HStack flexWrap="wrap" gap={2} align="center">
            <Text fontSize="sm" color="gray.600">
              Swap:
            </Text>
            {round.players.map((player) => (
              <Button key={player.id} size="xs" variant="outline" onClick={() => onSwapPlayer(player)}>
                <HStack gap={1}>
                  <LuArrowLeftRight />
                  <Text>{player.name}</Text>
                </HStack>
              </Button>
            ))}
          </HStack>
        )}

        <VStack align="stretch" gap={1} maxH={{ base: '50vh', md: '60vh' }} overflowY="auto" borderWidth="1px" borderColor="gray.200" borderRadius="md" p={2}>
          {Array.from({ length: TOTAL_SLOTS }, (_, i) => i + 1).map((slotNumber) => {
            const playerId = slots[slotNumber] ?? null
            const player = playerId ? (playersById.get(playerId) ?? null) : null
            const isAssigned = player !== null
            return (
              <HStack
                key={slotNumber}
                as="button"
                onClick={() => setPickerSlot(slotNumber)}
                justify="space-between"
                px={3}
                py={2}
                borderRadius="md"
                borderWidth="1px"
                borderColor={isAssigned ? 'brand.300' : 'gray.200'}
                bg={isAssigned ? 'brand.50' : 'white'}
                _hover={{ bg: isAssigned ? 'brand.100' : 'gray.50' }}
                textAlign="left"
              >
                <HStack gap={3} flex={1}>
                  <Box minW="2.5rem" textAlign="center" fontWeight="bold" fontSize="md" color={isAssigned ? 'brand.700' : 'gray.600'}>
                    {ordinal(slotNumber)}
                  </Box>
                  {isAssigned && player ? (
                    <HStack gap={2}>
                      <Avatar name={player.name} avatarFilename={player.avatarFilename} size="sm" />
                      <Text fontWeight="medium">{player.name}</Text>
                    </HStack>
                  ) : (
                    <Text color="gray.500">CPU</Text>
                  )}
                </HStack>
              </HStack>
            )
          })}
        </VStack>

        {(error || validationHint) && (
          <Box p={3} bg="red.50" borderRadius="button" borderWidth="1px" borderColor="red.300">
            <Text color="red.700" fontSize="sm" fontWeight="medium">
              {error || validationHint}
            </Text>
          </Box>
        )}

        <Button
          onClick={handleSubmit}
          colorScheme="yellow"
          bg="brand.400"
          color="gray.900"
          width="full"
          size={{ base: 'md', md: 'lg' }}
          borderRadius="button"
          fontWeight="bold"
          loading={submitting}
          disabled={!allAssigned || submitting}
          _hover={{ bg: 'brand.500', transform: 'translateY(-2px)' }}
          transition="all 0.2s"
        >
          {submitting ? 'Submitting...' : 'Submit Results'}
        </Button>
      </VStack>

      {pickerSlot !== null && (
        <GridSlotPicker
          open={pickerSlot !== null}
          onOpenChange={(open) => {
            if (!open) setPickerSlot(null)
          }}
          slotNumber={pickerSlot}
          currentAssignment={currentSlotAssignment}
          candidates={unassignedPlayers}
          onAssign={handleAssign}
        />
      )}
    </Box>
  )
}
