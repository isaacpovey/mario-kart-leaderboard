import { Button, Field, Heading, HStack, Input, Text, VStack } from '@chakra-ui/react'
import type { FormEvent } from 'react'

type Player = {
  id: string
  name: string
  eloRating: number
}

type Round = {
  roundNumber: number
  track?: {
    id: string
    name: string
  } | null
  players: Player[]
}

type RoundResultsFormProps = {
  round: Round
  positions: Record<string, string>
  onPositionChange: (playerId: string, position: string) => void
  onSubmit: (e: FormEvent) => void | Promise<void>
  onCancel: () => void
  error: string
  submitting: boolean
}

export const RoundResultsForm = ({ round, positions, onPositionChange, onSubmit, onCancel, error, submitting }: RoundResultsFormProps) => (
  <VStack gap={4} align="stretch" p={6} bg="bg.panel" borderRadius="md" borderWidth="1px">
    <Heading size="md">
      Record Results - Race {round.roundNumber}
      {round.track ? ` - ${round.track.name}` : ''}
    </Heading>
    <form onSubmit={onSubmit}>
      <VStack gap={4} align="stretch">
        {round.players.map((player) => (
          <Field.Root key={player.id}>
            <Field.Label>
              {player.name} (ELO: {player.eloRating})
            </Field.Label>
            <Input type="number" min={1} max={24} placeholder="Position" value={positions[player.id] || ''} onChange={(e) => onPositionChange(player.id, e.target.value)} />
          </Field.Root>
        ))}

        {error && (
          <Text color="red.500" fontSize="sm">
            {error}
          </Text>
        )}

        <HStack gap={2}>
          <Button type="submit" colorScheme="blue" width="full" loading={submitting}>
            {submitting ? 'Submitting...' : 'Submit Results'}
          </Button>
          <Button type="button" variant="outline" width="full" onClick={onCancel}>
            Cancel
          </Button>
        </HStack>
      </VStack>
    </form>
  </VStack>
)
