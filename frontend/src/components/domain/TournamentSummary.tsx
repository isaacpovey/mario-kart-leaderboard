import { Button, Heading, Text, VStack } from '@chakra-ui/react'
import { EloProgressionChart } from './EloProgressionChart'
import { FinalStandings } from './FinalStandings'
import { TournamentChampion } from './TournamentChampion'
import { TournamentHighlights } from './TournamentHighlights'

type DataPoint = {
  timestamp: string
  elo: number
}

type PlayerEloHistory = {
  playerId: string
  playerName: string
  dataPoints: DataPoint[]
}

type TournamentStat = {
  id: string
  statType: string
  playerId: string
  value: number
  extraData?: string | null
}

type LeaderboardEntry = {
  playerId: string
  playerName: string
  allTimeEloRating: number
  allTimeElo: number
  avatarFilename?: string | null
}

type TournamentData = {
  id: string
  startDate?: string | null
  endDate?: string | null
  winnerId?: string | null
  leaderboard: LeaderboardEntry[]
  stats: TournamentStat[]
  playerEloHistory: PlayerEloHistory[]
}

type TournamentSummaryProps = {
  tournament: TournamentData
  showStartButton?: boolean
  onStartTournament?: () => void
}

const formatDate = (dateStr: string): string => {
  const date = new Date(dateStr)
  return date.toLocaleDateString('en-US', { month: 'long', day: 'numeric', year: 'numeric' })
}

export const TournamentSummary = ({
  tournament,
  showStartButton = false,
  onStartTournament,
}: TournamentSummaryProps) => {
  const winner = tournament.winnerId
    ? tournament.leaderboard.find((entry) => entry.playerId === tournament.winnerId)
    : tournament.leaderboard[0]

  const dateRange =
    tournament.startDate && tournament.endDate
      ? `${formatDate(tournament.startDate)} - ${formatDate(tournament.endDate)}`
      : tournament.startDate
        ? `Started ${formatDate(tournament.startDate)}`
        : null

  return (
    <VStack gap={{ base: 6, md: 8 }} align="stretch">
      <VStack gap={2} align="start">
        <Heading size={{ base: 'lg', md: 'xl' }} color="gray.900">
          Tournament Wrap Up
        </Heading>
        {dateRange && (
          <Text fontSize={{ base: 'sm', md: 'md' }} color="gray.600">
            {dateRange}
          </Text>
        )}
      </VStack>

      {showStartButton && onStartTournament && (
        <Button
          onClick={onStartTournament}
          colorScheme="blue"
          size={{ base: 'md', md: 'lg' }}
          borderRadius="button"
          width={{ base: 'full', sm: 'auto' }}
          px={8}
        >
          Start New Tournament
        </Button>
      )}

      {winner && (
        <TournamentChampion
          name={winner.playerName}
          avatarFilename={winner.avatarFilename}
          score={winner.allTimeEloRating}
        />
      )}

      <VStack gap={{ base: 3, md: 4 }} align="stretch">
        <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
          ELO Progression
        </Heading>
        <EloProgressionChart playerEloHistory={tournament.playerEloHistory} />
      </VStack>

      {tournament.stats.length > 0 && (
        <VStack gap={{ base: 3, md: 4 }} align="stretch">
          <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
            Tournament Highlights
          </Heading>
          <TournamentHighlights stats={tournament.stats} leaderboard={tournament.leaderboard} />
        </VStack>
      )}

      <VStack gap={{ base: 3, md: 4 }} align="stretch">
        <Heading size={{ base: 'md', md: 'lg' }} color="gray.900">
          Final Standings
        </Heading>
        <FinalStandings leaderboard={tournament.leaderboard} winnerId={tournament.winnerId} />
      </VStack>
    </VStack>
  )
}
