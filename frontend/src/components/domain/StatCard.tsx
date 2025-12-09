import { Box, Circle, HStack, Text, VStack } from '@chakra-ui/react'
import type { ReactNode } from 'react'
import { LuChevronDown, LuChevronsRight, LuFrown, LuHeart, LuMedal, LuTarget, LuTrophy, LuUsers, LuX } from 'react-icons/lu'
import { Avatar } from '../common/Avatar'

type StatType = 'BEST_RACE' | 'WORST_RACE' | 'BIGGEST_SWING' | 'BEST_TEAMMATE' | 'WORST_TEAMMATE' | 'MOST_HELPED' | 'MOST_HURT' | 'BEST_MATCH' | 'WORST_MATCH'

type StatConfig = {
  label: string
  icon: ReactNode
  borderColor: string
  iconBg: string
}

const statConfigs: Record<StatType, StatConfig> = {
  BEST_RACE: {
    label: 'Best Single Race',
    icon: <LuTrophy />,
    borderColor: 'orange.400',
    iconBg: 'orange.100',
  },
  WORST_RACE: {
    label: 'Worst Single Race',
    icon: <LuChevronsRight />,
    borderColor: 'red.400',
    iconBg: 'red.100',
  },
  BIGGEST_SWING: {
    label: 'Biggest ELO Swing',
    icon: <LuX />,
    borderColor: 'purple.400',
    iconBg: 'purple.100',
  },
  BEST_TEAMMATE: {
    label: 'Best Teammate',
    icon: <LuHeart />,
    borderColor: 'pink.400',
    iconBg: 'pink.100',
  },
  WORST_TEAMMATE: {
    label: 'Worst Teammate',
    icon: <LuFrown />,
    borderColor: 'orange.400',
    iconBg: 'orange.100',
  },
  MOST_HELPED: {
    label: 'Most Helped Player',
    icon: <LuUsers />,
    borderColor: 'teal.400',
    iconBg: 'teal.100',
  },
  MOST_HURT: {
    label: 'Most Hurt Player',
    icon: <LuTarget />,
    borderColor: 'red.400',
    iconBg: 'red.100',
  },
  BEST_MATCH: {
    label: 'Best Match Performance',
    icon: <LuMedal />,
    borderColor: 'green.400',
    iconBg: 'green.100',
  },
  WORST_MATCH: {
    label: 'Worst Match Performance',
    icon: <LuChevronDown />,
    borderColor: 'red.400',
    iconBg: 'red.100',
  },
}

type StatCardProps = {
  statType: StatType
  playerName: string
  avatarFilename?: string | null
  value: number
  extraData?: string | null
}

const formatValue = (value: number, statType: StatType, extraData?: string | null): string => {
  if (statType === 'BIGGEST_SWING' && extraData) {
    try {
      const parsed = JSON.parse(extraData)
      const lowElo = parsed.low_value ?? parsed.low_elo ?? parsed.lowElo ?? parsed.low ?? parsed.min
      const highElo = parsed.high_value ?? parsed.high_elo ?? parsed.highElo ?? parsed.high ?? parsed.max

      if (lowElo !== undefined && highElo !== undefined) {
        return `${Math.abs(value)} pts (${lowElo} → ${highElo})`
      }
    } catch {
      // Fall through to default
    }
  }
  return value >= 0 ? `+${value}` : `${value}`
}

const getValueColor = (value: number): string => (value >= 0 ? 'green.600' : 'red.600')

export const StatCard = ({ statType, playerName, avatarFilename, value, extraData }: StatCardProps) => {
  const config = statConfigs[statType]

  if (!config) {
    return null
  }

  return (
    <Box
      p={{ base: 3, md: 4 }}
      bg="bg.panel"
      borderRadius="card"
      borderWidth="1px"
      borderColor="gray.200"
      borderLeftWidth="4px"
      borderLeftColor={config.borderColor}
      boxShadow="card"
    >
      <VStack align="start" gap={3}>
        <HStack gap={2}>
          <Circle size="32px" bg={config.iconBg} color={config.borderColor}>
            {config.icon}
          </Circle>
          <Text fontSize="sm" color="gray.600">
            {config.label}
          </Text>
        </HStack>

        <HStack gap={3}>
          <Avatar name={playerName} avatarFilename={avatarFilename} size="sm" />
          <VStack align="start" gap={0}>
            <Text fontWeight="bold" fontSize={{ base: 'sm', md: 'md' }}>
              {playerName}
            </Text>
            <Text fontSize="sm" fontWeight="medium" color={getValueColor(value)} bg={value >= 0 ? 'green.50' : 'red.50'} px={2} borderRadius="sm">
              {formatValue(value, statType, extraData)}
            </Text>
          </VStack>
        </HStack>
      </VStack>
    </Box>
  )
}
