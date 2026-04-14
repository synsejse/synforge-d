# synforge-d

synforge-d helps you keep package builds running and published from one simple dashboard.

You connect source repositories, sync updates, run builds, and view results in one place.

## Quick start

1. Copy `.env.example` to `.env` and set database credentials.
2. Configure storage paths in `.env` (defaults to `./data/*` subdirectories; point to large disks for production).
3. Start the stack with Docker Compose.
4. Open the daemon-served web app at `http://localhost:8080`.
5. Complete first-time setup and create an admin user.
6. Add packages and start syncing/building.

`phpMyAdmin` is available only when started with the `dev-tools` profile.

## Architecture

- `libs/rust/core`: shared contracts, config, and domain types
- `libs/rust/store`: Diesel persistence, schema, and migrations
- `libs/rust/runtime`: build/worker/repo/source runtime services
- `libs/rust/orchestrator`: application layer behind `SynforgeService`
- `libs/rust/serve`: Axum HTTP adapter and static frontend serving
- `apps/rust/daemon`: main process serving the API, docs, repo endpoints, and built WebUI
- `apps/webui`: Astro + React frontend, organized toward `src/features/*` ownership

## What you can do

- Track packages and build activity
- Refresh all package sources from one action
- View build outputs and repository contents
- Use a dedicated statistics page for system-level insights
- Optionally enable repository + package signing from the Signing page

For technical details, architecture, and contributor guidance, see `AGENTS.md`.
