import { Center, ChakraProvider, defaultSystem, Spinner } from '@chakra-ui/react'
import { Provider } from 'jotai'
import { useHydrateAtoms } from 'jotai/react/utils'
import { clientAtom } from 'jotai-urql'
import { Suspense } from 'react'
import { RouterProvider } from 'react-router'
import { Provider as UrqlProvider } from 'urql'
import { urqlClient } from './lib/urql'
import { router } from './routes'

const HydrateAtoms = ({ children }: { children: React.ReactNode }) => {
  useHydrateAtoms([[clientAtom, urqlClient]])
  return children
}

const App = () => (
  <UrqlProvider value={urqlClient}>
    <Provider>
      <HydrateAtoms>
        <ChakraProvider value={defaultSystem}>
          <Suspense
            fallback={
              <Center h="100vh">
                <Spinner size="xl" />
              </Center>
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
