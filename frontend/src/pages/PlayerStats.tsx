import { Box, Container, Heading, VStack } from '@chakra-ui/react'
import { useAtomValue } from 'jotai'
import { useEffect, useMemo } from 'react'
import { LuHistory, LuHouse, LuMap } from 'react-icons/lu'
import { useNavigate, useParams } from 'react-router'
import { ErrorState } from '../components/common/ErrorState'
import { BottomNav } from '../components/domain/BottomNav'
import type { BottomNavItem } from '../components/domain/BottomNav'
import { PlayerHeader } from '../components/domain/PlayerHeader'
import { PlayerMatchHistory } from '../components/domain/PlayerMatchHistory'
import { PlayerTournamentPlacings } from '../components/domain/PlayerTournamentPlacings'
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
    averagePosition: stat.averagePosition,
    racesPlayed: stat.racesPlayed,
    trackName: String(stat.trackName),
  }))

  const matchHistory = player.matchHistory.map((match) => ({
    eloChange: match.eloChange,
    matchId: match.matchId,
    matchTime: String(match.matchTime),
    position: match.position,
    tournamentEloChange: match.tournamentEloChange,
  }))

  const pastPlacings = (player.pastTournamentPlacings ?? []).map((placing) => ({
    endDate: placing.endDate,
    placing: placing.placing,
    startDate: placing.startDate,
    totalPlayers: placing.totalPlayers,
    tournamentId: placing.tournamentId,
  }))

  const navItems: BottomNavItem[] = [
    { dividerAfter: true, icon: LuHouse, id: 'home', label: 'Home', onClick: () => navigate('/') },
    ...(trackStats.length > 0 ? ([{ icon: LuMap, id: 'tracks', label: 'Tracks', targetId: 'tracks-section' }] satisfies BottomNavItem[]) : []),
    { icon: LuHistory, id: 'matches', label: 'Matches', targetId: 'matches-section' },
  ]

  return (
    <Box minH="100vh" bg="bg.canvas" pb={{ base: '80px', md: '88px' }}>
      <Container maxW="4xl" py={{ base: 4, lg: 8, md: 6 }}>
        <VStack gap={{ base: 6, md: 8 }} align="stretch">
          <Heading size={{ base: 'lg', lg: '2xl', md: 'xl' }} color="gray.900">
            Player Stats
          </Heading>

          <PlayerHeader name={player.name} avatarFilename={player.avatarFilename} tournamentElo={player.currentTournamentElo} allTimeElo={player.eloRating} />

          {pastPlacings.length > 0 && (
            <>
              <Box h="1px" bg="gray.200" />
              <VStack gap={{ base: 3, md: 4 }} align="stretch">
                <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
                  Tournament Placings
                </Heading>
                <PlayerTournamentPlacings placings={pastPlacings} />
              </VStack>
            </>
          )}

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
