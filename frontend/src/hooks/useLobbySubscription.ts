import { useSubscription } from 'urql'
import { lobbyUpdatedSubscription } from '../subscriptions/lobbyUpdated.subscription'

export const useLobbySubscription = () => {
  const [result] = useSubscription({ query: lobbyUpdatedSubscription })
  return result
}
