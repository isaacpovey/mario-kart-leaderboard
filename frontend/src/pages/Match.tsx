import { Button, Container, Heading, HStack, Text, VStack } from '@chakra-ui/react'
import { useAtomValue } from 'jotai'
import { useEffect, useMemo, useState } from 'react'
import { useNavigate, useParams } from 'react-router'
import { ErrorState } from '../components/common/ErrorState'
import { RoundResultsForm } from '../components/domain/RoundResultsForm'
import { RoundSelector } from '../components/domain/RoundSelector'
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
    <Container maxW="4xl" py={8}>
      <VStack gap={6} align="stretch">
        <HStack justify="space-between" flexWrap={{ base: 'wrap', sm: 'nowrap' }} gap={4}>
          <VStack align="start" gap={1}>
            <Heading size={{ base: 'xl', md: '2xl', lg: '3xl' }}>Match Details</Heading>
            <Text color="fg.subtle" fontSize={{ base: 'sm', md: 'md' }}>
              {new Date(match.time).toLocaleString()}
            </Text>
          </VStack>
          <Button onClick={() => navigate('/')} flexShrink={0}>
            Back to Home
          </Button>
        </HStack>

        <VStack gap={4} align="stretch">
          <Heading size="lg">Teams</Heading>
          {match.teams.map((team) => (
            <TeamCard key={team.id} team={team} />
          ))}
        </VStack>

        <VStack gap={4} align="stretch">
          <Heading size="lg">Races</Heading>
          <RoundSelector rounds={match.rounds} selectedRound={selectedRound} onSelectRound={handleSelectRound} />
        </VStack>

        {selectedRound !== null && selectedRoundData && !selectedRoundData.completed && (
          <RoundResultsForm
            round={selectedRoundData}
            positions={positions}
            onPositionChange={handlePositionChange}
            onSubmit={handleSubmit}
            onCancel={() => {
              setSelectedRound(null)
              setPositions({})
              setError('')
            }}
            error={error}
            submitting={isRecordingResults}
          />
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
