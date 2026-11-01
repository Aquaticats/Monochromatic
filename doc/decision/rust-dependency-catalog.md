# Rust dependencies come from one pnpm-style catalog without a root Cargo workspace

## Status

Accepted 2026-10-29 by the owner,
 resolving the design of [issue #1800](https://github.com/Aquaticat/Monochromatic/issues/1800).
Evidence,
 probes,
 rejected alternatives,
 and the pnpm feature map are in `doc/planning/rust-dependency-catalog.md`.
Complements `.out-of-scope/cargo-workspace.md` and `doc/planning/cargo-toml-file-enforcer.md`.

## Context

Every Rust crate is its own Cargo workspace root,
 so Cargo's `[workspace.dependencies]` cannot span them,
 and Cargo has no cross-root inheritance (rust-lang/cargo#5042 and #15435 are open without a design).
No tool was found that syncs one catalog across independent roots.
The JS side already runs pnpm catalogs (`catalogMode: strict`),
 `catalog:tighten`,
 `minimumReleaseAge`,
 and `trustPolicy: no-downgrade`;
 the Rust catalog follows what pnpm offers.

## Decision

### Catalog file

- `rust-catalog.toml` at the repository root,
   in Cargo dependency syntax,
   with `[catalog]` (default) and `[catalogs.<name>]` (named) tables.
- Registry entries are `>=` floors;
   named-catalog entries are caret or exact;
   git entries carry `git` and `rev`.
- Supply-chain keys use pnpm's names:
   `minimumReleaseAge` (1440),
   `minimumReleaseAgeExclude`,
   `minimumReleaseAgeStrict` (true),
   `minimumReleaseAgeIgnoreMissingTime` (false),
   `minimumReleaseAgeExcludePrune`,
   `trustPolicy` (`no-downgrade`),
   `trustPolicyExclude`,
   `trustPolicyIgnoreAfter`,
   `trustPolicyExcludePrune`.
- No version-family grouping.

### Crate manifests

- `file-enforcer` renders a generated `[workspace.dependencies]` table in each crate holding the catalog entries it uses.
   Use sites reference it with `workspace = true` and keep crate-local `features`,
   `optional`,
   and `default-features`,
   in every dependency table.
- Every registry and git dependency comes from the catalog.
- A crate selects a named catalog in `[package.metadata.rust-catalog.catalogs]`.
- Internal crates stay out of the catalog;
   they are path dependencies whose `version` the sync writes as `=<target package.version>`,
   the counterpart of `workspace:*`.

### Sync and check

- `file-enforcer` adopts literals left by `cargo add` into the catalog as `>=` floors,
   prunes unused entries,
   and has a check-only mode with named diagnostics and a list of affected crates.
- The check runs in `.github/workflows/advisory-core.yml` after `task-run prepare:pnpm`,
   beside the JS `pnpm install --frozen-lockfile` gate.

### Update and tightening

- A new package bin (now `package/dependency-catalog/cli-rust-update`),
   run by a thin mise task,
   runs `cargo update` in every crate root
   (the `pnpm -r --no-save update` counterpart),
   with dry run,
   pnpm's release-age semantics (newest version old enough;
   strict fails when none is)
   through `cargo -Zunstable-options generate-lockfile --publish-time`,
   with excluded packages updated afterwards and every new version checked against the sparse index `pubtime`,
   and a heuristic `no-downgrade` trust policy from the crates.io API
   (Trusted Publishing above a known token publisher above a first-time publisher;
   a changed trusted-publishing repository or declared `repository` fails),
   cached per `name@version`.
   Amended by the owner on 2026-10-29 ([issue #1806](https://github.com/Aquaticats/Monochromatic/issues/1806)):
   a listed individual crate owner counts as a known publisher
   (checked against `/api/v1/crates/<name>/owners` only for a release that would otherwise be refused),
   and only a change between two declared `repository` values fails;
   an undeclared value on either side carries no evidence.
   The CI check re-verified every locked registry version.
   Amended by the owner on 2026-10-29:
   the CI check covers only registry versions that no `Cargo.lock` held at a base revision
   (the pushed range's previous commit or a pull request's base),
   the counterpart of pnpm's `trustLockfile: true`,
   because the owner never accepts pull requests that change a lockfile,
   so committed lockfiles are part of the trusted base;
   and `trustPolicyIgnoreAfter = 525600` (365 days) skips older releases.
   The first full cold check had found 41 findings on long-locked releases.
   Amended by the owner on 2026-10-29:
   a cold cache is filled from the crates.io database dump (`https://static.crates.io/db-dump.tar.gz`,
   rebuilt daily, no rate limit), which crates.io's data-access policy lists ahead of the API,
   after a full API-only verification hit HTTP 429 after 657 requests.
   The dump keeps every Trusted Publishing column private,
   so a crate with a publisher-less release since 2025-07-01 still takes its history from the API,
   which is spaced at 1100 ms and retries HTTP 429 after `Retry-After`.
   The dump and CSV readers are hand-written, not third-party libraries (owner choice).
- `catalog-tighten` gains a Cargo source that raises `>=` floors to the highest version locked across consuming crates;
   it writes no lockfiles.

## Consequences

- Upstream majors arrive through the update by design;
   the owner treats falling behind as the failure.
- `i-slint-backend-testing` stays an exact pin equal to `slint`,
   so an update that moves `slint` breaks the tests of `package/ide/app` and `package/music-player/desktop-app`
   until the pin is raised by hand.
- `package/deps-update/cli` is not built on;
   its audit is [issue #1803](https://github.com/Aquaticat/Monochromatic/issues/1803).
- Lockfile drift reporting and transitive policy are
   [issue #1801](https://github.com/Aquaticat/Monochromatic/issues/1801);
   change intents for Rust crates are [issue #1802](https://github.com/Aquaticat/Monochromatic/issues/1802).
