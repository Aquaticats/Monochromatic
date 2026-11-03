# rust-catalog-update

Recursive no-save update of every first-party `Cargo.lock`,
 the Cargo counterpart of `pnpm --recursive --no-save update`,
 under the release-age and trust policy in the repository's `rust-catalog.toml`.
It also has a CI mode that checks newly locked versions against the trust policy,
 and a lock-only mode that makes every lockfile satisfy its manifests again after a floor raise.

Design and decisions:
 [`doc/decision/rust-dependency-catalog.md`](../../../doc/decision/rust-dependency-catalog.md)
 and [`doc/planning/rust-dependency-catalog.md`](../../../doc/planning/rust-dependency-catalog.md).

## Usage

Run from the repository root:

```sh
# package/dependency-catalog/cli-rust-update/README.md
mise run rust:update              # update and keep the result if every check passes
mise run rust:update -- --dry-run # report what would change, then restore every Cargo.lock
mise run rust:reconcile           # after sync:files: make every Cargo.lock satisfy its manifests, offline
mise run rust:reconcile -- --dry-run  # report which lockfiles are stale, then restore every Cargo.lock
mise run rust:verify              # check versions no lockfile held at HEAD (uncommitted changes)
mise run rust:verify -- --base <rev>  # CI: the same against another revision
```

`--verify` checks only registry versions that no `Cargo.lock` held at the base revision,
 the counterpart of pnpm's `trustLockfile: true`:
 the owner never accepts pull requests that change a lockfile,
 so committed lockfiles are part of the trusted base.
CI passes the pushed range's previous commit (or a pull request's base) and fetches only that commit.
A crate whose lockfile did not exist at the base counts as new in full.

The full update workflow is:

1.  `mise run rust:update` refreshes the lockfiles.
2.  `mise run catalog:tighten` raises each `rust-catalog.toml` floor to the highest locked version.
3.  `mise run sync:files` renders the new floors into each crate's generated `[workspace.dependencies]` table.
4.  `mise run rust:reconcile` makes the lockfiles of the crates that locked a lower release satisfy the raised floors.

The update never edits manifests or catalog floors.
It edits `rust-catalog.toml` only to prune stale exclude selectors when the matching prune setting is on.

## What a floor raise leaves behind, and the reconcile

The catalog's one-version premise makes `catalog:tighten` raise a floor to the highest release any consuming crate locks.
`sync:files` then writes that floor into every consuming crate, and also writes a path crate's new `package.version`
 into its dependents as `=<version>`.
A crate whose lockfile held a lower release, or the old path-crate version, now has a lockfile that its own manifest refuses:
 `cargo metadata --locked --offline` exits 101 with "cannot update the lock file", and every later `cargo` run
 rewrites the lockfile in whichever worktree ran it.
Tighten never writes lockfiles and the file-enforcer sync is a pure renderer, so before `rust:reconcile` no step repaired this.
The update does not either: it runs before the floors move and re-resolves from scratch.

`mise run rust:reconcile` is that step.
It covers every first-party crate root that has a `Cargo.lock`, found with the same globs as the update and as file-enforcer's
 Rust catalog sync (standalone roots, `.fuzz` and `.bench` sidecars, virtual workspaces, and roots with a path dependency).
For each root, offline:

1.  `cargo update --workspace --locked --offline` asks whether the lockfile already satisfies the manifests.
    It reads only the index and writes nothing, so a consistent lockfile stays byte-for-byte as committed.
2.  A refusal means the lockfile is stale, so `cargo update --workspace --offline` re-locks the workspace members and the
     packages their changed requirements force, and keeps every other locked release.
    It is not a full re-resolution (`rust:update` is), so unrelated packages do not move.
3.  `cargo metadata --locked --offline` confirms the result.

Like the update it is one transaction.
Any cargo failure, or a refusal by the release-age or trust policy for a registry version that no lockfile held before
 (a version some other lockfile already holds passed the policy when it arrived and costs no registry request),
 restores every lockfile.
With `--dry-run` it reports the stale roots and restores everything.
Because cargo runs offline, it can only pick releases that the local registry index and cache hold:
 when one is missing, cargo's message is reported with the remedy (fetch it once online, or run `rust:update`).
Run `mise run rust:verify` afterwards to check the newly locked versions against the trust policy.

## What the update does

1.  Reads the settings from `rust-catalog.toml`.
2.  Snapshots every `Cargo.lock` beside a manifest matched by `package/*/*/Cargo.toml` or `package/*/*/*/Cargo.toml`.
3.  Re-resolves each crate in turn with `cargo -Zunstable-options generate-lockfile --publish-time <now minus minimumReleaseAge>`,
     which picks the newest releases published before the cutoff, as pnpm's `minimumReleaseAge` does.
    Only `generate-lockfile` accepts `--publish-time`,
     so every run is a full re-resolution rather than a selective `cargo update`.
