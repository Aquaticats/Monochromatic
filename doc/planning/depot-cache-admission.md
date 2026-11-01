# Depot cache admission

Status:
work item 5 implementation.
Cache policy:
[the CI caching decision](../decision/ci-caching.md).
Release workflow:
[music-player-release.yml](../../.github/workflows/music-player-release.yml).

## Admitted formats

The 2026-10-15 cache decision admits the existing GitHub Actions cache families for cargo
target and registry bytes,
the CodeQL action's entries,
and the MSYS2 package downloads.
Each family is restored on every ref and saved only from `main` or an explicit dispatch.
The release workflow's `detect` job sets `save_cache` only for a push on `refs/heads/main`
or `workflow_dispatch`,
and each save step checks that flag.

## Gradle

The Gradle cache is restore-only.
The release workflow sets `save_gradle_cache=false` and the Gradle save step checks that separate
flag.
The restore step is unchanged,
so an already admitted cache can still be consumed without writing new bytes.

Gradle is not yet admitted because the cache-off,
invalidation and stale-output controls have not been demonstrated for the Gradle command
boundary.
Admitting it requires:

- a cache-off run that builds and attests the same inputs without the cache;
- an invalidation check that a changed Gradle,
  Android Gradle plugin,
  or wrapper input moves the key;
- a stale-output check that an old cache cannot make a later build skip a changed input;
- a size measurement against the 650 MB guide.

Until those pass,
the family cannot write and no new Gradle bytes enter the cache.

## Disabled formats

These remain disabled and have no cache steps:

- pnpm store;
- target archive;
- mise action cache,
  which saves nothing while `install: false`.

The disabled formats are recorded in the cache decision and are not reintroduced by the release
workflow.

## Pruning

The Gradle key family remains in `cache-prune.yml` so any entries written before this guard are
eventually removed.
The guard stops new writes;
the existing prune schedule handles the old entries.

## Acceptance gaps

- Cache-off,
  invalidation and stale-output evidence for Gradle is still missing.
- The cargo and MSYS2 families rely on the 2026-10-15 measurements,
  not a fresh cache-off rerun in this rollout.
- The first release run after this change should confirm the Gradle save step is skipped while
  the restore step still reports its key.
