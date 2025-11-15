import { Box, Button, Container, Field, Heading, Input, Tabs, Text, VStack } from '@chakra-ui/react'
import { useState } from 'react'
import { useNavigate } from 'react-router'
import { useAuth } from '../hooks/useAuth'

export const Login = () => {
  const navigate = useNavigate()
  const { login, createGroup } = useAuth()

  const [loginForm, setLoginForm] = useState({ groupId: '', password: '' })
  const [createForm, setCreateForm] = useState({ name: '', password: '' })
  const [error, setError] = useState('')
  const [isLoading, setIsLoading] = useState(false)

  const handleLogin = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setIsLoading(true)

    const result = await login({ groupId: loginForm.groupId, password: loginForm.password })

    setIsLoading(false)

    if (result.success) {
      navigate('/')
    } else {
      setError(result.error || 'Login failed')
    }
  }

  const handleCreateGroup = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setIsLoading(true)

    const result = await createGroup({ name: createForm.name, password: createForm.password })

    setIsLoading(false)

    if (result.success) {
      navigate('/')
    } else {
      setError(result.error || 'Failed to create group')
    }
  }

  return (
    <Container maxW="md" py={12}>
      <Box bg="bg.panel" p={8} borderRadius="lg" boxShadow="lg">
        <Heading size="2xl" mb={6} textAlign="center">
          Mario Kart Leaderboard
        </Heading>

        <Tabs.Root defaultValue="login">
          <Tabs.List mb={4}>
            <Tabs.Trigger value="login">Login</Tabs.Trigger>
            <Tabs.Trigger value="create">Create Group</Tabs.Trigger>
          </Tabs.List>

          <Tabs.Content value="login">
            <form onSubmit={handleLogin}>
              <VStack gap={4} align="stretch">
                <Field.Root required>
                  <Field.Label>Group ID</Field.Label>
                  <Input
                    value={loginForm.groupId}
                    onChange={(e) => setLoginForm((prev) => ({ ...prev, groupId: e.target.value }))}
                    placeholder="Enter group ID"
                    disabled={isLoading}
                  />
                </Field.Root>

                <Field.Root required>
                  <Field.Label>Password</Field.Label>
                  <Input
                    type="password"
                    value={loginForm.password}
                    onChange={(e) => setLoginForm((prev) => ({ ...prev, password: e.target.value }))}
                    placeholder="Enter password"
                    disabled={isLoading}
                  />
                </Field.Root>

                {error && (
                  <Text color="red.500" fontSize="sm">
                    {error}
                  </Text>
                )}

                <Button type="submit" colorScheme="blue" width="full" loading={isLoading}>
                  {isLoading ? 'Logging in...' : 'Login'}
                </Button>
              </VStack>
            </form>
          </Tabs.Content>

          <Tabs.Content value="create">
            <form onSubmit={handleCreateGroup}>
              <VStack gap={4} align="stretch">
                <Field.Root required>
                  <Field.Label>Group Name</Field.Label>
                  <Input
                    value={createForm.name}
                    onChange={(e) => setCreateForm((prev) => ({ ...prev, name: e.target.value }))}
                    placeholder="Enter group name"
                    disabled={isLoading}
                  />
                </Field.Root>

                <Field.Root required>
                  <Field.Label>Password</Field.Label>
                  <Input
                    type="password"
                    value={createForm.password}
                    onChange={(e) => setCreateForm((prev) => ({ ...prev, password: e.target.value }))}
                    placeholder="Create a password"
                    disabled={isLoading}
                  />
                </Field.Root>

                {error && (
                  <Text color="red.500" fontSize="sm">
                    {error}
                  </Text>
                )}

                <Button type="submit" colorScheme="green" width="full" loading={isLoading}>
                  {isLoading ? 'Creating...' : 'Create Group'}
                </Button>
              </VStack>
            </form>
          </Tabs.Content>
        </Tabs.Root>
      </Box>
    </Container>
  )
}
