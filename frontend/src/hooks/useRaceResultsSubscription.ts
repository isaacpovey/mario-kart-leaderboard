import { useSubscription } from 'urql'
import { raceResultsUpdatedSubscription } from '../subscriptions/raceResults.subscription'

export const useRaceResultsSubscription = (tournamentId: string | null | undefined) => {
  const [result] = useSubscription({
    pause: !tournamentId,
    query: raceResultsUpdatedSubscription,
    variables: { tournamentId: tournamentId || '' },
  })

  return result
}
