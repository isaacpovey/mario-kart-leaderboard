import { Box, Button, Dialog, HStack, Portal, Text, VStack } from '@chakra-ui/react'
import { useMatchManagement } from '../hooks/features/useMatchManagement'

type CancelMatchModalProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
  matchId: string
  onSuccess: () => void
}

export const CancelMatchModal = (props: CancelMatchModalProps) => {
  const { open, onOpenChange, matchId, onSuccess } = props
  const { cancelMatch, isCancellingMatch, cancelMatchError } = useMatchManagement()

  const handleConfirm = async () => {
    const success = await cancelMatch(matchId)
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
          <Dialog.Content maxW={{ base: '90vw', md: '400px' }}>
            <Dialog.Header>
              <Dialog.Title>Cancel Match</Dialog.Title>
            </Dialog.Header>
            <Dialog.Body>
              <VStack gap={4} align="stretch">
                <Text>Are you sure you want to cancel this match?</Text>
                <Box p={3} bg="red.50" borderRadius="md" borderWidth="1px" borderColor="red.200">
                  <Text color="red.700" fontSize="sm">
                    This action cannot be undone. The match and all team assignments will be permanently deleted.
                  </Text>
                </Box>

                {cancelMatchError && (
                  <Box p={3} bg="red.50" borderRadius="md" borderWidth="1px" borderColor="red.300">
                    <Text color="red.700" fontSize="sm" fontWeight="medium">
                      {cancelMatchError}
                    </Text>
                  </Box>
                )}

                <HStack gap={3} justify="flex-end">
                  <Button variant="outline" onClick={handleClose} disabled={isCancellingMatch}>
                    Keep Match
                  </Button>
                  <Button colorScheme="red" onClick={handleConfirm} loading={isCancellingMatch}>
                    Cancel Match
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
