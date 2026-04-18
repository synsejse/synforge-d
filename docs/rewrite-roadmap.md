# Rewrite Roadmap

## Target Architecture

The `app` and `orchestrator` crates are gone. The remaining rewrite target is to keep draining daemon-local service glue into the owning service crates so the daemon stays a composition root instead of becoming the next umbrella layer.

- `libs/rust/core`
  Shared config, DTOs, worker protocol, common model types, and small validation primitives.
- `libs/rust/database`
  Everything PostgreSQL-related: Diesel schema, records, queries, migrations, and persistence adapters.
- `libs/rust/state`
  Everything Redis- and memory-related: runtime cache, progress state, throttles, short-lived coordination state.
- `libs/rust/git-sync`
  Git mirrors, source inspection, package definition materialization, sync progress, refresh planning.
- `libs/rust/publish`
  Repo publication, repodata generation, signing, object storage, published artifact handling.
- `libs/rust/worker-host`
  Daemon-side worker orchestration: queueing, Docker launch, worker sessions, job lifecycle around container builds.
- `libs/rust/worker`
  In-container execution only: clone, spec parse, mock/rpmbuild, logging, artifact collection.
- `apps/rust/daemon`
  Main executable, HTTP routes, startup, background loops, composition root.

Long-term, daemon-local service glue should shrink to startup/background wiring plus thin HTTP-facing methods.

## Current To Target Mapping

- the old `store` + `database` split has been collapsed into `libs/rust/database`
- the old `app/src/git_sync/*` logic now lives in `libs/rust/git-sync`
- the old `app/src/repo/*` logic now lives in `libs/rust/publish`
- the old `app/src/builds/*` logic now lives in `libs/rust/worker-host`
- old orchestrator runtime cache/progress helpers
- any other Redis/in-memory progress/cache helpers
  -> `libs/rust/state`
- common pagination, config schema/runtime setting parsing, and user input validation live in `libs/rust/core`
- password hashing, account authentication/authorization, user CRUD, and user metrics live behind `libs/rust/database::accounts::AccountService`
- UI session cookies are opaque Redis-backed tokens owned by `libs/rust/state`; account state remains Postgres-backed
- `session_secret` is no longer active config; old stored runtime values are ignored during config overlay
- worker-image mock-chroot discovery, job usage sampling, Docker host hardware probing, and other Docker/session mechanics live in `libs/rust/worker-host`
- repo inventory/summary trait adapters live with `libs/rust/publish` service glue instead of daemon wrappers
- repo signing dependencies are now explicit `RepoSigningDeps` objects instead of publish trait impls on `SynforgeService`
- job retry dependencies are now explicit `JobRetryDeps` objects instead of worker-host trait impls on `SynforgeService`
- the old `libs/rust/orchestrator` crate has been collapsed into `apps/rust/daemon/src/service`, which should now keep shrinking toward a facade/composition root

## Boundary Rules

- `core` must not depend on Diesel, Redis, Docker, Git process helpers, or object storage.
- `database` owns all PostgreSQL access. No direct Diesel usage outside `database`.
- `database` owns durable account state and password verification against Postgres records.
- `state` owns Redis keys, cache TTLs, UI session tokens, in-memory snapshots, progress slots, and runtime coordination state.
- `git-sync` owns source-of-truth package sync behavior. It should not own repo publication or worker container mechanics.
- `publish` owns repo mutations, signing, MinIO/S3 interaction, and repo-file resolution.
- `worker-host` owns daemon-side build execution mechanics. It should not own in-container `mock`/`rpmbuild` logic.
- `worker` stays container-only.
- `daemon` should wire services and expose HTTP only. It should not become the new “god crate”.

## Phase Plan

### Phase 1: Rename To Final Crate Names

Create the final crate names before more code movement:

- `libs/rust/database` stays `libs/rust/database`
- `libs/rust/git-sync` stays `libs/rust/git-sync`
- `libs/rust/worker-host` stays `libs/rust/worker-host`
- create `libs/rust/state`

Keep behavior unchanged in this phase. This is naming and workspace cleanup only.

### Phase 2: Build `database`

Collapse PostgreSQL ownership into one crate. This is complete once the old `store` crate is deleted and all Diesel code lives under `database`.

- keep Diesel traits/impls/records/schema/migrations in `database`
- keep the PostgreSQL adapters in `database`
- keep Postgres-backed account service logic in `database`
- do not reintroduce a second PostgreSQL crate

End state:

- one PostgreSQL crate
- no direct `DieselStore` usage outside `database`
- no standalone `store` crate
- daemon does not own password hashing or account authorization logic

