import { Button, HStack, Text, VStack } from '@chakra-ui/react'

type Round = {
  roundNumber: number
  completed: boolean
  track?: {
    id: string
    name: string
  } | null
}

type RoundSelectorProps = {
  rounds: Round[]
  selectedRound: number | null
  onSelectRound: (roundNumber: number) => void
}

type RoundButtonProps = {
  round: Round
  selectedRound: number | null
  onSelectRound: (roundNumber: number) => void
}

const RoundButton = ({ round, selectedRound, onSelectRound }: RoundButtonProps) => {
  const isSelected = selectedRound === round.roundNumber

  return (
    <Button
      onClick={() => onSelectRound(round.roundNumber)}
      colorScheme={isSelected ? 'yellow' : round.completed ? 'green' : 'gray'}
      variant={isSelected ? 'solid' : 'outline'}
      size={{ base: 'md', md: 'lg' }}
      minH={{ base: '60px', md: '70px' }}
      px={{ base: 4, md: 5 }}
      borderRadius="button"
      borderWidth="2px"
      disabled={round.completed}
      _hover={{
        transform: round.completed ? 'none' : 'translateY(-2px)',
        boxShadow: round.completed ? 'none' : 'md',
      }}
      transition="all 0.2s"
      bg={isSelected ? 'brand.400' : round.completed ? 'green.50' : 'white'}
      color={isSelected ? 'gray.900' : round.completed ? 'green.700' : 'gray.700'}
      _disabled={{
        opacity: 0.7,
        cursor: 'not-allowed',
      }}
    >
      <VStack gap={0}>
        <Text fontWeight="bold" fontSize={{ base: 'sm', md: 'md' }}>
          Race {round.roundNumber}
        </Text>
        {round.track && (
          <Text fontSize={{ base: 'xs', md: 'sm' }} fontWeight="normal">
            {round.track.name}
          </Text>
        )}
        {round.completed && (
          <Text fontSize={{ base: 'xs', md: 'sm' }} fontWeight="semibold">
            ✓ Done
          </Text>
        )}
      </VStack>
    </Button>
  )
}

export const RoundSelector = ({ rounds, selectedRound, onSelectRound }: RoundSelectorProps) => (
  <HStack flexWrap="wrap" gap={{ base: 2, md: 3 }}>
    {rounds.map((round) => (
      <RoundButton key={round.roundNumber} round={round} selectedRound={selectedRound} onSelectRound={onSelectRound} />
    ))}
  </HStack>
)
