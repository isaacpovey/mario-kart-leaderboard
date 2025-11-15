import { Box, Center, Spinner } from '@chakra-ui/react'
import { useEffect, useState } from 'react'
import { Navigate, useSearchParams } from 'react-router'
import { useAuth } from '../hooks/useAuth'

export const ProtectedRoute = (dependencies: { children: React.ReactNode }) => {
  const { children } = dependencies
  const { isAuthenticated, login } = useAuth()
  const [searchParams, setSearchParams] = useSearchParams()
  const [isCheckingAuth, setIsCheckingAuth] = useState(true)

  useEffect(() => {
    const checkAuth = async () => {
      const groupId = searchParams.get('groupId')
      const password = searchParams.get('password')

      if (groupId && password) {
        const result = await login({ groupId, password })

        if (result.success) {
          setSearchParams((prev) => {
            const newParams = new URLSearchParams(prev)
            newParams.delete('groupId')
            newParams.delete('password')
            return newParams
          })
        }
      }

      setIsCheckingAuth(false)
    }

    checkAuth()
  }, [searchParams, login, setSearchParams])

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
