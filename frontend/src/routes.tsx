import { lazy, Suspense } from 'react'
import { createBrowserRouter } from 'react-router'
import { Spinner, Center } from '@chakra-ui/react'
import { ProtectedRoute } from './components/ProtectedRoute'

const Home = lazy(() => import('./pages/Home'))
const Login = lazy(() => import('./pages/Login').then(module => ({ default: module.Login })))
const Match = lazy(() => import('./pages/Match'))

const LoadingFallback = () => (
  <Center height="100vh">
    <Spinner size="xl" />
  </Center>
)

export const router = createBrowserRouter([
  {
    path: '/',
    element: (
      <Suspense fallback={<LoadingFallback />}>
        <ProtectedRoute>
          <Home />
        </ProtectedRoute>
      </Suspense>
    ),
  },
  {
    path: '/match/:matchId',
    element: (
      <Suspense fallback={<LoadingFallback />}>
        <ProtectedRoute>
          <Match />
        </ProtectedRoute>
      </Suspense>
    ),
  },
  {
    path: '/login',
    element: (
      <Suspense fallback={<LoadingFallback />}>
        <Login />
      </Suspense>
    ),
  },
])
