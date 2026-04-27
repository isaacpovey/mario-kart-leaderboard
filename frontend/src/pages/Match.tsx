import { Badge, Box, Button, Container, Heading, Text, VStack } from '@chakra-ui/react'
import { useAtomValue } from 'jotai'
import { useEffect, useMemo, useState } from 'react'
import { LuCheck, LuFlag, LuHouse, LuUsers } from 'react-icons/lu'
import { useNavigate, useParams } from 'react-router'
import { CancelMatchModal } from '../components/CancelMatchModal'
import { ErrorState } from '../components/common/ErrorState'
import { BottomNav, type BottomNavItem } from '../components/domain/BottomNav'
import { RaceList } from '../components/domain/RaceList'
import { RaceResultsDisplay } from '../components/domain/RaceResultsDisplay'
import { RoundResultsForm } from '../components/domain/RoundResultsForm'
import { TeamCard } from '../components/domain/TeamCard'
import { SwapPlayerModal } from '../components/SwapPlayerModal'
import { useMatchManagement } from '../hooks/features/useMatchManagement'
import { useRaceResultsSubscription } from '../hooks/useRaceResultsSubscription'
import { matchQueryAtom } from '../store/queries'

const Match = () => {
  const { matchId } = useParams()
  const navigate = useNavigate()
  const matchAtom = useMemo(() => matchQueryAtom(matchId || ''), [matchId])
  const matchResult = useAtomValue(matchAtom)
  const { recordResults, isRecordingResults, swapRoundPlayer, isSwappingPlayer, swapPlayerError } = useMatchManagement()

  const [selectedRound, setSelectedRound] = useState<number | null>(null)
  const [expandedCompletedRound, setExpandedCompletedRound] = useState<number | null>(null)
  const [positions, setPositions] = useState<Record<string, string>>({})
  const [error, setError] = useState('')
  const [cancelModalOpen, setCancelModalOpen] = useState(false)
  const [swapModalOpen, setSwapModalOpen] = useState(false)
  const [playerToSwap, setPlayerToSwap] = useState<{ id: string; name: string; avatarFilename?: string | null; teamId: string } | null>(null)
  const [roundToSwap, setRoundToSwap] = useState<number | null>(null)

  // Subscribe to race result updates for this match's tournament (must be called before any returns)
  useRaceResultsSubscription(matchResult.data?.matchById?.tournamentId)

  useEffect(() => {
    document.title = 'Match Details'
  }, [])

  if (matchResult.error || !matchResult.data?.matchById) {
    return <ErrorState message={`Error loading match data: ${matchResult.error?.message || 'Match not found'}`} />
  }

  const match = matchResult.data.matchById

  const handleSelectRound = (roundNumber: number) => {
    setSelectedRound(roundNumber)
    setExpandedCompletedRound(null)
    setPositions({})
    setError('')
  }

  const handleToggleExpanded = (roundNumber: number) => {
    setExpandedCompletedRound((prev) => (prev === roundNumber ? null : roundNumber))
    setSelectedRound(null)
  }

  const handlePositionChange = (playerId: string, position: string) => {
    setPositions((prev) => ({ ...prev, [playerId]: position }))
  }

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    if (selectedRound === null || !selectedRoundData) return

    setError('')

    const results = selectedRoundData.players
      .map((player) => ({
        playerId: player.id,
        position: Number.parseInt(positions[player.id] || '0', 10),
      }))
      .filter((result) => result.position > 0)

    if (results.length === 0) {
      setError('Please enter at least one position')
      return
    }

    const positionSet = new Set(results.map((r) => r.position))
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

  const selectedRoundData = match.rounds.find((r) => r.roundNumber === selectedRound)
  const hasAnyResults = match.rounds.some((r) => r.completed)
  const canCancelMatch = !match.completed && !hasAnyResults

  const handleCancelSuccess = () => {
    navigate('/')
  }

  const handleSwapPlayer = (player: { id: string; name: string; avatarFilename?: string | null; teamId?: string | unknown }, roundNumber: number) => {
    if (!player.teamId) return
    const teamId = typeof player.teamId === 'string' ? player.teamId : String(player.teamId)
    setPlayerToSwap({ ...player, teamId })
    setRoundToSwap(roundNumber)
    setSwapModalOpen(true)
  }

  const handleSwapConfirm = async (newPlayerId: string) => {
    if (!playerToSwap || roundToSwap === null || !matchId) return
    const result = await swapRoundPlayer({
      matchId,
      roundNumber: roundToSwap,
      currentPlayerId: playerToSwap.id,
      newPlayerId,
    })
    if (result) {
      setSwapModalOpen(false)
      setPlayerToSwap(null)
      setRoundToSwap(null)
    }
  }

  const navItems: BottomNavItem[] = [
    { id: 'home', label: 'Home', icon: LuHouse, onClick: () => navigate('/'), dividerAfter: true },
    { id: 'teams', label: 'Teams', icon: LuUsers, targetId: 'teams-section' },
    { id: 'races', label: 'Races', icon: LuFlag, targetId: 'races-section' },
  ]

  return (
    <Box minH="100vh" bg="bg.canvas" pb={{ base: '80px', md: '88px' }}>
      <Container maxW="4xl" py={{ base: 4, md: 6, lg: 8 }}>
        <VStack gap={{ base: 6, md: 8 }} align="stretch">
          <VStack align="start" gap={1}>
            <Heading size={{ base: 'lg', md: 'xl', lg: '2xl' }} color="gray.900">
              Match Details
            </Heading>
            <Text color="gray.600" fontSize={{ base: 'xs', md: 'sm' }}>
              {new Date(match.time).toLocaleString()}
            </Text>
          </VStack>

          <VStack id="teams-section" scrollMarginTop={{ base: 4, md: 6 }} gap={{ base: 3, md: 4 }} align="stretch">
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

          <VStack id="races-section" scrollMarginTop={{ base: 4, md: 6 }} gap={{ base: 3, md: 4 }} align="stretch">
            <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
              Races
            </Heading>
            <RaceList
              rounds={match.rounds}
              selectedRound={selectedRound}
              expandedCompletedRound={expandedCompletedRound}
              onSelectRound={handleSelectRound}
              onToggleExpanded={handleToggleExpanded}
              renderFormForRound={(roundNumber) => {
                const roundData = match.rounds.find((r) => r.roundNumber === roundNumber)
                if (!roundData) return null

                if (roundData.completed && roundData.results) {
                  return <RaceResultsDisplay results={roundData.results} trackName={roundData.track?.name} />
                }

                return (
                  <RoundResultsForm
                    round={roundData}
                    positions={positions}
                    onPositionChange={handlePositionChange}
                    onSubmit={handleSubmit}
                    error={error}
                    submitting={isRecordingResults}
                    onSwapPlayer={(player) => handleSwapPlayer(player, roundNumber)}
                  />
                )
              }}
            />
          </VStack>

          {match.completed && (
            <Box p={6} bg="green.50" borderRadius="card" borderWidth="2px" borderColor="green.400" textAlign="center">
              <Badge colorScheme="green" fontSize={{ base: 'md', md: 'lg' }} px={4} py={2} display="inline-flex" alignItems="center" gap={2}>
                <LuCheck size={18} />
                Match Completed!
              </Badge>
            </Box>
          )}

          {canCancelMatch && (
            <Box pt={4} borderTopWidth="1px" borderColor="gray.200">
              <Button colorScheme="red" variant="outline" size={{ base: 'sm', md: 'md' }} onClick={() => setCancelModalOpen(true)}>
                Cancel Match
              </Button>
            </Box>
          )}
        </VStack>
      </Container>

      <BottomNav items={navItems} />

      <CancelMatchModal open={cancelModalOpen} onOpenChange={setCancelModalOpen} matchId={matchId || ''} onSuccess={handleCancelSuccess} />

      {playerToSwap && roundToSwap !== null && (
        <SwapPlayerModal
          open={swapModalOpen}
          onOpenChange={setSwapModalOpen}
          currentPlayer={playerToSwap}
          roundNumber={roundToSwap}
          teams={match.teams}
          roundPlayerIds={match.rounds.find((r) => r.roundNumber === roundToSwap)?.players.map((p) => p.id) ?? []}
          onSwap={handleSwapConfirm}
          isSwapping={isSwappingPlayer}
          error={swapPlayerError}
        />
      )}
    </Box>
  )
}

export default Match
