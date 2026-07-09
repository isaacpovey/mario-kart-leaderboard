#!/usr/bin/env bash
# Startup script for the Mario Kart Leaderboard Cursor Cloud environment.
# Runs after `install`. Brings up the Docker daemon, the dev Postgres, and
# applies migrations + idempotent seed data. The dev servers themselves are
# started by the `terminals` in environment.json.
set -uo pipefail

echo "==> Starting Docker daemon"
if ! sudo service docker start 2>/dev/null; then
  sudo nohup dockerd >/tmp/dockerd.log 2>&1 &
fi

echo "==> Waiting for Docker to be ready"
for _ in $(seq 1 60); do
  if sudo docker info >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
# Let the ubuntu user talk to the daemon without sudo for the rest of setup.
sudo chmod 666 /var/run/docker.sock 2>/dev/null || true

echo "==> Ensuring backend/.env exists"
if [ ! -f backend/.env ]; then
  cp backend/.env.example backend/.env
fi

echo "==> Starting Postgres (docker compose)"
docker compose -f backend/docker-compose.yml up -d --wait postgres

echo "==> Applying migrations and seeding dev data"
( cd backend && cargo run --bin migrate up && cargo run --bin seed ) || \
  echo "WARN: migrate/seed failed (DB may already be initialized) — continuing"

echo "==> Startup complete"
