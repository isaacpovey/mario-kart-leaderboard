import { Box, Container, Heading, Spinner, Text, VStack } from '@chakra-ui/react'
import { useEffect } from 'react'
import { LuHistory, LuHouse } from 'react-icons/lu'
import { useNavigate, useParams } from 'react-router'
import { useQuery } from 'urql'
import { ErrorState } from '../components/common/ErrorState'
import { BottomNav } from '../components/domain/BottomNav'
import type { BottomNavItem } from '../components/domain/BottomNav'
import { TournamentSummary } from '../components/domain/TournamentSummary'
import { tournamentByIdQuery } from '../queries/tournamentById.query'

const TournamentDetail = () => {
  const { tournamentId } = useParams()
  const navigate = useNavigate()

  const [result] = useQuery({
    pause: !tournamentId,
    query: tournamentByIdQuery,
    variables: { id: tournamentId ?? '' },
  })

  const tournament = result.data?.tournamentById

  useEffect(() => {
    if (tournament) {
      document.title = 'Tournament Wrap Up - Mario Kart Leaderboard'
    }
  }, [tournament])

  const navItems: BottomNavItem[] = [
    { icon: LuHouse, id: 'home', label: 'Home', onClick: () => navigate('/') },
    { icon: LuHistory, id: 'history', label: 'History', onClick: () => navigate('/tournaments') },
  ]

  if (result.fetching && !tournament) {
    return (
      <Box minH="100vh" bg="bg.canvas">
        <Container maxW="4xl" py={{ base: 4, lg: 8, md: 6 }}>
          <VStack gap={4} align="center" justify="center" minH="50vh">
            <Spinner size="xl" color="brand.500" />
            <Text color="gray.600">Loading tournament...</Text>
          </VStack>
        </Container>
      </Box>
    )
  }

  if (result.error || !tournament) {
    return <ErrorState message={`Error loading tournament: ${result.error?.message ?? 'Tournament not found'}`} onRetry={() => navigate('/tournaments')} />
  }

  return (
    <Box minH="100vh" bg="bg.canvas" pb={{ base: '80px', md: '88px' }}>
      <Container maxW="4xl" py={{ base: 4, lg: 8, md: 6 }}>
        <VStack gap={{ base: 6, md: 8 }} align="stretch">
          <Heading size={{ base: 'lg', md: 'xl' }} color="gray.900">
            Past Tournament
          </Heading>
          <TournamentSummary tournament={tournament} />
        </VStack>
      </Container>
      <BottomNav items={navItems} />
    </Box>
  )
}

export default TournamentDetail
