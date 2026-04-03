# synforge-d

`synforge-d` is a Docker-based Fedora package build orchestrator, repo manager, and WebUI for git-backed RPM spec packages.

## Workspace

- `libs/rust/core`: shared package, config, API, and job models
- `libs/rust/orchestrator`: MariaDB-backed state, git source syncing, queueing, polling, Docker worker launcher, managed repo publication
- `libs/rust/serve`: Axum HTTP API and static repo serving
- `libs/rust/worker`: one-shot worker runtime and RPM spec execution
- `apps/rust/daemon`: combined daemon binary
- `apps/rust/worker`: worker container binary
- `apps/rust/webui`: Axum WebUI server for static assets and daemon proxying
- `apps/webui`: Astro/React operations UI

## Local layout

- Compose stack: `docker-compose.yml`
- Compose data volume: `synforge-runtime`
- Runtime configuration: `/var/lib/synforge/config/config.yaml`
- Example config: `config/config.yaml.example`

## Runtime Model

The daemon uses a single `runtime_root` and derives all managed paths from it:

- `runtime_root/metadata/packages/`
- `runtime_root/metadata/repo/fedora/`
- `runtime_root/metadata/tmp/`
- `runtime_root/jobs/<job-id>/`

Per-job state is stored under `jobs/<job-id>/`, including:

- `artifacts/`
- `logs/worker.log`
- `logs/mock-root.log`
- `logs/mock-build.log`
- `logs/mock-state.log`

This layout is intentional: when a build is removed, its owned job files can be removed as a single subtree, and its managed repo files can be unpublished separately.

## Package Sources

Packages are git-backed.

Each package definition stores:

- package name
- git repository URL
- repository-relative spec path
- target Fedora release/arch
- polling behavior and poll interval
- build timeout in seconds

Polling checks the tracked repository for new commits and triggers rebuilds when the source revision changes.

## Worker Transport

Workers are launched by the daemon as Docker containers.

- The daemon talks to workers over the socket-based worker protocol.
- HTTP is reserved for WebUI-facing APIs and repo serving.
- Worker containers do not run as standalone services in normal operation.

## Repository Management

The daemon is the source of truth for managed repo contents.

- successful builds publish files into the managed Fedora repo tree
- published repo files are tracked per build
- deleting a build removes the repo files owned by that build
- the package inventory API exposes package builds plus managed repo file ownership

## Notes

- The daemon expects Docker socket access so it can create worker containers.
- The daemon reads its runtime settings from the YAML config file in the runtime volume. `SYNFORGE_CONFIG_PATH` is only an optional override for the config file location.
- `worker_image` in the config selects the generic Fedora worker image, for example `synforge-worker-fedora:latest`.
- If `session_secret` is missing or still set to the example placeholder, the daemon generates and persists a random one on first run.
- `/api/v1` uses a signed session cookie for the WebUI. `/repo` uses HTTP Basic Auth so Fedora clients can consume packages with handle/password credentials.
- OpenAPI docs are available at `/docs`, with the raw spec at `/openapi.json`.
- First boot now goes through the setup screen, which writes the initial config and creates the first admin account.
- Schema changes should now be treated as forward-migration changes. The earlier reset-style schema churn was only for the pre-stabilization cleanup phase.
- The default compose setup stores daemon data in the named Docker volume `synforge-runtime`. To reset local state, use `docker compose down -v` or remove that volume explicitly.
- The repository is unsigned in this MVP.
- Supported targets are `fedora 43` and `fedora 44` on `x86_64`.
- Build the worker image with `docker compose build worker-fedora daemon`.
