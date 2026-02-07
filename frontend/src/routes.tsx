import { createBrowserRouter } from 'react-router'
import { ProtectedRoute } from './components/ProtectedRoute'
import Home from './pages/Home'
import { Login } from './pages/Login'
import Match from './pages/Match'
import PlayerStats from './pages/PlayerStats'

export const router = createBrowserRouter([
  {
    path: '/',
    element: (
      <ProtectedRoute>
        <Home />
      </ProtectedRoute>
    ),
  },
  {
    path: '/match/:matchId',
    element: (
      <ProtectedRoute>
        <Match />
      </ProtectedRoute>
    ),
  },
  {
    path: '/player/:playerId',
    element: (
      <ProtectedRoute>
        <PlayerStats />
      </ProtectedRoute>
    ),
  },
  {
    path: '/login',
    element: <Login />,
  },
])
