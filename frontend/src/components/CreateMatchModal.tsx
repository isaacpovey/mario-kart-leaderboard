import { Button, Dialog, Field, Portal, Text, VStack } from '@chakra-ui/react'
import { useMemo, useState } from 'react'
import { useNavigate } from 'react-router'
import { useMatchManagement } from '../hooks/features/useMatchManagement'
import { usePlayerSelection } from '../hooks/features/usePlayerSelection'
import { useFormState } from '../hooks/patterns/useFormState'
import { FormField } from './common/FormField'
import { PlayerSearchDropdown } from './common/PlayerSearchDropdown'

const DEFAULT_NUM_RACES = 6
const DEFAULT_PLAYERS_PER_RACE = 4
const FALLBACK_NUM_RACES = 4
const FALLBACK_PLAYERS_PER_RACE = 4

export const CreateMatchModal = (dependencies: { open: boolean; onOpenChange: (open: boolean) => void; tournamentId: string }) => {
  const { open, onOpenChange, tournamentId } = dependencies
  const navigate = useNavigate()
  const { formState, updateField, resetForm } = useFormState({
    numRaces: String(DEFAULT_NUM_RACES),
    playersPerRace: String(DEFAULT_PLAYERS_PER_RACE),
  })
  const [error, setError] = useState('')

  const playerSelection = usePlayerSelection()
  const { createMatchWithRounds, isCreatingMatch } = useMatchManagement()

  const totalSlots = (Number.parseInt(formState.numRaces, 10) || FALLBACK_NUM_RACES) * (Number.parseInt(formState.playersPerRace, 10) || FALLBACK_PLAYERS_PER_RACE)
  const selectedCount = playerSelection.selectedPlayerIds.length
  const isValidAllocation = selectedCount > 0 && totalSlots >= selectedCount

  const validationMessage = useMemo(() => {
    if (selectedCount === 0) return 'Select at least one player'
    if (totalSlots < selectedCount) {
      return `Total slots (${totalSlots}) must be at least equal to player count (${selectedCount})`
    }
    return ''
  }, [totalSlots, selectedCount])

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')

    if (!isValidAllocation) {
      setError(validationMessage)
      return
    }

    const match = await createMatchWithRounds({
      tournamentId,
      playerIds: playerSelection.selectedPlayerIds,
      numRaces: Number.parseInt(formState.numRaces, 10) || FALLBACK_NUM_RACES,
      playersPerRace: Number.parseInt(formState.playersPerRace, 10) || FALLBACK_PLAYERS_PER_RACE,
    })

    if (match) {
      resetForm()
      playerSelection.clearSelection()
      playerSelection.setSearchTerm('')
      onOpenChange(false)
      navigate(`/match/${match.id}`)
    }
  }

  const handleClose = () => {
    resetForm()
    playerSelection.clearSelection()
    playerSelection.setSearchTerm('')
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
              <Dialog.Title>Create New Match</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              <form onSubmit={handleSubmit}>
                <VStack gap={4} align="stretch">
                  <Field.Root>
                    <Field.Label>Players</Field.Label>
                    <PlayerSearchDropdown
                      searchTerm={playerSelection.searchTerm}
                      onSearchTermChange={playerSelection.setSearchTerm}
                      showDropdown={playerSelection.showDropdown}
                      onShowDropdownChange={playerSelection.setShowDropdown}
                      filteredPlayers={playerSelection.filteredPlayers}
                      selectedPlayers={playerSelection.selectedPlayers}
                      selectedPlayerIds={playerSelection.selectedPlayerIds}
                      onTogglePlayer={playerSelection.togglePlayer}
                      canCreateNewPlayer={playerSelection.canCreateNewPlayer}
                      isCreatingPlayer={playerSelection.isCreatingPlayer}
                      onCreateAndSelectPlayer={playerSelection.createAndSelectPlayer}
                    />
                  </Field.Root>

                  <FormField label="Number of Races" type="number" min={1} max={20} value={formState.numRaces} onChange={updateField('numRaces')} disabled={isCreatingMatch} />

                  <FormField
                    label="Players per Race"
                    type="number"
                    min={2}
                    max={12}
                    value={formState.playersPerRace}
                    onChange={updateField('playersPerRace')}
                    disabled={isCreatingMatch}
                  />

                  {validationMessage && (
                    <Text color="orange.500" fontSize="sm">
                      {validationMessage}
                    </Text>
                  )}

                  {error && (
                    <Text color="red.500" fontSize="sm">
                      {error}
                    </Text>
                  )}

                  <VStack gap={2}>
                    <Button type="submit" colorScheme="blue" width="full" disabled={!isValidAllocation} loading={isCreatingMatch}>
                      {isCreatingMatch ? 'Creating...' : 'Create Match'}
                    </Button>
                    <Button type="button" variant="outline" width="full" onClick={handleClose} disabled={isCreatingMatch}>
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
