import { Badge, Box, HStack, Text, VStack } from '@chakra-ui/react'
import type { ReactNode } from 'react'
import { LuCheck, LuClock, LuCrown, LuPencil } from 'react-icons/lu'
import { Avatar } from '../common/Avatar'

type Player = {
  id: string
  name: string
  currentTournamentElo: number | null
  avatarFilename?: string | null
}

type RaceResult = {
  player: Player
  position: number
  tournamentEloChange: number | null
}

type Round = {
  roundNumber: number
  completed: boolean
  track?: {
    id: string
    name: string
  } | null
  players?: Player[]
  results?: RaceResult[]
}

type RaceListProps = {
  rounds: Round[]
  selectedRound: number | null
  expandedCompletedRound: number | null
  onSelectRound: (roundNumber: number) => void
  onToggleExpanded: (roundNumber: number) => void
  renderFormForRound?: (roundNumber: number) => ReactNode
}

type RaceCardProps = {
  round: Round
  selectedRound: number | null
  expandedCompletedRound: number | null
  onSelectRound: (roundNumber: number) => void
  onToggleExpanded: (roundNumber: number) => void
  renderFormForRound?: (roundNumber: number) => ReactNode
}

const getBadgeIcon = (completed: boolean, isSelected: boolean) => {
  if (completed) return LuCheck
  if (isSelected) return LuPencil
  return LuClock
}

const getBadgeColorScheme = (completed: boolean, isSelected: boolean): string => {
  if (completed) return 'green'
  if (isSelected) return 'yellow'
  return 'gray'
}

const PendingRacePlayers = ({ players }: { players: Player[] }) => (
  <HStack gap={1}>
    {players.map((player) => (
      <Avatar key={player.id} name={player.name} avatarFilename={player.avatarFilename} size="sm" />
    ))}
  </HStack>
)

const getTopFinisher = (results?: RaceResult[]): RaceResult | undefined => {
  if (!results || results.length === 0) return undefined
  const sorted = [...results].sort((a, b) => a.position - b.position)
  return sorted[0]
}

const getFinisherBorderColor = (position: number): string => {
  if (position === 1) return 'yellow.400'
  if (position === 2) return 'gray.400'
  if (position === 3) return 'orange.400'
  return 'gray.300'
}

const getCrownColor = (position: number): string => {
  if (position === 1) return 'yellow.500'
  if (position === 2) return 'gray.500'
  if (position === 3) return 'orange.500'
  return 'gray.500'
}

const CompletedRaceTopFinisher = ({ finisher }: { finisher: RaceResult }) => {
  const borderColor = getFinisherBorderColor(finisher.position)
  const crownColor = getCrownColor(finisher.position)
  const isNumberPosition = finisher.position > 3

  return (
    <Box position="relative">
      <Box borderWidth="2px" borderColor={borderColor} borderRadius="full" p="10px" position="relative">
        <Avatar name={finisher.player.name} avatarFilename={finisher.player.avatarFilename} size="sm" />
        {isNumberPosition ? (
          <Box
            position="absolute"
            top="-10px"
            left="50%"
            transform="translateX(-50%)"
            bg="gray.400"
            borderRadius="full"
            w="20px"
            h="20px"
            display="flex"
            alignItems="center"
            justifyContent="center"
            zIndex={1}
          >
            <Text fontSize="xs" fontWeight="bold" color="white">
              {finisher.position}
            </Text>
          </Box>
        ) : (
          <Box position="absolute" top="-10px" left="50%" transform="translateX(-50%)" zIndex={1} color={crownColor}>
            <LuCrown size={16} fill="currentColor" />
          </Box>
        )}
      </Box>
    </Box>
  )
}

const RaceCard = ({ round, selectedRound, expandedCompletedRound, onSelectRound, onToggleExpanded, renderFormForRound }: RaceCardProps) => {
  const isSelected = selectedRound === round.roundNumber
  const isExpanded = expandedCompletedRound === round.roundNumber
  const topFinisher = getTopFinisher(round.results)

  const handleClick = () => {
    if (round.completed) {
      onToggleExpanded(round.roundNumber)
    } else {
      onSelectRound(round.roundNumber)
    }
  }

  return (
    <Box>
      <Box
        p={{ base: 3, md: 4 }}
        bg={isSelected || isExpanded ? 'brand.50' : 'bg.panel'}
        borderRadius="button"
        borderWidth={isSelected || isExpanded ? '2px' : '1px'}
        borderColor={isSelected || isExpanded ? 'brand.400' : 'gray.200'}
        boxShadow={isSelected || isExpanded ? 'card-hover' : 'card'}
        cursor="pointer"
        onClick={handleClick}
        _hover={{
          boxShadow: 'card-hover',
          transform: 'translateX(4px)',
          borderColor: 'brand.400',
        }}
        transition="all 0.2s"
      >
        <HStack justify="space-between" gap={4} align="center">
          <HStack gap={{ base: 3, md: 4 }} flex={1} minW={0}>
            <Text fontWeight="bold" fontSize={{ base: 'md', md: 'lg' }} color={isSelected || isExpanded ? 'brand.600' : 'gray.900'} flexShrink={0}>
              {round.roundNumber}
            </Text>
            {round.track && (
              <Text fontSize={{ base: 'sm', md: 'md' }} color="gray.600" truncate>
                {round.track.name}
              </Text>
            )}
          </HStack>

          <HStack gap={3} flexShrink={0}>
            {!round.completed && round.players && round.players.length > 0 && <PendingRacePlayers players={round.players} />}

            {round.completed && topFinisher && <CompletedRaceTopFinisher finisher={topFinisher} />}

            <Badge colorScheme={getBadgeColorScheme(round.completed, isSelected)} fontSize={{ base: 'md', md: 'lg' }} px={2} py={1} display="flex" alignItems="center">
              <Box as={getBadgeIcon(round.completed, isSelected)} boxSize={{ base: 4, md: 5 }} />
            </Badge>
          </HStack>
        </HStack>
      </Box>

      {isSelected && !round.completed && renderFormForRound && <Box mt={{ base: 2, md: 3 }}>{renderFormForRound(round.roundNumber)}</Box>}
      {isExpanded && round.completed && renderFormForRound && <Box mt={{ base: 2, md: 3 }}>{renderFormForRound(round.roundNumber)}</Box>}
    </Box>
  )
}

export const RaceList = ({ rounds, selectedRound, expandedCompletedRound, onSelectRound, onToggleExpanded, renderFormForRound }: RaceListProps) => (
  <VStack gap={{ base: 2, md: 3 }} align="stretch">
    {rounds.map((round) => (
      <RaceCard
        key={round.roundNumber}
        round={round}
        selectedRound={selectedRound}
        expandedCompletedRound={expandedCompletedRound}
        onSelectRound={onSelectRound}
        onToggleExpanded={onToggleExpanded}
        renderFormForRound={renderFormForRound}
      />
    ))}
  </VStack>
)
