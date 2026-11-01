# Rust dependencies

Every registry and git dependency version of the repository's Rust crates lives in `rust-catalog.toml`
 at the repository root,
 the Rust counterpart of the pnpm catalog in `pnpm-workspace.yaml`.
Each crate stays its own Cargo workspace root;
 `file-enforcer` renders the catalog entries a crate uses into that crate's generated `[workspace.dependencies]` table,
 and the crate's dependency tables reference them with `workspace = true`.
Design and rationale:
 [`doc/decision/rust-dependency-catalog.md`](../decision/rust-dependency-catalog.md).

## Add a dependency

1.  In the crate directory, run `cargo add <crate>`
     (with `--features`, `--optional`, or `--no-default-features` as needed).
2.  From the repository root, run `mise run sync:files`.
    The sync adopts the literal version into `rust-catalog.toml` as a `>=` floor
     (or raises an existing floor to satisfy it)
     and rewrites the crate's entry to `workspace = true`,
     keeping its `features`, `optional`, and `default-features`.
3.  Commit `rust-catalog.toml`, the crate's `Cargo.toml`, and its `Cargo.lock` together.

A crate that already uses the dependency elsewhere can write `name = { workspace = true }` directly
 when the catalog has the entry.

## Change features or optional

Edit the use site only:
 `features`, `optional`, and `default-features` stay crate-local.
When every site of a dependency in one crate writes `default-features = false`,
 the sync also writes `default-features = false` into that crate's generated entry,
 which Cargo needs before 1.99 and for pre-2024 editions.
Sites in one crate that disagree on `default-features` fail the sync with a named diagnostic;
 make them agree.

## Pin or bound a version

The default catalog holds only `>=` floors and git pins.
For an exact pin (`=0.7.0`) or a bounded requirement (`0.58`),
 put the entry in a named catalog and select it from the crate:

```toml
# rust-catalog.toml
[catalogs.pinned]
smithay = "=0.7.0"
```

```toml
# package/<category>/<crate>/Cargo.toml
[package.metadata.rust-catalog.catalogs]
smithay = "pinned"
```

Then run `mise run sync:files`.

## Depend on another first-party crate

Use a path dependency, not the catalog:
 `other = { path = "../other" }`.
The sync writes `version = "=<other's package.version>"` beside the path,
 the counterpart of `workspace:*`.

## Remove a dependency

Delete it from the crate's `Cargo.toml` and run `mise run sync:files`.
With `catalogPrune = true` (the repository's setting),
 the sync removes catalog entries no crate uses.

## Update every crate

1.  `mise run rust:update -- --dry-run` lists what would change
     and restores every lockfile.
2.  `mise run rust:update` re-resolves every `Cargo.lock`
     to the newest releases older than `minimumReleaseAge`,
     checks every newly locked version against the release age and the trust policy,
     and keeps the result only when all of them pass.
3.  `mise run catalog:tighten` raises each `>=` floor to the highest locked version.
4.  `mise run sync:files` renders the new floors into the crates.
5.  Build and test the crates whose lockfiles changed,
     then commit the lockfiles,
     `rust-catalog.toml`,
     and the manifests together.

The update needs the nightly cargo pinned in `toolchain.lock.json`.
Its trust checks cache crates.io histories under `.cache/rust-catalog-update/`.
See [`package/dependency-catalog/cli-rust-update/README.md`](../../package/dependency-catalog/cli-rust-update/README.md)
 for the policy details.

## Exceptions to the release age or trust policy

- `minimumReleaseAgeExclude = ["ring", "tokio*", "zbus@5.19.0"]` exempts names,
   name patterns, or exact versions from the release age.
- `trustPolicyExclude` takes the same selectors and exempts them from the trust policy.
   Review the release first;
   a refusal names the crate, the version, and the evidence (publisher change or repository change).
- With `minimumReleaseAgeExcludePrune` and `trustPolicyExcludePrune` on,
   the update removes selectors that no locked version needs any more.

## Check before pushing

`mise run check:files` (`file-enforcer check`) exits 1 and lists every generated file,
 manifests included,
 that differs from what the sync would write.
A file edited by hand in a generated section is overwritten by the next sync:
 edit `rust-catalog.toml` or the use site instead.
