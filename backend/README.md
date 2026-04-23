# Mario Kart Leaderboard - Backend

A Rust GraphQL API backend for tracking Mario Kart tournament leaderboards with JWT authentication.

## Tech Stack

- **Rust** - Systems programming language
- **Axum** - Modern web framework
- **async-graphql** - GraphQL server with async support
- **SQLx** - Async SQL toolkit with compile-time query checking
- **PostgreSQL** - Database
- **Argon2** - Password hashing
- **JWT** - Token-based authentication

## Prerequisites

- **Rust** 1.70+ ([Install Rust](https://rustup.rs/))
- **PostgreSQL** 14+ (or Podman/Docker for tests)
- **sqlx-cli** (for migrations)

```bash
# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres
```

### Optional: devbox

If you have [devbox](https://www.jetpack.io/devbox) installed, `devbox shell` from the repo root provides pinned versions of Rust, Node, `pnpm`, and `sqlx-cli` without needing to install any of them manually. First invocation warms the Nix store and can take several minutes; subsequent shells are near-instant.

## Setup

### 1. Environment Configuration

Create a `.env` file in the `backend/` directory:

```env
DATABASE_URL=postgresql://postgres:password@localhost:5432/mario_kart
JWT_SECRET=your-secret-key-change-in-production
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
ENABLE_PLAYGROUND=true
```

### 2. Database Setup

#### Option A: Install Postgres locally

Create the database:

```bash
# Using psql
createdb mario_kart

# Or using SQL
psql -U postgres -c "CREATE DATABASE mario_kart;"
```

#### Option B: Run Postgres via Docker Compose

Alternatively, start the bundled Postgres 16 service from `backend/docker-compose.yml`:

```bash
docker compose -f backend/docker-compose.yml up -d postgres
# or, if you prefer Podman:
podman compose -f backend/docker-compose.yml up -d postgres
```

Then copy the matching `.env` template:

```bash
cp .env.example .env
```

The credentials in `.env.example` match the compose service defaults. Data is persisted in the named volume `mario-kart-pgdata`; use `docker compose ... down -v` to wipe it. If port `5432` is already taken by a host-native Postgres, stop that service first or change the host port mapping in `docker-compose.yml` and update `DATABASE_URL` in your `.env` accordingly.

### 3. Run Migrations

Run all pending migrations:

```bash
# Using the migrate binary
cargo run --bin migrate up

# Or using sqlx-cli directly
sqlx migrate run
```

Other migration commands:

```bash
# Revert last migration
cargo run --bin migrate down

# Check migration status
cargo run --bin migrate status

# Create new migration
cargo run --bin migrate add <migration_name>
```

## Running the Dev Server

### Start the server

```bash
cargo run
```

The server will start at `http://localhost:8080`

- **GraphQL Endpoint**: `http://localhost:8080/graphql`
- **GraphQL Playground**: `http://localhost:8080/` (if `ENABLE_PLAYGROUND=true`)

### Development with auto-reload

Install and use `cargo-watch`:

```bash
cargo install cargo-watch
cargo watch -x run
```

## API Documentation

### Authentication

The API uses JWT tokens for authentication. Include the token in the `Authorization` header:

```
Authorization: Bearer <your-jwt-token>
```

### Core Operations

**Create a Group** (returns JWT token)
```graphql
mutation {
  createGroup(name: "My Group", password: "secret123")
}
```

**Login** (returns JWT token)
```graphql
query {
  login(groupId: "uuid-here", password: "secret123")
}
```

**Get Current Group**
```graphql
query {
  currentGroup {
    id
    name
    players {
      id
      name
      eloRating
    }
  }
}
```

**Create Player**
```graphql
mutation {
  createPlayer(name: "Player Name") {
    id
    name
    eloRating
  }
}
```

**Create Tournament**
```graphql
mutation {
  createTournament(
    startDate: "2024-01-01"
    endDate: "2024-01-07"
  ) {
    id
    startDate
    endDate
  }
}
```

**Get Leaderboard**
```graphql
query {
  tournaments {
    id
    leaderboard {
      playerName
      eloRating
      totalScore
    }
  }
}
```

## Testing

### Prerequisites for Tests

Tests use **testcontainers** with **Podman** (or Docker):

1. Install Podman: [Podman Installation Guide](https://podman.io/getting-started/installation)
2. Enable Docker compatibility:
   ```bash
   podman machine set --rootful=false
   ```

### Run Tests

```bash
# Set required environment variable
export TESTCONTAINERS_RYUK_DISABLED=true

# Run all tests (sequential to avoid port conflicts)
cargo test -- --test-threads=1

# Run specific test
cargo test test_create_group_mutation -- --nocapture

# Run with verbose output
cargo test -- --test-threads=1 --nocapture
```

See [TESTING.md](./TESTING.md) for detailed testing documentation.

## Project Structure

```
backend/
├── src/
│   ├── bin/
│   │   └── migrate.rs           # Database migration CLI
│   ├── graphql/                 # GraphQL layer (feature-based)
│   │   ├── auth/                # Authentication feature
│   │   │   ├── queries.rs       # login query
│   │   │   └── mutations.rs     # createGroup mutation
│   │   ├── groups/              # Groups feature
│   │   │   ├── queries.rs       # currentGroup query
│   │   │   ├── types.rs         # Group GraphQL type
│   │   │   └── loaders.rs       # DataLoader for batching
│   │   ├── players/             # Players feature
│   │   ├── tournaments/         # Tournaments feature
│   │   ├── matches/             # Matches feature
│   │   ├── tracks/              # Tracks feature
│   │   ├── context.rs           # GraphQL context
│   │   └── schema.rs            # Schema builder
│   ├── models/                  # Database models
│   ├── handlers/                # HTTP handlers
│   ├── middleware/              # Auth middleware
│   ├── auth.rs                  # JWT & password utilities
│   ├── config.rs                # Configuration
│   ├── db.rs                    # Database connection
│   ├── error.rs                 # Error types
│   ├── lib.rs                   # Library exports
│   └── main.rs                  # Application entry point
├── migrations/                  # SQL migrations
├── tests/                       # Integration tests
│   ├── common/                  # Test utilities
│   ├── graphql_queries_test.rs
│   └── graphql_mutations_test.rs
├── Cargo.toml
├── README.md
└── TESTING.md
```

## Development

### Code Style

This project follows functional programming principles:
- Prefer `const` over `let`
- Avoid mutation (use `map`, `filter`, `fold` instead of loops)
- Use `Result` and `Option` for error handling
- Use typed errors (no `anyhow`)

### GraphQL Schema

The GraphQL schema is organized by features using `MergedObject`. Each feature contains:
- **queries.rs** - Query resolvers
- **mutations.rs** - Mutation resolvers (if applicable)
- **types.rs** - GraphQL object types
- **loaders.rs** - DataLoaders for N+1 prevention (if applicable)

### Database

- Migrations are in `migrations/`
- Models use SQLx with `#[derive(FromRow)]`
- All queries are checked at compile time
- Database pool is shared via Axum `Extension`

### Adding a New Feature

1. Create a new folder in `src/graphql/<feature>/`
2. Add `queries.rs`, `mutations.rs`, `types.rs`, `loaders.rs` as needed
3. Create a `mod.rs` to export public APIs
4. Add the feature module to `src/graphql/mod.rs`
5. Include queries/mutations in `src/graphql/schema.rs` using `MergedObject`

## Building for Production

```bash
# Build optimized binary
cargo build --release

# Binary will be at
./target/release/mario-kart-leaderboard-backend
```

Set `ENABLE_PLAYGROUND=false` in production.

## Troubleshooting

### Database Connection Issues

```bash
# Check PostgreSQL is running
pg_isready

# Test connection
psql -U postgres -d mario_kart -c "SELECT 1;"
```

### Migration Issues

```bash
# Check migration status
sqlx migrate info

# Force migration version (careful!)
sqlx migrate revert
```

### Test Issues

```bash
# Verify Podman is running
podman machine list

# Check Docker compatibility socket
ls -la /var/run/docker.sock
```

## License

[Your License Here]

## Contributing

[Your Contributing Guidelines Here]
