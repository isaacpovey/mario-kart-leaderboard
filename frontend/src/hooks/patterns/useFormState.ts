import { useCallback, useState } from 'react'

type FormState<T> = T

export const useFormState = <T extends Record<string, unknown>>(initialState: T) => {
  const [formState, setFormState] = useState<FormState<T>>(initialState)

  const updateField = useCallback(
    <K extends keyof T>(field: K) =>
      (value: T[K]) => {
        setFormState((prev) => ({ ...prev, [field]: value }))
      },
    []
  )

  const setField = useCallback(<K extends keyof T>(field: K, value: T[K]) => {
    setFormState((prev) => ({ ...prev, [field]: value }))
  }, [])

  const resetForm = useCallback(() => {
    setFormState(initialState)
  }, [initialState])

  const setForm = useCallback((newState: T | ((prev: T) => T)) => {
    setFormState(newState)
  }, [])

  return { formState, updateField, setField, resetForm, setForm }
}
