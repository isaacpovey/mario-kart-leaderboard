import { Box, Button, Container, Text, VStack } from '@chakra-ui/react'

type ErrorStateProps = {
  message: string
  onRetry?: () => void
}

export const ErrorState = ({ message, onRetry }: ErrorStateProps) => (
  <Box minH="100vh" bg="bg.canvas">
    <Container maxW="4xl" py={8}>
      <Box p={{ base: 6, md: 8 }} bg="red.50" borderRadius="card" borderWidth="2px" borderColor="red.300" boxShadow="card">
        <VStack gap={4}>
          <Text fontSize={{ base: 'xl', md: '2xl' }} fontWeight="bold" color="red.700">
            ⚠ Error
          </Text>
          <Text color="red.700" fontSize={{ base: 'sm', md: 'md' }} textAlign="center">
            {message}
          </Text>
          {onRetry && (
            <Button colorScheme="red" variant="outline" onClick={onRetry}>
              Retry
            </Button>
          )}
        </VStack>
      </Box>
    </Container>
  </Box>
)
