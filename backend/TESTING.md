# Testing Guide

This project uses integration tests with testcontainers and Podman.

## Prerequisites

1. **Podman** must be installed and running
2. **Docker compatibility** must be enabled in Podman

### Enabling Podman Docker Compatibility

Run the following command to enable Docker compatibility:
```bash
podman machine set --rootful=false
```

Verify the Docker socket exists:
```bash
ls -la /var/run/docker.sock
```

## Running Tests

To run all tests:
```bash
export TESTCONTAINERS_RYUK_DISABLED=true
cargo test -- --test-threads=1
```

To run a specific test:
```bash
export TESTCONTAINERS_RYUK_DISABLED=true
cargo test test_create_group_mutation -- --nocapture
```

## Test Structure

- `tests/common/setup.rs` - Test database setup with testcontainers
- `tests/common/fixtures.rs` - Helper functions for creating test data
- `tests/graphql_mutations_test.rs` - Tests for GraphQL mutations
- `tests/graphql_queries_test.rs` - Tests for GraphQL queries

## Test Coverage

### Mutation Tests (5 tests)
- `test_create_group_mutation` - Create group and verify JWT
- `test_create_player_mutation` - Create player with authentication
- `test_create_player_without_auth` - Verify authentication is required
- `test_create_tournament_mutation` - Create tournament with dates
- `test_create_tournament_with_invalid_dates` - Verify date validation

### Query Tests (9 tests)
- `test_login_query_success` - Login with valid credentials
- `test_login_query_invalid_credentials` - Login with invalid credentials
- `test_current_group_query` - Fetch authenticated group
- `test_current_group_without_auth` - Verify authentication required
- `test_players_query` - Fetch players for a group
- `test_tournaments_query` - Fetch tournaments
- `test_matches_query` - Fetch matches for a tournament
- `test_matches_query_unauthorized` - Verify cross-group access blocked
- `test_tracks_query` - Fetch all tracks

## Notes

- Tests use PostgreSQL containers via testcontainers
- Each test gets a fresh database with migrations applied
- Ryuk is disabled for Podman compatibility
- Tests run sequentially (`--test-threads=1`) to avoid port conflicts
