import { Box, Button, Container, Heading, HStack, Spinner, Stack, Text, VStack } from '@chakra-ui/react'
import type { ResultOf } from 'gql.tada'
import { useAtomValue } from 'jotai'
import { useCallback, useEffect, useRef, useState } from 'react'
import { useClient, useQuery } from 'urql'
import { CompleteTournamentModal } from '../components/CompleteTournamentModal'
import { CreateMatchModal } from '../components/CreateMatchModal'
import { CreateTournamentModal } from '../components/CreateTournamentModal'
import { ErrorState } from '../components/common/ErrorState'
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

type RetryState = {
  isRetrying: boolean
  hasFailedCompletely: boolean
  handleManualRetry: () => void
}

type UseQueryRetryParams = {
  urqlClient: ReturnType<typeof useClient>
  hasError: boolean
  error: Error | undefined
  hasData: boolean
}

const useQueryRetry = ({ urqlClient, hasError, error, hasData }: UseQueryRetryParams): RetryState => {
  const [retryCount, setRetryCount] = useState(0)
  const lastErrorRef = useRef<Error | null>(null)

  const isNewError = hasError && error !== lastErrorRef.current

  useEffect(() => {
    if (isNewError && retryCount < MAX_RETRIES) {
      lastErrorRef.current = error ?? null
      const delay = 1000 * 2 ** retryCount
      const timer = setTimeout(() => {
        urqlClient
          .query(activeTournamentQuery, {}, { requestPolicy: 'network-only' })
          .toPromise()
          .finally(() => setRetryCount((prev) => prev + 1))
      }, delay)
      return () => clearTimeout(timer)
    }
  }, [isNewError, retryCount, error, urqlClient])

  useEffect(() => {
    if (hasData && !hasError) {
      setRetryCount(0)
      lastErrorRef.current = null
    }
  }, [hasData, hasError])

  const handleManualRetry = useCallback(() => {
    setRetryCount(0)
    lastErrorRef.current = null
    urqlClient.query(activeTournamentQuery, {}, { requestPolicy: 'network-only' }).toPromise()
  }, [urqlClient])

  return {
    isRetrying: hasError && retryCount < MAX_RETRIES,
    hasFailedCompletely: hasError && retryCount >= MAX_RETRIES,
    handleManualRetry,
  }
}

type ActiveTournament = NonNullable<ResultOf<typeof activeTournamentQuery>['activeTournament']>

type ActiveTournamentContentProps = {
  tournament: ActiveTournament
}

const ActiveTournamentContent = ({ tournament }: ActiveTournamentContentProps) => (
  <>
    <Heading size={{ base: 'lg', md: 'xl' }} color="gray.900">
      Current Tournament
    </Heading>

    {(tournament.startDate || tournament.endDate) && (
      <HStack gap={4} flexWrap="wrap" fontSize={{ base: 'xs', md: 'sm' }} color="gray.600">
        {tournament.startDate && <Text>Started: {tournament.startDate}</Text>}
        {tournament.endDate && <Text>Ended: {tournament.endDate}</Text>}
      </HStack>
    )}

    <VStack gap={{ base: 3, md: 4 }} align="stretch">
      <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
        Leaderboard
      </Heading>
      <LeaderboardList entries={tournament.leaderboard} />
    </VStack>

    <VStack gap={{ base: 3, md: 4 }} align="stretch">
      <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
        Races
      </Heading>
      <MatchList matches={tournament.matches} />
    </VStack>
  </>
)

type EmptyStateProps = {
  onStartTournament: () => void
}

const EmptyState = ({ onStartTournament }: EmptyStateProps) => (
  <Box p={8} bg="bg.panel" borderRadius="card" borderWidth="1px" borderColor="gray.200" textAlign="center">
    <Text color="gray.600" fontSize={{ base: 'md', md: 'lg' }}>
      No tournaments yet. Create one to get started!
    </Text>
    <Button mt={4} onClick={onStartTournament} colorScheme="blue" size={{ base: 'md', md: 'lg' }} borderRadius="button" px={8}>
      Start Tournament
    </Button>
  </Box>
)

const LoadingState = () => (
  <Box minH="100vh" bg="bg.canvas">
    <Container maxW="4xl" py={{ base: 4, md: 6, lg: 8 }}>
      <VStack gap={4} align="center" justify="center" minH="50vh">
        <Spinner size="xl" color="blue.500" />
        <Text color="gray.600">Loading tournament data...</Text>
      </VStack>
    </Container>
  </Box>
)

const Home = () => {
  const { logout } = useAuth()
  const urqlClient = useClient()
  const activeTournamentResult = useAtomValue(activeTournamentQueryAtom)
  const [isMatchModalOpen, setIsMatchModalOpen] = useState(false)
  const [isTournamentModalOpen, setIsTournamentModalOpen] = useState(false)
  const [isCompleteTournamentModalOpen, setIsCompleteTournamentModalOpen] = useState(false)

  const currentTournament = activeTournamentResult?.data?.activeTournament ?? null
  const hasError = activeTournamentResult?.error !== undefined

  const { isRetrying, hasFailedCompletely, handleManualRetry } = useQueryRetry({
    urqlClient,
    hasError,
    error: activeTournamentResult?.error,
    hasData: activeTournamentResult?.data !== undefined,
  })

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
    return <LoadingState />
  }

  return (
    <Box minH="100vh" bg="bg.canvas">
      <Container maxW="4xl" py={{ base: 4, md: 6, lg: 8 }}>
        <VStack gap={{ base: 6, md: 8 }} align="stretch">
          <HeroBanner onStartRace={() => setIsMatchModalOpen(true)} showStartButton={currentTournament !== null} />

          {currentTournament && <ActiveTournamentContent tournament={currentTournament} />}
          {!currentTournament && completedTournamentData && (
            <TournamentSummary tournament={completedTournamentData} showStartButton={true} onStartTournament={() => setIsTournamentModalOpen(true)} />
          )}
          {!currentTournament && !completedTournamentData && <EmptyState onStartTournament={() => setIsTournamentModalOpen(true)} />}

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
