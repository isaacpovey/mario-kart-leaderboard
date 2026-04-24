import { Box, Container, Heading, VStack } from '@chakra-ui/react'
import { useAtomValue } from 'jotai'
import { useEffect, useMemo } from 'react'
import { LuHistory, LuHouse, LuMap } from 'react-icons/lu'
import { useNavigate, useParams } from 'react-router'
import { ErrorState } from '../components/common/ErrorState'
import { BottomNav, type BottomNavItem } from '../components/domain/BottomNav'
import { PlayerHeader } from '../components/domain/PlayerHeader'
import { PlayerMatchHistory } from '../components/domain/PlayerMatchHistory'
import { PlayerTrackStats } from '../components/domain/PlayerTrackStats'
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

  const trackStats = player.trackStats.map((stat) => ({
    trackName: String(stat.trackName),
    averagePosition: stat.averagePosition,
    racesPlayed: stat.racesPlayed,
  }))

  const matchHistory = player.matchHistory.map((match) => ({
    matchId: match.matchId,
    matchTime: String(match.matchTime),
    position: match.position,
    eloChange: match.eloChange,
    tournamentEloChange: match.tournamentEloChange,
  }))

  const navItems: BottomNavItem[] = [
    { id: 'home', label: 'Home', icon: LuHouse, onClick: () => navigate('/'), dividerAfter: true },
    ...(trackStats.length > 0 ? ([{ id: 'tracks', label: 'Tracks', icon: LuMap, targetId: 'tracks-section' }] satisfies BottomNavItem[]) : []),
    { id: 'matches', label: 'Matches', icon: LuHistory, targetId: 'matches-section' },
  ]

  return (
    <Box minH="100vh" bg="bg.canvas" pb={{ base: '80px', md: '88px' }}>
      <Container maxW="4xl" py={{ base: 4, md: 6, lg: 8 }}>
        <VStack gap={{ base: 6, md: 8 }} align="stretch">
          <Heading size={{ base: 'lg', md: 'xl', lg: '2xl' }} color="gray.900">
            Player Stats
          </Heading>

          <PlayerHeader name={player.name} avatarFilename={player.avatarFilename} tournamentElo={player.currentTournamentElo} allTimeElo={player.eloRating} />

          {trackStats.length > 0 && (
            <>
              <Box h="1px" bg="gray.200" />
              <VStack id="tracks-section" scrollMarginTop={{ base: 4, md: 6 }} gap={{ base: 3, md: 4 }} align="stretch">
                <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
                  Track Performance
                </Heading>
                <PlayerTrackStats trackStats={trackStats} />
              </VStack>
            </>
          )}

          <Box h="1px" bg="gray.200" />

          <VStack id="matches-section" scrollMarginTop={{ base: 4, md: 6 }} gap={{ base: 3, md: 4 }} align="stretch">
            <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
              Recent Matches
            </Heading>
            <PlayerMatchHistory matches={matchHistory} />
          </VStack>
        </VStack>
      </Container>

      <BottomNav items={navItems} />
    </Box>
  )
}

export default PlayerStats
