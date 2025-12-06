import { Box, Button, Container, Heading, HStack, Spinner, Stack, Text, VStack } from '@chakra-ui/react'
import { useAtomValue } from 'jotai'
import { useCallback, useEffect, useRef, useState } from 'react'
import { useClient, useQuery } from 'urql'
import { ErrorState } from '../components/common/ErrorState'
import { CompleteTournamentModal } from '../components/CompleteTournamentModal'
import { CreateMatchModal } from '../components/CreateMatchModal'
import { CreateTournamentModal } from '../components/CreateTournamentModal'
import { HeroBanner } from '../components/domain/HeroBanner'
import { LeaderboardList } from '../components/domain/LeaderboardList'
import { MatchList } from '../components/domain/MatchList'
import { NewMatchNotification } from '../components/domain/NewMatchNotification'
import { TournamentSummary } from '../components/domain/TournamentSummary'
import { useAuth } from '../hooks/useAuth'
import { useRaceResultsSubscription } from '../hooks/useRaceResultsSubscription'
import { activeTournamentQuery } from '../queries/activeTournament.query'
import { tournamentByIdQuery } from '../queries/tournamentById.query'
import { tournamentsQuery } from '../queries/tournaments.query'
import { activeTournamentQueryAtom } from '../store/queries'

const MAX_RETRIES = 3

