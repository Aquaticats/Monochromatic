# Testing Slint apps headless (agent-driven UI testing)

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/desktop-app/terminal` is now `package/terminal/app`.

How a future session drives,
 inspects,
 and screenshots this repo's Slint 1.17 GUIs without a physical display,
 and
writes regression tests for UI behavior.
 Two apps use Slint:
 `package/music-player/desktop-app` and
`package/desktop-app/terminal`.
 Every seam below is wired and was verified end to end.

Mechanism internals and the traps behind these seams live in
`doc/troubleshooting/slint-embedded-mcp-server.md`;
 read it before changing any of the wiring.

## Seam 1: in-process ElementHandle regression tests (deterministic, CI)

Best for asserting UI behavior (bindings,
 models,
 interaction) as permanent regression tests.
 Runs headless with a
mock renderer,
 no display.

- Run:
   `mise run //package/music-player/desktop-app:test` (the whole `cargo nextest` suite,
   UI tests included).
- Where:
   `package/music-player/desktop-app/src/ui_binding_tests.rs`,
   a `#[cfg(test)]` bin module wired from
  `main.rs`.
   It uses `i-slint-backend-testing` (dev-dependency,
   pinned `=1.17.0`) to locate the seek/volume
  `Slider`s,
   drive their accessibility `SetValue`,
   and assert the thumb still tracks engine-pushed `position`/`volume`
  (guards the two-way `value <=> ...` bindings;
   see `doc/troubleshooting/slint-slider-binding-breaks-on-input.md`).
- The `test` task sets `SLINT_EMIT_DEBUG_INFO=1` for you;
   the `ElementHandle` API refuses to run without it.
- To add a test:
   locate elements with `ElementHandle::find_by_element_type_name(&app, "Slider")` (no id metadata
  needed) or `find_by_element_id(&app, "AppWindow::<id>")`;
   give the element an id in the `.slint` (`foo := Slider {}`)
  when you need to target it precisely.

## Seam 2: embedded MCP server (drive the running app live)

Best for exploratory driving,
 real-data screenshots,
 and clicking through flows.
 Slint 1.17's embedded MCP server
(the `slint/mcp` feature,
 activated by `SLINT_MCP_PORT`) runs inside the app and exposes 14 tools over MCP Streamable
HTTP on loopback.

Launch the app (each `mcp` task builds `--features slint/mcp`,
 runs headless,
 isolates state under `target/mcp-xdg`,
and loads paused so no audio plays):

```bash
mise run //package/music-player/desktop-app:mcp -- fixtures   # binds 127.0.0.1:9315
mise run //package/desktop-app/terminal:mcp                   # binds 127.0.0.1:9316
```

Run from the repo root with the pinned `//path:mcp` task id;
 a bare `mise run mcp` is ambiguous (both apps define
`mcp`).
 Each task leaves the app running in the foreground;
 background it (or run in a separate shell) while you
drive.

Reach the tools two ways:

- Via `.mcp.json` (committed at the repo root,
   entries `slint-music-player` -> 9315 and `slint-terminal` -> 9316):
  Claude Code exposes the tools natively,
   but only after the app is running AND the session has (re)connected the MCP
  server.
   A server reads "unavailable" until its app is launched.
   The `.mcp.json` carries a `//` warning:
   only local,
  loopback,
   no-auth servers belong in that tracked file.
- Via raw JSON-RPC (works in the current session without a reconnect):

```bash
curl -s http://127.0.0.1:9315/mcp \
  -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"list_windows","arguments":{}}}'
```

Tool workflow and argument-name traps (verified):

1. `list_windows` -> a `windowHandle` object `{generation, index}` (nested as JSON in `content[0].text`).
2. `find_elements_by_id` needs `{windowHandle, elementsId}` (the field is `elementsId`,
    not `id`);
    returns
   `elementHandles`.
3. `get_element_properties` / `set_element_value` / `click_element` take `{elementHandle, ...}` (element handle,
    not
   window handle);
    `set_element_value` takes `value` as a string.
4. `take_screenshot` takes `{windowHandle, imageMimeType}` and returns an inline base64 PNG in an `image` content
   item.

