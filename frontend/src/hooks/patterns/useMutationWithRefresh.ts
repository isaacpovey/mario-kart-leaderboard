import { useCallback, useState } from 'react'
import type { AnyVariables, OperationResult, TypedDocumentNode } from 'urql'
import { useMutation } from 'urql'

type MutationOptions = {
  onSuccess?: (data: unknown) => void | Promise<void>
  onError?: (error: Error) => void
  refreshAtoms?: (() => void)[]
}

export const useMutationWithRefresh = <Data, Variables extends AnyVariables>(mutation: TypedDocumentNode<Data, Variables>, options?: MutationOptions) => {
  const [isLoading, setIsLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [, executeMutation] = useMutation(mutation)

  const execute = useCallback(
    async (variables: Variables): Promise<OperationResult<Data, Variables>> => {
      setIsLoading(true)
      setError(null)

      const result = await executeMutation(variables)

      setIsLoading(false)

      if (result.error) {
        const errorMessage = result.error.message
        setError(errorMessage)
        options?.onError?.(new Error(errorMessage))
      } else if (result.data) {
        await options?.onSuccess?.(result.data)
        options?.refreshAtoms?.forEach((refresh) => {
          refresh()
        })
      }

      return result
    },
    [executeMutation, options]
  )

  const clearError = useCallback(() => {
    setError(null)
  }, [])

  return { execute, isLoading, error, clearError }
}