const Home = () => {
  const { logout } = useAuth()
  const urqlClient = useClient()
  const activeTournamentResult = useAtomValue(activeTournamentQueryAtom)
  const [isMatchModalOpen, setIsMatchModalOpen] = useState(false)
  const [isTournamentModalOpen, setIsTournamentModalOpen] = useState(false)
  const [isCompleteTournamentModalOpen, setIsCompleteTournamentModalOpen] = useState(false)
  const [retryCount, setRetryCount] = useState(0)
  const lastErrorRef = useRef<Error | null>(null)

  const currentTournament = activeTournamentResult?.data?.activeTournament ?? null
  const hasError = activeTournamentResult?.error !== undefined

  // Track if this is a new error (different from last one we tried to handle)
  const isNewError = hasError && activeTournamentResult.error !== lastErrorRef.current

  // Auto-retry on error with exponential backoff
  useEffect(() => {
    if (isNewError && retryCount < MAX_RETRIES) {
      lastErrorRef.current = activeTournamentResult.error ?? null
      const delay = 1000 * Math.pow(2, retryCount) // 1s, 2s, 4s
      const timer = setTimeout(() => {
        urqlClient
          .query(activeTournamentQuery, {}, { requestPolicy: 'network-only' })
          .toPromise()
          .finally(() => setRetryCount((prev) => prev + 1))
      }, delay)
      return () => clearTimeout(timer)
    }
  }, [isNewError, retryCount, urqlClient, activeTournamentResult.error])

  // Reset retry count when we successfully load data
  useEffect(() => {
    if (activeTournamentResult?.data && !hasError) {
      setRetryCount(0)
      lastErrorRef.current = null
    }
  }, [activeTournamentResult?.data, hasError])

  const handleManualRetry = useCallback(() => {
    setRetryCount(0)
    lastErrorRef.current = null
    urqlClient.query(activeTournamentQuery, {}, { requestPolicy: 'network-only' }).toPromise()
  }, [urqlClient])

  // Determine loading and error states
  const isRetrying = hasError && retryCount < MAX_RETRIES
  const hasFailedCompletely = hasError && retryCount >= MAX_RETRIES

  const [tournamentsResult] = useQuery({
    query: tournamentsQuery,
    pause: currentTournament !== null,
  })

  const mostRecentCompletedTournament = tournamentsResult.data?.tournaments?.find((t) => t.winnerId !== null) ?? null

  const [completedTournamentResult] = useQuery({
    query: tournamentByIdQuery,
    variables: { id: mostRecentCompletedTournament?.id ?? '' },
    pause: !mostRecentCompletedTournament,
  })

  const completedTournamentData = completedTournamentResult.data?.tournamentById ?? null

  // Subscribe to race result updates for live leaderboard and match list
  const subscriptionResult = useRaceResultsSubscription(currentTournament?.id)

  // Refetch tournament data when subscription receives updates (race results or match creation)
  useEffect(() => {
    if (subscriptionResult.data || subscriptionResult.error) {
      // Trigger a refetch of the tournament data
      // The query will fetch fresh data and update urql's cache, which automatically updates the atom
      urqlClient
        .query(activeTournamentQuery, {}, { requestPolicy: 'network-only' })
        .toPromise()
        .catch((err) => {
          console.error('Failed to refetch tournament data:', err)
        })
    }
  }, [subscriptionResult.data, subscriptionResult.error, urqlClient])

  useEffect(() => {
    document.title = 'Mario Kart Leaderboard'
  }, [])

  // Show error state if all retries have failed
  if (hasFailedCompletely) {
    return <ErrorState message="Failed to load tournament data. Please check your connection and try again." onRetry={handleManualRetry} />
  }

  // Show loading state while retrying (and no cached data)
  if (isRetrying && !currentTournament) {
    return (
      <Box minH="100vh" bg="bg.canvas">
        <Container maxW="4xl" py={{ base: 4, md: 6, lg: 8 }}>
          <VStack gap={4} align="center" justify="center" minH="50vh">
            <Spinner size="xl" color="blue.500" />
            <Text color="gray.600">Loading tournament data...</Text>
          </VStack>
        </Container>
      </Box>
    )
  }

  return (
    <Box minH="100vh" bg="bg.canvas">
      <Container maxW="4xl" py={{ base: 4, md: 6, lg: 8 }}>
        <VStack gap={{ base: 6, md: 8 }} align="stretch">
          <HeroBanner
            onStartRace={() => setIsMatchModalOpen(true)}
            showStartButton={currentTournament !== null}
          />

          {currentTournament ? (
            <>
              <Heading size={{ base: 'lg', md: 'xl' }} color="gray.900">
                Current Tournament
              </Heading>

              {(currentTournament.startDate || currentTournament.endDate) && (
                <HStack gap={4} flexWrap="wrap" fontSize={{ base: 'xs', md: 'sm' }} color="gray.600">
                  {currentTournament.startDate && <Text>Started: {currentTournament.startDate}</Text>}
                  {currentTournament.endDate && <Text>Ended: {currentTournament.endDate}</Text>}
                </HStack>
              )}

              <VStack gap={{ base: 3, md: 4 }} align="stretch">
                <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
                  Leaderboard
                </Heading>
                <LeaderboardList entries={currentTournament.leaderboard} />
              </VStack>

              <VStack gap={{ base: 3, md: 4 }} align="stretch">
                <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
                  Races
                </Heading>
                <MatchList matches={currentTournament.matches} />
              </VStack>
            </>
          ) : completedTournamentData ? (
            <TournamentSummary
              tournament={completedTournamentData}
              showStartButton={true}
              onStartTournament={() => setIsTournamentModalOpen(true)}
            />
          ) : (
            <Box p={8} bg="bg.panel" borderRadius="card" borderWidth="1px" borderColor="gray.200" textAlign="center">
              <Text color="gray.600" fontSize={{ base: 'md', md: 'lg' }}>
                No tournaments yet. Create one to get started!
              </Text>
              <Button mt={4} onClick={() => setIsTournamentModalOpen(true)} colorScheme="blue" size={{ base: 'md', md: 'lg' }} borderRadius="button" px={8}>
                Start Tournament
              </Button>
            </Box>
          )}

          <Box h="1px" bg="gray.200" my={4} />

          <Stack direction={{ base: 'column', sm: 'row' }} gap={3} justify="center" pb={4}>
            {currentTournament && (
              <Button
                onClick={() => setIsCompleteTournamentModalOpen(true)}
                colorScheme="green"
                size={{ base: 'md', md: 'lg' }}
                borderRadius="button"
                width={{ base: 'full', sm: 'auto' }}
                px={8}
              >
                Complete Tournament
              </Button>
            )}
            <Button onClick={logout} variant="outline" size={{ base: 'md', md: 'lg' }} borderRadius="button" borderWidth="2px" width={{ base: 'full', sm: 'auto' }} px={8}>
              Logout
            </Button>
          </Stack>
        </VStack>
        {currentTournament && <CreateMatchModal open={isMatchModalOpen} onOpenChange={setIsMatchModalOpen} tournamentId={currentTournament.id} />}
        {currentTournament && <NewMatchNotification matches={currentTournament.matches} tournamentId={currentTournament.id} />}
        {currentTournament && (
          <CompleteTournamentModal
            open={isCompleteTournamentModalOpen}
            onOpenChange={setIsCompleteTournamentModalOpen}
            tournamentId={currentTournament.id}
            endDate={currentTournament.endDate}
            onSuccess={() => {
              urqlClient.query(activeTournamentQuery, {}, { requestPolicy: 'network-only' }).toPromise()
            }}
          />
        )}
        <CreateTournamentModal open={isTournamentModalOpen} onOpenChange={setIsTournamentModalOpen} />
      </Container>
    </Box>
  )
}

export default Home
