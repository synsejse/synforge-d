# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

`AGENTS.md` is the authoritative technical guide for this repo (design system, refactor guardrails, product/behavior notes). Read it for anything not covered below; don't duplicate it here.

## What this is

`synforge-d` orchestrates Fedora RPM package **sync → build → publish** workflows. A long-running daemon serves the HTTP API + bundled WebUI and spawns short-lived Docker worker containers (one per build job). Cargo workspace (edition 2024) + a Vite/React SPA.

## Commands

Rust (run from repo root):
- `cargo fmt --all` — format
- `cargo check --workspace` — fast validation
- `cargo test --workspace` — all tests
- `cargo test -p <crate> <test_name>` — single test (e.g. `cargo test -p synforge-core validation`)
- `cargo build --release -p synforge-daemon -p synforge-worker-bin` — release binaries (`daemon`, `worker`)

Frontend (run from `apps/webui/`):
- `npm run dev` — dev server (runs `generate:api` first via `predev`)
- `npm run build` — `tsc --noEmit && vite build` (regenerates TS types from existing `openapi.json` if present)
- `npm run typecheck` — type-check only
- `npm run generate:api` — regenerate the OpenAPI client (see below)

Local stack:
- `docker compose build worker-fedora daemon` — build images (worker is behind the `build-images-only` profile)
- `docker compose up` — starts postgres + redis + daemon; daemon spawns worker containers on demand
- `docker compose --profile debug-tools up` — adds pgAdmin
- WebUI + API at `http://localhost:8080`. Copy `.env.example` → `.env` and set `SYNFORGE_DB_PASSWORD` first.

## Architecture

### Crate dependency graph (libs/rust)
Dependencies point downward; nothing depends upward. Respect these boundaries — they are the main structural invariant.

```
core                      shared API contracts, config, domain models, constants, wire protocol
 ├─ database              Diesel persistence, schema, migrations, Postgres adapters (DieselStore)
 ├─ state                 Redis-backed runtime cache + ephemeral coordination
 ├─ git-sync   (→ database)   git mirrors, source inspection, package materialization
 ├─ publish    (→ database)   repo publication (createrepo_c), signing, repo-file resolution
 ├─ worker-host(→ database, state, git-sync, publish)  daemon-side worker orchestration + build lifecycle
 └─ worker     (→ core ONLY)  build/mock executor logic that runs INSIDE the worker container
```

`synforge-worker` depends on **only** `core` by design — it has no access to the database, Redis, or daemon services because it executes in an isolated container. Keep it that way.

### Two binaries
- `apps/rust/daemon` (`daemon` bin) — depends on every lib + `worker-host`. Thin `main.rs`; all wiring lives in `SynforgeService` (`src/service/`). HTTP layer in `src/http/`. Also ships the `openapi-export` bin.
- `apps/rust/worker` (`worker` bin) — wraps `synforge-worker`; runs inside the per-build container.

### Daemon ↔ worker model
The daemon launches **one worker container per build job** over the mounted Docker socket (via `bollard`). The worker connects **back** to the daemon's worker socket on port **8090** (TCP, length-delimited framed frames carrying `WorkerWireMessage`, defined in `core`'s wire protocol). Port 8090 is **compose-network-only — never host-bound by design**. Build orchestration, sessions, and the socket listener live in `libs/rust/worker-host` (`worker_socket.rs`, `sessions.rs`, `launcher/`, `build_runner.rs`, `job_lifecycle.rs`).

### Data stores
- **Postgres** (Diesel) — source of truth: accounts, password hashes, permissions, packages, build jobs, runtime settings, signature state. Migrations in `libs/rust/database/migrations/` (sequentially numbered `00000000000000_*`).
- **Redis** (`state`) — hot runtime cache/state and **opaque session-cookie tokens** (the cookie is just a Redis key; Postgres stays authoritative for the account behind it).
- **Local filesystem runtime tree** — `/var/lib/synforge` (repo workspace for `createrepo_c` + signing keys + cache) and `/var/lib/synforge-worker-jobs` (per-job artifacts/logs, shared with worker containers).

### WebUI ↔ API contract (generated, do not hand-edit)
The TS API client is generated from the Rust API:
1. `cargo run -p synforge-daemon --bin openapi-export` → `src/generated/api/openapi.json` (utoipa-derived OpenAPI from the Rust handlers)
2. `openapi-typescript` → `src/generated/api/api-schema.ts`

When you change Rust API DTOs/handlers, regenerate with `npm run generate:api`. Everything under `apps/webui/src/generated/api` is generated output.

### WebUI layout
- `src/routes/` — thin TanStack Router route files (target < 50 lines)
- `src/features/<feature>/` — feature-owned UI, helpers, local state (auth, dashboard, jobs, packages, repository, settings, setup, signing, statistics, users)
- `src/lib/` — cross-feature generic utilities only
- `src/styles/global.css` — Tailwind v4 theme tokens (see AGENTS.md "WebUI Design System")

## Conventions worth knowing

- **File-size targets** (non-generated): source < 400 lines, composition roots < 250, route files < 50. Generated files/lockfiles exempt.
- **Diesel query builder over raw SQL** for application query logic.
- **Stable contracts during refactors**: don't change HTTP routes, page URLs, env/config names, or DB schema during structural refactors unless that contract is the explicit target.
- Runtime settings are DB-backed (`runtime_settings`) and overlaid on static config-file defaults.
- App is **dark-only** — no `data-theme` / `prefers-color-scheme` branches without proposing a light-theme phase first.
- Rust: 4-space indent, `snake_case` modules/fns, `PascalCase` types. Frontend: 2-space, double quotes in `.tsx`, `PascalCase` components.
