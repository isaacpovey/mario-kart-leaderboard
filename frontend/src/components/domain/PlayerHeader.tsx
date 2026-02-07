import { Box, Heading, HStack, Text, VStack } from '@chakra-ui/react'
import { Avatar } from '../common/Avatar'

type PlayerHeaderProps = {
  name: string
  avatarFilename?: string | null
  tournamentElo?: number | null
  allTimeElo: number
}

export const PlayerHeader = ({ name, avatarFilename, tournamentElo, allTimeElo }: PlayerHeaderProps) => (
  <Box p={{ base: 5, md: 6 }} bg="bg.panel" borderRadius="card" boxShadow="card" borderWidth="1px" borderColor="gray.200">
    <HStack gap={{ base: 4, md: 6 }} align="center">
      <Avatar name={name} avatarFilename={avatarFilename} size="lg" />
      <VStack align="start" gap={2} flex={1}>
        <Heading size={{ base: 'lg', md: 'xl' }} color="gray.900">
          {name}
        </Heading>
        <HStack gap={{ base: 4, md: 6 }} flexWrap="wrap">
          {tournamentElo !== null && tournamentElo !== undefined && (
            <VStack align="start" gap={0}>
              <Text fontSize={{ base: 'xs', md: 'sm' }} color="gray.600" fontWeight="medium">
                Tournament
              </Text>
              <Text fontSize={{ base: 'xl', md: '2xl' }} fontWeight="bold" color="brand.600">
                {tournamentElo}
              </Text>
            </VStack>
          )}
          <VStack align="start" gap={0}>
            <Text fontSize={{ base: 'xs', md: 'sm' }} color="gray.600" fontWeight="medium">
              All Time
            </Text>
            <Text fontSize={{ base: 'xl', md: '2xl' }} fontWeight="bold" color="gray.900">
              {allTimeElo}
            </Text>
          </VStack>
        </HStack>
      </VStack>
    </HStack>
  </Box>
)
