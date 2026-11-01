# CI caching policy

Decided on branch `chore/ci-caching`,
measured on real GitHub runs on 2026-10-15.

The owner deleted `.github/workflows/final-newline.yml` on 2026-10-15 because it recompiled the wrapper on every pull
request.
Every mention of that workflow below is a record of what was decided and measured;
today the `cli-git-native` cache
belongs to the version job of `npm-release.yml` alone,
and the push to `main` that runs it is the only run that saves.

## Amended on 2026-10-18

[The Depot CI adoption decision](depot-ci-adoption.md) extends the cache write policy to every
cache system this repository uses.
The replaced scope is the statement at "pnpm store (not cached)" that "The tested cache system is
GitHub Actions `actions/cache` ... not Depot Cache or Cargo artifact caching"
(lines 422 to 423):
that remains true of the measurements below,
and is no longer the whole policy.
Under the current decision,
GitHub Actions cache,
Depot Actions cache,
Depot project BuildKit cache and
organization-global Disk all admit writes only from `main` and explicit dispatches,
restore only for pull requests and merge groups,
and cache no shipped,
attested or published bytes.
The measurements,
key shapes,
save authority and size rules below remain the 2026-10-15 record for the GitHub Actions cache they
describe.

## Starting point

The repository was not caching nothing,
but the caches that existed did little.

- Every `cargo-publish.yml` publish job keyed its cache on the hash of `Cargo.lock` alone,
  with no `restore-keys`.
  That workflow only runs on a version bump,
  and a version bump changes the crate's own entry in `Cargo.lock`,
  so the
  key
  never matched on the push path.
  Every run stored a new 130 to 420 MB entry and restored nothing.
- The `cache: true` of `jdx/mise-action` saved nothing in any workflow.
  At the pinned `c2a8761` (v4.3.0),
  `src/index.ts` calls `saveCache` only inside `if (core.getBooleanInput('install'))`,
  and every workflow sets `install: false`.
  The cache listing held no `mise-v1-*` entry,
  and every run log says `mise cache not found` and never
  `Saving mise cache`.
  A working mise cache would also not hold the Rust toolchain,
  because mise links `installs/rust/<version>` to
  `~/.cargo/bin` and keeps the toolchain under `~/.rustup`.
- `final-newline.yml` (every pull request and merge group) and the version job of `npm-release.yml` compile
  `cli-git-native` in release mode with no cache.
  That compile takes 157 s cold.
- The repository was over its cap:
  11.3 GB of 10 GB at the first reading,
  7.5 GB after eviction,
  most of it music player
  feature-branch entries of 600 MB to 1 GB.

## What each workflow caches now

- `cargo-publish.yml`,
  publish jobs for `forbidden-strings`,
  `nested-wayland-session`,
  `monochromatic-lint`,
  and
  `monochromatic-git-policy-cli`:
  the crate's `target/` directory without `target/package`.
  The registry directories are not cached:
  the log shows the downloads finishing in about 1 s,
  and `target/` alone
  restores a full hit.
- `cargo-publish.yml`,
  publish jobs for `forbidden-regex` and `monochromatic-jsonc-edit`:
  nothing.
  The cold verify build takes 6 s and under 1 s,
  so an entry cannot pay for its restore and save steps.
- `cargo-publish.yml`,
  release binary matrices:
  nothing,
  by decision (see the rules below).
- `final-newline.yml` and `npm-release.yml` (version job):
  one shared `target/` for `cli-git-native`.
  Pull request and merge group runs only restore;
  push and dispatch runs save.
- `logger-fuzz.yml`,
  `toml-edit-fuzz.yml`,
  `readonly-semantic-bridge.yml`:
  nothing.
  A pnpm store cache was measured and
  loses on every operating system (see the measurements).
- `npm-release.yml`,
  `pnpr-publish.yml`,
  `kotlin-linter-publish.yml`:
  nothing.
  Their mise cache is inert (see above) and
  stays as it is in this change;
  a working one would save about 4 s of Node and pnpm installation per job.
- `codeql.yml`:
  the CodeQL action's own `codeql-trap-*` entries (about 145 MB per main commit) and
  `codeql-overlay-base-database-*` entries (about 5 MB).
  They are not configured here.
- `forbidden-strings.yml` (the per-push scan):
  nothing;
  it downloads a prebuilt binary.
- `fs-id.yml`,
  `scorecard.yml`,
  `claude.yml`:
  nothing;
  they build nothing worth caching.
