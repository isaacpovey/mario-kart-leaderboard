import { Badge, Box, HStack, Text, VStack } from '@chakra-ui/react'
import type { ReactNode } from 'react'

type Round = {
  roundNumber: number
  completed: boolean
  track?: {
    id: string
    name: string
  } | null
}

type RaceListProps = {
  rounds: Round[]
  selectedRound: number | null
  onSelectRound: (roundNumber: number) => void
  renderFormForRound?: (roundNumber: number) => ReactNode
}

type RaceCardProps = {
  round: Round
  selectedRound: number | null
  onSelectRound: (roundNumber: number) => void
  renderFormForRound?: (roundNumber: number) => ReactNode
}

const getBadgeColorScheme = (completed: boolean, isSelected: boolean): string => {
  if (completed) return 'green'
  if (isSelected) return 'yellow'
  return 'gray'
}

const getBadgeText = (completed: boolean, isSelected: boolean): string => {
  if (completed) return '✓ Completed'
  if (isSelected) return 'Selected'
  return 'Pending'
}

const RaceCard = ({ round, selectedRound, onSelectRound, renderFormForRound }: RaceCardProps) => {
  const isSelected = selectedRound === round.roundNumber

  return (
    <Box>
      <Box
        p={{ base: 3, md: 4 }}
        bg={isSelected ? 'brand.50' : 'bg.panel'}
        borderRadius="button"
        borderWidth={isSelected ? '2px' : '1px'}
        borderColor={isSelected ? 'brand.400' : 'gray.200'}
        boxShadow={isSelected ? 'card-hover' : 'card'}
        cursor={round.completed ? 'default' : 'pointer'}
        onClick={() => !round.completed && onSelectRound(round.roundNumber)}
        _hover={
          round.completed
            ? {}
            : {
                boxShadow: 'card-hover',
                transform: 'translateX(4px)',
                borderColor: 'brand.400',
              }
        }
        transition="all 0.2s"
        opacity={round.completed ? 0.7 : 1}
      >
        <HStack justify="space-between" gap={4}>
          <HStack gap={{ base: 3, md: 4 }} flex={1}>
            <Text fontWeight="bold" fontSize={{ base: 'md', md: 'lg' }} color={isSelected ? 'brand.600' : 'gray.900'} minW="80px">
              Race {round.roundNumber}
            </Text>
            {round.track && (
              <Text fontSize={{ base: 'sm', md: 'md' }} color="gray.600" truncate>
                {round.track.name}
              </Text>
            )}
          </HStack>

          <Badge colorScheme={getBadgeColorScheme(round.completed, isSelected)} fontSize={{ base: 'xs', md: 'sm' }} px={3} py={1}>
            {getBadgeText(round.completed, isSelected)}
          </Badge>
        </HStack>
      </Box>

      {isSelected && renderFormForRound && <Box mt={{ base: 2, md: 3 }}>{renderFormForRound(round.roundNumber)}</Box>}
    </Box>
  )
}

export const RaceList = ({ rounds, selectedRound, onSelectRound, renderFormForRound }: RaceListProps) => (
  <VStack gap={{ base: 2, md: 3 }} align="stretch">
    {rounds.map((round) => (
      <RaceCard key={round.roundNumber} round={round} selectedRound={selectedRound} onSelectRound={onSelectRound} renderFormForRound={renderFormForRound} />
    ))}
  </VStack>
)
