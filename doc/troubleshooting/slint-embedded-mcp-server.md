# Slint 1.17 embedded MCP server: it never starts for an app that installs its own platform with `set_platform`, and `SLINT_BACKEND=headless` is silently ignored

Tool under test:
 Slint 1.17.0 (`slint`,
 `i-slint-backend-winit`,
 `slint-build`,
 all pinned to `1.17.0`),
 embedded MCP
server (the `slint/mcp` Cargo feature,
 activated at runtime by `SLINT_MCP_PORT`).
 Surface trigger:
 a binary that
constructs the winit backend explicitly and calls `slint::platform::set_platform(...)` before creating its first
window (both desktop apps here do this,
 to stamp the Wayland `app_id`).
 Failure mode:
 with the `mcp` feature compiled
in and `SLINT_MCP_PORT` set,
 nothing ever listens on that port,
 and `SLINT_BACKEND=headless` opens a real window
instead of rendering off-screen.

## Symptom

Build the app with `--features slint/mcp`,
 run it with `SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=9315
SLINT_BACKEND=headless`,
 and:

- No TCP listener appears on `127.0.0.1:9315` (`ss -ltn | grep 9315` is empty;
   `curl http://127.0.0.1:9315/mcp`
  exits 7,
   connection refused).
- The process is otherwise healthy and stays running.
- Despite `SLINT_BACKEND=headless`,
   a real window is created on the live Wayland session (the headless request is
  ignored),
   so on a machine with a display you see the app pop up;
   on a headless box it fails to find a display
  instead of rendering off-screen.

There is no error message:
 the MCP server simply never initializes.
 The Slint runtime prints
`Slint MCP server listening on http://127.0.0.1:<port>/mcp` when it does start,
 and that line is absent.

## Root cause

Slint starts the embedded MCP server only when Slint itself creates the platform backend through its selector.
 An
app that calls `set_platform` supplies the backend first,
 so the selector's post-creation hook never runs.

The hook lives in the backend selector's `with_global_context`,
 which creates the backend and then,
 only if it
created one this call,
 initializes the testing/MCP backends:

```rust
// internal/backends/selector/lib.rs:189
pub fn with_global_context<R>(f: impl FnOnce(&SlintContext) -> R) -> Result<R, PlatformError> {
    let mut platform_created = false;
    let result = i_slint_core::with_global_context(
        || {
            let backend = create_backend();
            platform_created = backend.is_ok();
            backend
        },
        f,
    );

    #[cfg(any(feature = "system-testing", feature = "mcp"))]
    if result.is_ok() && platform_created {
        init_testing_backends();
    }

    result
}
```

`i_slint_core::with_global_context` runs its `create_backend` closure only when no platform has been set yet.
 If the
app already called `set_platform`,
 the closure never runs,
 `platform_created` stays `false`,
 and the
`init_testing_backends()` call is skipped.
 That function is the only caller of the MCP server init:

```rust
// internal/backends/selector/lib.rs:166
#[cfg(any(feature = "system-testing", feature = "mcp"))]
pub(crate) fn init_testing_backends() {
    #[cfg(feature = "system-testing")]
    if let Err(e) = i_slint_backend_testing::systest::init() {
        i_slint_core::debug_log!("System testing init failed: {e:?}");
    }

    #[cfg(feature = "mcp")]
    if let Err(e) = i_slint_backend_testing::mcp_server::init() {
        i_slint_core::debug_log!("MCP server init failed: {e:?}");
    }
}
```

Both consuming apps install their own platform.
 The music player builds the winit backend with a window-attributes
hook and sets it as the process platform:

```rust
// package/music-player/desktop-app/src/main.rs (before the fix)
let backend = builder.build()?;
slint::platform::set_platform(Box::new(backend))
    .expect("no Slint platform should already be set");
```

The terminal does the same in its `install_backend()`.
 Because `set_platform` wins the race,
 `create_backend()` is
never reached,
 so neither the selector's `SLINT_BACKEND` parsing nor `init_testing_backends()` runs.
 That single fact
explains both symptoms:
 the MCP server never binds,
 and `SLINT_BACKEND=headless` (which is handled inside
`create_backend`) has no effect.

`SLINT_BACKEND=headless` is itself gated on the `mcp` feature plus a software or skia renderer,
 so it only exists as a
value when the app is built with the feature:

```rust
// internal/backends/selector/lib.rs:130
#[cfg(all(feature = "mcp", supports_headless))]
"headless" => return create_headless_backend(_renderer),
```

`supports_headless` is `any(renderer-software, renderer-skia)` (selector `build.rs`),
 and both apps enable
`renderer-software`,
 so the headless path is available once the app stops bypassing the selector.

Separately,
 `ElementHandle` and the MCP introspection tools require debug info in the generated UI.
 Without it the
tools return an error rather than silently finding nothing:

