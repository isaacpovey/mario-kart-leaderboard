import { useSubscription } from 'urql'
import { lobbyUpdatedSubscription } from '../subscriptions/lobbyUpdated.subscription'

export const useLobbySubscription = (enabled: boolean) => {
  const [result] = useSubscription({
    query: lobbyUpdatedSubscription,
    pause: !enabled,
  })

  return result
}
