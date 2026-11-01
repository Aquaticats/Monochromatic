# Floating `nightly` breaks the `branches` crate with E0425 `core::intrinsics::abort`

Tool under test:
rustc `1.101.0-nightly (6f5da406b 2026-10-10)` (toolchain `nightly-2026-10-11`),
crate `branches` 0.4.4 and 0.4.6,
reached through `turso_core` 0.6.1 and 0.8.2.

## Symptom

Desktop-app builds through mise fail while compiling a dependency, not the app:

```text
error[E0425]: cannot find function `abort` in module `core::intrinsics`
  --> .../branches-0.4.6/src/lib.rs:45:23
   |
45 |     core::intrinsics::abort();
   |                       ^^^^^ not found in `core::intrinsics`

error[E0308]: mismatched types
  --> .../branches-0.4.6/src/lib.rs:41:30
   |
41 | pub extern "C" fn abort() -> ! {
   |                   -----      ^ expected `!`, found `()`
```

## Cause

The root `mise.toml` pins `rust = { version = "nightly", ... }`, a floating channel,
so the newest installed nightly is used.
That nightly no longer exposes `core::intrinsics::abort`.
`branches::abort` calls it on nightly when its `std` feature is off
(`src/lib.rs:44-45`, selected by the `branches_nightly` cfg that `build.rs` sets for every nightly).
`turso_core` depends on `branches ^0.4.3` with default features off, so `std` is off in our graph.
Releases of `branches` 0.4.4 to 0.4.6 all share the code, and `turso_core` 0.8.2 and 0.8.3-pre.1 still require `^0.4.3`,
so neither a `branches` patch release nor a Turso upgrade removes the failure.

A scratch crate that depends on `branches` with default features builds fine on the same nightly, because `std` is on
there. Only the real graph reproduces the failure, so reproduce with the app's own build, not a scratch crate.

## Fix

Enable `std` through Cargo feature unification, in `package/music-player/truepeak-core/Cargo.toml`:

```toml
branches = { version = "0.4.6", default-features = false, features = ["std"], optional = true }
```

behind the `service` feature next to `turso`.
With `std`, `branches::abort` calls `std::process::abort`.
Verified on 2026-10-11 with the newest nightly: `lint:slint`, the 616 desktop tests and `lint:clippy` all passed.
Drop the line once `turso_core` builds on the newest nightly by itself.

## Notes

- The Android builds use `nightly-2026-09-22`, the only installed toolchain with the Android standard-library targets,
  so they do not meet the failure.
- `branches` 0.5.1 fixes nothing here because `turso_core` cannot select it (`^0.4.3`).
