import { Box, Center, Spinner } from '@chakra-ui/react'
import { useEffect, useState } from 'react'
import { Navigate, useSearchParams } from 'react-router'
import { useAuth } from '../hooks/useAuth'

export const ProtectedRoute = (dependencies: { children: React.ReactNode }) => {
  const { children } = dependencies
  const { isAuthenticated, login } = useAuth()
  const [searchParams, setSearchParams] = useSearchParams()
  const groupId = searchParams.get('groupId')
  const password = searchParams.get('password')
  // Only block on the auto-login URL flow; otherwise decide immediately so an
  // Unauthenticated visit to `/` never sits on an indefinite spinner.
  const [isCheckingAuth, setIsCheckingAuth] = useState(() => Boolean(groupId && password))

  useEffect(() => {
    if (!groupId || !password) {
      setIsCheckingAuth(false)
      return
    }

    let cancelled = false

    const checkAuth = async () => {
      const result = await login({ groupId, password })

      if (cancelled) {
        return
      }

      if (result.success) {
        setSearchParams((prev) => {
          const newParams = new URLSearchParams(prev)
          newParams.delete('groupId')
          newParams.delete('password')
          return newParams
        })
      }

      setIsCheckingAuth(false)
    }

    void checkAuth()

    return () => {
      cancelled = true
    }
  }, [groupId, password, login, setSearchParams])

  if (isCheckingAuth) {
    return (
      <Box minH="100vh" bg="bg.canvas">
        <Center h="100vh">
          <Spinner size="xl" color="brand.500" />
        </Center>
      </Box>
    )
  }

  if (!isAuthenticated) {
    return <Navigate to="/login" replace />
  }

  return <>{children}</>
}
