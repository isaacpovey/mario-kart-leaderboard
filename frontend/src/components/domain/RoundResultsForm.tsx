import { Box, Button, Field, Heading, HStack, Input, Text, VStack } from '@chakra-ui/react'
import type { FormEvent } from 'react'
import { Avatar } from '../common/Avatar'

type Player = {
  id: string
  name: string
  currentTournamentElo: number | null
  avatarFilename?: string | null
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
  error: string
  submitting: boolean
}

export const RoundResultsForm = ({ round, positions, onPositionChange, onSubmit, error, submitting }: RoundResultsFormProps) => (
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

      <form onSubmit={onSubmit}>
        <VStack gap={{ base: 3, md: 4 }} align="stretch">
          {round.players.map((player) => (
            <Field.Root key={player.id}>
              <Field.Label fontSize={{ base: 'sm', md: 'md' }} fontWeight="medium" mb={2}>
                <HStack gap={2}>
                  <Avatar name={player.name} avatarFilename={player.avatarFilename} size="sm" />
                  <Text>{player.name}</Text>
                </HStack>
              </Field.Label>
              <Input
                type="number"
                min={1}
                max={24}
                placeholder="Enter position (1-24)"
                value={positions[player.id] || ''}
                onChange={(e) => onPositionChange(player.id, e.target.value)}
                size={{ base: 'md', md: 'lg' }}
                borderRadius="button"
                borderWidth="2px"
                _focus={{ borderColor: 'brand.400', boxShadow: '0 0 0 1px var(--chakra-colors-brand-400)' }}
              />
            </Field.Root>
          ))}

          {error && (
            <Box p={3} bg="red.50" borderRadius="button" borderWidth="1px" borderColor="red.300">
              <Text color="red.700" fontSize="sm" fontWeight="medium">
                {error}
              </Text>
            </Box>
          )}

          <Button
            type="submit"
            colorScheme="yellow"
            bg="brand.400"
            color="gray.900"
            width="full"
            size={{ base: 'md', md: 'lg' }}
            borderRadius="button"
            fontWeight="bold"
            loading={submitting}
            _hover={{ bg: 'brand.500', transform: 'translateY(-2px)' }}
            transition="all 0.2s"
            mt={2}
          >
            {submitting ? 'Submitting...' : 'Submit Results'}
          </Button>
        </VStack>
      </form>
    </VStack>
  </Box>
)
