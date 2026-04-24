export type SlotPositions = Record<string, string>

/**
 * Apply a slot assertion to a positions map.
 *
 * Sets `slotNumber` to `playerId` (or clears it if null), removing whoever
 * was previously at `slotNumber`. Pure: does not mutate `positions`.
 */
export const applyAssignment = (positions: SlotPositions, slotNumber: number, playerId: string | null): SlotPositions => {
  const next: SlotPositions = { ...positions }
  for (const [otherPlayerId, posStr] of Object.entries(next)) {
    if (Number.parseInt(posStr, 10) === slotNumber) {
      delete next[otherPlayerId]
    }
  }
  if (playerId !== null) {
    next[playerId] = String(slotNumber)
  }
  return next
}