### Phase 3: Build `state`

Extract Redis and ephemeral runtime state:

- completed: runtime cache and mock-chroot cache persistence live in `state`
- completed: refresh/signing progress state lives in `state`
- completed: job-usage sample caching lives in `state`
- completed: UI session cookies are opaque Redis-backed tokens stored by `state`
- centralize Redis key naming and TTL policy

End state:

- one crate owns Redis and in-memory runtime state
- no Redis client usage outside `state`
- no self-contained signed UI session cookies; browser cookies carry opaque Redis session tokens
- no ad hoc `Arc<Mutex<Option<_>>>` state living in unrelated service crates unless it is strictly local and short-lived

### Phase 4: Build `git-sync`

Merge service logic and implementation:

- completed: the old `app/src/git_sync/*` slice was moved into `git-sync`
- keep the existing source mechanics from current `source`
- move package refresh orchestration, source inspection, package materialization, repository browse flows
- move package CRUD logic that is fundamentally source/sync owned

End state:

- no separate `app::git_sync`
- `git-sync` exposes a small service-facing API directly

### Phase 5: Build `publish`

Merge repo app logic into repo implementation:

- completed: the old `app/src/repo/*` slice was moved into `publish`
- keep repo manager, signing, storage, and artifact publication in the same crate
- move repo summary/file resolution/reconcile/use-case logic there

End state:

- no separate `app::repo`
- `publish` owns all repo/signing/object-storage behavior and service verbs

### Phase 6: Build `worker-host`

Merge daemon-side build orchestration:

- completed: the old `app/src/builds/*` slice was moved into `worker-host`
- completed: worker-image mock-chroot discovery moved into `worker-host`
- completed: Docker job usage sampling moved into `worker-host`
- completed: Docker host hardware probing moved into `worker-host`
- keep queueing, Docker launch, worker socket/session handling, and job lifecycle together
- define a narrow API for build actions: queue, retry, kill, finalize, abort unfinished jobs

End state:

- no separate `app::builds`
- `worker-host` owns daemon-side build orchestration
- `worker` remains container-only

### Phase 7: Remove `app`

Completed:

- `libs/rust/app` is gone from the workspace
- imports now point at `git-sync`, `publish`, and `worker-host` directly

### Phase 8: Remove `orchestrator`

Completed at the crate boundary:

- `libs/rust/orchestrator` is gone from the workspace
- `SynforgeService` now lives under `apps/rust/daemon/src/service`
- daemon-local service glue has been reduced by moving common config/validation/pagination to `core`, Docker worker host mechanics to `worker-host`, and repo query adapters to `publish`
- auth glue has been reduced by moving account behavior to `database` and UI session storage to `state`
- publish and worker-host use-case traits are no longer implemented directly on `SynforgeService`; daemon builds explicit dependency objects for those flows
- daemon-local service glue still needs further draining into `git-sync`, `publish`, `worker-host`, `state`, and `database`

End state:

- service crates own their own use-case logic
- daemon wires them together
- no extra umbrella crate

## Next Concrete Moves

1. Split `apps/rust/daemon/src/service/packages/deps/*` by ownership: source/materialization adapters toward `git-sync`, build queue adapters toward `worker-host`, and DB-only adapters toward `database`/service trait impls where dependency direction permits.
2. Continue reducing `apps/rust/daemon/src/service/repo/deps.rs`: `RepoSigningDeps` is explicit now, but it still hosts repetitive RuntimeRepoAdapter/PostgresRepoStore delegation.
3. Continue shrinking auth/user daemon facade methods; account behavior is now database-backed, but route-facing methods still live in `SynforgeService`.
4. Move job/log/config DB-heavy query methods behind narrower database/service APIs where useful.
5. Keep shrinking `SynforgeService` until it is startup/background wiring plus HTTP-facing facade methods only.

## Success Criteria

- Workspace crates are:
  - `core`
  - `database`
  - `state`
  - `git-sync`
  - `publish`
  - `worker-host`
  - `worker`
  - `daemon`
- No direct Diesel usage outside `database`
- No direct Redis usage outside `state`
- No `app` crate
- No `orchestrator` crate
- `worker` contains no daemon-side Docker/session logic
- `daemon` is a composition root, not a behavior dump

## Non-Goals

- keeping current crate names just because they already exist
- preserving the current `app` / `orchestrator` split
- splitting PostgreSQL concerns across two crates
- mixing repo publication into `git-sync`
- mixing daemon-side worker orchestration into container worker code
