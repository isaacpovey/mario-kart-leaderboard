import { Badge, Box, HStack, Text, VStack } from '@chakra-ui/react'
import { Link } from 'react-router'

type Match = {
  id: string
  time: string
  completed: boolean
}

type MatchListProps = {
  matches: Match[]
}

const formatMatchDate = (dateString: string) => {
  const date = new Date(dateString)
  const now = new Date()
  const diffInHours = (now.getTime() - date.getTime()) / (1000 * 60 * 60)

  if (diffInHours < 24) {
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }
  return date.toLocaleDateString([], { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' })
}

export const MatchList = ({ matches }: MatchListProps) => {
  if (matches.length === 0) {
    return <Text color="fg.subtle">No matches yet</Text>
  }

  return (
    <VStack gap={{ base: 3, md: 4 }} align="stretch">
      {matches.map((match) => (
        <Link key={match.id} to={`/match/${match.id}`} style={{ width: '100%', textDecoration: 'none' }}>
          <Box
            p={{ base: 4, md: 5 }}
            bg="bg.panel"
            borderRadius="card"
            borderWidth="1px"
            borderColor="gray.200"
            boxShadow="card"
            cursor="pointer"
            _hover={{ boxShadow: 'card-hover', transform: 'translateY(-2px)', borderColor: 'brand.400' }}
            transition="all 0.2s"
          >
            <HStack justify="space-between" gap={{ base: 3, md: 4 }}>
              <VStack align="start" gap={1} flex={1}>
                <Text fontWeight="bold" fontSize={{ base: 'md', md: 'lg' }} color="gray.900">
                  {formatMatchDate(match.time)}
                </Text>
                <Text fontSize={{ base: 'xs', md: 'sm' }} color="gray.600">
                  {new Date(match.time).toLocaleDateString()}
                </Text>
              </VStack>

              <Badge colorScheme={match.completed ? 'green' : 'orange'} fontSize={{ base: 'sm', md: 'md' }} px={3} py={1} borderRadius="md">
                {match.completed ? 'Completed' : 'In Progress'}
              </Badge>
            </HStack>
          </Box>
        </Link>
      ))}
    </VStack>
  )
}
