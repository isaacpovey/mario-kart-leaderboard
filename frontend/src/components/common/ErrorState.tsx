import { Container, Text } from '@chakra-ui/react'

type ErrorStateProps = {
  message: string
}

export const ErrorState = ({ message }: ErrorStateProps) => (
  <Container maxW="4xl" py={8}>
    <Text color="red.500">{message}</Text>
  </Container>
)
