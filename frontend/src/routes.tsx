import { createBrowserRouter } from 'react-router'
import { ProtectedRoute } from './components/ProtectedRoute'
import Home from './pages/Home'
import { Login } from './pages/Login'
import Match from './pages/Match'
import PlayerStats from './pages/PlayerStats'

export const router = createBrowserRouter([
  {
    element: (
      <ProtectedRoute>
        <Home />
      </ProtectedRoute>
    ),
    path: '/',
  },
  {
    element: (
      <ProtectedRoute>
        <Match />
      </ProtectedRoute>
    ),
    path: '/match/:matchId',
  },
  {
    element: (
      <ProtectedRoute>
        <PlayerStats />
      </ProtectedRoute>
    ),
    path: '/player/:playerId',
  },
  {
    element: <Login />,
    path: '/login',
  },
])
