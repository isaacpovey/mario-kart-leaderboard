import { Badge, Box, Button, Container, Heading, Text, VStack } from '@chakra-ui/react'
import { useAtomValue } from 'jotai'
import { useEffect, useMemo, useState } from 'react'
import { LuCheck, LuFlag, LuHouse, LuUsers } from 'react-icons/lu'
import { useNavigate, useParams } from 'react-router'
import { CancelMatchModal } from '../components/CancelMatchModal'
import { ErrorState } from '../components/common/ErrorState'
import { BottomNav } from '../components/domain/BottomNav'
import type { BottomNavItem } from '../components/domain/BottomNav'
import { RaceList } from '../components/domain/RaceList'
import { RaceResultsDisplay } from '../components/domain/RaceResultsDisplay'
import { ResultsGrid } from '../components/domain/ResultsGrid'
import type { SlotAssignments } from '../components/domain/ResultsGrid'
import { TeamCard } from '../components/domain/TeamCard'
import { SwapPlayerModal } from '../components/SwapPlayerModal'
import { useMatchManagement } from '../hooks/features/useMatchManagement'
import { useRaceResultsSubscription } from '../hooks/useRaceResultsSubscription'
import { useSlotAssignmentSync } from '../hooks/useSlotAssignmentSync'
import { applyAssignment } from '../lib/slotAssignments'
import { matchQueryAtom } from '../store/queries'

const Match = () => {
  const { matchId } = useParams()
  const navigate = useNavigate()
  const matchAtom = useMemo(() => matchQueryAtom(matchId || ''), [matchId])
  const matchResult = useAtomValue(matchAtom)
  const { recordResults, isRecordingResults, swapRoundPlayer, isSwappingPlayer, swapPlayerError } = useMatchManagement()

  const [selectedRound, setSelectedRound] = useState<number | null>(null)
  const [expandedCompletedRound, setExpandedCompletedRound] = useState<number | null>(null)
  // Positions are indexed per round so switching rounds and returning preserves
  // Local entry state (cross-device late-joiners are still empty by design —
  // Ephemeral broadcast, no DB snapshot).
  const [positionsByRound, setPositionsByRound] = useState<Record<number, Record<string, string>>>({})

  const positions = useMemo<Record<string, string>>(() => (selectedRound !== null ? (positionsByRound[selectedRound] ?? {}) : {}), [selectedRound, positionsByRound])

  const slots = useMemo<SlotAssignments>(
    () =>
      Object.fromEntries(
        Object.entries(positions)
          .map(([playerId, posStr]) => [Number.parseInt(posStr, 10), playerId] as const)
          .filter(([position]) => position >= 1 && position <= 24)
      ),
    [positions]
  )

  const updateRoundPositions = (round: number, update: (prev: Record<string, string>) => Record<string, string>) => {
    setPositionsByRound((prev) => ({ ...prev, [round]: update(prev[round] ?? {}) }))
  }

  const { publish } = useSlotAssignmentSync({
    matchId: matchId ?? null,
    onAssignment: (slotNumber, playerId) => {
      if (selectedRound === null) {
        return
      }
      updateRoundPositions(selectedRound, (prev) => applyAssignment(prev, slotNumber, playerId))
    },
    roundNumber: selectedRound,
  })

  const handleTogglePlayerInSlot = (slotNumber: number, playerId: string) => {
    if (selectedRound === null) {
      return
    }
    const round = selectedRound
    const prevSlot = positions[playerId] ? Number.parseInt(positions[playerId], 10) : null
    const newPlayerId = prevSlot === slotNumber ? null : playerId
    const snapshot = positions

    updateRoundPositions(round, (prev) => applyAssignment(prev, slotNumber, newPlayerId))

    publish(slotNumber, newPlayerId).then((result) => {
      if (!result.ok) {
        // Best-effort revert: if a concurrent legitimate event landed during
        // The round-trip, this restores the snapshot and overwrites that
        // Event. Acceptable for the small group / sub-second event scale.
        updateRoundPositions(round, () => snapshot)
      }
    })
  }
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
    // Don't clear positions — they're per-round, so leaving previous-round
    // State in `positionsByRound` doesn't affect this round and lets the user
    // Bounce between rounds without losing local entry state.
    setError('')
  }

  const handleToggleExpanded = (roundNumber: number) => {
    setExpandedCompletedRound((prev) => (prev === roundNumber ? null : roundNumber))
    setSelectedRound(null)
  }

  const handleSubmitResults = async (results: Array<{ playerId: string; position: number }>) => {
    if (selectedRound === null) {
      return
    }

    setError('')

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
      results,
      roundNumber: selectedRound,
    })

    if (updated) {
      const submittedRound = selectedRound
      setSelectedRound(null)
      setPositionsByRound((prev) => {
        const { [submittedRound]: _submitted, ...rest } = prev
        return rest
      })
    }
  }

  const hasAnyResults = match.rounds.some((r) => r.completed)
  const canCancelMatch = !match.completed && !hasAnyResults

  const handleCancelSuccess = () => {
    navigate('/')
  }

  const handleSwapPlayer = (player: { id: string; name: string; avatarFilename?: string | null; teamId?: string | unknown }, roundNumber: number) => {
    if (!player.teamId) {
      return
    }
    const teamId = typeof player.teamId === 'string' ? player.teamId : String(player.teamId)
    setPlayerToSwap({ ...player, teamId })
    setRoundToSwap(roundNumber)
    setSwapModalOpen(true)
  }

  const handleSwapConfirm = async (newPlayerId: string) => {
    if (!playerToSwap || roundToSwap === null || !matchId) {
      return
    }
    const result = await swapRoundPlayer({
      currentPlayerId: playerToSwap.id,
      matchId,
      newPlayerId,
      roundNumber: roundToSwap,
    })
    if (result) {
      setSwapModalOpen(false)
      setPlayerToSwap(null)
      setRoundToSwap(null)
    }
  }

  const navItems: BottomNavItem[] = [
    { dividerAfter: true, icon: LuHouse, id: 'home', label: 'Home', onClick: () => navigate('/') },
    { icon: LuUsers, id: 'teams', label: 'Teams', targetId: 'teams-section' },
    { icon: LuFlag, id: 'races', label: 'Races', targetId: 'races-section' },
  ]

  return (
    <Box minH="100vh" bg="bg.canvas" pb={{ base: '80px', md: '88px' }}>
      <Container maxW="4xl" py={{ base: 4, lg: 8, md: 6 }}>
        <VStack gap={{ base: 6, md: 8 }} align="stretch">
          <VStack align="start" gap={1}>
            <Heading size={{ base: 'lg', lg: '2xl', md: 'xl' }} color="gray.900">
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
                if (!roundData) {
                  return null
                }

                if (roundData.completed && roundData.results) {
                  return <RaceResultsDisplay results={roundData.results} trackName={roundData.track?.name} />
                }

                return (
                  <ResultsGrid
                    round={roundData}
                    slots={slots}
                    onTogglePlayer={handleTogglePlayerInSlot}
                    error={error}
                    submitting={isRecordingResults}
                    onSubmit={handleSubmitResults}
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
