# Repository Guidelines

## Project Structure & Module Organization

This repository is a mixed Rust and web workspace for `synforge-d`, a Docker-backed Fedora package build orchestrator.

- `libs/rust/core`: shared models, config, API types, runtime paths, and worker protocol
- `libs/rust/orchestrator`: SQLite-backed orchestration, scheduling, repo publication, and migrations
- `libs/rust/serve`: Axum API and static repository serving
- `libs/rust/worker`: reusable worker runtime code
- `apps/rust/daemon`, `apps/rust/worker`, `apps/rust/webui`: binary entry points
- `apps/webui/src`: Astro + React frontend source
- `docker-compose.yml`: local stack for daemon, web UI, and worker image builds
- `docs/`: design and cleanup notes

Treat `target/`, `apps/webui/dist/`, and `apps/webui/node_modules/` as generated output.

## Build, Test, and Development Commands

- `cargo check --workspace`: fast validation for all Rust crates
- `cargo build --workspace`: build all Rust libraries and binaries
- `cargo test --workspace`: run Rust tests when present
- `docker compose build worker-fedora daemon`: build the local worker and daemon images
- `docker compose up`: start the local stack
- `cd apps/webui && npm run dev`: run the Astro frontend with API proxying to `http://localhost:8080`
- `cd apps/webui && npm run build`: produce the frontend bundle

## Coding Style & Naming Conventions

Rust uses the workspace defaults in `Cargo.toml` with 4-space indentation, `snake_case` for modules/functions, and `PascalCase` for types. Prefer small modules with explicit re-exports from `lib.rs`.

Frontend code uses TypeScript, React, and Astro. Follow the existing style: 2-space indentation, double quotes in `.tsx`, and `PascalCase` component names such as `Dashboard.tsx`. Keep utility code under `apps/webui/src/lib`.

Run `cargo fmt --all` before submitting Rust changes. For frontend edits, keep imports grouped and match the surrounding file style.

## Testing Guidelines

There is not yet a broad automated test suite, so add tests with new behavior where practical. Prefer unit tests beside Rust modules with `#[cfg(test)]`, and name tests for the behavior they prove, for example `publishes_repo_files_on_success`.

Before opening a PR, run `cargo check --workspace`, `cargo test --workspace`, and `cd apps/webui && npm run build`.

## Commit & Pull Request Guidelines

Current history uses short, imperative commit subjects, for example `Initial project import`. Keep commit titles concise and specific.

PRs should include:

- a brief summary of the user-visible or operational change
- linked issues or design notes when relevant
- screenshots for `apps/webui` changes
- notes about new environment variables, schema changes, or Docker/runtime impacts
