import { Box, Button, HStack, Text, VStack } from '@chakra-ui/react'
import { useEffect, useMemo, useState } from 'react'
import { LuRotateCcw } from 'react-icons/lu'
import { Avatar } from '../common/Avatar'

type DataPoint = {
  timestamp: string
  elo: number
}

type PlayerEloHistory = {
  playerId: string
  playerName: string
  dataPoints: DataPoint[]
}

type EloProgressionChartProps = {
  playerEloHistory: PlayerEloHistory[]
}

const playerColors = [
  '#f97316', // orange
  '#ef4444', // red
  '#84cc16', // lime
  '#06b6d4', // cyan
  '#8b5cf6', // violet
  '#ec4899', // pink
  '#14b8a6', // teal
  '#f59e0b', // amber
  '#6366f1', // indigo
]

const getPlayerColor = (index: number): string => playerColors[index % playerColors.length]

const BAR_HEIGHT = 48
const ELO_MIN = 1100
const ELO_MAX = 1350

type PlayerState = {
  playerId: string
  playerName: string
  elo: number
  colorIndex: number
}

export const EloProgressionChart = ({ playerEloHistory }: EloProgressionChartProps) => {
  const [visibleTimestampIndex, setVisibleTimestampIndex] = useState(0)
  const [animationKey, setAnimationKey] = useState(0)

  const uniqueTimestamps = useMemo(() => {
    const all = playerEloHistory.flatMap((p) => p.dataPoints.map((d) => d.timestamp))
    return [...new Set(all)].sort()
  }, [playerEloHistory])

  const playerColorMap = useMemo(
    () =>
      playerEloHistory.reduce<Record<string, number>>(
        (acc, player, index) => ({ ...acc, [player.playerId]: index }),
        {}
      ),
    [playerEloHistory]
  )

  useEffect(() => {
    setVisibleTimestampIndex(0)

    if (uniqueTimestamps.length <= 1) {
      return
    }

    const interval = setInterval(() => {
      setVisibleTimestampIndex((prev) => {
        if (prev >= uniqueTimestamps.length - 1) {
          clearInterval(interval)
          return prev
        }
        return prev + 1
      })
    }, 500)

    return () => clearInterval(interval)
  }, [uniqueTimestamps.length, animationKey])

  const currentPlayers = useMemo((): PlayerState[] => {
    if (uniqueTimestamps.length === 0) return []

    const currentTimestamp = uniqueTimestamps[visibleTimestampIndex] ?? uniqueTimestamps[0]

    return playerEloHistory
      .map((player) => {
        const relevantPoints = player.dataPoints.filter((d) => d.timestamp <= currentTimestamp)
        const latestPoint = relevantPoints.length > 0 ? relevantPoints[relevantPoints.length - 1] : null

        return {
          playerId: player.playerId,
          playerName: player.playerName,
          elo: latestPoint?.elo ?? 1200,
          colorIndex: playerColorMap[player.playerId] ?? 0,
        }
      })
      .sort((a, b) => b.elo - a.elo)
  }, [playerEloHistory, uniqueTimestamps, visibleTimestampIndex, playerColorMap])

  const currentDate = useMemo(() => {
    if (uniqueTimestamps.length === 0) return ''
    const timestamp = uniqueTimestamps[visibleTimestampIndex] ?? uniqueTimestamps[0]
    const date = new Date(timestamp)
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
  }, [uniqueTimestamps, visibleTimestampIndex])

  if (playerEloHistory.length === 0 || playerEloHistory.every((p) => p.dataPoints.length === 0)) {
    return (
      <Box p={8} bg="bg.panel" borderRadius="card" borderWidth="1px" borderColor="gray.200" textAlign="center">
        <Text color="gray.600">No ELO history available</Text>
      </Box>
    )
  }

  const handleReplay = () => setAnimationKey((prev) => prev + 1)

  const calculateBarWidth = (elo: number): string => {
    const percentage = ((elo - ELO_MIN) / (ELO_MAX - ELO_MIN)) * 100
    return `${Math.max(10, Math.min(100, percentage))}%`
  }

  const containerHeight = currentPlayers.length * BAR_HEIGHT

  return (
    <VStack align="stretch" gap={4}>
      <Box p={{ base: 3, md: 4 }} bg="bg.panel" borderRadius="card" borderWidth="1px" borderColor="gray.200">
        <VStack align="stretch" gap={4}>
          <Text fontSize="sm" color="gray.600" fontWeight="medium">
            {currentDate}
          </Text>

          <Box position="relative" height={`${containerHeight}px`}>
            {currentPlayers.map((player, sortedIndex) => {
              const color = getPlayerColor(player.colorIndex)

              return (
                <Box
                  key={player.playerId}
                  position="absolute"
                  left={0}
                  right={0}
                  top={`${sortedIndex * BAR_HEIGHT}px`}
                  height={`${BAR_HEIGHT}px`}
                  transition="top 0.4s ease-out"
                  display="flex"
                  alignItems="center"
                  px={2}
                >
                  <HStack gap={2} width="120px" flexShrink={0}>
                    <Avatar name={player.playerName} size="sm" />
                    <Text fontSize="sm" fontWeight="medium" truncate flex={1}>
                      {player.playerName}
                    </Text>
                  </HStack>

                  <Box flex={1} position="relative" height="28px" mx={2}>
                    <Box
                      position="absolute"
                      left={0}
                      top={0}
                      bottom={0}
                      width={calculateBarWidth(player.elo)}
                      bg={color}
                      borderRadius="md"
                      transition="width 0.4s ease-out"
                    />
                  </Box>

                  <Text fontSize="sm" fontWeight="bold" width="50px" textAlign="right" flexShrink={0}>
                    {player.elo}
                  </Text>
                </Box>
              )
            })}
          </Box>
        </VStack>
      </Box>

      <HStack justify="flex-end">
        <Button variant="outline" size="sm" onClick={handleReplay}>
          <LuRotateCcw />
          <Text ml={1}>Replay</Text>
        </Button>
      </HStack>
    </VStack>
  )
}
