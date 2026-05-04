# syntax=docker/dockerfile:1.7
#
# Two runtime images come out of this Dockerfile:
#   - daemon-runtime  — the long-running synforge daemon + bundled webui
#   - worker-runtime  — short-lived per-build mock executor, launched by
#                       the daemon over docker socket
#
# Build chain:
#   chef → planner → cooker → rust-builder → {daemon,worker}-runtime
#                                          ↘ webui-builder ↗
#   webui-builder is independent of rust-builder so a frontend-only
#   change doesn't trigger any cargo work.

FROM rust:1.94 AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Snapshot the workspace's dependency graph as recipe.json so the
# cooker can build only the deps without the source.
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY apps/rust ./apps/rust
COPY libs/rust ./libs/rust
RUN cargo chef prepare --recipe-path recipe.json

# Pre-cook the dependencies into the cargo target cache mount. Output
# is consumed by rust-builder via the same id=synforge-cargo-target-v2
# cache mount. Source changes don't invalidate this layer.
FROM chef AS cooker
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,id=synforge-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=synforge-cargo-target-v2,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

# Source-build stage. Starts FROM cooker so the cooked deps are the
# cache base — without this the cooker's work would be orphaned.
FROM cooker AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY apps/rust ./apps/rust
COPY libs/rust ./libs/rust
RUN --mount=type=cache,id=synforge-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=synforge-cargo-target-v2,target=/app/target \
    cargo build --release \
      -p synforge-daemon \
      -p synforge-worker-bin \
    && mkdir -p /out \
    && cp /app/target/release/daemon /out/daemon \
    && cp /app/target/release/worker /out/worker

# Webui build is decoupled from rust-builder. The committed
# api-schema.ts is the source of truth for webui types; bumping the
# OpenAPI surface requires regenerating + committing it
# (`npm run generate:api`). With this split, a webui-only edit never
# invalidates the Rust build cache.
FROM node:22-alpine AS webui-builder
WORKDIR /app/apps/webui
COPY apps/webui/package*.json ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY apps/webui ./
RUN npm run build

# --- Runtime images -----------------------------------------------------

FROM fedora:44 AS daemon-runtime
RUN dnf -y --setopt=install_weak_deps=False --nodocs upgrade-minimal \
    && dnf -y --setopt=install_weak_deps=False --nodocs install \
        ca-certificates \
        createrepo_c \
        curl \
        git \
        gnupg2 \
        rpm-sign \
    && dnf clean all

COPY --from=rust-builder /out/daemon /usr/local/bin/daemon
COPY --from=webui-builder /app/apps/webui/dist /opt/synforge/webui

RUN mkdir -p \
    /var/lib/synforge/repo \
    /var/lib/synforge/state/jobs \
    /var/lib/synforge/state/signing \
    /var/lib/synforge/cache \
    /var/lib/synforge/work

# 8080 = HTTP API + webui;  8090 = inter-container worker socket.
EXPOSE 8080 8090
ENTRYPOINT ["/usr/local/bin/daemon"]


FROM fedora:44 AS worker-runtime
RUN dnf -y --setopt=install_weak_deps=False --nodocs upgrade-minimal \
    && dnf -y --setopt=install_weak_deps=False --nodocs install \
        bash \
        createrepo_c \
        dnf-plugins-core \
        git \
        mock \
        python3 \
        rpm-build \
        rpmdevtools \
    && dnf clean all

COPY docker/mock-site-defaults.cfg /etc/mock/site-defaults.cfg
COPY --from=rust-builder /out/worker /usr/local/bin/worker

ENTRYPOINT ["/usr/local/bin/worker"]
