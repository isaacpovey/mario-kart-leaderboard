import { Box, Button, Drawer, HStack, Portal, Text, VStack } from '@chakra-ui/react'
import { Avatar } from '../common/Avatar'

type Player = {
  id: string
  name: string
  avatarFilename?: string | null
}

type GridSlotPickerProps = {
  open: boolean
  onOpenChange: (open: boolean) => void
  slotNumber: number
  currentAssignment: Player | null
  candidates: Player[]
  onAssign: (playerId: string | null) => void
}

const ordinal = (n: number): string => {
  const mod100 = n % 100
  if (mod100 >= 11 && mod100 <= 13) return `${n}th`
  const mod10 = n % 10
  if (mod10 === 1) return `${n}st`
  if (mod10 === 2) return `${n}nd`
  if (mod10 === 3) return `${n}rd`
  return `${n}th`
}

export const GridSlotPicker = (props: GridSlotPickerProps) => {
  const { open, onOpenChange, slotNumber, currentAssignment, candidates, onAssign } = props

  const handleAssign = (playerId: string | null) => {
    onAssign(playerId)
    onOpenChange(false)
  }

  return (
    <Drawer.Root open={open} onOpenChange={(details) => onOpenChange(details.open)} placement="bottom" size="full">
      <Portal>
        <Drawer.Backdrop />
        <Drawer.Positioner>
          <Drawer.Content height="100dvh">
            <Drawer.Header borderBottomWidth="1px">
              <HStack justify="space-between" width="100%">
                <Drawer.Title>Who finished {ordinal(slotNumber)}?</Drawer.Title>
                <Drawer.CloseTrigger asChild>
                  <Button variant="ghost" size="sm">
                    ✕
                  </Button>
                </Drawer.CloseTrigger>
              </HStack>
            </Drawer.Header>

            <Drawer.Body overflowY="auto" p={0}>
              <VStack align="stretch" gap={0}>
                <Box px={4} py={3} borderBottomWidth="1px" cursor="pointer" onClick={() => handleAssign(null)} bg="gray.50" _hover={{ bg: 'gray.100' }}>
                  <HStack gap={3}>
                    <Box width="32px" height="32px" borderRadius="full" bg="gray.300" color="gray.700" display="flex" alignItems="center" justifyContent="center" fontWeight="bold">
                      —
                    </Box>
                    <Text fontWeight="semibold" color="gray.700">
                      Unassign (set to CPU)
                    </Text>
                  </HStack>
                </Box>

                {currentAssignment && (
                  <>
                    <Box px={4} py={2} bg="brand.50" position="sticky" top={0} zIndex={1}>
                      <Text fontSize="sm" fontWeight="semibold" color="brand.700">
                        Currently assigned
                      </Text>
                    </Box>
                    <PlayerRow player={currentAssignment} isCurrent={true} onSelect={() => handleAssign(currentAssignment.id)} />
                  </>
                )}

                {candidates.length > 0 && (
                  <Box px={4} py={2} bg="gray.50" position="sticky" top={0} zIndex={1}>
                    <Text fontSize="sm" fontWeight="semibold" color="gray.600">
                      Unassigned players ({candidates.length})
                    </Text>
                  </Box>
                )}
                {candidates.map((player) => (
                  <PlayerRow key={player.id} player={player} isCurrent={false} onSelect={() => handleAssign(player.id)} />
                ))}

                {candidates.length === 0 && !currentAssignment && (
                  <Box px={4} py={4}>
                    <Text color="gray.500">No players left to assign.</Text>
                  </Box>
                )}
              </VStack>
            </Drawer.Body>
          </Drawer.Content>
        </Drawer.Positioner>
      </Portal>
    </Drawer.Root>
  )
}

const PlayerRow = (props: { player: Player; isCurrent: boolean; onSelect: () => void }) => {
  const { player, isCurrent, onSelect } = props

  return (
    <Box
      px={4}
      py={3}
      borderBottomWidth="1px"
      cursor="pointer"
      onClick={onSelect}
      bg={isCurrent ? 'brand.50' : 'white'}
      _hover={{ bg: isCurrent ? 'brand.100' : 'gray.50' }}
      transition="background 0.15s"
    >
      <HStack gap={3}>
        <Avatar name={player.name} avatarFilename={player.avatarFilename} size="sm" />
        <Text fontWeight="medium">{player.name}</Text>
      </HStack>
    </Box>
  )
}
