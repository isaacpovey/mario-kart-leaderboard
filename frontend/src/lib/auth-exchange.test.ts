import { describe, expect, it } from 'vitest'
import { isAuthError } from './auth-exchange'

describe('isAuthError', () => {
  it('detects UNAUTHORIZED extension code', () => {
    expect(
      isAuthError({
        graphQLErrors: [{ extensions: { code: 'UNAUTHORIZED' }, message: 'Authentication required' }],
      })
    ).toBe(true)
  })

  it('detects Authentication required message without extensions', () => {
    expect(
      isAuthError({
        graphQLErrors: [{ message: 'Authentication required' }],
      })
    ).toBe(true)
  })

  it('detects Unauthorized message', () => {
    expect(
      isAuthError({
        graphQLErrors: [{ message: 'Unauthorized' }],
      })
    ).toBe(true)
  })

  it('detects HTTP 401 responses', () => {
    expect(
      isAuthError({
        graphQLErrors: [],
        response: { status: 401 } as Response,
      })
    ).toBe(true)
  })

  it('ignores unrelated GraphQL errors', () => {
    expect(
      isAuthError({
        graphQLErrors: [{ message: 'Match not found' }],
      })
    ).toBe(false)
  })
})
