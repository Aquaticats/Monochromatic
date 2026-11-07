# `libghostty-vt-sys` 0.2.2 fails to build on Zig 0.17.1: the vendored Ghostty requires Zig 0.15.x

Recorded 2026-11-07T10:55:00-05:00.

## Symptom

`cargo build` of `package/terminal/app` in an image with Zig 0.17.1 stopped in the build script of `libghostty-vt-sys` 0.2.2:

```text
src/build/zig.zig:13:9: error: Your Zig version v0.17.1 does not meet the required build version of v0.15.2
```

with further Zig errors (`@cImport` removed,
`b.build_root` missing) behind it.

## Root cause

The crate vendors Ghostty,
whose build compares the Zig version with the one it was written for and refuses any other.
The lock moved Zig from 0.15.2 to 0.17.1 (see `doc/planning/toolchain-always-latest.md`),
and the Containerfiles were made
to install the lock's Zig,
so the vendored Ghostty met a Zig it does not support.
The release `libghostty-vt` 0.2.2 on
crates.io still vendors that Ghostty.

## Verification

Reproduced in the Zig 0.17.1 proof image of the earlier agent.
The upstream `Uzaaft/libghostty-rs` main sys crate built in
2m10s on 0.17.1,
and `cargo check --all-targets`,
`cargo clippy --locked --release --all-targets -- --deny warnings`
and `cargo nextest run --locked --release` (16 tests) pass in that image with the fix below.

## Verified workarounds

`rust-catalog.toml` takes `libghostty-vt` from `Uzaaft/libghostty-rs` at revision
`84373f8d48b8f0b794e259f74a71a52686616f5d`,
and `package/terminal/app/src/engine.rs` follows that revision's API.

Tradeoff:
an unreleased git revision.
Return to a crates.io release once one vendors a Ghostty that builds with the
current Zig (the earlier agent noted Ghostty 1.4 or newer).

## What does not work

Keeping Zig 0.15.2 in the Containerfiles:
the owner's decision records follow the lock.

## Upstream filing decision

Not filed;
the break is the vendored Ghostty's own version gate,
and upstream main already carries the fix.
