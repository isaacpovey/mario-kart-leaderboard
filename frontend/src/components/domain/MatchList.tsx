import { HStack, Text, VStack } from '@chakra-ui/react'
import { Link } from 'react-router'

type Match = {
  id: string
  time: string
  completed: boolean
}

type MatchListProps = {
  matches: Match[]
}

export const MatchList = ({ matches }: MatchListProps) => {
  if (matches.length === 0) {
    return <Text color="fg.subtle">No matches yet</Text>
  }

  return (
    <>
      {matches.map((match) => (
        <Link key={match.id} to={`/match/${match.id}`} style={{ width: '100%', textDecoration: 'none' }}>
          <HStack p={4} bg="bg.panel" borderRadius="md" borderWidth="1px" justify="space-between" width="full" cursor="pointer" _hover={{ bg: 'bg.subtle' }}>
            <VStack align="start" gap={0}>
              <Text fontWeight="semibold">{new Date(match.time).toLocaleString()}</Text>
            </VStack>
            <HStack gap={1}>
              <Text fontSize="sm" fontWeight="semibold" color={match.completed ? 'green.500' : 'orange.500'}>
                {match.completed ? '✓' : '○'}
              </Text>
              <Text fontSize="sm" fontWeight="semibold" color={match.completed ? 'green.500' : 'orange.500'}>
                {match.completed ? 'Completed' : 'In Progress'}
              </Text>
            </HStack>
          </HStack>
        </Link>
      ))}
    </>
  )
}
