# synforge-d

synforge-d helps you keep package builds running and published from one simple dashboard.

You connect source repositories, sync updates, run builds, and view results in one place.

## Quick start

1. Copy `.env.example` to `.env` and set a non-default PostgreSQL password.
2. Configure storage paths in `.env` (defaults under `/data/*`; point to large disks for production).
3. Start the stack with Docker Compose.
4. Open the daemon-served web app at `http://localhost:8080`.
5. Complete first-time setup and create an admin user.
6. Add packages and start syncing/building.

The default compose stack runs PostgreSQL for relational state and Redis for hot runtime cache/state. Job artifacts, logs, and published repository contents live on the host volumes mounted at `SYNFORGE_RUNTIME_PATH` and `SYNFORGE_WORKER_JOBS_PATH` — point those at large disks in production. The daemon requires Postgres and Redis at startup.

## Architecture

- `libs/rust/core`: shared contracts, config, and domain types
- `libs/rust/database`: Diesel persistence, schema, migrations, and PostgreSQL-backed adapters
- `libs/rust/state`: Redis-backed runtime cache/state and ephemeral coordination data
- `libs/rust/git-sync`: git/source inspection, mirrors, and package sync mechanics
- `libs/rust/worker-host`: worker launch, session/socket protocol, and build execution
- `libs/rust/publish`: repo publication, signing, and repo-file resolution
- `apps/rust/daemon`: main process, service composition, API, docs, repo endpoints, and built WebUI
- `apps/webui`: Vite + React + TanStack Router SPA, organized toward `src/features/*` ownership

## What you can do

- Track packages and build activity
- Refresh all package sources from one action
- View build outputs and repository contents
- Use a dedicated statistics page for system-level insights
- Optionally enable repository + package signing from the Signing page

For technical details, architecture, and contributor guidance, see `AGENTS.md`.
