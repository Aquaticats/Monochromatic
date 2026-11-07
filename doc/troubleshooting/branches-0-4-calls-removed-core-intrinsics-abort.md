# `branches` 0.4.6 fails with E0425 on nightly-2026-11-07 when its `std` feature is off, and turso's 0.4 requirement keeps it there

Recorded 2026-11-07T10:55:00-05:00.

## Symptom

`cargo zigbuild --locked --release --target x86_64-unknown-linux-gnu.2.39` of `package/music-player/desktop-app`
on `nightly-2026-11-07` stopped in the dependency `branches` 0.4.6 with:

```text
error[E0425]: cannot find function `abort` in module `core::intrinsics`
```

Zig was not involved:
Zig 0.17.1 built the same crates up to that point.

## Root cause

Three facts combine.

- `branches` 0.4.6 calls the compiler intrinsic by its old name,
  only when its `std` feature is off and the build script
  sees a nightly compiler (`branches-0.4.6/src/lib.rs`,
  the `abort` function):

  ```rust
  #[cfg(feature = "std")]
  std::process::abort();
  #[cfg(all(not(feature = "std"), branches_nightly))]
  core::intrinsics::abort();
  ```

- Nightly renamed that intrinsic to `abort_immediate` on 2026-10-03
  ([rust-lang/rust#163574](https://github.com/rust-lang/rust/pull/163574),
  merge commit
  `99f6df7fc740fdd5deda70dcb4015769bb2e6366`,
  "intrinsics:
  Rename `abort` to `abort_immediate`").
  `branches` 0.5.1 already calls `core::intrinsics::abort_immediate()`,
  and still takes `std::process::abort()` first when
  `std` is on.
- `turso_core` 0.8.3 (and 0.9.0-pre.2) depends on `branches` as `^0.4.3` with `default_features = false`
  (crates.io dependency list of each release),
  so `turso_core` can never use 0.5,
  and it never turns `std` on.

An earlier fix (commit `b5b415fe3c`,
2026-10-11) turned `std` on from `truepeak-core` through feature unification with a
workspace `branches` entry.
Cargo unifies features only inside one semver line.
When the catalog floor became
`branches >= 0.5.1`,
that entry resolved 0.5.1,
a different crate from the 0.4.6 that `turso_core` uses,
so the `std`
feature stopped reaching it.
The lock then held both versions,
and 0.4.6 compiled without `std`.

## Verification

The first line of the full music-player build log after the fix is `Compiling branches v0.4.6`,
and the build no longer
reports E0425 (later errors in that run came from `zbus`,
see `doc/troubleshooting/zbus-6-removed-the-blocking-api.md`).

## Verified workarounds

`package/music-player/truepeak-core/Cargo.toml` selects a named catalog,
`branches-0-4` (`^0.4.3`),
declared in
`rust-catalog.toml`,
and enables `std` on that line (pattern of `[catalogs.bincode-1]`).
The default `branches` entry went
away because nothing else uses it,
and the three affected lock files lost the 0.5.1 entry.

Tradeoff:
the dependency has no code user,
it exists only to unify a feature,
and it caps `branches` at 0.4 for that crate
until `turso_core` requires 0.5 or newer;
remove the named catalog then.

## What does not work

- Pinning the nightly back:
  the owner's decision is to fix forward.
- Bumping `branches`:
  0.5.1 is not allowed by `turso_core`'s `^0.4.3`,
  and 0.4.6 is the newest 0.4 release.
- Enabling `std` on the catalog's 0.5 entry:
  the 0.4 crate is a different lock entry and never sees the feature.

## Upstream filing decision

Not assessed and not filed;
the default is not to file.
`turso_core` could allow `branches` 0.5 or enable `std`,
and
`branches` 0.4 could be patched,
but none of the six constraints or the duplicate search was run.
Filing would be the
poster's decision,
with the risk of a trust or blocklist system in the target repository.
