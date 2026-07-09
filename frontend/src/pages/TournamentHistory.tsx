import { Box, Container, Heading, Spinner, Text, VStack } from '@chakra-ui/react'
import { useCallback, useEffect, useRef, useState } from 'react'
import { LuHistory, LuHouse } from 'react-icons/lu'
import { useNavigate } from 'react-router'
import { useClient } from 'urql'
import { ErrorState } from '../components/common/ErrorState'
import { BottomNav } from '../components/domain/BottomNav'
import type { BottomNavItem } from '../components/domain/BottomNav'
import { TournamentListItem } from '../components/domain/TournamentListItem'
import { completedTournamentsQuery } from '../queries/completedTournaments.query'

const PAGE_SIZE = 10

type TournamentItem = {
  id: string
  startDate?: string | null
  endDate?: string | null
  winnerId?: string | null
  winnerName?: string | null
  winnerAvatarFilename?: string | null
  participantCount: number
}

const TournamentHistory = () => {
  const navigate = useNavigate()
  const urqlClient = useClient()
  const [tournaments, setTournaments] = useState<TournamentItem[]>([])
  const [offset, setOffset] = useState(0)
  const [hasMore, setHasMore] = useState(true)
  const [totalCount, setTotalCount] = useState(0)
  const [isLoading, setIsLoading] = useState(true)
  const [isLoadingMore, setIsLoadingMore] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const loadMoreRef = useRef<HTMLDivElement>(null)
  const isFetchingRef = useRef(false)

  const fetchPage = useCallback(
    async (pageOffset: number, append: boolean) => {
      if (isFetchingRef.current) {
        return
      }
      isFetchingRef.current = true

      if (append) {
        setIsLoadingMore(true)
      } else {
        setIsLoading(true)
      }
      setError(null)

      try {
        const result = await urqlClient.query(completedTournamentsQuery, { limit: PAGE_SIZE, offset: pageOffset }, { requestPolicy: 'network-only' }).toPromise()

        if (result.error) {
          setError(result.error.message)
          return
        }

        const page = result.data?.completedTournaments
        if (!page) {
          setError('Failed to load tournaments')
          return
        }

        setTotalCount(page.totalCount)
        setHasMore(page.hasMore)
        setOffset(pageOffset + page.items.length)

        setTournaments((prev) => (append ? [...prev, ...page.items] : page.items))
      } finally {
        isFetchingRef.current = false
        setIsLoading(false)
        setIsLoadingMore(false)
      }
    },
    [urqlClient]
  )

  useEffect(() => {
    fetchPage(0, false)
  }, [fetchPage])

  useEffect(() => {
    document.title = 'Tournament History - Mario Kart Leaderboard'
  }, [])

  useEffect(() => {
    const sentinel = loadMoreRef.current
    if (!sentinel || !hasMore || isLoading || isLoadingMore) {
      return
    }

    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting) {
          fetchPage(offset, true)
        }
      },
      { rootMargin: '200px' }
    )

    observer.observe(sentinel)
    return () => observer.disconnect()
  }, [fetchPage, hasMore, isLoading, isLoadingMore, offset])

  const navItems: BottomNavItem[] = [
    { icon: LuHouse, id: 'home', label: 'Home', onClick: () => navigate('/') },
    { icon: LuHistory, id: 'history', label: 'History', targetId: 'history-section' },
  ]

  if (error && tournaments.length === 0) {
    return <ErrorState message={`Error loading tournament history: ${error}`} onRetry={() => fetchPage(0, false)} />
  }

  return (
    <Box minH="100vh" bg="bg.canvas" pb={{ base: '80px', md: '88px' }}>
      <Container maxW="4xl" py={{ base: 4, lg: 8, md: 6 }}>
        <VStack id="history-section" scrollMarginTop={{ base: 4, md: 6 }} gap={{ base: 6, md: 8 }} align="stretch">
          <VStack align="start" gap={1}>
            <Heading size={{ base: 'lg', md: 'xl' }} color="gray.900">
              Tournament History
            </Heading>
            <Text fontSize={{ base: 'sm', md: 'md' }} color="gray.600">
              {totalCount > 0 ? `${totalCount} completed tournament${totalCount === 1 ? '' : 's'}` : 'Browse past tournaments and champions'}
            </Text>
          </VStack>

          {isLoading ? (
            <VStack gap={4} py={12}>
              <Spinner size="lg" color="brand.500" />
              <Text color="gray.600">Loading tournaments...</Text>
            </VStack>
          ) : tournaments.length === 0 ? (
            <Box p={8} bg="bg.panel" borderRadius="card" borderWidth="1px" borderColor="gray.200" textAlign="center">
              <Text color="gray.600">No completed tournaments yet.</Text>
            </Box>
          ) : (
            <VStack gap={{ base: 3, md: 4 }} align="stretch">
              {tournaments.map((tournament) => (
                <TournamentListItem
                  key={tournament.id}
                  id={tournament.id}
                  startDate={tournament.startDate}
                  endDate={tournament.endDate}
                  winnerName={tournament.winnerName}
                  winnerAvatarFilename={tournament.winnerAvatarFilename}
                  participantCount={tournament.participantCount}
                />
              ))}

              <Box ref={loadMoreRef} h="1px" />

              {isLoadingMore && (
                <VStack gap={2} py={4}>
                  <Spinner size="md" color="brand.500" />
                  <Text fontSize="sm" color="gray.500">
                    Loading more...
                  </Text>
                </VStack>
              )}

              {!hasMore && tournaments.length > 0 && (
                <Text textAlign="center" fontSize="sm" color="gray.500" py={2}>
                  All tournaments loaded
                </Text>
              )}
            </VStack>
          )}
        </VStack>
      </Container>
      <BottomNav items={navItems} />
    </Box>
  )
}

export default TournamentHistory
