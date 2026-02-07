import { Box, Button, Dialog, Field, Portal, Tabs, Text, VStack } from '@chakra-ui/react'
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
  const [activeTab, setActiveTab] = useState('players')

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
    setActiveTab('players')
    onOpenChange(false)
  }

  return (
    <Dialog.Root open={open} onOpenChange={(details) => onOpenChange(details.open)}>
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content maxW={{ base: '90vw', md: '600px' }}>
            <Dialog.Header>
              <Dialog.Title>Create New Match</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              <form onSubmit={handleSubmit}>
                <VStack gap={4} align="stretch">
                  <Tabs.Root value={activeTab} onValueChange={(details) => setActiveTab(details.value)}>
                    <Tabs.List>
                      <Tabs.Trigger value="players">Players</Tabs.Trigger>
                      <Tabs.Trigger value="settings">Settings</Tabs.Trigger>
                    </Tabs.List>

                    <Box mt={4}>
                      <Tabs.Content value="players">
                        <VStack gap={4} align="stretch">
                          <Field.Root>
                            <Field.Label fontSize={{ base: 'sm', md: 'md' }} fontWeight="medium" mb={2}>
                              Select Players
                            </Field.Label>
                            <PlayerSearchDropdown
                              selectedPlayers={playerSelection.selectedPlayers}
                              selectedPlayerIds={playerSelection.selectedPlayerIds}
                              onTogglePlayer={playerSelection.togglePlayer}
                              isCreatingPlayer={playerSelection.isCreatingPlayer}
                              onCreatePlayerByName={playerSelection.createAndSelectPlayerByName}
                              allPlayers={playerSelection.players}
                            />
                          </Field.Root>

                          {validationMessage && (
                            <Box p={3} bg="orange.50" borderRadius="button" borderWidth="1px" borderColor="orange.300">
                              <Text color="orange.700" fontSize="sm" fontWeight="medium">
                                {validationMessage}
                              </Text>
                            </Box>
                          )}
                        </VStack>
                      </Tabs.Content>

                      <Tabs.Content value="settings">
                        <VStack gap={4} align="stretch">
                          <FormField
                            label="Number of Races"
                            type="number"
                            min={1}
                            max={20}
                            value={formState.numRaces}
                            onChange={updateField('numRaces')}
                            disabled={isCreatingMatch}
                          />

                          <FormField
                            label="Players per Race"
                            type="number"
                            min={2}
                            max={12}
                            value={formState.playersPerRace}
                            onChange={updateField('playersPerRace')}
                            disabled={isCreatingMatch}
                          />

                          <Box p={4} bg="blue.50" borderRadius="button" borderWidth="1px" borderColor="blue.200">
                            <Text fontSize="sm" color="blue.900">
                              Total slots:{' '}
                              <Text as="span" fontWeight="bold">
                                {totalSlots}
                              </Text>
                            </Text>
                            <Text fontSize="sm" color="blue.900">
                              Selected players:{' '}
                              <Text as="span" fontWeight="bold">
                                {selectedCount}
                              </Text>
                            </Text>
                          </Box>
                        </VStack>
                      </Tabs.Content>
                    </Box>
                  </Tabs.Root>

                  {error && (
                    <Box p={3} bg="red.50" borderRadius="button" borderWidth="1px" borderColor="red.300">
                      <Text color="red.700" fontSize="sm" fontWeight="medium">
                        {error}
                      </Text>
                    </Box>
                  )}

                  <VStack gap={2} mt={2}>
                    <Button
                      type="submit"
                      colorScheme="yellow"
                      bg="brand.400"
                      color="gray.900"
                      width="full"
                      size={{ base: 'md', md: 'lg' }}
                      borderRadius="button"
                      fontWeight="bold"
                      disabled={!isValidAllocation}
                      loading={isCreatingMatch}
                      _hover={{ bg: 'brand.500' }}
                    >
                      {isCreatingMatch ? 'Creating...' : 'Create Match'}
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      width="full"
                      size={{ base: 'md', md: 'lg' }}
                      borderRadius="button"
                      borderWidth="2px"
                      onClick={handleClose}
                      disabled={isCreatingMatch}
                    >
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
