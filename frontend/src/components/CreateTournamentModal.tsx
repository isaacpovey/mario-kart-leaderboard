import { Button, Dialog, Field, Input, Portal, Text, VStack } from '@chakra-ui/react'
import { useState } from 'react'
import { useClient, useMutation } from 'urql'
import { createTournamentMutation } from '../queries/createTournament.mutation'
import { tournamentsQuery } from '../queries/tournaments.query'

export const CreateTournamentModal = (dependencies: { open: boolean; onOpenChange: (open: boolean) => void }) => {
  const { open, onOpenChange } = dependencies
  const [form, setForm] = useState({ startDate: '', endDate: '' })
  const [error, setError] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const [, executeMutation] = useMutation(createTournamentMutation)
  const client = useClient()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setIsLoading(true)

    const result = await executeMutation({
      startDate: form.startDate || undefined,
      endDate: form.endDate || undefined,
    })

    setIsLoading(false)

    if (result.error) {
      setError(result.error.message)
      return
    }

    if (result.data?.createTournament) {
      setForm({ startDate: '', endDate: '' })
      onOpenChange(false)

      await client.query(tournamentsQuery, {}, { requestPolicy: 'network-only' }).toPromise()
    }
  }

  const handleClose = () => {
    setForm({ startDate: '', endDate: '' })
    setError('')
    onOpenChange(false)
  }

  return (
    <Dialog.Root open={open} onOpenChange={(details) => onOpenChange(details.open)}>
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content>
            <Dialog.Header>
              <Dialog.Title>Create New Tournament</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              <form onSubmit={handleSubmit}>
                <VStack gap={4} align="stretch">
                  <Field.Root>
                    <Field.Label>Start Date</Field.Label>
                    <Input type="date" value={form.startDate} onChange={(e) => setForm((prev) => ({ ...prev, startDate: e.target.value }))} disabled={isLoading} />
                  </Field.Root>

                  <Field.Root>
                    <Field.Label>End Date</Field.Label>
                    <Input type="date" value={form.endDate} onChange={(e) => setForm((prev) => ({ ...prev, endDate: e.target.value }))} disabled={isLoading} />
                  </Field.Root>

                  {error && (
                    <Text color="red.500" fontSize="sm">
                      {error}
                    </Text>
                  )}

                  <VStack gap={2}>
                    <Button type="submit" colorScheme="blue" width="full" loading={isLoading}>
                      {isLoading ? 'Creating...' : 'Create Tournament'}
                    </Button>
                    <Button type="button" variant="outline" width="full" onClick={handleClose} disabled={isLoading}>
                      Cancel
                    </Button>
                  </VStack>
                </VStack>
              </form>
            </Dialog.Body>
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  )
}
