import { Badge, HStack, Text, Wrap, WrapItem } from '@chakra-ui/react'
import { LuCrown } from 'react-icons/lu'
import { Link } from 'react-router'

type Placing = {
  tournamentId: string
  startDate?: string | null
  endDate?: string | null
  placing: number
  totalPlayers: number
}

type PlayerTournamentPlacingsProps = {
  placings: Placing[]
  compact?: boolean
}

const formatTournamentLabel = (startDate?: string | null, endDate?: string | null): string => {
  const format = (dateStr: string) => new Date(dateStr).toLocaleDateString('en-US', { month: 'short', year: 'numeric' })

  if (startDate) {
    return format(startDate)
  }
  if (endDate) {
    return format(endDate)
  }
  return 'Tournament'
}

const getPlacingColor = (placing: number): string => {
  if (placing === 1) {
    return 'yellow'
  }
  if (placing === 2) {
    return 'gray'
  }
  if (placing === 3) {
    return 'orange'
  }
  return 'gray'
}

const PlacingBadge = ({ placing, compact }: { placing: Placing; compact?: boolean }) => {
  const label = formatTournamentLabel(placing.startDate, placing.endDate)
  const isWinner = placing.placing === 1

  return (
    <Link to={`/tournament/${placing.tournamentId}`} style={{ textDecoration: 'none' }}>
      <Badge
        colorScheme={getPlacingColor(placing.placing)}
        variant={isWinner ? 'solid' : 'subtle'}
        fontSize="xs"
        px={2}
        py={1}
        borderRadius="md"
        display="flex"
        alignItems="center"
        gap={1}
        cursor="pointer"
        _hover={{ opacity: 0.85 }}
      >
        {isWinner && <LuCrown size={12} fill="currentColor" />}
        {compact ? (
          <Text as="span">
            {label}: #{placing.placing}
          </Text>
        ) : (
          <Text as="span">
            {label} · #{placing.placing}/{placing.totalPlayers}
          </Text>
        )}
      </Badge>
    </Link>
  )
}

export const PlayerTournamentPlacings = ({ placings, compact = false }: PlayerTournamentPlacingsProps) => {
  if (placings.length === 0) {
    return (
      <Text fontSize="xs" color="gray.500">
        No previous tournaments
      </Text>
    )
  }

  if (compact) {
    return (
      <Wrap gap={1}>
        {placings.map((placing) => (
          <WrapItem key={placing.tournamentId}>
            <PlacingBadge placing={placing} compact />
          </WrapItem>
        ))}
      </Wrap>
    )
  }

  return (
    <HStack gap={2} flexWrap="wrap">
      {placings.map((placing) => (
        <PlacingBadge key={placing.tournamentId} placing={placing} />
      ))}
    </HStack>
  )
}
