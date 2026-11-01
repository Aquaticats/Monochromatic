# macOS release builds warn `stripping debug info with rust-objcopy failed: SIGABRT`
(libLLVM.dylib), but the binary is still stripped

## Symptom

A `cargo build --release` of `package/music-player/desktop-app` on the Apple
Silicon Mac (`ssh m1`,
 rustc 1.98.0-nightly,
 `nightly-aarch64-apple-darwin`)
finishes successfully (exit 0) but prints one warning,
 because the
`[profile.release] strip = "symbols"` setting runs `rust-objcopy` as a post-link
step and it aborts:

```text
warning: stripping debug info with `rust-objcopy` failed: signal: 6 (SIGABRT)
  = note: dyld[NNNNN]: Library not loaded: @rpath/libLLVM.dylib
            Referenced from: <...> /Users/user/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/rust-objcopy
            Reason: tried: '.../aarch64-apple-darwin/bin/../lib/libLLVM.dylib' (no such file), ...
warning: `music-player` (build script) generated 1 warning
    Finished `release` profile [optimized] target(s) in ...
```

It also appears on the unmodified crate:
 the `strip = "symbols"` profile setting
predates the cross-platform work,
 so it is not caused by the cpal/QoS/identity
changes.

## Root cause

`rustc`'s `strip = "symbols"` invokes the toolchain's bundled `rust-objcopy`,
which dynamically loads `libLLVM.dylib` via the `@rpath/libLLVM.dylib` install
name and the `LC_RPATH` `@loader_path/../lib`.
 Two facts,
 both verified on the
box:

- The dylib exists at exactly the searched path
  (`.../rustlib/aarch64-apple-darwin/lib/libLLVM.dylib`,
   139 MB),
   and `rust-objcopy`
  loads it fine when invoked directly:
   `rust-objcopy --version` prints
  `LLVM version 22.1.6-rust-1.98.0-nightly` (exit 0,
   three consecutive runs),
   and
  a manual `rust-objcopy --strip-debug <copy>` succeeds (exit 0).
   So the tool and
  the dylib are not broken.
- The failure happens only when `cargo`/`rustc` spawns `rust-objcopy` as its strip
  sub-step.
   This is an upstream rustc-on-macOS-nightly interaction (the spawn
  context fails to resolve the same `@rpath`),
   not a defect in this repo.
   Earlier
  the `llvm-tools` component was simply absent;
   installing it
  (`rustup component add llvm-tools-preview`) made `rust-objcopy` loadable
  standalone but did NOT silence the build warning.

The warning is non-fatal:
 the build still produces the binary.

## Verification

Toolchain:
 `nightly-aarch64-apple-darwin`,
 rustc 1.98.0-nightly,
 libLLVM
22.1.6-rust-1.98.0-nightly,
 macOS 26.5.1 (arm64).

The produced binary is already stripped despite the warning:

```bash
# On ssh m1, after `cargo build --release`:
ls -la target/release/music-player          # 8,622,864 bytes (~8.6 MB)
nm target/release/music-player | wc -l      # 370  (a non-stripped release binary
                                            #       has thousands)

# A manual rust-objcopy strip of a copy removes almost nothing, confirming the
# artifact going into the redundant step is already stripped:
OBJ=~/.rustup/toolchains/nightly-aarch64-apple-darwin/lib/rustlib/aarch64-apple-darwin/bin/rust-objcopy
cp target/release/music-player /tmp/mp-copy
"$OBJ" --strip-debug /tmp/mp-copy           # exit 0
ls -la /tmp/mp-copy                         # 8,622,848 bytes (16 bytes smaller)
```

So the symbol table is already gone before the warned step runs;
 the failing
`rust-objcopy` pass is redundant on this target.
 The shipped Mach-O (and therefore
the signed `.app`) is small and stripped.

## Verified workarounds

- Do nothing.
   The warning is cosmetic on this toolchain:
   the binary builds and is
  effectively stripped.
   This is the chosen course;
   the signing pipeline is
  unaffected.
   Tradeoff:
   the warning line stays in release-build output on m1.
- If the warning must be silenced,
   update the toolchain (`rustup update nightly`,
  or reinstall `rust-std`/`llvm-tools` for the nightly).
   Tradeoff:
   it moves the
  pinned-rev Slint build onto a newer rustc;
   that build only requires rustc >= 1.92,
  so a newer nightly is compatible,
   but it is a toolchain change to make
  deliberately,
   not silently.
- Setting `strip = false` in `[profile.release]` would also remove the warning,
   but
  the `Cargo.toml` comment marks the symbol strip as wanted for size,
   and the artifact
  is already stripped here anyway,
   so this is not worth the size regression on
  non-macOS targets.

## What does not work

- `rustup component add llvm-tools-preview`:
   makes `rust-objcopy` loadable when run
  directly,
   but the `cargo`/`rustc`-spawned strip step still SIGABRTs.
   Verified on
  the box.

## Upstream filing decision

Nothing to file from this repo.
 Walking the 6-constraint check stops at the first
constraint:

1. Is it upstream's fault?
    Partly:
    it is a rustc-nightly + macOS `rust-objcopy`
   spawn-context quirk,
    not a defect in `music-player`.
    But it does not affect our
   output (the binary is stripped regardless),
    so there is no user-facing problem to
   report,
    and a fast-moving nightly is the wrong target for a bug report about a
   redundant,
    non-fatal step.
    If it ever blocks a build,
    search the rust-lang/rust
   tracker for `rust-objcopy` + `libLLVM.dylib` strip failures before filing;
    this
   class of issue has appeared there before.

Because there is no impact on the artifact,
 this doc records the diagnosis and the
"do nothing" decision rather than keeping a fileable issue draft.
