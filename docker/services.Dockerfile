# syntax=docker/dockerfile:1.7

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
    --mount=type=cache,id=synforge-cargo-target,target=/app/target \
    cargo chef cook --release --recipe-path recipe.json

FROM chef AS rust-builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY apps/rust ./apps/rust
COPY libs/rust ./libs/rust
RUN --mount=type=cache,id=synforge-cargo-registry,target=/usr/local/cargo/registry \
    --mount=type=cache,id=synforge-cargo-target,target=/app/target \
    cargo build --release \
      -p synforge-daemon \
      -p synforge-worker-bin \
      -p synforge-webui \
    && mkdir -p /out \
    && cp /app/target/release/daemon /out/daemon \
    && cp /app/target/release/worker /out/worker \
    && cp /app/target/release/synforge-webui /out/synforge-webui

FROM node:22-alpine AS webui-builder
WORKDIR /app/apps/webui
COPY apps/webui/package*.json ./
RUN --mount=type=cache,target=/root/.npm npm ci
COPY apps/webui ./
RUN npm run build

FROM fedora:44 AS daemon-runtime
RUN dnf -y upgrade-minimal \
    && dnf -y install \
        ca-certificates \
        createrepo_c \
        curl \
        git \
    && dnf clean all

COPY --from=rust-builder /out/daemon /usr/local/bin/daemon

RUN mkdir -p /var/lib/synforge/metadata/database /var/lib/synforge/metadata/packages /var/lib/synforge/metadata/repo /var/lib/synforge/jobs

EXPOSE 8080
EXPOSE 8090
ENTRYPOINT ["/usr/local/bin/daemon"]

FROM fedora:44 AS worker-runtime
RUN dnf -y upgrade-minimal \
    && dnf -y install \
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

FROM fedora:44 AS webui-runtime
RUN dnf -y upgrade-minimal \
    && dnf -y install \
        ca-certificates \
    && dnf clean all

COPY --from=rust-builder /out/synforge-webui /usr/local/bin/synforge-webui
COPY --from=webui-builder /app/apps/webui/dist /opt/synforge/webui

ENV SYNFORGE_WEBUI_LISTEN_ADDR=0.0.0.0:80
ENV SYNFORGE_WEBUI_DAEMON_URL=http://daemon:8080
ENV SYNFORGE_WEBUI_STATIC_DIR=/opt/synforge/webui

EXPOSE 80

ENTRYPOINT ["/usr/local/bin/synforge-webui"]
