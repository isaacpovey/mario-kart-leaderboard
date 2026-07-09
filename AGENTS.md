# AGENTS.md

## Cursor Cloud specific instructions

This repo is a Mario Kart Leaderboard app with two services:

- **backend/** — Rust (edition 2024, toolchain pinned in `rust-toolchain.toml`) Axum + async-graphql API backed by PostgreSQL. Runs on `http://localhost:8080` (`/graphql` endpoint, playground at `/`).
- **frontend/** — React 19 + Vite + urql SPA. Runs on `http://localhost:5173` and defaults to `http://localhost:8080/graphql` when `VITE_GRAPHQL_URL` is unset (do NOT copy `frontend/.env.example` — it points at the wrong port `8000`).

The update script keeps dependencies fresh (`pnpm install` in `frontend/`, `cargo fetch` in `backend/`). Everything below is startup/runtime context that is NOT handled automatically.

### Postgres + Docker (required for running the backend and backend tests)

- Docker is pre-installed in the snapshot but the daemon is NOT auto-started. Start it once per session:
  `sudo nohup dockerd > /tmp/dockerd.log 2>&1 &` (daemon is configured for `fuse-overlayfs` with the containerd snapshotter disabled in `/etc/docker/daemon.json`).
- The `ubuntu` user is in the `docker` group, so `docker` works without `sudo` once the daemon is up. If you hit a socket permission error, `sudo chmod 666 /var/run/docker.sock`.
- Start Postgres (only the `postgres` service; the `aspire-dashboard` service is optional/not needed):
  `docker compose -f backend/docker-compose.yml up -d --wait postgres`
- `backend/.env` is gitignored. Create it from `backend/.env.example` (values there already match the docker-compose Postgres: user `postgres`, password `password`, db `mario_kart`, port `5432`). `JWT_SECRET` must be >= 32 chars.

### Backend

- Migrate + seed dev data: `cd backend && cargo run --bin migrate up && cargo run --bin seed`. The seed is idempotent and prints an auto-login URL, login credentials (`Dev Group` / `devpassword`), and a JWT for the playground.
- Run dev server: `cd backend && cargo run --bin mario-kart-leaderboard-backend`.
- Lint: `cd backend && cargo clippy --all-targets` (only harmless dead-code warnings currently).
- Tests use **testcontainers**, which spins up its own throwaway Postgres via Docker — they do NOT use the docker-compose Postgres, but the Docker daemon MUST be running. Run:
  `cd backend && TESTCONTAINERS_RYUK_DISABLED=true cargo test -- --test-threads=1` (Ryuk must be disabled and tests run single-threaded).
- No sqlx compile-time query macros are used, so the backend builds without a live database (only running/seeding needs Postgres). First build is slow (~4 min).

### Frontend

- Install: `cd frontend && pnpm install`. pnpm blocks the `esbuild` build script by default; this is harmless — Vite dev/build/test all work regardless (esbuild ships its native binary via an optional dependency).
- Dev server: `cd frontend && pnpm dev`. Lint: `pnpm lint`. Build: `pnpm build`. Tests: `pnpm test` (vitest).
- `pnpm graphql:generate` regenerates GraphQL types and needs the backend running first; it is not required for normal dev.
