import { Box, Button, Container, Heading, HStack, Stack, Text, VStack } from '@chakra-ui/react'
import { useAtomValue } from 'jotai'
import { useEffect, useState } from 'react'
import { useClient, useQuery } from 'urql'
import { CreateMatchModal } from '../components/CreateMatchModal'
import { CreateTournamentModal } from '../components/CreateTournamentModal'
import { HeroBanner } from '../components/domain/HeroBanner'
import { LeaderboardList } from '../components/domain/LeaderboardList'
import { MatchList } from '../components/domain/MatchList'
import { NewMatchNotification } from '../components/domain/NewMatchNotification'
import { TournamentSummary } from '../components/domain/TournamentSummary'
import { useTournamentManagement } from '../hooks/features/useTournamentManagement'
import { useAuth } from '../hooks/useAuth'
import { useRaceResultsSubscription } from '../hooks/useRaceResultsSubscription'
import { activeTournamentQuery } from '../queries/activeTournament.query'
import { tournamentByIdQuery } from '../queries/tournamentById.query'
import { tournamentsQuery } from '../queries/tournaments.query'
import { activeTournamentQueryAtom } from '../store/queries'

const Home = () => {
  const { logout } = useAuth()
  const urqlClient = useClient()
  const activeTournamentResult = useAtomValue(activeTournamentQueryAtom)
  const [isMatchModalOpen, setIsMatchModalOpen] = useState(false)
  const [isTournamentModalOpen, setIsTournamentModalOpen] = useState(false)
  const { completeTournament, isCompleting } = useTournamentManagement()

  const currentTournament = activeTournamentResult?.data?.activeTournament ?? null

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

  const handleCompleteTournament = async () => {
    if (!currentTournament) return
    await completeTournament(currentTournament.id)
    urqlClient.query(activeTournamentQuery, {}, { requestPolicy: 'network-only' }).toPromise()
  }

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
                onClick={handleCompleteTournament}
                colorScheme="green"
                size={{ base: 'md', md: 'lg' }}
                borderRadius="button"
                width={{ base: 'full', sm: 'auto' }}
                px={8}
                loading={isCompleting}
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
        <CreateTournamentModal open={isTournamentModalOpen} onOpenChange={setIsTournamentModalOpen} />
      </Container>
    </Box>
  )
}

export default Home
