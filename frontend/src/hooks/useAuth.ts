import { useCallback } from 'react'
import { useAtom, useAtomValue } from 'jotai'
import { useClient } from 'urql'
import { createGroupMutation } from '../queries/createGroup.mutation'
import { loginQuery } from '../queries/login.query'
import { isAuthenticatedAtom, tokenAtom } from '../store/auth'

export const useAuth = () => {
  const [token, setToken] = useAtom(tokenAtom)
  const isAuthenticated = useAtomValue(isAuthenticatedAtom)
  const client = useClient()

  const login = useCallback(async (dependencies: { groupId: string; password: string }): Promise<{ success: boolean; error?: string }> => {
    const { groupId, password } = dependencies

    const result = await client.query(loginQuery, { groupId, password }).toPromise()

    if (result.error) {
      return { success: false, error: result.error.message }
    }

    if (result.data?.login) {
      setToken(result.data.login)
      return { success: true }
    }

    return { success: false, error: 'Login failed' }
  }, [client, setToken])

  const createGroup = useCallback(async (dependencies: { name: string; password: string }): Promise<{ success: boolean; error?: string }> => {
    const { name, password } = dependencies

    const result = await client.mutation(createGroupMutation, { name, password }).toPromise()

    if (result.error) {
      return { success: false, error: result.error.message }
    }

    if (result.data?.createGroup) {
      setToken(result.data.createGroup)
      return { success: true }
    }

    return { success: false, error: 'Group creation failed' }
  }, [client, setToken])

  const logout = useCallback(() => {
    setToken(null)
  }, [setToken])

  return {
    token,
    isAuthenticated,
    login,
    createGroup,
    logout,
  }
}
