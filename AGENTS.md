# Repository Guidelines (Technical)

## Purpose

`synforge-d` is a Rust + web workspace for orchestrating Fedora RPM package sync/build/publish workflows with a Docker worker runtime and a WebUI.

## Project Structure

- `libs/rust/core`: shared API contracts, runtime config, and domain models
- `libs/rust/orchestrator`: DB-backed orchestration logic, sync/build/repo services, migrations
- `libs/rust/serve`: Axum API + static serving layer
- `libs/rust/worker`: worker runtime logic used by containerized build workers
- `apps/rust/daemon`: main daemon binary
- `apps/rust/worker`: worker binary
- `apps/rust/webui`: backend for serving/proxying the frontend
- `apps/webui`: Astro + React frontend

Generated artifacts: `target/`, `apps/webui/dist/`, `apps/webui/node_modules/`.

## Current Product/Behavior Notes

- Overview dashboard is intentionally reduced to high-signal cards; detailed system metrics moved to `/statistics`.
- Packages page supports **Refresh All** (manual source refresh queueing across enabled packages).
- Dedicated **Signing** page (`/signing`) manages optional repository/package signing state and key lifecycle.
- Permission model is API-centric:
  - `write` implies `read` for session/API access checks.
  - `repo` remains separate for repository consumption/auth use-cases.
- WebUI deauth trigger is narrow by design: only session endpoint auth failures (`/api/v1/session` 401) force auth reset.
- Git mirror cache state persistence is DB-backed (`git_mirror_cache_states`) instead of file metadata.
- Repo summary/stat queries were optimized to DB aggregates (Diesel query builder `COUNT DISTINCT`/`SUM` style paths), replacing in-memory aggregation loops.
- Repository metadata signing is optional and default-off. When enabled, `repodata/repomd.xml` is signed and `repomd.xml.asc` is emitted.
- Published RPM/SRPM/debug artifacts can be signed during successful build finalization; per-artifact signing state is persisted in DB (`artifact_signatures`).

## Database and Migrations

- Migrations live in `libs/rust/orchestrator/migrations/`.
- Performance index migration exists: `00000000000006_performance_indexes`.
- Artifact signature/runtime settings migration exists: `00000000000007_artifact_signatures_and_runtime_settings`.
- Migration execution now logs whether migrations were applied and prints applied migration versions at startup.
- Prefer Diesel query builder patterns over raw SQL strings for application query logic.
- Dynamic runtime settings are DB-backed (`runtime_settings`) and overlaid on top of static config-file defaults.

## Build/Test/Validation

- Rust formatting + validation:
  - `cargo fmt --all`
  - `cargo check --workspace`
  - `cargo test --workspace`
- Frontend build validation:
  - `cd apps/webui && npm run build`
- Local stack:
  - `docker compose build worker-fedora daemon`
  - `docker compose up`

## Coding Conventions

- Rust: 4 spaces, `snake_case` modules/functions, `PascalCase` types, small focused modules.
- Frontend: 2 spaces, double quotes in `.tsx`, `PascalCase` components, utilities under `apps/webui/src/lib`.
- Follow existing patterns over introducing parallel abstractions.

## Change and PR Expectations

- Keep commits short, imperative, and scoped.
- Include schema/migration and runtime impact notes when relevant.
- Include UI screenshots when frontend behavior changes.
