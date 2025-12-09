import { SimpleGrid } from '@chakra-ui/react'
import { StatCard } from './StatCard'

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
  avatarFilename?: string | null
}

type TournamentHighlightsProps = {
  stats: TournamentStat[]
  leaderboard: LeaderboardEntry[]
}

const statTypeMap: Record<string, string> = {
  BestRace: 'BEST_RACE',
  WorstRace: 'WORST_RACE',
  BiggestSwing: 'BIGGEST_SWING',
  BestTeammate: 'BEST_TEAMMATE',
  WorstTeammate: 'WORST_TEAMMATE',
  MostHelped: 'MOST_HELPED',
  MostHurt: 'MOST_HURT',
  BestMatch: 'BEST_MATCH',
  WorstMatch: 'WORST_MATCH',
}

const normalizeStatType = (statType: string): string => statTypeMap[statType] ?? statType

export const TournamentHighlights = ({ stats, leaderboard }: TournamentHighlightsProps) => {
  const playerMap = Object.fromEntries(leaderboard.map((entry) => [entry.playerId, entry]))

  const getPlayer = (playerId: string): LeaderboardEntry => playerMap[playerId] ?? { playerId, playerName: 'Unknown Player', avatarFilename: null }

  return (
    <SimpleGrid columns={{ base: 1, md: 2 }} gap={{ base: 3, md: 4 }}>
      {stats.map((stat) => {
        const player = getPlayer(stat.playerId)
        const normalizedType = normalizeStatType(stat.statType)

        return (
          <StatCard
            key={stat.id}
            statType={normalizedType as Parameters<typeof StatCard>[0]['statType']}
            playerName={player.playerName}
            avatarFilename={player.avatarFilename}
            value={stat.value}
            extraData={stat.extraData}
          />
        )
      })}
    </SimpleGrid>
  )
}
