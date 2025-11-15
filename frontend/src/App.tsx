import { Box, Center, ChakraProvider, Spinner } from '@chakra-ui/react'
import { Provider } from 'jotai'
import { useHydrateAtoms } from 'jotai/react/utils'
import { clientAtom } from 'jotai-urql'
import { Suspense } from 'react'
import { RouterProvider } from 'react-router'
import { Provider as UrqlProvider } from 'urql'
import { urqlClient } from './lib/urql'
import { router } from './routes'
import { system } from './theme/theme'

const HydrateAtoms = ({ children }: { children: React.ReactNode }) => {
  useHydrateAtoms([[clientAtom, urqlClient]])
  return children
}

const App = () => (
  <UrqlProvider value={urqlClient}>
    <Provider>
      <HydrateAtoms>
        <ChakraProvider value={system}>
          <Suspense
            fallback={
              <Box minH="100vh" bg="bg.canvas">
                <Center h="100vh">
                  <Spinner size="xl" color="brand.500" />
                </Center>
              </Box>
            }
          >
            <RouterProvider router={router} />
          </Suspense>
        </ChakraProvider>
      </HydrateAtoms>
    </Provider>
  </UrqlProvider>
)

export default App
