import { Button, HStack } from '@chakra-ui/react'

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

export const RoundSelector = ({ rounds, selectedRound, onSelectRound }: RoundSelectorProps) => (
  <HStack flexWrap="wrap" gap={2}>
    {rounds.map((round) => (
      <Button
        key={round.roundNumber}
        onClick={() => onSelectRound(round.roundNumber)}
        colorScheme={selectedRound === round.roundNumber ? 'blue' : round.completed ? 'green' : 'gray'}
        variant={selectedRound === round.roundNumber ? 'solid' : 'outline'}
        disabled={round.completed}
      >
        Race {round.roundNumber}
        {round.track ? ` - ${round.track.name}` : ''} {round.completed && '✓'}
      </Button>
    ))}
  </HStack>
)