4.  Moves crates that a name or pattern in `minimumReleaseAgeExclude` covers to their newest release with `cargo update`.
5.  Collects the registry versions that no lockfile held before (the union across all crates).
6.  Checks each new version's `pubtime` from the crates.io sparse index against `minimumReleaseAge`,
     and its crates.io version history against the trust policy.
7.  Sets aside a version the trust policy refuses, as pnpm's picker does:
     every crate that locks it runs `cargo update --package <name>@<version> --precise <candidate>`
     with the newest older release in the same caret class that was published before the cutoff,
     is not yanked, and passes the trust policy against the full history.
    Rounds repeat (at most 5, with 10 attempts per version and crate) because a move can lock new transitive versions,
     and each move is printed as a warning.
    A refusal with no workable candidate stays a finding.
8.  Keeps the new lockfiles only when nothing failed and it is not a dry run;
     otherwise it restores every lockfile, so a refused version never stays in some crates and not others.

With `minimumReleaseAgeStrict = false`,
 a crate that cannot resolve under the cutoff resolves without it,
 and young versions are reported as warnings instead of refusals.

## Trust policy

The trust levels are heuristics from crates.io data,
 ranked from highest to lowest:

- Trusted Publishing (`trustpub_data` present).
- A token publish by an account that published an earlier release or is a listed individual crate owner.
- A token publish by any other account.

With `trustPolicy = "no-downgrade"`,
 a version fails when an earlier stable release ranked higher,
 when its Trusted Publishing repository differs from earlier trusted releases,
 or when its declared `repository` differs from a declared `repository` on the release before it
 (an undeclared value on either side is not a change).
The owner list comes from `https://crates.io/api/v1/crates/<name>/owners`,
 fetched only for a crate that would otherwise be refused.
Team owners do not count,
 because the API does not list their members.
`trustPolicyExclude` exempts versions,
 and `trustPolicyIgnoreAfter` (minutes) skips versions older than that.

Version histories are cached under `.cache/rust-catalog-update/crates-io/` (gitignored),
 since published versions never change;
 CI keeps the cache with `actions/cache`.
When at least 100 of the checked crates have no cache file,
 the cache is filled from the crates.io database dump (`https://static.crates.io/db-dump.tar.gz`),
 which crates.io's data-access policy asks bulk readers to use and serves without a rate limit.
The reader streams the 2.1 GB archive,
 reads only `crates.csv` and `versions.csv`,
 and stops before the later tables
 (22 seconds for all 1024 locked crates on 2026-10-29, plus the download).
The dump keeps every Trusted Publishing column private,
 and a Trusted Publishing release has no publisher,
 so a crate with a publisher-less release since 2025-07-01 is left to the API
 (141 of 1024 crates on 2026-10-29).
Those crates,
 fewer than 100 uncached crates,
 and versions newer than the dump come from `https://crates.io/api/v1/crates/<name>/versions`,
 spaced 1100 ms apart with a user agent naming the repository;
 an HTTP 429 is retried after its `Retry-After`,
 or a doubling 30-second backoff,
 at most five times.

## Limits

- The reconcile cannot apply the release-age cutoff, because `cargo update` has no `--publish-time`:
   offline cargo picks the newest release in the local index and cache, which can be younger than `minimumReleaseAge`.
  The post-check refuses such a version (restoring every lockfile) when no other lockfile held it before.
- The update needs a nightly cargo for `-Zunstable-options`.
   It runs cargo under the nightly `toolchain.lock.json` pins (`.tools.rust.version`),
   overriding an inherited `RUSTUP_TOOLCHAIN` (logged as a warning),
   so it works outside a mise shell too:
   on the development host the systemd user manager exports `RUSTUP_TOOLCHAIN=1.98.1`,
   which made the first real run use stable cargo.
- An exclude selector that names exact versions (`zbus@5.19.0`) exempts only those versions from the checks;
   it does not move the crate, because the cutoff already chose its version.
- `--verify` does not re-check release age:
   a newly locked version passed the release-age cutoff when the update resolved it,
   and its age only grows.
   (pnpm's default `trustLockfile: false` re-applies both rules to every lockfile entry on install;
   this repository uses the `trustLockfile: true` counterpart for the reason given in the usage section.)

## Development

```sh
# package/dependency-catalog/cli-rust-update/README.md
mise run //package/dependency-catalog/cli-rust-update:lint:types
mise run //package/dependency-catalog/cli-rust-update:lint:oxlint
node package/dependency-catalog/cli-rust-update/src/update.unit.test.ts
node package/dependency-catalog/cli-rust-update/src/reconcile.unit.test.ts
```

Most tests use fake cargo runners,
 a fake registry,
 and disposable repositories,
 so they need neither the network nor cargo.
The reconcile test also runs the real cargo (the repository's pinned one on `PATH`) against a directory source that stands
 in for crates.io, so it needs cargo but not the network;
 without cargo on `PATH` those two tests report a skip with the reason and the rest still run.
