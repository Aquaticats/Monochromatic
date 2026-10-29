# rust-catalog-update

Recursive no-save update of every first-party `Cargo.lock`,
 the Cargo counterpart of `pnpm --recursive --no-save update`,
 under the release-age and trust policy in the repository's `rust-catalog.toml`.
It also has a CI mode that re-checks every locked version against the trust policy.

Design and decisions:
 [`doc/decision/rust-dependency-catalog.md`](../../../doc/decision/rust-dependency-catalog.md)
 and [`doc/planning/rust-dependency-catalog.md`](../../../doc/planning/rust-dependency-catalog.md).

## Usage

Run from the repository root:

```sh
# package/dev-script/rust-catalog-update/README.md
mise run rust:update              # update and keep the result if every check passes
mise run rust:update -- --dry-run # report what would change, then restore every Cargo.lock
mise run rust:verify              # CI: check every locked version against the trust policy
```

The full update workflow is:

1.  `mise run rust:update` refreshes the lockfiles.
2.  `mise run catalog:tighten` raises each `rust-catalog.toml` floor to the highest locked version.
3.  `mise run sync:files` renders the new floors into each crate's generated `[workspace.dependencies]` table.

The update never edits manifests or catalog floors.
It edits `rust-catalog.toml` only to prune stale exclude selectors when the matching prune setting is on.

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
7.  Keeps the new lockfiles only when nothing failed and it is not a dry run;
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

Version histories come from `https://crates.io/api/v1/crates/<name>/versions`,
 spaced at one request per second as crates.io's data-access policy requires,
 with a user agent naming the repository.
They are cached under `.cache/rust-catalog-update/crates-io/` (gitignored),
 since published versions never change;
 a history is refetched only when a needed version is missing from its cache file.
A cold `--verify` over the current lockfiles is about 1024 requests,
 so about 17 minutes;
 CI keeps the cache with `actions/cache`.

## Limits

- The update needs a nightly cargo for `-Zunstable-options`.
   It runs cargo under the nightly `toolchain.lock.json` pins (`.tools.rust.version`),
   overriding an inherited `RUSTUP_TOOLCHAIN` (logged as a warning),
   so it works outside a mise shell too:
   on the development host the systemd user manager exports `RUSTUP_TOOLCHAIN=1.98.1`,
   which made the first real run use stable cargo.
- An exclude selector that names exact versions (`zbus@5.19.0`) exempts only those versions from the checks;
   it does not move the crate, because the cutoff already chose its version.
- `--verify` does not re-check release age on committed lockfiles,
   as pnpm does not re-gate a frozen lockfile on `minimumReleaseAge`.

## Development

```sh
# package/dev-script/rust-catalog-update/README.md
mise run //package/dev-script/rust-catalog-update:lint:types
mise run //package/dev-script/rust-catalog-update:lint:oxlint
node package/dev-script/rust-catalog-update/src/update.unit.test.ts
```

The tests use fake cargo runners,
 a fake registry,
 and disposable repositories,
 so they need neither the network nor cargo.