- `music-player-release.yml`,
  job `build-desktop-musl` (the Alpine release tarballs):
  nothing,
  decided 2026-10-15
  while the repository held 9.98 GB of 10 GB.
  It ships its bytes,
  which this document allows to stay uncached.
  A cache could save at most the dependency compile:
  9 min 07 s (x86_64) and 5 min 16 s (aarch64) per release.
  The release build leaves 684 MB of cargo home and 2.3 GB of target directory before any trim,
  and the compressed
  size of a trimmed entry was not measured (the Alpine guard's debug-build entries are 949 MB and 928 MB).
  Details:
   `doc/decision/music-player-releases.md`.
- `music-player-release.yml`,
   the Windows GNU job (`x86_64-pc-windows-gnu`,
   release asset `windows-x64-gnu.exe`):
  no cargo or target cache,
   decided 2026-10-15 while the repository held 11.4 GB of 10 GB.
  The cold job took 16 min 07 s,
   of which a cache could save at most the 7 min 14 s of dependency compilation,
   and
  it finished before the slowest job of the run.
   It restores and saves only the MSYS2 package downloads (70 MB) under
  the key that the `test-windows` job of the same workflow also uses.
   Details:
   `doc/decision/music-player-releases.md`.
- `music-player-release.yml`,
   both `jdx/mise-action` steps:
   pinned to mise 2026.10.9 without `minimum_release_age`,
   so
  they make no GitHub API call to resolve the version (a 403 from that call failed two jobs of run 38688512043).

## Key shapes

Cargo publish jobs:

```text
cargo-v2-publish-<crate>-<runner.os>-<ImageOS>-<rustc commit hash>-<hash of Cargo.lock>
restore-keys: cargo-v2-publish-<crate>-<runner.os>-<ImageOS>-<rustc commit hash>-
```

`<crate>` is `fs`,
`nws`,
`ml`,
or `gpc`.

Shared `cli-git-native` cache:

```text
cargo-v2-cli-git-release-<runner.os>-<ImageOS>-<rustc commit hash>-<hash of package/git-policy/cli/Cargo.lock>
restore-keys: cargo-v2-cli-git-release-<runner.os>-<ImageOS>-<rustc commit hash>-
```

Rules behind the shapes:

- The `v2` token invalidates the old `cargo-publish-*` keys,
  so a first run on a new key is a guaranteed cold run.
  Raise it to invalidate every entry at once.
- The toolchain and the runner image belong in the key and in the restore prefix.
  Cargo never prunes artifacts of an
  older compiler,
  so restoring a stale `target/` only enlarges the next save.
  The repository floats `rust = "nightly"`,
  so an entry serves runs that share a nightly day;
  it still serves every version bump of that day.
  `ubuntu-latest` becomes Ubuntu 26 on 2026-10-19,
  and the image name in the key keeps 24.04 artifacts from
  being restored there.
- The lock hash is the last segment and the restore prefix ends right before it.
  A lock change then restores the newest
  entry of the same toolchain and recompiles only what changed.
- No key may be a prefix of another workflow's key,
  so each family names its crate and operating system.
- `restore-keys` improves hit rate and does not reduce the number of entries:
  a prefix hit still saves a new entry under
  the new primary key.
  Entry count is controlled by deleting superseded entries.
- Never cache what the workflow ships.
  The `.crate` files and release binaries are built from source in the run,
  and
  `target/package` is excluded from the cache so the attested bytes cannot come from a restored directory.

## Who may save

- A cache can be restored from its own ref and from the default branch only.
  A pull request ref cannot read the branch
  that opened it,
  and no pull request can read another pull request.
- Pull request and merge group runs restore and never save,
  so a pull request cannot add an entry per pull request,
  and
  only runs of `main` write the entries that other refs read.
  The `cli-git-native` cache is seeded by the push-to-main
  run of `npm-release.yml` and by a dispatch of `final-newline.yml` from `main`.
- Proof runs from a feature branch write entries that nothing else can read.
  Delete them when the proof is done.

## Size budget against the 10 GB cap

The cap covers every workflow.
At the limit GitHub evicts the least recently used entry,
whatever workflow stored it,
and
also evicts an entry unread for 7 days.
The steady state below keeps the newest entry of each family on `main`.

- `cargo-publish.yml`:
  101 MB (`fs`),
  262 MB (`nws`),
  351 MB (`ml`),
  117 MB (`gpc`),
  about 0.85 GB in total.
- `final-newline.yml` and `npm-release.yml`:
  80 MB.
- `codeql.yml`:
  about 145 MB per daily main run kept for 7 days,
  at most about 1 GB.
- `music-player-release.yml`:
  about 4.0 GB,
  as reported by its author (seven entries of 387 MB to 651 MB).
- The `test-windows` job of `music-player-release.yml`
  (a standalone Windows workflow until it moved into the release workflow on 2026-10-22):
  four entries of about 610 MB plus msys2 entries of about 69 MB,
  about 3 GB if each
  family keeps one entry.
- The `test-alpine` job of `music-player-release.yml`
  (a standalone Alpine workflow until it moved into the release workflow on 2026-10-22):
  two entries (x86_64 and aarch64) of 949 MB and 928 MB,
  about 1.9 GB.
  They exceed the 650 MB guide because the debug build of the `turso` crates alone holds about 400 MB of libraries;
  the entry is already trimmed of the registry sources and of every output of this repository's own crates.
  `doc/decision/music-player-alpine.md` has the measurements.

That sums to about 9 GB with no headroom,
so the rules for large entries below are binding.

Superseded entries are removed daily by `.github/workflows/cache-prune.yml`,
which the human approved on 2026-10-15
with the music player families (`music-player-desktop-v1`,
`music-player-android-native-v1`,
`music-player-gradle-v1`
and `mp-win-v1`) added to its `FAMILIES` list;
the patterns were tested against the live key list.
It keeps the newest
entry of each family on each ref and deletes the rest.
Without it,
superseded entries leave only through the 7-day idle rule or through cap eviction,
which can pick a music player entry
instead.

## Rules for the music player workflows

These apply to `music-player-release.yml`,
including its `test-alpine` and `test-windows` jobs,
and to any later
workflow that stores more than 100 MB.

- Save only on `main` and on dispatch;
  restore everywhere.
  No save on pull request or merge group events.
- One family per target and operating system.
  The lock hash is the last key segment,
  and the restore prefix ends before
  it.
  Keys that embed a nightly date create a new full set every day the workflow runs,
  so either drop the date from the
  family used for pruning or key the toolchain by week.
- Cache the smallest directory that works.
  Measure `du` of each candidate and compare the cold step time with restore
  plus warm step time.
  Do not store the registry and `target/` twice across jobs of the same lock hash.
- Each entry stays under 650 MB compressed and each workflow stays inside its share of the budget above.
  A new large
  family needs an edit to this document first.
- Add the workflow's key families to the `FAMILIES` list of `cache-prune.yml`,
  or its superseded entries stay.
- Delete the entries a feature branch wrote once the branch has merged or been dropped.
- Never cache a path whose bytes are shipped,
  attested,
  or published.

## How to check hit rates and size

- Hit or miss of one run:
  `gh run view <run id> --log | rg "Cache hit|Cache not found|Cache restored from key"`.
  `Cache hit for:` is an exact hit,
  `Cache hit for restore-key:` is a prefix hit,
  and `Cache not found` is a miss.
- Time of one run's cache steps:
  `gh run view <run id> --json jobs`,
  comparing `Cache cargo build artifacts`,
  the build
  step,
  and the `Post` step.
- Whole repository:
  `gh api repos/{owner}/{repo}/actions/cache/usage` and
  `gh api "repos/{owner}/{repo}/actions/caches?per_page=100&sort=size_in_bytes&direction=desc"`.
- The cargo publish jobs log `target/` and registry sizes in the `Log cargo directory sizes` step.

## Measurements

All runs are GitHub-hosted `ubuntu-latest` unless noted.
Run-to-run spread on an unchanged build is about 10 percent
(`nws` cold:
49 s and 54.7 s).

### nested-wayland-session verify build, entry 262 MB

- Cold (run 38671738833):
  restore miss,
  build 54.7 s,
  save 5 s.
- Warm,
  exact hit (run 38671352265,
  previous key shape):
  restore 6 s,
  build 9 s.
- Prefix hit (run 38672459720):
  the log reads `Cache hit for restore-key`,
  restore 4 s,
  build 9.2 s,
  and a new entry
  is saved
  under the new primary key.
  The primary key was replaced by a literal suffix in a temporary commit,
  because a valid
  lock
  edit that changes only the hash needs a version bump.
  The temporary commit is reverted on the branch.
- A first attempt that edited the lock file by hand restored the prefix correctly and then failed in `cargo package`,
  because cargo rewrote the lock (run 38672141946).
  That run proves the restore only.

### monochromatic-lint verify build, entry 351 MB

- Cold (run 38672750851):
  build 104 s,
  save 12 s.
- Warm (run 38673762722):
  restore 11 s,
  build 7 s.
  The saving is 86 s of 104 s.
- The previous entry was 404 to 424 MB because it also held the registry.

### forbidden-strings verify build, entry 101 MB

- Cold (run 38674820030):
  build 133 s,
  save 3 s.
- Warm (run 38675277251):
  restore 3 s,
  build 82 s.
  The crate itself is the long part.

### forbidden-regex and jsonc-edit verify builds

- Cold build 6.5 s (run 38675138974,
  21 MB entry) and 0.22 s (run 38675533214,
  under 1 MB entry).
  Both caches were
  removed.

### monochromatic-git-policy-cli verify build

- Cold (run 38675660904):
  build 138 s,
  save 4 s,
  entry 117 MB.
- Warm (run 38676006582):
  restore 4 s,
  build 6.2 s.
  The saving is 128 s of 138 s.

### cli-git-native release build, shared entry 80 MB

- Cold dispatch of `final-newline.yml` (run 38671742063):
  build 157 s,
  save 3 s.
- Warm dispatch (run 38672083181):
  restore 1 s,
  build 100 s.
  The four workspace path crates rebuild because a fresh
  checkout gives their sources newer modification times than the cached fingerprints,
  so the saving is 57 s of 157 s.
- Pull request run (run 38672732073):
  `Cache not found`,
  build 160 s,
  no save.
  This shows that a pull request cannot
  read
  the entry a branch dispatch wrote,
  and that it does not save.
  Restoring from a `main` entry is documented GitHub
  behavior and is not yet exercised,
  because no such entry exists until this branch merges.

### Release binary matrices (not cached)

Release builds of `monochromatic-lint` in run 38672750851:

- Target directories:
  714 MB (Linux x86_64),
  702 MB (Linux arm64 musl),
  723 MB (macOS x86_64),
  828 MB (Windows arm64).
- Build step times:
  245 s to 279 s on Linux,
  340 s and 366 s on macOS,
  419 s and 622 s on Windows.
- Eight entries would cost roughly 1 GB of cap for this one crate at the 17 to 19 percent compressed-to-`du` ratio the
  verify
  builds show (not measured for release builds),
  and four crates run such matrices.
  The final link with fat link-time
  optimization is not cacheable,
  and these jobs ship attested binaries,
  so the shipped bytes come from a clean build.
  The matrices stay uncached;
  a saving for them was not measured.

### pnpm store (not cached)

The measured workflow is `readonly-semantic-bridge.yml`,
displayed as `readonly semantic bridge host evidence`.
Its jobs are `semantic bridge (ubuntu-latest)`,
`semantic bridge (windows-latest)` and `semantic bridge (macos-15)`.
The tested cache system is GitHub Actions `actions/cache` archiving the pnpm store,
not Depot Cache or Cargo artifact caching.
The comparison covers cache restoration plus workspace dependency installation,
excluding cache-save time unless explicitly stated.
The policy applies this result to other pnpm-installing workflows;
they are not separately measured regressions in this record.

Entry size 298 MB (Linux),
377 MB (Windows),
580 MB (macOS).
Run 38672732092 and its re-run:

- Linux:
  install 8 to 9 s without a cache;
  restore 7 s plus install 8 s with one.
- Windows:
  install 15.6 s without a cache;
  restore 15 s plus install 13 s,
  and the cold save took 11 s.
- macOS:
  install 14.7 s without a cache;
  restore 37 s plus install 8 s,
  and the cold save took 44 s.

Restoring costs as much as the whole install,
so the cache is reverted on the branch (`git log` shows the pair).

### mise-action cache

- Restore attempt 2 s and a miss in every run;
  no save ever.
  Rust toolchain installation takes 15.5 s and Node plus pnpm
  3 to 4 s on Linux.
  The `cache: false` set in `cargo-publish.yml` removes the inert restore.
- Resolving the mise version through `minimum_release_age` takes about 10 s per job (log lines `Selected mise` after the
  restore line).
   A fixed `version` skips that lookup.
   This is not a cache;
   the two steps of
  `music-player-release.yml` use it since 2026-10-15 (see above),
   and the other workflows are left as they were.

## Not verified

- The version job of `npm-release.yml` was not run:
  a dispatch of that workflow can open pull requests or publish.
  Its cache steps are the same as the proven ones in `final-newline.yml`,
  and the file parses as YAML.
- Restoring the `cli-git-native` entry from a pull request after a `main` run saved it.
- The deletion path of `cache-prune.yml`:
  the listing and the selection were run against the real repository as a dry
  run
  and picked the expected entries,
  and the deletion call was not executed.
- `cache-prune.yml` has not run on GitHub as of its merge,
  because a new workflow file cannot be dispatched before it
  exists on `main`;
  its first dry-run dispatch is recorded in the handover.

## Other findings outside caching

- `kotlin-linter-publish.yml` does not set `MISE_AUTO_INSTALL: 'false'`,
  so `mise run` installs the whole root toolset;
  the 2026-08-16 run spent 472 s in that step.
- `cargo-publish.yml` uses one concurrency group for every ref.
  A dispatch from a branch can replace the pending run
  of a
  version-bump push to `main`.
  Dispatch only when no run is queued.
- `logger-fuzz.yml` fails at its coverage gate on this branch and on 2026-09-24,
  before this change.
- `toml-edit-fuzz.yml` spends 174 s in its conformance step (2026-09-24 run),
  more than everything a cache could remove.
