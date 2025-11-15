import { Button, Dialog, Field, Input, Portal, Text, VStack } from '@chakra-ui/react'
import { useTournamentManagement } from '../hooks/features/useTournamentManagement'
import { useFormState } from '../hooks/patterns/useFormState'

export const CreateTournamentModal = (dependencies: { open: boolean; onOpenChange: (open: boolean) => void }) => {
  const { open, onOpenChange } = dependencies
  const { formState, updateField, resetForm } = useFormState({ startDate: '', endDate: '' })
  const { createTournament, isCreating, createError } = useTournamentManagement()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()

    const tournament = await createTournament({
      startDate: formState.startDate || null,
      endDate: formState.endDate || null,
    })

    if (tournament) {
      resetForm()
      onOpenChange(false)
    }
  }

  const handleClose = () => {
    resetForm()
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
                    <Input type="date" value={formState.startDate} onChange={(e) => updateField('startDate')(e.target.value)} disabled={isCreating} />
                  </Field.Root>

                  <Field.Root>
                    <Field.Label>End Date</Field.Label>
                    <Input type="date" value={formState.endDate} onChange={(e) => updateField('endDate')(e.target.value)} disabled={isCreating} />
                  </Field.Root>

                  {createError && (
                    <Text color="red.500" fontSize="sm">
                      {createError}
                    </Text>
                  )}

                  <VStack gap={2}>
                    <Button type="submit" colorScheme="blue" width="full" loading={isCreating}>
                      {isCreating ? 'Creating...' : 'Create Tournament'}
                    </Button>
                    <Button type="button" variant="outline" width="full" onClick={handleClose} disabled={isCreating}>
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
