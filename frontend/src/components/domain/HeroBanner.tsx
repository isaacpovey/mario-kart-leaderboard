import { Box, Button, Heading, VStack } from '@chakra-ui/react'

type HeroBannerProps = {
  onStartRace?: () => void
  showStartButton?: boolean
}

export const HeroBanner = ({ onStartRace, showStartButton = true }: HeroBannerProps) => (
  <Box
    position="relative"
    w="full"
    h={{ base: '200px', md: '280px', lg: '320px' }}
    borderRadius="card"
    overflow="hidden"
    bg="linear-gradient(135deg, #667eea 0%, #764ba2 100%)"
    mb={{ base: 6, md: 8 }}
  >
    <Box position="absolute" top={0} left={0} right={0} bottom={0} backgroundImage="url('/hero_banner.jpg')" backgroundSize="cover" backgroundPosition="center" opacity={0.8} />

    <VStack position="relative" h="full" justify="center" align="center" gap={{ base: 4, md: 6 }} px={4}>
      <Heading size={{ base: '2xl', md: '3xl', lg: '4xl' }} color="white" textAlign="center" textShadow="0 2px 4px rgba(0,0,0,0.3)">
        Mario Kart Leaderboard
      </Heading>

      {showStartButton && onStartRace && (
        <Button
          size={{ base: 'lg', md: 'xl' }}
          colorScheme="yellow"
          bg="brand.400"
          color="gray.900"
          fontWeight="bold"
          fontSize={{ base: 'md', md: 'lg' }}
          px={{ base: 6, md: 8 }}
          py={{ base: 6, md: 7 }}
          borderRadius="button"
          _hover={{ bg: 'brand.500', transform: 'translateY(-2px)' }}
          _active={{ transform: 'translateY(0)' }}
          boxShadow="0 4px 12px rgba(0,0,0,0.2)"
          transition="all 0.2s"
          onClick={onStartRace}
        >
          Start New Race
        </Button>
      )}
    </VStack>
  </Box>
)
