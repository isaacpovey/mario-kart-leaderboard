import { Badge, Box, HStack, Text, VStack } from '@chakra-ui/react'
import { LuCrown, LuUsers } from 'react-icons/lu'
import { Link } from 'react-router'
import { Avatar } from '../common/Avatar'

type TournamentListItemProps = {
  id: string
  startDate?: string | null
  endDate?: string | null
  winnerName?: string | null
  winnerAvatarFilename?: string | null
  participantCount: number
}

const formatDate = (dateStr: string): string => {
  const date = new Date(dateStr)
  return date.toLocaleDateString('en-US', { day: 'numeric', month: 'short', year: 'numeric' })
}

const formatDateRange = (startDate?: string | null, endDate?: string | null): string => {
  if (startDate && endDate) {
    return `${formatDate(startDate)} – ${formatDate(endDate)}`
  }
  if (startDate) {
    return `Started ${formatDate(startDate)}`
  }
  if (endDate) {
    return `Ended ${formatDate(endDate)}`
  }
  return 'Tournament'
}

export const TournamentListItem = ({ id, startDate, endDate, winnerName, winnerAvatarFilename, participantCount }: TournamentListItemProps) => (
  <Link to={`/tournament/${id}`} style={{ textDecoration: 'none', width: '100%' }}>
    <Box
      p={{ base: 4, md: 5 }}
      bg="bg.panel"
      borderRadius="card"
      borderWidth="2px"
      borderColor="yellow.300"
      boxShadow="card"
      cursor="pointer"
      _hover={{ borderColor: 'brand.400', boxShadow: 'card-hover', transform: 'translateY(-2px)' }}
      transition="all 0.2s"
    >
      <HStack justify="space-between" gap={{ base: 3, md: 4 }} align="start">
        <VStack align="start" gap={2} flex={1} minW={0}>
          <Text fontWeight="bold" fontSize={{ base: 'md', md: 'lg' }} color="gray.900">
            {formatDateRange(startDate, endDate)}
          </Text>

          {winnerName && (
            <HStack gap={3}>
              <Box color="yellow.500" flexShrink={0}>
                <LuCrown size={20} fill="currentColor" />
              </Box>
              <Avatar name={winnerName} avatarFilename={winnerAvatarFilename} size="sm" />
              <VStack align="start" gap={0} minW={0}>
                <Text fontSize="xs" color="gray.500" fontWeight="medium" textTransform="uppercase" letterSpacing="wide">
                  Champion
                </Text>
                <Text fontWeight="bold" fontSize={{ base: 'sm', md: 'md' }} color="brand.600" truncate>
                  {winnerName}
                </Text>
              </VStack>
            </HStack>
          )}
        </VStack>

        <VStack align="end" gap={2} flexShrink={0}>
          <Badge colorScheme="green" fontSize="xs" px={2} py={1} borderRadius="md">
            Completed
          </Badge>
          <HStack gap={1} color="gray.500" fontSize="xs">
            <LuUsers size={14} />
            <Text>{participantCount} players</Text>
          </HStack>
        </VStack>
      </HStack>
    </Box>
  </Link>
)
