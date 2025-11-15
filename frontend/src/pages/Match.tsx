import { Badge, Box, Button, Container, Heading, HStack, Text, VStack } from '@chakra-ui/react'
import { useAtomValue } from 'jotai'
import { useEffect, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { ErrorState } from '../components/common/ErrorState'
import { RaceList } from '../components/domain/RaceList'
import { RoundResultsForm } from '../components/domain/RoundResultsForm'
import { TeamCard } from '../components/domain/TeamCard'
import { useMatchManagement } from '../hooks/features/useMatchManagement'
import { matchQueryAtom } from '../store/queries'

const Match = () => {
  const { matchId } = useParams()
  const navigate = useNavigate()
  const matchAtom = useMemo(() => matchQueryAtom(matchId || ''), [matchId])
  const matchResult = useAtomValue(matchAtom)
  const { recordResults, isRecordingResults } = useMatchManagement()

  const [selectedRound, setSelectedRound] = useState<number | null>(null)
  const [positions, setPositions] = useState<Record<string, string>>({})
  const [error, setError] = useState('')

  useEffect(() => {
    document.title = 'Match Details'
  }, [])

  if (matchResult.error || !matchResult.data?.matchById) {
    return <ErrorState message={`Error loading match data: ${matchResult.error?.message || 'Match not found'}`} />
  }

  const match = matchResult.data.matchById

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

    const roundPlayers = selectedRoundData.players
    const results = roundPlayers
      .map((player: { id: string }) => ({
        playerId: player.id,
        position: Number.parseInt(positions[player.id] || '0', 10),
      }))
      .filter((result: { position: number }) => result.position > 0)

    if (results.length === 0) {
      setError('Please enter at least one position')
      return
    }

    const positionSet = new Set(results.map((r: { position: number }) => r.position))
    if (positionSet.size !== results.length) {
      setError('Each position must be unique')
      return
    }

    const updated = await recordResults({
      matchId: matchId || '',
      roundNumber: selectedRound,
      results,
    })

    if (updated) {
      setSelectedRound(null)
      setPositions({})
    }
  }

  const selectedRoundData = match.rounds.find((r: { roundNumber: number }) => r.roundNumber === selectedRound)

  return (
    <Box minH="100vh" bg="bg.canvas">
      <Container maxW="4xl" py={{ base: 4, md: 6, lg: 8 }}>
        <VStack gap={{ base: 6, md: 8 }} align="stretch">
          <HStack justify="space-between" flexWrap="wrap" gap={{ base: 3, md: 4 }}>
            <VStack align="start" gap={1} flex={1}>
              <Heading size={{ base: 'lg', md: 'xl', lg: '2xl' }} color="gray.900">
                Match Details
              </Heading>
              <Text color="gray.600" fontSize={{ base: 'xs', md: 'sm' }}>
                {new Date(match.time).toLocaleString()}
              </Text>
            </VStack>
            <Button
              onClick={() => navigate('/')}
              variant="outline"
              size={{ base: 'sm', md: 'md' }}
              borderRadius="button"
              borderWidth="2px"
              flexShrink={0}
              _hover={{ bg: 'gray.50' }}
            >
              ← Back to Home
            </Button>
          </HStack>

          <VStack gap={{ base: 3, md: 4 }} align="stretch">
            <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
              Teams
            </Heading>
            <VStack gap={{ base: 3, md: 4 }} align="stretch">
              {match.teams.map((team) => (
                <TeamCard key={team.id} team={team} playerResults={match.playerResults} />
              ))}
            </VStack>
          </VStack>

          <Box h="1px" bg="gray.200" />

          <VStack gap={{ base: 3, md: 4 }} align="stretch">
            <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
              Races
            </Heading>
            <RaceList
              rounds={match.rounds}
              selectedRound={selectedRound}
              onSelectRound={handleSelectRound}
              renderFormForRound={(roundNumber) => {
                const roundData = match.rounds.find((r: { roundNumber: number }) => r.roundNumber === roundNumber)
                if (!roundData || roundData.completed) return null

                return (
                  <RoundResultsForm
                    round={roundData}
                    positions={positions}
                    onPositionChange={handlePositionChange}
                    onSubmit={handleSubmit}
                    error={error}
                    submitting={isRecordingResults}
                  />
                )
              }}
            />
          </VStack>

          {match.completed && (
            <Box p={6} bg="green.50" borderRadius="card" borderWidth="2px" borderColor="green.400" textAlign="center">
              <Badge colorScheme="green" fontSize={{ base: 'md', md: 'lg' }} px={4} py={2}>
                ✓ Match Completed!
              </Badge>
            </Box>
          )}
        </VStack>
      </Container>
    </Box>
  )
}

export default Match
