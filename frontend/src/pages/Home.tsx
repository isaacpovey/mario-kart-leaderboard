import { Box, Button, Container, HStack, Heading, Link as ChakraLink, Spinner, Stack, Text, VStack } from '@chakra-ui/react'
import type { ResultOf } from 'gql.tada'
import { useAtomValue } from 'jotai'
import { useCallback, useEffect, useState } from 'react'
import { LuFlag, LuHistory, LuPlay, LuTrophy, LuUsers } from 'react-icons/lu'
import { Link, useNavigate } from 'react-router'
import { useClient, useQuery } from 'urql'
import { CompleteTournamentModal } from '../components/CompleteTournamentModal'
import { CreateMatchModal } from '../components/CreateMatchModal'
import { CreateTournamentModal } from '../components/CreateTournamentModal'
import { ErrorState } from '../components/common/ErrorState'
import { BottomNav } from '../components/domain/BottomNav'
import type { BottomNavItem } from '../components/domain/BottomNav'
import { HeroBanner } from '../components/domain/HeroBanner'
import { LeaderboardList } from '../components/domain/LeaderboardList'
import { Lobby } from '../components/domain/Lobby'
import { MatchList } from '../components/domain/MatchList'
import { NewMatchNotification } from '../components/domain/NewMatchNotification'
import { TournamentSummary } from '../components/domain/TournamentSummary'
import { useAuth } from '../hooks/useAuth'
import { useRaceResultsSubscription } from '../hooks/useRaceResultsSubscription'
import { activeTournamentQuery } from '../queries/activeTournament.query'
import { tournamentByIdQuery } from '../queries/tournamentById.query'
import { tournamentsQuery } from '../queries/tournaments.query'
import { activeTournamentQueryAtom, lobbyQueryAtom } from '../store/queries'

const MAX_RETRIES = 3

type RetryState = {
  isRetrying: boolean
  hasFailedCompletely: boolean
  handleManualRetry: () => void
}

type UseQueryRetryParams = {
  urqlClient: ReturnType<typeof useClient>
  hasError: boolean
  hasData: boolean
}

const useQueryRetry = ({ urqlClient, hasError, hasData }: Omit<UseQueryRetryParams, 'error'>): RetryState => {
  const [retryCount, setRetryCount] = useState(0)

  // Depend on hasError (boolean), not the error object identity. Fresh CombinedError
  // Instances on each failed refetch used to cancel the backoff timer forever.
  useEffect(() => {
    if (!hasError || retryCount >= MAX_RETRIES) {
      return
    }

    const delay = 1000 * 2 ** retryCount
    const timer = setTimeout(() => {
      urqlClient
        .query(activeTournamentQuery, {}, { requestPolicy: 'network-only' })
        .toPromise()
        .finally(() => setRetryCount((prev) => prev + 1))
    }, delay)
    return () => clearTimeout(timer)
  }, [hasError, retryCount, urqlClient])

  useEffect(() => {
    if (hasData && !hasError) {
      setRetryCount(0)
    }
  }, [hasData, hasError])

  const handleManualRetry = useCallback(() => {
    setRetryCount(0)
    urqlClient.query(activeTournamentQuery, {}, { requestPolicy: 'network-only' }).toPromise()
  }, [urqlClient])

  return {
    handleManualRetry,
    hasFailedCompletely: hasError && retryCount >= MAX_RETRIES,
    isRetrying: hasError && retryCount < MAX_RETRIES,
  }
}

type ActiveTournament = NonNullable<ResultOf<typeof activeTournamentQuery>['activeTournament']>

type BuildNavItemsArgs = {
  hasActiveTournament: boolean
  onStartRace: () => void
  onViewHistory: () => void
}

