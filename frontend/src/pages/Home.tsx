import { Box, Button, Container, Heading, HStack, Stack, Text, VStack } from '@chakra-ui/react'
import { useAtomValue } from 'jotai'
import { useEffect, useMemo, useState } from 'react'
import { CreateMatchModal } from '../components/CreateMatchModal'
import { CreateTournamentModal } from '../components/CreateTournamentModal'
import { HeroBanner } from '../components/domain/HeroBanner'
import { LeaderboardList } from '../components/domain/LeaderboardList'
import { MatchList } from '../components/domain/MatchList'
import { useAuth } from '../hooks/useAuth'
import { tournamentsQueryAtom } from '../store/queries'

const Home = () => {
  const { logout } = useAuth()
  const tournamentsResult = useAtomValue(tournamentsQueryAtom)
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [isMatchModalOpen, setIsMatchModalOpen] = useState(false)

  // Derive current tournament directly from query result
  const currentTournament = useMemo(() => {
    if (!tournamentsResult?.data?.tournaments) {
      return null
    }

    const tournaments = tournamentsResult.data.tournaments
      .filter((tournament) => tournament.startDate != null)
      .sort((a, b) => {
        if (!a.startDate || !b.startDate) return 0
        return b.startDate.localeCompare(a.startDate)
      })

    return tournaments[0] || null
  }, [tournamentsResult])

  useEffect(() => {
    document.title = 'Mario Kart Leaderboard'
  }, [])

  return (
    <Box minH="100vh" bg="bg.canvas">
      <Container maxW="4xl" py={{ base: 4, md: 6, lg: 8 }}>
        <VStack gap={{ base: 6, md: 8 }} align="stretch">
          <HeroBanner onStartRace={() => setIsMatchModalOpen(true)} />

          <Heading size={{ base: 'lg', md: 'xl' }} color="gray.900">
            Current Tournament
          </Heading>

          {currentTournament ? (
            <>
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
          ) : (
            <Box p={8} bg="bg.panel" borderRadius="card" borderWidth="1px" borderColor="gray.200" textAlign="center">
              <Text color="gray.600" fontSize={{ base: 'md', md: 'lg' }}>
                No active tournament. Create one to get started!
              </Text>
            </Box>
          )}

          <Box h="1px" bg="gray.200" my={4} />

          <Stack direction={{ base: 'column', sm: 'row' }} gap={3} justify="center" pb={4}>
            <Button onClick={() => setIsModalOpen(true)} colorScheme="blue" size={{ base: 'md', md: 'lg' }} borderRadius="button" width={{ base: 'full', sm: 'auto' }} px={8}>
              Create Tournament
            </Button>
            <Button onClick={logout} variant="outline" size={{ base: 'md', md: 'lg' }} borderRadius="button" borderWidth="2px" width={{ base: 'full', sm: 'auto' }} px={8}>
              Logout
            </Button>
          </Stack>
        </VStack>
        <CreateTournamentModal open={isModalOpen} onOpenChange={setIsModalOpen} />
        {currentTournament && <CreateMatchModal open={isMatchModalOpen} onOpenChange={setIsMatchModalOpen} tournamentId={currentTournament.id} />}
      </Container>
    </Box>
  )
}

export default Home
