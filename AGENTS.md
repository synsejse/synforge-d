# Repository Guidelines (Technical)

## Purpose

`synforge-d` is a Rust + web workspace for orchestrating Fedora RPM package sync/build/publish workflows with a Docker worker runtime and a daemon-served WebUI.

## Project Structure

- `libs/rust/core`: shared API contracts, runtime config, and domain models
- `libs/rust/database`: Diesel persistence, schema, migrations, and PostgreSQL-backed adapters
- `libs/rust/state`: Redis-backed runtime cache/state and short-lived coordination data
- `libs/rust/git-sync`: git/source inspection, mirrors, package materialization, and sync mechanics
- `libs/rust/publish`: object storage, repo publication, and signing
- `libs/rust/worker-host`: daemon-side worker launch, sessions, socket handling, queueing, and build lifecycle
- `libs/rust/worker`: worker runtime logic used by containerized build workers
- `apps/rust/daemon`: main daemon binary serving the API, repo endpoints, docs, and built frontend
- `apps/rust/worker`: worker binary
- `apps/webui`: Vite + React + TanStack Router SPA

Generated artifacts: `target/`, `apps/webui/dist/`, `apps/webui/node_modules/`.

## Architecture Boundaries

- `synforge-core` owns shared models, DTOs, config, validation primitives, and constants.
- `synforge-database` owns Diesel-only persistence concerns, schema, migrations, and PostgreSQL-backed adapters.
- `synforge-state` owns Redis and in-memory runtime state.
- `synforge-git-sync` owns package source sync, git mirrors, source inspection, and package materialization.
- `synforge-publish` owns repo publication, signing, object storage, and repo-file resolution.
- `synforge-worker-host` owns daemon-side worker orchestration and build lifecycle.
- `apps/rust/daemon` owns HTTP transport, startup, background loops, and composition wiring.
- `apps/webui/src/routes` should stay thin TanStack Router route files.
- `apps/webui/src/features` should own feature-specific UI, helpers, and local state.
- `apps/webui/src/lib` should remain cross-feature and generic only.
- `apps/webui/src/generated/api` is the home for generated OpenAPI artifacts.

## Refactor Guardrails

- Non-generated source files should target `< 600` lines.
- Composition roots should target `< 250` lines.
- TanStack Router route files should target `< 50` lines.
- Generated files and lockfiles are excluded from these size targets.
- Keep HTTP routes, page URLs, env/config names, and DB schema stable during structural refactors unless the change explicitly targets one of those contracts.

## WebUI Design System

- Theme tokens live in `apps/webui/src/styles/global.css` as CSS custom properties (`--theme-*`, `--shadow-brutal-*`, `--shadow-card-*`) and are wired to Tailwind v4 via `@theme`.
- Prefer the **semantic utilities** exposed by `@theme` (`bg-surface`, `bg-surface-alt`, `text-muted`, `text-soft`, `border-strong`, `shadow-brutal-sm/md`, `shadow-card-sm/md/lg`, `text-accent-lime`, `text-accent-cyan`, `text-accent-violet`) over `var()` references in className strings, and over raw `bg-zinc-*` / `text-zinc-*` utilities.
- Existing zinc-* utilities (`bg-zinc-950`, `text-zinc-400`, `border-zinc-700` etc.) are bridged to the same tokens via `@layer utilities` `!important` overrides in global.css. Do not introduce new zinc-* usage in new code; the bridge will be removed once feature pages migrate.
- Off-brand color literals (`text-cyan-400`, `border-purple-400`, etc.) are not allowed in components — use `--theme-accent-cyan`, `--theme-accent-violet`, or another brand token.
- Inline shadow literals (`shadow-[2px_2px_0_rgba(...)]`) are not allowed when a token matches; reach for `shadow-brutal-sm/md` for black-cast button shadows and `shadow-card-sm/md/lg` for white-cast surface shadows.
- Reusable section patterns: `.app-section-band` (horizontal gradient header strip) and `.app-section-band-vertical` (sidebar). Use these instead of inlining `bg-gradient-to-* from-zinc-900 to-black`.
- The app is dark-only. Do not add `data-theme` switches or `prefers-color-scheme` branches without first proposing a light-theme phase.

## WebUI Spacing Rhythm

- Vertical page rhythm: `space-y-6` between top-level page sections, `space-y-4` between sub-sections.
- Section padding: `p-5 sm:p-6` for standard cards; `p-6 sm:p-8` for hero/PageHeader containers; `px-6 py-4` for card header bands.
- Grid/flex gaps: `gap-4` is the default, `gap-3` for dense filter rows, `gap-6` for split-pane layouts.
- Deviate only with reason — compact tables, dense toolbars, or fixed-aspect cards may justify smaller scales.

## Current Product/Behavior Notes

- Overview dashboard is intentionally reduced to high-signal cards; detailed system metrics moved to `/statistics`.
- Packages page supports **Refresh All** (manual source refresh queueing across enabled packages).
- Dedicated **Signing** page (`/signing`) manages optional repository/package signing state and key lifecycle.
- Permission model is API-centric:
  - `write` implies `read` for session/API access checks.
  - `repo` remains separate for repository consumption/auth use-cases.
- WebUI deauth trigger is narrow by design: only session endpoint auth failures (`/api/v1/session` 401) force auth reset.
- UI session cookies are opaque Redis-backed tokens; Postgres remains source of truth for accounts, password hashes, permissions, and user metrics.
- Git mirror cache state persistence is DB-backed (`git_mirror_cache_states`) instead of file metadata.
- Repo summary/stat queries were optimized to DB aggregates (Diesel query builder `COUNT DISTINCT`/`SUM` style paths), replacing in-memory aggregation loops.
- Repository metadata signing is optional and default-off. When enabled, `repodata/repomd.xml` is signed and `repomd.xml.asc` is emitted.
- Published RPM/SRPM/debug artifacts can be signed during successful build finalization; per-artifact signing state is persisted in DB (`artifact_signatures`).

## Database and Migrations

- Migrations live in `libs/rust/database/migrations/`.
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
- Frontend: 2 spaces, double quotes in `.tsx`, `PascalCase` components, feature-owned code under `apps/webui/src/features`, generic utilities only under `apps/webui/src/lib`.
- Follow existing patterns over introducing parallel abstractions.

## Change and PR Expectations

- Keep commits short, imperative, and scoped.
- Include schema/migration and runtime impact notes when relevant.
- Include UI screenshots when frontend behavior changes.
