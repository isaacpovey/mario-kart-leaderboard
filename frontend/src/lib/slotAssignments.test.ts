import { describe, expect, it } from 'vitest'
import { applyAssignment } from './slotAssignments'

describe('applyAssignment', () => {
  it('assigns a player to an empty slot', () => {
    expect(applyAssignment({}, 5, 'alice')).toEqual({ alice: '5' })
  })

  it('moves a player from one slot to another, clearing the old slot', () => {
    expect(applyAssignment({ alice: '3' }, 5, 'alice')).toEqual({ alice: '5' })
  })

  it('assigning a slot clears whoever was previously at that slot', () => {
    expect(applyAssignment({ alice: '5', bob: '6' }, 5, 'bob')).toEqual({ bob: '5' })
  })

  it('null playerId clears the slot (assign no one)', () => {
    expect(applyAssignment({ alice: '5', bob: '6' }, 5, null)).toEqual({ bob: '6' })
  })

  it('null playerId on an empty slot is a no-op', () => {
    expect(applyAssignment({ alice: '5' }, 7, null)).toEqual({ alice: '5' })
  })

  it('is idempotent when re-applied with the same inputs', () => {
    const once = applyAssignment({ alice: '3' }, 5, 'alice')
    expect(applyAssignment(once, 5, 'alice')).toEqual({ alice: '5' })
  })

  it('does not mutate its input', () => {
    const input = { alice: '3' }
    applyAssignment(input, 5, 'alice')
    expect(input).toEqual({ alice: '3' })
  })
})
