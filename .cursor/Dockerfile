# Base image for the Mario Kart Leaderboard Cursor Cloud environment.
# Installs system-level dependencies only; project dependencies are installed
# by the `install` command in environment.json, and services are started by the
# `start` command / `terminals`.
#
# NOTE: Do NOT COPY the repository into this image — Cursor checks out the
# correct commit into the workspace itself.

FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# Core system tooling (build-essential/pkg-config/libssl-dev are needed to
# compile some Rust crates; postgresql-client is handy for DB debugging).
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates \
      curl \
      gnupg \
      git \
      sudo \
      build-essential \
      pkg-config \
      libssl-dev \
      postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Non-root `ubuntu` user with passwordless sudo (present in ubuntu:24.04 as UID 1000).
RUN id -u ubuntu >/dev/null 2>&1 || useradd -m -s /bin/bash ubuntu
RUN groupadd -f docker && usermod -aG docker ubuntu && usermod -aG sudo ubuntu
RUN echo "ubuntu ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/ubuntu && chmod 0440 /etc/sudoers.d/ubuntu

# ---------------------------------------------------------------------------
# Docker (required for testcontainers-based backend tests and the dev Postgres
# started from backend/docker-compose.yml). Runs inside a nested container, so
# it needs fuse-overlayfs + iptables-legacy. Pinned to 28.x so the default
# storage driver works with fuse-overlayfs without extra tweaks.
# ---------------------------------------------------------------------------
RUN install -m 0755 -d /etc/apt/keyrings \
    && curl --retry 3 --retry-delay 5 -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg \
    && chmod a+r /etc/apt/keyrings/docker.gpg \
    && echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(. /etc/os-release && echo "$VERSION_CODENAME") stable" > /etc/apt/sources.list.d/docker.list \
    && apt-get update \
    && apt-get install -y \
      docker-ce=5:28.5.2-1~ubuntu.24.04~noble \
      docker-ce-cli=5:28.5.2-1~ubuntu.24.04~noble \
      containerd.io \
      docker-buildx-plugin \
      docker-compose-plugin \
      fuse-overlayfs \
      iptables \
    && rm -rf /var/lib/apt/lists/*
RUN mkdir -p /etc/docker && printf '%s\n' '{' '  "storage-driver": "fuse-overlayfs"' '}' > /etc/docker/daemon.json
RUN update-alternatives --set iptables /usr/sbin/iptables-legacy \
    && update-alternatives --set ip6tables /usr/sbin/ip6tables-legacy

# ---------------------------------------------------------------------------
# Node.js 22 + pnpm 10.20.0 (matches devbox.json / packageManager pin).
# ---------------------------------------------------------------------------
RUN curl --retry 3 --retry-delay 5 -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y nodejs \
    && rm -rf /var/lib/apt/lists/* \
    && npm install -g pnpm@10.20.0

# ---------------------------------------------------------------------------
# Rust toolchain, pinned to 1.95.0 (rust-toolchain.toml) with rustfmt + clippy.
# Installed for the ubuntu user so cargo caches live under its home.
# ---------------------------------------------------------------------------
USER ubuntu
ENV RUSTUP_HOME=/home/ubuntu/.rustup \
    CARGO_HOME=/home/ubuntu/.cargo \
    PATH=/home/ubuntu/.cargo/bin:/usr/local/bin:/usr/bin:/bin
# The `default` rustup profile already includes rustfmt and clippy.
RUN curl --retry 3 --retry-delay 5 --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
      | sh -s -- -y --default-toolchain 1.95.0 --profile default \
    && echo 'export PATH=/home/ubuntu/.cargo/bin:$PATH' >> /home/ubuntu/.bashrc \
    && /home/ubuntu/.cargo/bin/rustc --version

USER root
