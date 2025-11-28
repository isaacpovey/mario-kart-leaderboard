import { Box, Button, HStack, Text, VStack } from '@chakra-ui/react'
import { useEffect, useMemo, useRef, useState } from 'react'
import { LuRotateCcw } from 'react-icons/lu'
import { Avatar } from '../common/Avatar'

type DataPoint = {
  timestamp: string
  elo: number
}

type PlayerEloHistory = {
  playerId: string
  playerName: string
  avatarFilename?: string | null
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
const ANIMATION_DURATION_PER_STEP = 300

type PlayerState = {
  playerId: string
  playerName: string
  avatarFilename?: string | null
  elo: number
  colorIndex: number
}

const lerp = (start: number, end: number, t: number): number => start + (end - start) * t

const getEloAtTimestamp = (dataPoints: DataPoint[], timestamp: string): number => {
  const relevantPoints = dataPoints.filter((d) => d.timestamp <= timestamp)
  return relevantPoints.length > 0 ? relevantPoints[relevantPoints.length - 1].elo : 1200
}

export const EloProgressionChart = ({ playerEloHistory }: EloProgressionChartProps) => {
  const [animationProgress, setAnimationProgress] = useState(0)
  const [animationKey, setAnimationKey] = useState(0)
  const animationRef = useRef<number | null>(null)
  const startTimeRef = useRef<number | null>(null)

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

  const { eloMin, eloMax } = useMemo(() => {
    const allElos = playerEloHistory.flatMap((p) => p.dataPoints.map((d) => d.elo))
    if (allElos.length === 0) return { eloMin: 1100, eloMax: 1300 }
    const min = Math.min(...allElos)
    const max = Math.max(...allElos)
    const padding = Math.max(20, (max - min) * 0.1)
    return {
      eloMin: Math.floor(min - padding),
      eloMax: Math.ceil(max + padding),
    }
  }, [playerEloHistory])

  const totalDuration = useMemo(
    () => Math.max(1, uniqueTimestamps.length - 1) * ANIMATION_DURATION_PER_STEP,
    [uniqueTimestamps.length]
  )

  useEffect(() => {
    setAnimationProgress(0)
    startTimeRef.current = null

    if (uniqueTimestamps.length <= 1) {
      setAnimationProgress(1)
      return
    }

    const animate = (currentTime: number) => {
      if (startTimeRef.current === null) {
        startTimeRef.current = currentTime
      }

      const elapsed = currentTime - startTimeRef.current
      const progress = Math.min(1, elapsed / totalDuration)

      setAnimationProgress(progress)

      if (progress < 1) {
        animationRef.current = requestAnimationFrame(animate)
      }
    }

    animationRef.current = requestAnimationFrame(animate)

    return () => {
      if (animationRef.current !== null) {
        cancelAnimationFrame(animationRef.current)
      }
    }
  }, [uniqueTimestamps.length, totalDuration, animationKey])

  const currentPlayers = useMemo((): PlayerState[] => {
    if (uniqueTimestamps.length === 0) return []

    const numSteps = uniqueTimestamps.length - 1
    const exactStep = animationProgress * numSteps
    const currentStepIndex = Math.min(Math.floor(exactStep), numSteps - 1)
    const stepProgress = exactStep - currentStepIndex

    const currentTimestamp = uniqueTimestamps[currentStepIndex]
    const nextTimestamp = uniqueTimestamps[Math.min(currentStepIndex + 1, numSteps)]

    return playerEloHistory
      .map((player) => {
        const currentElo = getEloAtTimestamp(player.dataPoints, currentTimestamp)
        const nextElo = getEloAtTimestamp(player.dataPoints, nextTimestamp)
        const interpolatedElo = Math.round(lerp(currentElo, nextElo, stepProgress))

        return {
          playerId: player.playerId,
          playerName: player.playerName,
          avatarFilename: player.avatarFilename,
          elo: interpolatedElo,
          colorIndex: playerColorMap[player.playerId] ?? 0,
        }
      })
      .sort((a, b) => b.elo - a.elo)
  }, [playerEloHistory, uniqueTimestamps, animationProgress, playerColorMap])

  const currentDate = useMemo(() => {
    if (uniqueTimestamps.length === 0) return ''
    const numSteps = Math.max(1, uniqueTimestamps.length - 1)
    const currentIndex = Math.min(Math.floor(animationProgress * numSteps), uniqueTimestamps.length - 1)
    const timestamp = uniqueTimestamps[currentIndex]
    const date = new Date(timestamp)
    return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric', year: 'numeric' })
  }, [uniqueTimestamps, animationProgress])

  if (playerEloHistory.length === 0 || playerEloHistory.every((p) => p.dataPoints.length === 0)) {
    return (
      <Box p={8} bg="bg.panel" borderRadius="card" borderWidth="1px" borderColor="gray.200" textAlign="center">
        <Text color="gray.600">No ELO history available</Text>
      </Box>
    )
  }

  const handleReplay = () => setAnimationKey((prev) => prev + 1)

  const calculateBarWidth = (elo: number): string => {
    const percentage = ((elo - eloMin) / (eloMax - eloMin)) * 100
    return `${Math.max(10, Math.min(100, percentage))}%`
  }

  const containerHeight = currentPlayers.length * BAR_HEIGHT

  return (
    <VStack align="stretch" gap={4}>
      <Box p={{ base: 3, md: 4 }} bg="bg.panel" borderRadius="card" borderWidth="1px" borderColor="gray.200">
        <VStack align="stretch" gap={4}>
          <HStack justify="space-between">
            <Text fontSize="sm" color="gray.600" fontWeight="medium">
              {currentDate}
            </Text>
            <Text fontSize="xs" color="gray.400">
              {eloMin} - {eloMax}
            </Text>
          </HStack>

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
                  transition="top 0.15s ease-out"
                  display="flex"
                  alignItems="center"
                  px={2}
                >
                  <HStack gap={2} width="120px" flexShrink={0}>
                    <Avatar name={player.playerName} avatarFilename={player.avatarFilename} size="sm" />
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
