import { Box, Text, VStack } from '@chakra-ui/react'
import { LuCrown } from 'react-icons/lu'
import { Avatar } from '../common/Avatar'

type TournamentChampionProps = {
  name: string
  avatarFilename?: string | null
  score: number
}

export const TournamentChampion = ({ name, avatarFilename, score }: TournamentChampionProps) => (
  <Box p={{ base: 6, md: 8 }} bg="bg.panel" borderRadius="card" borderWidth="3px" borderColor="yellow.400" textAlign="center">
    <VStack gap={4}>
      <Box color="yellow.500">
        <LuCrown size={48} fill="currentColor" />
      </Box>

      <Text fontSize={{ base: 'xs', md: 'sm' }} fontWeight="bold" color="gray.600" letterSpacing="wide" textTransform="uppercase">
        Tournament Champion
      </Text>

      <Box position="relative">
        <Box position="absolute" inset="-4px" borderRadius="full" borderWidth="3px" borderColor="yellow.400" />
        <Avatar name={name} avatarFilename={avatarFilename} size="lg" />
      </Box>

      <VStack gap={1}>
        <Text fontWeight="bold" fontSize={{ base: 'xl', md: '2xl' }}>
          {name}
        </Text>
        <Text fontSize={{ base: 'md', md: 'lg' }} color="gray.600">
          Final Score:{' '}
          <Text as="span" fontWeight="bold" color="brand.500">
            {score}
          </Text>
        </Text>
      </VStack>
    </VStack>
  </Box>
)
