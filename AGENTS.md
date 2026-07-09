# AGENTS.md

## Cursor Cloud specific instructions

This repo is a Mario Kart Leaderboard app with two services:

- **backend/** — Rust (edition 2024, toolchain pinned in `rust-toolchain.toml`) Axum + async-graphql API backed by PostgreSQL. Runs on `http://localhost:8080` (`/graphql` endpoint, playground at `/`).
- **frontend/** — React 19 + Vite + urql SPA. Runs on `http://localhost:5173` and defaults to `http://localhost:8080/graphql` when `VITE_GRAPHQL_URL` is unset (do NOT copy `frontend/.env.example` — it points at the wrong port `8000`).

The environment is defined in code by `.cursor/environment.json` + `.cursor/Dockerfile` (not a snapshot). On boot Cursor:

1. builds the `.cursor/Dockerfile` base image (system deps: Rust 1.95.0, Node 22, pnpm 10.20.0, Docker + fuse-overlayfs/iptables-legacy);
2. runs `install` to refresh deps (`pnpm install` in `frontend/`, `cargo fetch` in `backend/`);
3. runs `start` (`.cursor/start.sh`): starts the Docker daemon, brings up the dev Postgres via docker-compose, creates `backend/.env` from the example if missing, and applies migrations + idempotent seed data;
4. launches the `backend` and `frontend` dev servers as `terminals`.

So in a fresh Cloud environment the DB, migrations, seed, and both dev servers are already running. The notes below are for restarting/troubleshooting individual pieces.

### Postgres + Docker (required for running the backend and backend tests)

- If the Docker daemon is not running, `start.sh` starts it; to start manually: `sudo service docker start` (or `sudo nohup dockerd >/tmp/dockerd.log 2>&1 &`). The daemon uses `fuse-overlayfs` (`/etc/docker/daemon.json`) for the nested-container environment.
- The `ubuntu` user is in the `docker` group, so `docker` works without `sudo`. If you hit a socket permission error, `sudo chmod 666 /var/run/docker.sock`.
- Start Postgres manually (only the `postgres` service; `aspire-dashboard` is optional/not needed):
  `docker compose -f backend/docker-compose.yml up -d --wait postgres`
- `backend/.env` is gitignored; `start.sh` copies it from `backend/.env.example` (values match the docker-compose Postgres: user `postgres`, password `password`, db `mario_kart`, port `5432`). `JWT_SECRET` must be >= 32 chars.

### Backend

- Migrate + seed dev data (run automatically by `start.sh`): `cd backend && cargo run --bin migrate up && cargo run --bin seed`. The seed is idempotent and prints an auto-login URL, login credentials (`Dev Group` / `devpassword`), and a JWT for the playground.
- Dev server auto-starts in the `backend` terminal; to run manually: `cd backend && cargo run --bin mario-kart-leaderboard-backend`.
- Lint: `cd backend && cargo clippy --all-targets` (only harmless dead-code warnings currently).
- Tests use **testcontainers**, which spins up its own throwaway Postgres via Docker — they do NOT use the docker-compose Postgres, but the Docker daemon MUST be running. Run:
  `cd backend && TESTCONTAINERS_RYUK_DISABLED=true cargo test -- --test-threads=1` (Ryuk must be disabled and tests run single-threaded).
- No sqlx compile-time query macros are used, so the backend builds without a live database (only running/seeding needs Postgres). First build is slow (~4 min).

### Frontend

- Install: `cd frontend && pnpm install`. pnpm blocks the `esbuild` build script by default; this is harmless — Vite dev/build/test all work regardless (esbuild ships its native binary via an optional dependency).
- Dev server auto-starts in the `frontend` terminal; to run manually: `cd frontend && pnpm dev`. Lint: `pnpm lint`. Build: `pnpm build`. Tests: `pnpm test` (vitest).
- `pnpm graphql:generate` regenerates GraphQL types and needs the backend running first; it is not required for normal dev.