const buildNavItems = ({ hasActiveTournament, onStartRace, onViewHistory }: BuildNavItemsArgs): BottomNavItem[] => {
  if (!hasActiveTournament) {
    return [
      { icon: LuUsers, id: 'lobby', label: 'Lobby', targetId: 'lobby-section' },
      { icon: LuHistory, id: 'history', label: 'History', onClick: onViewHistory },
    ]
  }
  return [
    { icon: LuUsers, id: 'lobby', label: 'Lobby', targetId: 'lobby-section' },
    { icon: LuTrophy, id: 'leaderboard', label: 'Leaderboard', targetId: 'leaderboard-section' },
    { dividerAfter: true, icon: LuFlag, id: 'races', label: 'Races', targetId: 'races-section' },
    { icon: LuPlay, id: 'start-race', label: 'Start Race', onClick: onStartRace },
    { icon: LuHistory, id: 'history', label: 'History', onClick: onViewHistory },
  ]
}

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

    <VStack id="leaderboard-section" scrollMarginTop={{ base: 4, md: 6 }} gap={{ base: 3, md: 4 }} align="stretch">
      <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
        Leaderboard
      </Heading>
      <LeaderboardList entries={tournament.leaderboard} />
    </VStack>

    <VStack id="races-section" scrollMarginTop={{ base: 4, md: 6 }} gap={{ base: 3, md: 4 }} align="stretch">
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
    <Container maxW="4xl" py={{ base: 4, lg: 8, md: 6 }}>
      <VStack gap={4} align="center" justify="center" minH="50vh">
        <Spinner size="xl" color="blue.500" />
        <Text color="gray.600">Loading tournament data...</Text>
      </VStack>
    </Container>
  </Box>
)

const Home = () => {
  const { logout } = useAuth()
  const navigate = useNavigate()
  const urqlClient = useClient()
  const activeTournamentResult = useAtomValue(activeTournamentQueryAtom)
  const [isMatchModalOpen, setIsMatchModalOpen] = useState(false)
  const [isTournamentModalOpen, setIsTournamentModalOpen] = useState(false)
  const [isCompleteTournamentModalOpen, setIsCompleteTournamentModalOpen] = useState(false)

  const currentTournament = activeTournamentResult?.data?.activeTournament ?? null
  const hasError = activeTournamentResult?.error !== undefined

  const { isRetrying, hasFailedCompletely, handleManualRetry } = useQueryRetry({
    hasData: activeTournamentResult?.data !== undefined,
    hasError,
    urqlClient,
  })

  const [tournamentsResult] = useQuery({
    pause: currentTournament !== null,
    query: tournamentsQuery,
  })

  const mostRecentCompletedTournament = tournamentsResult.data?.tournaments?.find((t) => t.winnerId !== null) ?? null

  const [completedTournamentResult] = useQuery({
    pause: !mostRecentCompletedTournament,
    query: tournamentByIdQuery,
    variables: { id: mostRecentCompletedTournament?.id ?? '' },
  })

  const completedTournamentData = completedTournamentResult.data?.tournamentById ?? null

  const lobbyResult = useAtomValue(lobbyQueryAtom)
  const lobbyPlayerIds = lobbyResult?.data?.currentGroup?.lobby.map((p) => p.id) ?? []

  const navItems = buildNavItems({
    hasActiveTournament: currentTournament !== null,
    onStartRace: () => setIsMatchModalOpen(true),
    onViewHistory: () => navigate('/tournaments'),
  })

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
    <Box minH="100vh" bg="bg.canvas" pb={{ base: '80px', md: '88px' }}>
      <Container maxW="4xl" py={{ base: 4, lg: 8, md: 6 }}>
        <VStack gap={{ base: 6, md: 8 }} align="stretch">
          <HeroBanner onStartRace={() => setIsMatchModalOpen(true)} showStartButton={currentTournament !== null} />

          <Box id="lobby-section" scrollMarginTop={{ base: 4, md: 6 }}>
            <Lobby />
          </Box>

          {currentTournament && <ActiveTournamentContent tournament={currentTournament} />}
          {!currentTournament && completedTournamentData && (
            <>
              <TournamentSummary tournament={completedTournamentData} showStartButton onStartTournament={() => setIsTournamentModalOpen(true)} />
              <ChakraLink asChild color="brand.600" fontWeight="medium" fontSize={{ base: 'sm', md: 'md' }} alignSelf="center">
                <Link to="/tournaments">View all past tournaments →</Link>
              </ChakraLink>
            </>
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
        {currentTournament && (
          <CreateMatchModal open={isMatchModalOpen} onOpenChange={setIsMatchModalOpen} tournamentId={currentTournament.id} initialSelectedIds={lobbyPlayerIds} />
        )}
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
      <BottomNav items={navItems} />
    </Box>
  )
}

export default Home
