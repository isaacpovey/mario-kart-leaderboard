import { Button, Center, Container, Heading, HStack, Spinner, Text, VStack, Stack } from '@chakra-ui/react'
import { useEffect, useState } from 'react'
import { Link } from 'react-router'
import { useQuery } from 'urql'
import { CreateTournamentModal } from '../components/CreateTournamentModal'
import { CreateMatchModal } from '../components/CreateMatchModal'
import { useAuth } from '../hooks/useAuth'
import { tournamentsQuery } from '../queries/tournaments.query'

const Home = () => {
  const { logout } = useAuth()
  const [result] = useQuery({ query: tournamentsQuery })
  const [isModalOpen, setIsModalOpen] = useState(false)
  const [isMatchModalOpen, setIsMatchModalOpen] = useState(false)

  useEffect(() => {
    document.title = 'Mario Kart Leaderboard'
  }, [])

  const { data, fetching, error } = result

  if (fetching) {
    return (
      <Center h="100vh">
        <Spinner size="xl" />
      </Center>
    )
  }

  if (error) {
    return (
      <Container maxW="4xl" py={8}>
        <Text color="red.500">Error loading tournament data: {error.message}</Text>
      </Container>
    )
  }

  const currentTournament = data?.tournaments
    .filter((tournament) => tournament.startDate && typeof tournament.startDate === 'string')
    .sort((a, b) => {
      if (!a.startDate || !b.startDate) return 0
      return b.startDate.localeCompare(a.startDate)
    })[0]

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
              {currentTournament.leaderboard.length > 0 ? (
                currentTournament.leaderboard.map((entry, index) => (
                  <HStack
                    key={entry.playerId}
                    p={4}
                    bg={index === 0 ? 'yellow.50' : 'bg.panel'}
                    borderRadius="md"
                    borderWidth="1px"
                    borderColor={index === 0 ? 'yellow.300' : 'border'}
                    justify="space-between"
                  >
                    <HStack gap={4} flex={1} minW={0}>
                      <Text fontWeight="bold" fontSize={{ base: 'lg', md: 'xl' }} minW="8">
                        #{index + 1}
                      </Text>
                      <VStack align="start" gap={0} flex={1} minW={0}>
                        <Text fontWeight="semibold" fontSize={{ base: 'md', md: 'lg' }} truncate>
                          {entry.playerName}
                        </Text>
                        <Text fontSize="sm" color="fg.subtle">
                          ELO: {entry.eloRating}
                        </Text>
                      </VStack>
                    </HStack>
                    <Text fontWeight="bold" fontSize={{ base: 'lg', md: 'xl' }} flexShrink={0}>
                      {entry.totalScore} pts
                    </Text>
                  </HStack>
                ))
              ) : (
                <Text color="fg.subtle">No players yet</Text>
              )}
            </VStack>

            <VStack gap={2} align="start">
              <Heading size="lg">Matches</Heading>
              {currentTournament.matches.length > 0 ? (
                currentTournament.matches.map((match) => (
                  <Link key={match.id} to={`/match/${match.id}`} style={{ width: '100%', textDecoration: 'none' }}>
                    <HStack p={4} bg="bg.panel" borderRadius="md" borderWidth="1px" justify="space-between" width="full" cursor="pointer" _hover={{ bg: 'bg.subtle' }}>
                      <VStack align="start" gap={0}>
                        <Text fontWeight="semibold">{new Date(match.time).toLocaleString()}</Text>
                      </VStack>
                      <HStack gap={1}>
                        <Text fontSize="sm" fontWeight="semibold" color={match.completed ? 'green.500' : 'orange.500'}>
                          {match.completed ? '✓' : '○'}
                        </Text>
                        <Text fontSize="sm" fontWeight="semibold" color={match.completed ? 'green.500' : 'orange.500'}>
                          {match.completed ? 'Completed' : 'In Progress'}
                        </Text>
                      </HStack>
                    </HStack>
                  </Link>
                ))
              ) : (
                <Text color="fg.subtle">No matches yet</Text>
              )}
            </VStack>
          </>
        ) : (
          <Text color="fg.subtle">No tournament data available</Text>
        )}
      </VStack>
      <CreateTournamentModal open={isModalOpen} onOpenChange={setIsModalOpen} />
      {currentTournament && (
        <CreateMatchModal
          open={isMatchModalOpen}
          onOpenChange={setIsMatchModalOpen}
          tournamentId={currentTournament.id}
        />
      )}
    </Container>
  )
}

export default Home
