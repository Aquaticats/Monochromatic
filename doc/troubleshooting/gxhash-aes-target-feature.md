# gxhash 3.5.0 requires the `aes` CPU target-feature at build time, has no software fallback, and panics on aarch64 debug builds

Historical record, kept for the external facts below.
Update on 2026-10-14:
the music player left `gxhash` for XXH3-64 from `twox-hash`
([`doc/decision/music-player-fingerprint-hash.md`](../decision/music-player-fingerprint-hash.md)),
and every `-C target-feature=+aes` setting, the `gxhash` dependency, and the startup concerns below are gone from it.
Update on 2026-09-17:
the monorepo manager left `gxhash` after collision findings and now uses XXH3-128
([`doc/decision/monorepo-manager-cache-key-hash.md`](../decision/monorepo-manager-cache-key-hash.md)).
The collision evidence is in
[`doc/planning/monorepo-manager-route-research/gxhash-owned.md`](../planning/monorepo-manager-route-research/gxhash-owned.md).
Everything below describes `gxhash` 3.5.0 itself and the music player's setup as of 2026-09-16,
and no current build follows it.

The music-player peak-cache fingerprint hash migrated from hand-written FNV-1a to the
`gxhash` crate (both flavors:
 `desktop-app/src/peakcache.rs` and the Android native
`android-app/rust/src/fingerprint.rs`).
 gxhash is a hardware-AES hash with constraints that
FNV-1a did not have.
 This doc records them so a future session does not rediscover them by
hitting a build break,
 a runtime crash,
 or a debug-build panic.

Why gxhash:
 per the user on 2026-09-16,
 it benchmarked best among the hashes tried;
 the benchmark inputs and results were not recorded.

Runtime guard gotcha:
 in a build with `-C target-feature=+aes`,
 `is_x86_feature_detected!("aes")` is `true` at compile time,
 because std's detection macro evaluates `cfg!(target_feature = ...)` before runtime detection
 (`library/std_detect/src/detect/macros.rs:9-10` in the nightly-2026-09-12 Rust sources).
A guard that warns instead of crashing on a CPU without AES-NI must call `core::arch::x86_64::__cpuid(1)` directly
 and read ECX bit 25,
 the bit std itself uses (`std_detect/src/detect/os/x86.rs:108`).

## Symptom

Three distinct failure modes,
 plus one capability gap that shaped the design.

- Build break (feature absent):
   a default `cargo build` for a target that does not enable
  `aes` fails to compile with,
   on x86:

  ```text
  Gxhash requires aes and sse2 intrinsics. Make sure the processor supports it and build with
  RUSTFLAGS="-C target-cpu=native" or RUSTFLAGS="-C target-feature=+aes,+sse2".
  ```

  and the analogous `aes and neon` message on aarch64.
   Default Rust targets do NOT enable
  `aes` (except `aarch64-apple-darwin`,
   where it is on by default):
   `x86_64-unknown-linux-gnu`,
  `x86_64-pc-windows-msvc`,
   `aarch64-linux-android`,
   and `x86_64-linux-android` all need it
  added.
- Runtime crash (feature forced on,
   CPU lacks it):
   a binary built with `+aes` but run on a CPU
  without AES acceleration dies with SIGILL (illegal instruction).
   There is no runtime feature
  detection and no software fallback.
