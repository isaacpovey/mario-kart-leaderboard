import { Box, Button, Dialog, HStack, Portal, Text, VStack } from '@chakra-ui/react'
import { useTournamentManagement } from '../hooks/features/useTournamentManagement'

type CompleteTournamentModalProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
  tournamentId: string
  endDate: string | null
  onSuccess: () => void
}

export const CompleteTournamentModal = (props: CompleteTournamentModalProps) => {
  const { open, onOpenChange, tournamentId, endDate, onSuccess } = props
  const { completeTournament, isCompleting, completeError } = useTournamentManagement()

  const handleConfirm = async () => {
    const success = await completeTournament(tournamentId)
    if (success) {
      onOpenChange(false)
      onSuccess()
    }
  }

  const handleClose = () => {
    onOpenChange(false)
  }

  return (
    <Dialog.Root open={open} onOpenChange={(details) => onOpenChange(details.open)}>
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content maxW={{ base: '90vw', md: '450px' }}>
            <Dialog.Header>
              <Dialog.Title>Complete Tournament?</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              <VStack gap={4} align="stretch">
                <Text>Are you sure you want to complete this tournament?</Text>
                <Box p={3} bg="orange.50" borderRadius="md" borderWidth="1px" borderColor="orange.200">
                  <VStack gap={2} align="stretch">
                    <Text color="orange.700" fontSize="sm" fontWeight="medium">
                      Only complete the tournament when all races have been played.
                    </Text>
                    <Text color="orange.700" fontSize="sm">
                      This action will crown the winner and cannot be undone.
                    </Text>
                    {endDate && (
                      <Text color="orange.800" fontSize="sm" fontWeight="semibold">
                        Scheduled end date: {endDate}
                      </Text>
                    )}
                  </VStack>
                </Box>

                {completeError && (
                  <Box p={3} bg="red.50" borderRadius="md" borderWidth="1px" borderColor="red.300">
                    <Text color="red.700" fontSize="sm" fontWeight="medium">
                      {completeError}
                    </Text>
                  </Box>
                )}

                <HStack gap={3} justify="flex-end">
                  <Button variant="outline" onClick={handleClose} disabled={isCompleting}>
                    Cancel
                  </Button>
                  <Button colorScheme="green" onClick={handleConfirm} loading={isCompleting}>
                    Complete Tournament
                  </Button>
                </HStack>
              </VStack>
            </Dialog.Body>
          </Dialog.Content>
        </Dialog.Positioner>
      </Portal>
    </Dialog.Root>
  )
}
