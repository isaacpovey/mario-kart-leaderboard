import { ChakraProvider, defaultSystem } from '@chakra-ui/react'
import { RouterProvider } from 'react-router'
import { Provider as UrqlProvider } from 'urql'
import { urqlClient } from './lib/urql'
import { router } from './routes'

const App = () => (
  <UrqlProvider value={urqlClient}>
    <ChakraProvider value={defaultSystem}>
      <RouterProvider router={router} />
    </ChakraProvider>
  </UrqlProvider>
)

export default App