```txt
The use of the ElementHandle API requires the presence of debug info in Slint compiler generated code.
Set the `SLINT_EMIT_DEBUG_INFO=1` environment variable at application build time or use `compile_with_config`
and `with_debug_info` with `slint_build`'s `CompilerConfiguration`
```

## Verification

Version under test:
 `slint`/`i-slint-backend-winit`/`slint-build` 1.17.0 (crates.io,
 released 2026-06-24);
 source read
from a shallow clone of `slint-ui/slint` whose `CHANGELOG.md` top section is `## [1.17.0] - 2026-06-24`
(`CHANGELOG.md:120` records `Added MCP server feature. (#11542)`).

Harness (from `package/music-player/desktop-app`):

```bash
# FAILS to bind before the fix (explicit set_platform runs unconditionally):
SLINT_EMIT_DEBUG_INFO=1 SLINT_MCP_PORT=9315 SLINT_BACKEND=headless \
  cargo run --features slint/mcp -- fixtures &
ss -ltn | grep 9315          # empty; server never bound
curl -s -m 3 http://127.0.0.1:9315/mcp   # exit 7, connection refused

# WORKS after the fix (set_platform skipped when SLINT_MCP_PORT is set):
mise run mcp -- fixtures &    # sets the three env vars, builds --features slint/mcp
ss -ltn | grep 9315          # LISTEN 127.0.0.1:9315
curl -s http://127.0.0.1:9315/mcp -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -d '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"p","version":"0"}}}'
# -> {"result":{...,"serverInfo":{"name":"slint-mcp-embedded","version":"0.1.0"}}}
```

What works after the fix (verified end to end):
 `initialize`,
 `tools/list` (14 tools),
 `list_windows`,
`find_elements_by_id` with `elementsId: "AppWindow::seek-slider"` plus a `windowHandle`,
 `get_element_properties`
(read `accessibleValue`),
 `set_element_value` (set the seek slider to `0.5`,
 read back `0.5`),
 and `take_screenshot`
(a valid PNG showing the seek thumb at the midpoint).
 Both the music player (port 9315) and the terminal (port 9316)
bind and drive headless.

### Settle a headless frame before reading timer-driven Flickable geometry

Issue #457 added a timer-delayed horizontal reveal.
A direct selected-page write set `selected=25`,
but an immediate `get_element_tree` still returned `Alpha` through `Golf`,
`viewport-x = 0`,
and reveal generation `0`.
Calling `take_screenshot` once as a settlement probe,
then reading the tree,
returned `Tango` through selected `Zulu`,
`viewport-x = -1110`,
and settled generation `5`.

Slint 1.17.0 source tag `v1.17.0` at commit
`fdde7a535305d2ab2d4072dee637bad186a49723` explains the operation boundary:

- `internal/backends/testing/search_api.rs:348-383` walks the current item tree and filters by current visibility;
- `internal/backends/testing/search_api.rs:859-866` reports the current geometry through `map_to_window`;
- `internal/backends/testing/introspection/mod.rs:249-279` makes `take_screenshot` call
  `window.take_snapshot()` and encode the rendered buffer.

Tree/property reads inspect current state but do not request a rendered snapshot.
A headless verification whose result depends on a timer or deferred layout must therefore:

1. call `take_screenshot` as a frame-settlement probe;
2. read the element tree and verify intended geometry;
3. call `take_screenshot` again only if a retained evidence image is needed.

Tradeoff:
the first PNG is a probe and may show a transitional frame,
so it must not be presented as final visual evidence.

What fails (each is its own trap):

