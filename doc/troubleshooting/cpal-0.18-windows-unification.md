# cpal 0.15 pulls a second `windows` crate (0.54) and cpal 0.18's API breaks surface only on native macOS/Windows builds

`package/music-player/desktop-app` uses `cpal` for its non-Linux audio backend
(`src/output_cpal.rs`:
 CoreAudio on macOS,
 WASAPI on Windows).
 Two related
problems showed up while making the player properly cross-platform.

## Symptom

1.  The dependency graph compiled two different major versions of the `windows`
    crate.
     The resolved lockfile (any platform,
     since `Cargo.lock` is
    target-agnostic) listed both:

    ```text
    name = "windows"   version = "0.54.0"   # pulled only by cpal 0.15.3
    name = "windows"   version = "0.62.2"   # pulled by the Slint/skia stack
    ```

    Two copies of a large crate is wasted compile time and binary surface.

2.  After bumping cpal to unify them,
     the **native** (macOS/Windows) build failed,
    while a Linux `cargo check` stayed green.
     The error,
     from compiling
    `output_cpal.rs`:

    ```text
    error[E0423]: expected function, tuple struct or tuple variant, found type alias `SampleRate`
       --> src/output_cpal.rs:321:26
        |
    321 |             sample_rate: SampleRate(rate),
        |                          ^^^^^^^^^^ not a function, tuple struct or tuple variant
    ```

    The Linux check passes because `output_cpal.rs` is gated `cfg(not(target_os =
    "linux"))`,
     so cpal is never compiled on Linux.
     The break is invisible until the
    code is built on a Mac (`ssh m1`) or Windows box (`ssh x13-win`).

## Root cause

### Why two `windows` crates

`windows` is pulled by both cpal and the Slint/skia GUI stack,
 and the version
ranges did not overlap until a recent cpal release.
 Verified from each cpal tag's
`Cargo.toml` (`https://raw.githubusercontent.com/RustAudio/cpal/<tag>/Cargo.toml`,
`[target.'cfg(target_os = "windows")'.dependencies]`):

```text
cpal 0.15.3   windows = "0.54.0"
cpal 0.16.0   windows = "0.54.0"
cpal 0.17.0   windows = ">=0.59, <=0.62"
cpal 0.18.x   windows-core = ">=0.61, <=0.62" ; windows = ">=0.61, <=0.62"
```

The Slint/skia consumers in this graph all pin `windows` 0.62.2 (verified by
scanning the lockfile's dependency lists:
 `accesskit_windows` 0.32.1,
`fontique` 0.9.0,
 `i-slint-core`/`i-slint-renderer-skia`/`i-slint-backend-winit`
1.17.0,
 `skia-safe` 0.97.2).
 So `windows` 0.62.2 is the only version that
satisfies both the GUI stack and cpal once cpal is at >= 0.17.
 cpal 0.15.3 (and
0.16.0) sit at exactly `0.54.0`,
 which cannot unify,
 so cargo compiles a second
copy.

### Why the API break only shows on native builds

cpal 0.18 changed two pieces of the small surface `output_cpal.rs` touches,
 both
documented in cpal's `UPGRADING.md` for 0.18:

- "1.
   SampleRate is now a u32 type alias" (`UPGRADING.md`):
   `pub type SampleRate =
  u32;` in `src/lib.rs`.
   So `SampleRate(rate)` (the 0.15 newtype constructor) is no
  longer callable;
   the field takes a plain `u32`.
- "2.
   StreamConfig is now passed by value" (`UPGRADING.md`):
  `DeviceTrait::build_*_stream` takes `StreamConfig` by value (it is now `Copy`),
  so `build_output_stream(&config, ...)` must drop the `&`.

cpal's error types were also unified into one `cpal::Error`,
 but the player only
`Debug`-prints them (`format!("{e:?}")`),
 so that change needed no code edit.

Because cpal is `cfg(not(target_os = "linux"))`,
 none of this is compiled by a
Linux `cargo check`/`clippy`/`test`.
 The host Linux gates all pass while the macOS
and Windows builds fail;
 only building on the native toolchains catches it.

## Verification

Versions under test:
 cpal 0.15.3 -> 0.18.1;
 `windows` 0.62.2;
 Slint pinned at
`slint-ui/slint@85e3eb76819762cdcaa732fa87533ff896546bac` (1.17.0).
 Built on
`nightly-aarch64-apple-darwin` (m1,
 macOS 26.5.1) and `x86_64-pc-windows-msvc`
(x13-win,
 Windows 10 19044).
 Each native build needs
`SLINT_ENABLE_EXPERIMENTAL_FEATURES=1` (see
[slint-lsp-experimental-features](slint-lsp-experimental-features.md));
 the mise
tasks set it.

Inspect the windows-crate count from the lockfile (no build needed,
 the lock is
target-agnostic):

```bash
rg -A1 '^name = "windows"$' package/music-player/desktop-app/Cargo.lock
```

What fails:

- cpal 0.15.3 / 0.16.0:
   two `windows` entries (0.54.0 and 0.62.2).
- cpal 0.18 with the cpal-0.15 code shape:
   `output_cpal.rs` does not compile on
  macOS/Windows (`SampleRate(rate)` -> E0423;
   `build_output_stream(&config, ...)`
  -> a by-value/borrow mismatch).

What works:

- cpal 0.18.1 plus the two edits (assign `sample_rate: rate` and drop the import of
  `SampleRate`;
   pass `config` by value):
   one `windows` 0.62.2 and one
  `windows-core` 0.62.2 in the lock,
   and clean `cargo build` + `cargo clippy --
  -D warnings` on Linux (check only),
   macOS,
   and Windows.

## Verified workarounds

- Bump cpal to 0.18 and apply the two API edits.
   Chosen.
   Tradeoff:
   a 0.15 -> 0.18
  major jump (MSRV 1.85,
   fine on the nightly toolchain),
   but it unifies `windows`
  and keeps the player on a maintained cpal.
   Recorded in the `Cargo.toml`
  dependency comment.
- Stay on cpal 0.15 and accept two `windows` crates.
   Tradeoff:
   no code churn,
   but a
  second large crate compiles and the version skew can resurface other duplicate
  transitive deps.
- `[patch.crates-io]` to force cpal's `windows` to 0.62 without a cpal bump.
  Tradeoff:
   brittle (cpal 0.15 was written against the `windows` 0.54 API,
   so a
  patched 0.62 may not even compile cpal);
   rejected in favor of the clean bump.

## What does not work

- Bumping cpal only to 0.16.0:
   it still pins `windows = "0.54.0"`,
   so the duplicate
  remains.
   The unification needs cpal >= 0.17 (`windows >=0.59, <=0.62`).
- Relying on a Linux `cargo check` to catch the API break:
   cpal is not compiled on
  Linux,
   so the native-only failure is invisible there.
   Building on the real
  macOS/Windows toolchains is the only gate that catches it.

## Upstream filing decision

Nothing to file.
 Walking the 6-constraint check stops at constraint 1:

1.  Is it upstream's fault?
     No. cpal's per-release `windows` version ranges and its
    0.18 API changes (`SampleRate` alias,
     by-value `StreamConfig`) are deliberate,
    documented in cpal's own `UPGRADING.md`.
     There is no defect:
     the duplicate
    `windows` was our graph holding cpal back at 0.15,
     and the API break is a normal,
    announced major-version migration we completed.

Because constraint 1 fails,
 the remaining constraints do not apply and there is no
tracker to file against.
 No `.out-of-scope/` exemption is needed;
 cpal behaves as
documented.
