# syntax=docker/dockerfile:1.7
#
# Build chain: chef → planner → cooker → rust-builder → {daemon,worker}-runtime
#                                      ↘ webui-builder ↗
# webui-builder is parallel to rust-builder so webui-only edits don't
# invalidate the cargo cache. api-schema.ts is committed; regenerate
# via `npm run generate:api` when the OpenAPI surface changes.

FROM rust:1.94 AS chef
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY apps/rust ./apps/rust
COPY libs/rust ./libs/rust
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cooker
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,id=synforge-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=synforge-cargo-target-v2,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

# FROM cooker so the cooked deps are the cache base for the source build.
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

FROM node:22.12.0-alpine AS webui-builder
WORKDIR /app/apps/webui
COPY apps/webui/package*.json ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY apps/webui ./
RUN npm run build

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

# 8080 = HTTP API + webui; 8090 = worker socket (compose-network only).
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
