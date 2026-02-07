import { Box, Button, Container, Heading, HStack, VStack } from '@chakra-ui/react'
import { useAtomValue } from 'jotai'
import { useEffect, useMemo } from 'react'
import { useNavigate, useParams } from 'react-router'
import { ErrorState } from '../components/common/ErrorState'
import { PlayerHeader } from '../components/domain/PlayerHeader'
import { PlayerMatchHistory } from '../components/domain/PlayerMatchHistory'
import { playerByIdQueryAtom } from '../store/queries'

const PlayerStats = () => {
  const { playerId } = useParams()
  const navigate = useNavigate()
  const playerAtom = useMemo(() => playerByIdQueryAtom(playerId ?? ''), [playerId])
  const playerResult = useAtomValue(playerAtom)

  useEffect(() => {
    if (playerResult.data?.playerById?.name) {
      document.title = `${playerResult.data.playerById.name} - Stats`
    }
  }, [playerResult.data?.playerById?.name])

  if (playerResult.error || !playerResult.data?.playerById) {
    return <ErrorState message={`Error loading player data: ${playerResult.error?.message ?? 'Player not found'}`} />
  }

  const player = playerResult.data.playerById

  const matchHistory = player.matchHistory.map((match) => ({
    matchId: match.matchId,
    matchTime: String(match.matchTime),
    position: match.position,
    eloChange: match.eloChange,
    tournamentEloChange: match.tournamentEloChange,
  }))

  return (
    <Box minH="100vh" bg="bg.canvas">
      <Container maxW="4xl" py={{ base: 4, md: 6, lg: 8 }}>
        <VStack gap={{ base: 6, md: 8 }} align="stretch">
          <HStack justify="space-between" flexWrap="wrap" gap={{ base: 3, md: 4 }}>
            <Heading size={{ base: 'lg', md: 'xl', lg: '2xl' }} color="gray.900">
              Player Stats
            </Heading>
            <Button
              onClick={() => navigate('/')}
              variant="outline"
              size={{ base: 'sm', md: 'md' }}
              borderRadius="button"
              borderWidth="2px"
              flexShrink={0}
              _hover={{ bg: 'gray.50' }}
            >
              Back to Home
            </Button>
          </HStack>

          <PlayerHeader name={player.name} avatarFilename={player.avatarFilename} tournamentElo={player.currentTournamentElo} allTimeElo={player.eloRating} />

          <Box h="1px" bg="gray.200" />

          <VStack gap={{ base: 3, md: 4 }} align="stretch">
            <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
              Recent Matches
            </Heading>
            <PlayerMatchHistory matches={matchHistory} />
          </VStack>
        </VStack>
      </Container>
    </Box>
  )
}

export default PlayerStats
