import { atomWithStorage } from 'jotai/utils'

// Maps groupId -> playerId representing the "me" identity on this device.
// Keying by groupId means switching groups clears the stored identity for the new group.
export const mePlayerIdByGroupAtom = atomWithStorage<Record<string, string>>('mario-kart/me-player-id-by-group', {})