Default backend is `SLINT_BACKEND=headless` (real frames off-screen,
 software renderer).
 Override to a live GPU
render with the nested-niri path below.

## Seam 3: slint-viewer (static markup check and snapshot)

Best for a fast markup compile-check and a default-state layout snapshot.
 No Rust build,
 no backend.

```bash
mise run //package/music-player/desktop-app:lint:slint    # slint-viewer --check ui/app.slint
mise run //package/music-player/desktop-app:screenshot    # -> target/app.slint.png (headless software render)
```

The terminal package has the same two tasks.
 `screenshot` renders the markup with default property values only (no
engine/model data);
 use Seam 2's `take_screenshot` for real-data snapshots.

## Live GPU render (optional): nested niri

The default headless path renders with the software renderer.
 To exercise the real GPU/winit render path off your
main workspace,
 run the same `mcp` build inside a nested niri compositor.
 niri is installed on this Bazzite host through
rpm-ostree live apply:

```bash
rpm-ostree install --apply-live --assumeyes --idempotent niri
```

Do not install niri with `mise install cargo:niri@latest`:
 that installs upstream's sentinel crate,
 not a runnable
compositor.
 Details and the source trace live in `doc/troubleshooting/niri-mise-cargo-install.md`.
Issue #272 tracks replacing this dependency with a repo-owned minimal nested Wayland session tailored to GUI tests.

1. Launch niri nested from the live Wayland session:

   ```bash
   mkdir --parents target/niri-gpu-test/xdg-config target/niri-gpu-test/xdg-cache target/niri-gpu-test/xdg-data
   : > target/niri-gpu-test/empty.kdl
   XDG_CONFIG_HOME="$PWD/target/niri-gpu-test/xdg-config" \
     XDG_CACHE_HOME="$PWD/target/niri-gpu-test/xdg-cache" \
     XDG_DATA_HOME="$PWD/target/niri-gpu-test/xdg-data" \
     NIRI_CONFIG="$PWD/target/niri-gpu-test/empty.kdl" \
     niri
   ```

2. In a second shell,
    run the app as a nested Wayland client with the GPU backend.
    Use the `WAYLAND_DISPLAY` and
   `NIRI_SOCKET` values printed by niri:

   ```bash
   WAYLAND_DISPLAY=wayland-1 \
     NIRI_SOCKET=/run/user/1000/niri.wayland-1.<pid>.sock \
     SLINT_BACKEND= SLINT_MCP_PORT=9315 \
     mise run //package/music-player/desktop-app:mcp -- fixtures
   ```

3. Capture the nested compositor output with niri IPC:

   ```bash
   NIRI_SOCKET=/run/user/1000/niri.wayland-1.<pid>.sock \
     niri msg action screenshot-screen --show-pointer false --path "$PWD/target/niri-gpu-test/niri-screen.png"
   ```

4. Drive and capture app internals through Slint MCP as usual.
    Empty `SLINT_BACKEND` lets Slint pick the winit backend,
   while `SLINT_MCP_PORT` still starts the embedded MCP server for `take_screenshot`,
    property reads,
    and input.

Installed fallback:
 `cage` (a single-app nested wlroots compositor) can host the app similarly
(`cage -- <binary>`);
 its wlr-screencopy surface is capturable by wlroots screenshot tools if one is installed.

## Prerequisites and notes

- `slint-viewer` and `slint-lsp` 1.17.0 are provided by the root mise config;
   `SLINT_ENABLE_EXPERIMENTAL_FEATURES=1`
  is set per package (the UI uses the experimental FlexboxLayout).
- The `mcp` tasks isolate `XDG_CONFIG_HOME`/`XDG_CACHE_HOME`/`XDG_DATA_HOME` under `target/mcp-xdg`,
   so a test
  instance never collides with an installed app's cache lock or touches the real session;
   state resets each run.
- Never pass `--start-playing` and do not click the music player's Play unless volume is neutralized first:
   a driven
  Play emits real audio on the live PipeWire session.
- The apps skip their explicit `set_platform` (and its Wayland app-id hook) only when `SLINT_MCP_PORT` is set;
  production runs are unaffected.
   This gate is why the MCP server starts at all (see the troubleshooting doc).
