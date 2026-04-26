export type SlotPositions = Record<string, string>

/**
 * Apply a slot assertion to a positions map.
 *
 * Sets `slotNumber` to `playerId` (or clears it if null), removing whoever
 * was previously at `slotNumber`. Pure: returns a new map; does not mutate.
 */
export const applyAssignment = (positions: SlotPositions, slotNumber: number, playerId: string | null): SlotPositions => {
  const cleared = Object.fromEntries(Object.entries(positions).filter(([, posStr]) => Number.parseInt(posStr, 10) !== slotNumber))
  return playerId !== null ? { ...cleared, [playerId]: String(slotNumber) } : cleared
}
