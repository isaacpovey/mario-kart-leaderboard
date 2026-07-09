import { createBrowserRouter } from 'react-router'
import { ProtectedRoute } from './components/ProtectedRoute'
import Home from './pages/Home'
import { Login } from './pages/Login'
import Match from './pages/Match'
import PlayerStats from './pages/PlayerStats'
import TournamentDetail from './pages/TournamentDetail'
import TournamentHistory from './pages/TournamentHistory'

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
        <TournamentHistory />
      </ProtectedRoute>
    ),
    path: '/tournaments',
  },
  {
    element: (
      <ProtectedRoute>
        <TournamentDetail />
      </ProtectedRoute>
    ),
    path: '/tournament/:tournamentId',
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
