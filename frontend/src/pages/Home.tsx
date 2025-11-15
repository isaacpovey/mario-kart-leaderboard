import { Button, Container, Heading, Stack, Text, VStack } from '@chakra-ui/react'
import { useAtomValue } from 'jotai'
import { useEffect, useState } from 'react'
import { CreateMatchModal } from '../components/CreateMatchModal'
import { CreateTournamentModal } from '../components/CreateTournamentModal'
import { ErrorState } from '../components/common/ErrorState'
import { LeaderboardList } from '../components/domain/LeaderboardList'
import { MatchList } from '../components/domain/MatchList'
import { useAuth } from '../hooks/useAuth'
import { currentTournamentAtom } from '../store/derived'
import { tournamentsQueryAtom } from '../store/queries'

const Home = () => {
  const { logout } = useAuth()
  const tournamentsResult = useAtomValue(tournamentsQueryAtom)
  const currentTournament = useAtomValue(currentTournamentAtom)
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [isMatchModalOpen, setIsMatchModalOpen] = useState(false)

  useEffect(() => {
    document.title = 'Mario Kart Leaderboard'
  }, [])

  if (tournamentsResult.error) {
    return <ErrorState message={`Error loading tournament data: ${tournamentsResult.error.message}`} />
  }

  return (
    <Container maxW="4xl" py={8}>
      <VStack gap={6} align="stretch">
        <Stack direction={{ base: 'column', md: 'row' }} justify="space-between" gap={{ base: 4, md: 0 }}>
          <Heading size={{ base: 'xl', md: '2xl', lg: '3xl' }}>Mario Kart Leaderboard</Heading>
          <Stack direction={{ base: 'column', sm: 'row' }} gap={2} width={{ base: 'full', md: 'auto' }}>
            <Button onClick={() => setIsModalOpen(true)} colorScheme="blue" width={{ base: 'full', sm: 'auto' }}>
              Create Tournament
            </Button>
            {currentTournament && (
              <Button onClick={() => setIsMatchModalOpen(true)} colorScheme="green" width={{ base: 'full', sm: 'auto' }}>
                New Match
              </Button>
            )}
            <Button onClick={logout} colorScheme="red" width={{ base: 'full', sm: 'auto' }}>
              Logout
            </Button>
          </Stack>
        </Stack>

        {currentTournament ? (
          <>
            <VStack gap={2} align="start">
              <Heading size="xl">Current Tournament</Heading>
              {currentTournament.startDate && <Text color="fg.subtle">Started: {currentTournament.startDate}</Text>}
              {currentTournament.endDate && <Text color="fg.subtle">Ended: {currentTournament.endDate}</Text>}
            </VStack>

            <VStack gap={4} align="stretch">
              <Heading size="lg">Leaderboard</Heading>
              <LeaderboardList entries={currentTournament.leaderboard} />
            </VStack>

            <VStack gap={2} align="start">
              <Heading size="lg">Matches</Heading>
              <MatchList matches={currentTournament.matches} />
            </VStack>
          </>
        ) : (
          <Text color="fg.subtle">No tournament data available</Text>
        )}
      </VStack>
      <CreateTournamentModal open={isModalOpen} onOpenChange={setIsModalOpen} />
      {currentTournament && <CreateMatchModal open={isMatchModalOpen} onOpenChange={setIsMatchModalOpen} tournamentId={currentTournament.id} />}
    </Container>
  )
}

export default Home
