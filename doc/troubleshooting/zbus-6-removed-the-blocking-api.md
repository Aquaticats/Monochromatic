# zbus 6 removed `zbus::blocking`, so a lock moved to zbus 6.2.1 stops compiling the music player

Recorded 2026-11-07T10:55:00-05:00.

## Symptom

After the daily update `d61dc1934a` (2026-11-07T08:33:16-05:00) locked zbus 6.2.1 in
`package/music-player/desktop-app/Cargo.lock`, the crate fails with:

```text
error[E0433]: cannot find `blocking` in `zbus`
error[E0432]: unresolved import `zbus::blocking`
```

in `src/analysis_notification_wire.rs`, `src/launcher.rs` and others (plus follow-on `E0599` and `E0277` errors that only
repeat the missing types). `package/nested-wayland-session/cli` also names `zbus::blocking` (`src/a11y_tree.rs`,
`src/a11y_service.rs`) and takes zbus from the same default catalog entry; this was not built.

## Root cause

`rust-catalog.toml` had `zbus = ">=6.2.1"`, an open-ended floor, so the update crossed the 5.19.0 to 6.0.0 major boundary
(6.0.0 was published 2026-10-16). zbus's own upgrade guide says
"`zbus::blocking`, the `*ProxyBlocking` types and the `blocking-api` feature have no equivalent in 6.0. Drive the async API
with `zbus::block_on` instead" (`book/src/upgrading-to-6.md`, section "The blocking API is gone", zbus
`d656c1f`, [source](https://github.com/z-galaxy/zbus/blob/d656c1f/book/src/upgrading-to-6.md)). Other 6.0 changes the
port meets: `Value` is now `Variant`, a blocking iterator becomes a stream, and the match-rule builder no longer returns a
`Result`.

## Verification

With `zbus-5` selected (below), `cargo metadata` on the three music-player lock files leaves one zbus, 5.19.0, and the full
release build no longer reports these errors (see the planning document for the build result).

## Verified workarounds

`package/music-player/desktop-app` selects the named catalog `zbus-5` (`^5.19.0`), the repair that
`doc/planning/toolchain-always-latest.md` records for open-ended floors (as `bincode-1`).

Tradeoff: the crate stays one major behind. The port to `zbus::block_on` touches the launcher, the analysis and unplayable
notification workers, the thread-priority call and the dialog backend (about ten files); a connection made inside one
`block_on` only runs inside it, so each worker needs its own. Open work; remove the named catalog when it lands.

## What does not work

Moving the call sites by renaming the import: `zbus::blocking` has no replacement type.

## Upstream filing decision

Nothing to file: the removal is documented and intended by zbus.