- `set_platform` present and no gate:
   MCP server never binds,
   `SLINT_BACKEND=headless` ignored (this doc's cause).
- `mcp` feature compiled but `SLINT_MCP_PORT` unset:
   `mcp_server::init()` returns immediately,
   no server (by design).
- Debug info absent (`SLINT_EMIT_DEBUG_INFO` unset at build):
   `find_elements_by_id` / `get_element_tree` and the
  `ElementHandle` unit-test API error with the message quoted above.
- `find_elements_by_id` argument named `id`:
   `Error: Invalid parameters: unknown field 'id'` (the field is
  `elementsId`);
   and calling it without a `windowHandle`:
   `Error: missing windowHandle`.

## Verified workarounds

All three are consumer-side and live in this repo.

Gate the explicit `set_platform` on `SLINT_MCP_PORT` being unset,
 so Slint creates the backend itself (and starts the
MCP server,
 honoring `SLINT_BACKEND`) whenever the port is set:

```rust
// package/music-player/desktop-app/src/main.rs
if std::env::var_os("SLINT_MCP_PORT").is_none() {
    // ... build the winit backend with the app-id hook and set_platform ...
}
// package/terminal/app/src/main.rs: install_backend() early-returns Ok(()) when the port is set.
```

Tradeoff:
 in MCP mode the Wayland `app_id` hook is dropped,
 so KDE taskbar grouping and taskbar progress do not apply
to the test instance.
 This is invisible to a headless test run and is the whole point (let Slint own the backend).
Production runs (`SLINT_MCP_PORT` unset) are byte-identical to before.

Pass the feature on the command line and set debug info only for the introspection build (the `mise run mcp` task
does both),
 never in `Cargo.toml [features]`:

```bash
SLINT_EMIT_DEBUG_INFO=1 cargo build --features slint/mcp
```

Tradeoff:
 `SLINT_EMIT_DEBUG_INFO` embeds element ids and source locations,
 which slightly enlarges the UI data and is
why it is kept out of the size-optimized production build.
 `cargo` rebuilds the UI when the variable toggles between
the `mcp`/`test` tasks and the production `build`/`run` tasks.

Isolate the test instance's state so it never collides with a running installed app.
 The music player opens a Turso
cache at `~/.config/musicplayer/peaks.db`;
 a second instance hits
`Locking error: Failed locking file ... locked by another process` (logged,
 non-fatal,
 from
`package/music-player/desktop-app/src/peakcache_service.rs`).
 The `mise run mcp` task points
`XDG_CONFIG_HOME`/`XDG_CACHE_HOME`/`XDG_DATA_HOME` at a throwaway `target/mcp-xdg` so it never touches the user's real
session or cache.

Tradeoff:
 the test instance starts from an empty session every run (no restored queue/volume),
 which is the correct
behavior for a reproducible test.

## What does not work

- `SLINT_BACKEND=testing` for screenshots:
   that backend keeps the mock renderer with fixed font metrics and renders
  no pixels,
   so `take_screenshot` produces nothing.
   Only `SLINT_BACKEND=headless` (mcp feature + software/skia
  renderer) renders real frames off-screen.
   `SLINT_BACKEND=headless` is marked unstable by Slint and its exact value
  may change between releases.
- Putting `mcp` in `Cargo.toml [features]` (or a default feature):
   it would compile the introspection server into the
  production binary.
   Keep it command-line only.
- A nested `mise run` from inside a package task to stage build artifacts:
   on this repo it re-loads the
  sops-encrypted mise config and aborts with `failed to decrypt sops file ... no data key retrieved`.
   The terminal
  `mcp` task stages `libghostty-vt` inline (glob plus copy) instead of shelling out to
  `mise run ...:stage:runtime-libs:debug` for that reason.
- Treating an immediate `get_element_tree` response as proof of final timer-driven placement:
   it can describe the pre-frame Flickable viewport.
   Trigger a snapshot frame,
  verify through a fresh tree read,
  then retain a second screenshot if needed.

## Upstream filing decision

`.out-of-scope/` holds no Slint or MCP exemption (checked `bun-install.md`,
 `cargo-workspace.md`,
`claude-code-upstream-bugs.md`,
 `codex-harness.md`,
 `jsr.md`,
 `lightningcss.md`,
`low-impact-typescript-formatting.md`,
 `module-es-monolith.md`,
 `pi-gpt55-long-context.md`,
`terminal-title-fork-parity-tests.md`),
 so the 6-constraint check applies.

1. Really upstream's fault?
    Partly.
    The behavior is a consequence of `set_platform` bypassing the selector,
    which is
   defensible design (if you supply your own platform you opt out of the selector's extras).
    The gap is that this is
   undocumented:
    the MCP docs assume `cargo run --features slint/mcp` with the default backend and never mention that
   a custom `set_platform` disables the server.
    So this reads as a documentation gap,
    not a code defect.
2. Can upstream fix it?
    Yes,
    cheaply (call the MCP init from a `set_platform` path too,
    or document the interaction),
   so this constraint does not fail.
3. Supporting this use case?
    The `mcp` feature clearly targets agent-driven UIs,
    but nothing in the docs covers the
   custom-platform combination;
    support for it is unstated.
4. Would the repo welcome the contribution?
    Not established here;
    a doc-only note is low value and the code change is
   debatable design,
    so this is weak.
5. Will they likely fix it?
    No signal either way;
    plausibly "working as intended" for a self-set platform.
6. Prototyped a minimal fix?
    Not attempted.
    The user-facing problem is fully solved consumer-side by the one-line
   gate above,
    which is the correct place for it (an app that opts into a custom platform opts into managing this).

Constraints 3 to 5 are soft-to-weak and the clean fix is consumer-side,
 so this does not clear the bar to file,
 and
the auto-prototype trigger (1 to 5 all holding or sorta-holding) is not met.
 Decision:
 do not file.
 No duplicate
search escalation is warranted for a documentation-gap that is resolved at our boundary;
 if a future session wants
to push a docs note upstream,
 re-run the duplicate search (`gh search issues --repo slint-ui/slint mcp set_platform`)
and the constraint check first.
