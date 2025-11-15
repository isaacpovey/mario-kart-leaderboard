import { createBrowserRouter } from 'react-router'
import { ProtectedRoute } from './components/ProtectedRoute'
import Home from './pages/Home'
import { Login } from './pages/Login'
import Match from './pages/Match'

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
    path: '/login',
    element: <Login />,
  },
])