- Debug-build panic on aarch64 (gxhash issue #111):
   with `debug-assertions` on (so:
   any debug
  build,
   and the desktop `test` task which runs `cargo nextest run` in debug),
   hashing on
  aarch64 can panic.
   Reproduces on Apple-Silicon macOS;
   does NOT reproduce on x86_64.
- Capability gap:
   gxhash has no JVM/Kotlin port.
   The Android fingerprint,
   previously pure
  Kotlin,
   therefore had to move into the native Rust crate behind a JNI call
  (`NativeBridge.nativeFingerprint`);
   it cannot be computed in Kotlin.

## Root cause

Verified against gxhash 3.5.0 (crates.
io checksum recorded in each crate's `Cargo.lock`;
 git
tag `3.5.0`,
 cloned from `github.com/ogxd/gxhash`).

The `aes` requirement is an unconditional `compile_error!`,
 not a graceful fallback.
 On x86
(`src/gxhash/platform/x86.rs:1`):

```rust
#[cfg(not(any(all(target_feature = "aes", target_feature = "sse2"), docsrs, doc)))] // docs.rs bypasses the target_feature check
compile_error!{"Gxhash requires aes and sse2 intrinsics. Make sure the processor supports it and build with RUSTFLAGS=\"-C target-cpu=native\" or RUSTFLAGS=\"-C target-feature=+aes,+sse2\"."}
```

On aarch64 (`src/gxhash/platform/arm.rs:1`) the same gate requires `aes` and `neon`.
 There is
no `else` arm compiling a portable path,
 which is why the crate "will not build on these
platforms" (README "Hardware Acceleration").
 Feature requests for a software fallback (issue
#47) and runtime detection (PR #59) are unmerged,
 so the SIGILL-on-missing-AES behavior is
current.

The `hybrid` (AVX2/VAES) feature is the ONLY part needing nightly Rust
(`src/lib.rs:3`);
 we leave it off,
 so the default build is stable-Rust:

```rust
#![cfg_attr(feature = "hybrid", feature(stdarch_x86_avx512))]
```

Hash stability is per-major-version,
 identical across supported platforms (README:
43):

> All generated hashes for a given major version of GxHash are stable,
>  meaning that for a given
> input the output hash will be the same across all supported platforms.
>  This also means that
> the hash may change between majors versions (eg gxhash 2.
> x and 3.
> x).

So pinning `gxhash = "3"` keeps the on-disk cache key stable across minor/patch upgrades and
across x86_64/aarch64;
 a future major bump silently re-keys the cache,
 which only re-measures
each track once (a cache miss is treated as "not measured yet",
 so it self-heals).

Issue #111's panic comes from the partial-block read.
 `get_partial`
(`src/gxhash/platform/mod.rs:18`) takes a fast "same page" path that reads a full vector even
when fewer bytes remain,
 guarded by a page-boundary check:

```rust
pub unsafe fn get_partial(p: *const State, len: usize) -> State {
    // Safety check
    if check_same_page(p) {
        get_partial_unsafe(p, len)
    } else {
        get_partial_safe(p, len)
    }
}
```

The read-beyond in `get_partial_unsafe` is what trips a bounds check under `debug-assertions`
on aarch64 (reporter-confirmed on 3.5.0).
 The fingerprint material here (path bytes plus 24
bytes) is almost never a whole multiple of the vector width,
 so this path is exercised.
 PRs
#98 / #100 / #118 ("use inline assembly to read beyond bounds") address it on `main` but are
not in any published release.

## Verification

Version under test:
 gxhash 3.5.0 (`grep '^version' Cargo.toml` in the clone returned `3.5.0`;
both flavors' `Cargo.lock` pin 3.5.0).
 Public one-shot API used
(`src/gxhash/mod.rs:32`):
 `pub fn gxhash64(input: &[u8], seed: i64) -> u64`.

Builds and runs that pass with the `+aes` config in place:

```bash
# Desktop: gxhash compiles under +aes (the compile_error stays silent), tests pass on x86_64.
mise run //package/music-player/desktop-app:lint        # cargo check
mise run //package/music-player/desktop-app:test         # 77 tests pass (debug, x86_64)
mise run //package/music-player/desktop-app:lint:clippy  # -D warnings, clean

# Android: cross-compile both ABIs (the compile_error canary) and run the fingerprint on device.
mise run //package/music-player/android-app:build:native # gxhash v3.5.0 compiles for arm64 + x86_64
adb shell am instrument -w \
  -e class 'dev.monochromatic.musicplayer.NativeBridgeTest#fingerprintIsDeterministicOpaqueAndChangeSensitiveOnDevice' \
  dev.monochromatic.musicplayer.test/androidx.test.runner.AndroidJUnitRunner   # OK (1 test) on a Pixel 6
```

The on-device pass proves the `+aes`-built `.so` runs gxhash on a real arm64 CPU (Pixel 6,
`/proc/cpuinfo` Features line includes `aes`) with no SIGILL,
 and that the JNI fingerprint is
deterministic,
 opaque,
 16 hex chars,
 and change-sensitive to path,
 size,
 and mtime.

The on-device test is the coverage mechanism for the Android fingerprint,
 not a host
`cargo test`:
 the native crate is Android-only and will not build off-Android,
 because its
`ndk` dependency pulls `ndk-sys`,
 which emits `compile_error!("ndk-sys only supports compiling
for Android")` for any non-Android target (a host `cargo test --no-run` dies there).
 This is why
the crate carries no `#[cfg(test)]` for any of its logic (decode,
 true-peak,
 opus,
 fingerprint
are all device-tested).
 The fingerprint's hashing algorithm is nonetheless host-tested
indirectly:
 the desktop twin's `peakcache_tests.rs` exercises the byte-identical material plus
`gxhash64` on x86_64.

What fails (build) without the config:
 a plain `cargo build` for any of the four non-Apple
target triples hits the `compile_error!` above.
 What fails (runtime):
 a `+aes` binary on a CPU
without AES SIGILLs.
 What fails (debug,
 aarch64):
 `debug-assertions`-on hashing can panic per
#111.

## Verified workarounds

- Enable `aes` per target triple in `.cargo/config.toml` (what both flavors do).
   Desktop covers
  `x86_64-unknown-linux-gnu`,
   `x86_64-apple-darwin`,
   `aarch64-apple-darwin`,
  `x86_64-pc-windows-msvc`;
   Android covers `aarch64-linux-android`,
   `x86_64-linux-android`,
   and
  the host triple.
   Example:

  ```toml
  [target.aarch64-linux-android]
  rustflags = ["-C", "target-feature=+aes,+neon"]
  ```

  Tradeoff:
   the produced binary REQUIRES a CPU with AES acceleration.
   This is safe for desktop
  (AES-NI is universal on x86_64 since ~2010,
   and every Apple-Silicon Mac has the ARM crypto
  extensions) and for modern arm64 Android phones,
   but it drops support for pre-AES-NI x86 CPUs
  and any arm64 device lacking the ARMv8 crypto extensions,
   which would SIGILL at runtime.
   Check
  `aes` in the device `/proc/cpuinfo` before shipping to a new arm64 target.
- Avoid issue #111 by using release builds,
   where `debug-assertions` is off.
   The shipped
  artifacts (desktop release,
   Android `build:native --release`) are unaffected.
   Tradeoff:
   on
  Apple-Silicon macOS,
   debug builds and the debug `test` task can still panic in gxhash;
   until a
  release past 3.5.0 ships PR #118,
   run the desktop tests on x86_64 (the project's primary CI
  target) or expect the debug-build panic on Apple Silicon.
   The Android `build:native` is always
  `--release`,
   so it is not exposed.

## What does not work

- `-C target-cpu=native` (gxhash's own `.cargo/config.toml` example):
   on a distributed,
  signed binary it bakes the build host's full feature set in,
   so the binary SIGILLs on any
  other machine;
   on a cross-compiled Android `.so` it is meaningless.
   `+aes` is the minimal,
  portable flag and is what both flavors use instead.
- A pure-Kotlin gxhash for the Android flavor:
   no JVM/Java/Kotlin port exists (Maven Central
  and GitHub searches were empty;
   the only ports are C# and C++,
   plus a Python binding that
  wraps the Rust crate).
   gxhash is defined in terms of hardware AES rounds the JVM does not
  expose,
   so a bit-exact reimplementation is impractical.
   This is why the Android fingerprint
  moved into the native crate over JNI rather than staying pure Kotlin.
- Pinning to gxhash `main` to get the #111 fix (PR #118):
   it is unreleased,
   uses inline
  assembly,
   and disables miri verification.
   Not worth a git pin for a debug-only,
  aarch64-only,
   release-unaffected panic.

## Upstream filing decision

`.out-of-scope/` has no gxhash or ogxd exemption (checked:
 `bun-install`,
 `cargo-workspace`,
`claude-code-upstream-bugs`,
 `codex-harness`,
 `jsr`,
 `lightningcss`,
`low-impact-typescript-formatting`,
 `module-es-monolith`,
 `pi-gpt55-long-context`,
`terminal-title-fork-parity-tests`,
 `typescript-project-references`).

The `aes` requirement,
 the no-software-fallback design,
 and the nightly-only `hybrid` feature
are all DOCUMENTED,
 intended behavior (README "Hardware Acceleration"),
 not bugs.
 Nothing to
file for those.

The debug-assertions panic IS a bug,
 but it is already tracked upstream:
 issue #111 ("Panic
when debug-assertions=true in release profile",
 OPEN,
 filed 2025-01-10),
 reporter-confirmed on
3.5.0,
 with PRs #98 / #100 / #118 addressing the underlying read-beyond.
 A duplicate report is
itself a publicity incident,
 so this is a comment-or-nothing decision,
 and the thread already
contains the root cause,
 a reproduction,
 affected-version data,
 and an in-progress fix.
 We have
nothing additive (no sharper trace,
 no missing repro,
 no unmentioned workaround),
 so per the
6-constraint check this resolves at the duplicate-search step:
 post nothing.
 No new-issue draft
is kept.
