import { Button, Dialog, HStack, Portal, Text, VStack } from '@chakra-ui/react'
import { useEffect, useRef, useState } from 'react'
import { LuX } from 'react-icons/lu'
import { useNavigate } from 'react-router'

type Match = {
  id: string
  time: string
  completed: boolean
}

type NewMatchNotificationProps = {
  matches: Match[]
  tournamentId: string | null
}

export const NewMatchNotification = ({ matches, tournamentId }: NewMatchNotificationProps) => {
  const navigate = useNavigate()
  const isInitialMount = useRef(true)
  const [isOpen, setIsOpen] = useState(false)
  const [newMatchId, setNewMatchId] = useState<string | null>(null)
  const [lastSeenMatchCount, setLastSeenMatchCount] = useState<number>(() => {
    if (!tournamentId) return 0
    const stored = localStorage.getItem(`lastSeenMatchCount_${tournamentId}`)
    return stored ? Number.parseInt(stored, 10) : 0
  })

  useEffect(() => {
    if (!tournamentId) return

    const currentMatchCount = matches.length

    // On initial mount, just set the count without showing modal
    if (isInitialMount.current) {
      isInitialMount.current = false
      setLastSeenMatchCount(currentMatchCount)
      if (currentMatchCount > 0) {
        localStorage.setItem(`lastSeenMatchCount_${tournamentId}`, currentMatchCount.toString())
      }
      return
    }

    // Only show modal if matches increased after initial mount
    if (currentMatchCount > lastSeenMatchCount) {
      const latestMatch = matches[matches.length - 1]
      if (latestMatch) {
        setNewMatchId(latestMatch.id)
        setIsOpen(true)
        setLastSeenMatchCount(currentMatchCount)
        localStorage.setItem(`lastSeenMatchCount_${tournamentId}`, currentMatchCount.toString())

        const timer = setTimeout(() => {
          setIsOpen(false)
        }, 10000)

        return () => clearTimeout(timer)
      }
    }
  }, [matches.length, tournamentId, lastSeenMatchCount, matches])

  const handleViewMatch = () => {
    if (newMatchId) {
      navigate(`/match/${newMatchId}`)
      setIsOpen(false)
    }
  }

  const handleClose = () => {
    setIsOpen(false)
  }

  return (
    <Dialog.Root open={isOpen} onOpenChange={(details) => setIsOpen(details.open)}>
      <Portal>
        <Dialog.Backdrop />
        <Dialog.Positioner>
          <Dialog.Content maxW={{ base: '90vw', md: '500px' }}>
            <Dialog.Header>
              <Dialog.Title>New Race Created!</Dialog.Title>
              <Dialog.CloseTrigger asChild>
                <Button variant="ghost" size="sm">
                  <LuX />
                </Button>
              </Dialog.CloseTrigger>
            </Dialog.Header>
            <Dialog.Body>
              <VStack gap={4} align="stretch">
                <Text>A new race has been created and is ready to play!</Text>
                <HStack gap={3} justify="flex-end">
                  <Button onClick={handleClose} variant="outline">
                    Dismiss
                  </Button>
                  <Button onClick={handleViewMatch} colorScheme="blue">
                    View Race
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
