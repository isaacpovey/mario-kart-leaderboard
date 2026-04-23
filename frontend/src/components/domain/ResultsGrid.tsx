import { Box, Button, Heading, HStack, SimpleGrid, Text, VStack } from '@chakra-ui/react'
import { useMemo, useState } from 'react'
import { LuArrowLeftRight } from 'react-icons/lu'

const TOTAL_SLOTS = 24
const SLOTS_PER_PAGE = 12

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

export type SlotAssignments = Record<number, string | null>

type ResultsGridProps = {
  round: Round
  slots: SlotAssignments
  onTogglePlayer: (slotNumber: number, playerId: string) => void
  error: string
  submitting: boolean
  onSubmit: (results: Array<{ playerId: string; position: number }>) => void | Promise<void>
  onSwapPlayer?: (player: Player) => void
}

type Page = 'top' | 'bottom'

const ordinal = (n: number): string => {
  const mod100 = n % 100
  if (mod100 >= 11 && mod100 <= 13) return `${n}th`
  const mod10 = n % 10
  if (mod10 === 1) return `${n}st`
  if (mod10 === 2) return `${n}nd`
  if (mod10 === 3) return `${n}rd`
  return `${n}th`
}

const pageSlots = (page: Page): number[] => {
  const start = page === 'top' ? 1 : SLOTS_PER_PAGE + 1
  const end = page === 'top' ? SLOTS_PER_PAGE : TOTAL_SLOTS
  return Array.from({ length: end - start + 1 }, (_, i) => i + start)
}

export const ResultsGrid = ({ round, slots, onTogglePlayer, error, submitting, onSubmit, onSwapPlayer }: ResultsGridProps) => {
  const [page, setPage] = useState<Page>('top')

  const assignedPlayerIds = useMemo(() => new Set(Object.values(slots).filter((v): v is string => v !== null)), [slots])
  const unassignedPlayers = useMemo(() => round.players.filter((p) => !assignedPlayerIds.has(p.id)), [round.players, assignedPlayerIds])

  const allAssigned = unassignedPlayers.length === 0
  const assignedCount = round.players.length - unassignedPlayers.length
  const validationHint = !allAssigned ? `Assign every player to a position before submitting. (${assignedCount}/${round.players.length} assigned)` : ''

  const handleSubmit = () => {
    const results = Object.entries(slots)
      .filter(([, playerId]) => playerId !== null)
      .map(([position, playerId]) => ({
        playerId: playerId as string,
        position: Number(position),
      }))
    onSubmit(results)
  }

  const renderSlot = (slotNumber: number) => {
    const assignedId = slots[slotNumber] ?? null
    const isAssigned = assignedId !== null
    return (
      <HStack
        key={slotNumber}
        gap={2}
        px={3}
        py={2}
        borderRadius="md"
        borderWidth="1px"
        borderColor={isAssigned ? 'brand.300' : 'gray.200'}
        bg={isAssigned ? 'brand.50' : slotNumber % 2 === 0 ? 'gray.50' : 'white'}
        align="start"
      >
        <Box minW="2.5rem" textAlign="right" fontWeight="bold" fontSize="sm" color={isAssigned ? 'brand.700' : 'gray.500'} flexShrink={0} pt={1}>
          {ordinal(slotNumber)}
        </Box>
        <SimpleGrid columns={{ base: 2, md: 4 }} gap={2} flex={1}>
          {round.players.map((player) => {
            const isHere = assignedId === player.id
            const isElsewhere = !isHere && assignedPlayerIds.has(player.id)
            return (
              <Button
                key={player.id}
                size="xs"
                variant={isHere ? 'solid' : 'outline'}
                colorScheme={isHere ? 'yellow' : undefined}
                bg={isHere ? 'brand.400' : undefined}
                color={isHere ? 'gray.900' : isElsewhere ? 'gray.400' : undefined}
                borderColor={isElsewhere ? 'gray.200' : undefined}
                opacity={isElsewhere ? 0.6 : 1}
                onClick={() => onTogglePlayer(slotNumber, player.id)}
                px={2}
                minW={0}
              >
                <Text truncate>{player.name}</Text>
              </Button>
            )
          })}
        </SimpleGrid>
      </HStack>
    )
  }

  return (
    <Box p={{ base: 4, md: 6 }} bg="bg.panel" borderRadius="card" borderWidth="1px" borderColor="brand.400" boxShadow="card-hover">
      <VStack gap={{ base: 3, md: 5 }} align="stretch">
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

        <HStack gap={2} justify="center">
          <Button
            size="sm"
            variant={page === 'top' ? 'solid' : 'outline'}
            colorScheme={page === 'top' ? 'yellow' : undefined}
            bg={page === 'top' ? 'brand.400' : undefined}
            color={page === 'top' ? 'gray.900' : undefined}
            onClick={() => setPage('top')}
          >
            1st – 12th
          </Button>
          <Button
            size="sm"
            variant={page === 'bottom' ? 'solid' : 'outline'}
            colorScheme={page === 'bottom' ? 'yellow' : undefined}
            bg={page === 'bottom' ? 'brand.400' : undefined}
            color={page === 'bottom' ? 'gray.900' : undefined}
            onClick={() => setPage('bottom')}
          >
            13th – 24th
          </Button>
        </HStack>

        <VStack gap={1} align="stretch">
          {pageSlots(page).map(renderSlot)}
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
    </Box>
  )
}
