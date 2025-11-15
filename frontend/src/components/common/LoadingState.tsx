import { Center, Spinner } from '@chakra-ui/react'

export const LoadingState = () => (
  <Center h="100vh" bg="bg.canvas">
    <Spinner size="xl" color="brand.500" />
  </Center>
)
