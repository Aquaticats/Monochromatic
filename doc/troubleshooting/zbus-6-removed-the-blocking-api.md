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

The ports are in the Resolution section; `mise run //package/nested-wayland-session/cli:test` and
`mise run //package/music-player/desktop-app:test` pass on zbus 6.2.1.

## Resolution

Recorded 2026-11-07T12:09:19-05:00: the hold is gone. Both crates are ported to zbus 6's async API, the named catalog
`zbus-5` and its selector in `package/music-player/desktop-app/Cargo.toml` were removed through file-enforcer, and the
desktop app's lock moved to zbus 6.2.1 through the repository's `cargo` wrapper (`cargo-shadow`).

The replacement for a blocking call is `zbus::block_on` around the async one (zbus `CHANGELOG.md` 6.0.0, "Remove the
blocking API"; the book's "Upgrading to zbus 6.0", section "The blocking API is gone"; `src/utils.rs` of zbus 6.2.1,
documentation of `block_on`). Each synchronous thread drives its own calls with it. With the default `default-rt`
feature it runs zbus's own `zruntime` on the calling thread, and a connection built inside the call is served by that
thread while it is inside `block_on` and by a helper thread of zbus between calls, so a connection made inside one
`block_on` call keeps working in the next one and on another thread (the first draft of this note said otherwise; the
tests of both crates serve and call across threads). No second runtime and no event-loop integration were added.

What the port had to change besides the call sites:

- `zbus::blocking::*` is gone; clients use `zbus::Connection`, `zbus::Proxy` and `zbus::fdo::DBusProxy`.
- `zbus::zvariant::Value` and `OwnedValue` are `zbus::Variant` and `OwnedVariant` (the old paths still compile with a
  deprecation warning, which the repository's clippy gate turns into an error).
- A blocking iterator is a `zbus::MessageStream` read with `StreamExt::next`; `MessageStream::for_match_rule`
  registers the rule and opens the stream in one step.
- Builder and match-rule setters no longer return `Result`; `build()` returns the first error.
- A reply whose error name `zbus::fdo::Error` defines (`NotSupported`, `ServiceUnknown`, `InvalidArgs`) is
  `zbus::Error::FDO`, not `zbus::Error::MethodError`; tests that matched the variant now read the wire name from either.
- `Builder::method_timeout` bounds method calls only, zbus's own timeout helper is crate-private, and a connect or a
  signal send has no bound of its own. Both crates race `zruntime`'s timer (`SharedRuntime::current().timeout`) from
  inside `block_on`: `src/bus_step_bound.rs` in `package/nested-wayland-session/cli` and `src/dbus_step_bound.rs` in
  `package/music-player/desktop-app`. A signal listener thread's read of its stream has no bound by design; it ends
  when the connection closes.

Slint (`i-slint-backend-winit`), accesskit and `ksni` still depend on zbus 5.19.0, so
`package/music-player/desktop-app/Cargo.lock`, `package/terminal/app/Cargo.lock` and `package/ide/app/Cargo.lock`
hold zbus 5.19.0 beside the 6.2.1 the desktop app names. That is those crates' choice; this repository cannot move it.

## What does not work

Moving the call sites by renaming the import: `zbus::blocking` has no replacement type.

## Upstream filing decision

Nothing to file: the removal is documented and intended by zbus.
