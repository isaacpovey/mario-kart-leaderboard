import { Button, Center, Container, Heading, HStack, Spinner, Text, VStack, Input, Field } from '@chakra-ui/react'
import { useEffect, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { useMutation, useQuery } from 'urql'
import { matchQuery } from '../queries/match.query'
import { recordRoundResultsMutation } from '../queries/recordRoundResults.mutation'

const Match = () => {
  const { matchId } = useParams()
  const navigate = useNavigate()
  const [result, reexecuteQuery] = useQuery({
    query: matchQuery,
    variables: { matchId: matchId || '' },
    requestPolicy: 'cache-first'
  })
  const [, executeRecordRoundResults] = useMutation(recordRoundResultsMutation)

  const [selectedRound, setSelectedRound] = useState<number | null>(null)
  const [positions, setPositions] = useState<Record<string, string>>({})
  const [error, setError] = useState('')
  const [submitting, setSubmitting] = useState(false)

  useEffect(() => {
    document.title = 'Match Details'
  }, [])

  const { data, fetching, error: queryError } = result

  if (fetching) {
    return (
      <Center h="100vh">
        <Spinner size="xl" />
      </Center>
    )
  }

  if (queryError || !data?.matchById) {
    return (
      <Container maxW="4xl" py={8}>
        <Text color="red.500">Error loading match data: {queryError?.message || 'Match not found'}</Text>
        <Button mt={4} onClick={() => navigate('/')}>
          Back to Home
        </Button>
      </Container>
    )
  }

  const match = data.matchById

  const handleSelectRound = (roundNumber: number) => {
    setSelectedRound(roundNumber)
    setPositions({})
    setError('')
  }

  const handlePositionChange = (playerId: string, position: string) => {
    setPositions((prev) => ({ ...prev, [playerId]: position }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (selectedRound === null || !selectedRoundData) return

    setError('')
    setSubmitting(true)

    const roundPlayers = selectedRoundData.players
    const results = roundPlayers
      .map((player) => ({
        playerId: player.id,
        position: Number.parseInt(positions[player.id] || '0', 10),
      }))
      .filter((result) => result.position > 0)

    if (results.length === 0) {
      setError('Please enter at least one position')
      setSubmitting(false)
      return
    }

    const positionSet = new Set(results.map((r) => r.position))
    if (positionSet.size !== results.length) {
      setError('Each position must be unique')
      setSubmitting(false)
      return
    }

    const result = await executeRecordRoundResults({
      matchId: matchId || '',
      roundNumber: selectedRound,
      results,
    })

    setSubmitting(false)

    if (result.error) {
      setError(result.error.message)
      return
    }

    setSelectedRound(null)
    setPositions({})
    reexecuteQuery({ requestPolicy: 'network-only' })
  }

  const selectedRoundData = match.rounds.find((r) => r.roundNumber === selectedRound)

  return (
    <Container maxW="4xl" py={8}>
      <VStack gap={6} align="stretch">
        <HStack justify="space-between" flexWrap={{ base: 'wrap', sm: 'nowrap' }} gap={4}>
          <VStack align="start" gap={1}>
            <Heading size={{ base: 'xl', md: '2xl', lg: '3xl' }}>Match Details</Heading>
            <Text color="fg.subtle" fontSize={{ base: 'sm', md: 'md' }}>{new Date(match.time).toLocaleString()}</Text>
          </VStack>
          <Button onClick={() => navigate('/')} flexShrink={0}>Back to Home</Button>
        </HStack>

        <VStack gap={4} align="stretch">
          <Heading size="lg">Teams</Heading>
          {match.teams.map((team) => (
            <VStack key={team.id} p={{ base: 3, md: 4 }} bg="bg.panel" borderRadius="md" borderWidth="1px" align="stretch" gap={2}>
              <HStack justify="space-between">
                <Text fontWeight="bold" fontSize={{ base: 'md', md: 'lg' }} truncate>
                  {team.name}
                </Text>
                <Text fontWeight="semibold" fontSize={{ base: 'md', md: 'lg' }} flexShrink={0}>
                  {team.score} pts
                </Text>
              </HStack>
              <VStack align="stretch" gap={1}>
                {team.players.map((player) => (
                  <HStack key={player.id} justify="space-between" gap={2}>
                    <Text fontSize={{ base: 'sm', md: 'md' }} truncate>{player.name}</Text>
                    <Text fontSize="sm" color="fg.subtle" flexShrink={0}>
                      ELO: {player.eloRating}
                    </Text>
                  </HStack>
                ))}
              </VStack>
            </VStack>
          ))}
        </VStack>

        <VStack gap={4} align="stretch">
          <Heading size="lg">Races</Heading>
          <HStack flexWrap="wrap" gap={2}>
            {match.rounds.map((round) => (
              <Button
                key={round.roundNumber}
                onClick={() => handleSelectRound(round.roundNumber)}
                colorScheme={selectedRound === round.roundNumber ? 'blue' : round.completed ? 'green' : 'gray'}
                variant={selectedRound === round.roundNumber ? 'solid' : 'outline'}
                disabled={round.completed}
              >
                Race {round.roundNumber}{round.track ? ` - ${round.track.name}` : ''} {round.completed && '✓'}
              </Button>
            ))}
          </HStack>
        </VStack>

        {selectedRound !== null && selectedRoundData && !selectedRoundData.completed && (
          <VStack gap={4} align="stretch" p={6} bg="bg.panel" borderRadius="md" borderWidth="1px">
            <Heading size="md">
              Record Results - Race {selectedRound}{selectedRoundData.track ? ` - ${selectedRoundData.track.name}` : ''}
            </Heading>
            <form onSubmit={handleSubmit}>
              <VStack gap={4} align="stretch">
                {selectedRoundData.players.map((player) => (
                  <Field.Root key={player.id}>
                    <Field.Label>
                      {player.name} (ELO: {player.eloRating})
                    </Field.Label>
                    <Input
                      type="number"
                      min={1}
                      max={24}
                      placeholder="Position"
                      value={positions[player.id] || ''}
                      onChange={(e) => handlePositionChange(player.id, e.target.value)}
                    />
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
                  <Button
                    type="button"
                    variant="outline"
                    width="full"
                    onClick={() => {
                      setSelectedRound(null)
                      setPositions({})
                      setError('')
                    }}
                  >
                    Cancel
                  </Button>
                </HStack>
              </VStack>
            </form>
          </VStack>
        )}

        {match.completed && (
          <Text fontSize="lg" fontWeight="semibold" color="green.500" textAlign="center">
            Match Completed!
          </Text>
        )}
      </VStack>
    </Container>
  )
}

export default Match
