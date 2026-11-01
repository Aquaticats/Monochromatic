# Terminal

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

A Slint desktop terminal prototype backed by Ghostty's `libghostty-vt` terminal core.

The prototype runs the user's shell inside a PTY,
 feeds shell output through `libghostty-vt`,
 and keeps smooth
Slint-owned pixel scrolling over scrollback.
A background PTY reader thread forwards bytes to the Slint UI thread,
 where `TerminalEngine` remains because
`libghostty-vt` handles are `!Send + !Sync`.

## Scope

- Terminal core:
   `libghostty-vt` 0.1.1,
   not `alacritty_terminal`,
   `termwiz`,
   or raw `vte`.
- UI:
   Slint `Flickable` owns pixel scroll state and smooth wheel or touch motion.
- Rendering:
   Rust extracts rows,
   cells,
   resolved colors,
   and style flags from `RenderState`.
- Resize:
   Slint reports viewport pixels to Rust.
   Rust computes terminal columns and rows from fixed cell metrics,
   calls
  `Terminal::resize`,
   and resizes the PTY.
- Input:
   `FocusScope` forwards text,
   arrows,
   editing keys,
   Ctrl-letter shortcuts,
   and Alt-prefixed keys to the PTY.
- PTY:
   `portable-pty` spawns `$SHELL` or `/bin/sh`,
   reads output on a worker thread,
   writes keyboard bytes,
   and resizes
  the kernel PTY.

## Scroll bridge

Slint stores the pixel scroll offset in the `Flickable` viewport.
Rust maps that offset to the row-aligned `libghostty-vt` viewport with this formula:

```text
whole_row_offset = floor(pixel_scroll / cell_height)
fractional_px = pixel_scroll - whole_row_offset * cell_height
```

`TerminalEngine::set_pixel_scroll` clamps the pixel offset to Ghostty's current scrollback length,
uses `Terminal::scroll_viewport` with `Top`,
 `Bottom`,
 or `Delta` to move the whole-row viewport,
and then `TerminalEngine::snapshot` extracts visible cells from `RenderState`.

The Slint content places each visible cell at:

```text
y = (whole_row_offset + viewport_row) * cell_height
```

The `Flickable` then translates the content by the original pixel offset.
When the pixel offset changes inside the same row,
 Slint moves the content smoothly without asking Ghostty for new rows.
When it crosses a row boundary,
 Rust asks Ghostty for the next whole-row viewport.

## libghostty-vt research notes

The current Rust binding was verified from `Uzaaft/libghostty-rs`,
 cloned under `/tmp/agent/libghostty-rs-20260601`.
The crate README and `crates/libghostty-vt/src/terminal.rs` document:

- `Terminal::new(TerminalOptions { cols, rows, max_scrollback })` creates the VT state.
- `Terminal::vt_write(&[u8])` feeds VT bytes.
- `Terminal::resize(cols, rows, cell_width_px, cell_height_px)` updates grid and pixel dimensions.
- `Terminal::scroll_viewport(ScrollViewport::{Top, Bottom, Delta})` moves the viewport by rows.
- `Terminal::total_rows()` and `Terminal::scrollback_rows()` expose scrollback size.

`crates/libghostty-vt/src/render.rs` documents the render path used here:

- `RenderState::new()` allocates reusable render state.
- `RenderState::update(&terminal)` returns a snapshot.
- `RowIterator` iterates visible rows.
- `CellIterator` iterates cells within each row.
- `CellIteration::graphemes`,
   `style`,
   `fg_color`,
   and `bg_color` expose text and resolved style data.

`crates/libghostty-vt-sys/src/bindings.rs` confirms the C ABI row-scroll tags:
`TOP`,
 `BOTTOM`,
 and `DELTA`,
 where delta up is negative.
It also confirms `TerminalScrollbar { total, offset, len }`,
 `TOTAL_ROWS`,
 and `SCROLLBACK_ROWS` type definitions.

`lib.rs` notes all handles are `!Send + !Sync`.
This prototype keeps `TerminalEngine` on the Slint UI thread inside `Rc<RefCell<_>>`.
The PTY reader thread sends byte chunks through `std::sync::mpsc`;
 a Slint `Timer` drains that channel on the UI thread and
feeds `TerminalEngine`.

## PTY dependency

`portable-pty` is used for PTY process management.
It supplies safe shell spawn,
 reader,
 writer,
 and resize operations without hand-written `forkpty` setup.
The dependency is pinned to WezTerm revision
`f78f72f2f18bf459561e3681f016365273d3e281`,
the first revision after the `portable-pty` 0.9.0 release that upgrades `nix` from 0.28 to 0.29.
This removes `cfg_aliases` 0.1.1 and its future-incompatible macro expansion from the build.
Use a crates.io release again when one contains that dependency upgrade.
The decision and rejected alternatives are recorded in `doc/decision/terminal.md`.

## Ghostty debug logging

Debug builds of the vendored Ghostty core can write `debug(stream): unimplemented OSC callback: ...` directly to process
stderr when a shell emits OSC commands that Ghostty parses but does not implement yet.
`src/stderr_filter.rs` installs a Unix stderr pipe filter after the PTY shell has spawned and before the UI timer drains
PTY output into Ghostty.
The filter drops only lines containing `unimplemented OSC callback` and forwards every other stderr line to the original
stderr destination unchanged.

## Font rendering

The Slint UI uses `JetBrains Mono` because the target's generic system `monospace` resolves to Noto Sans Mono,
 whose
narrow glyphs leave visibly loose prompt spacing when Slint renders terminal output one cell at a time.
A hidden Slint `Text` probe measures a 32-character monospace sample through the same renderer used for terminal cells;
Slint exposes that measured width to Rust so Ghostty resize math and Slint cell placement share one metric.
Slint's `font-weight` and `font-italic` properties render VT bold and italic styles.
The terminal content starts after a 2px horizontal gutter,
 and resize math subtracts that gutter before computing PTY
columns.
The Slint scene paints cell backgrounds first,
 then glyphs with one-cell horizontal bleed,
 then underlines.
This keeps fallback Nerd Font icons used by tools such as `lsd` from being clipped at the window edge or erased by the
next cell's background while preserving PTY column alignment.
Do not fake bold by drawing the same glyph a second time with an x-offset;
 shell prompts commonly use bold SGR,
 and the
duplicate glyph pass makes those prompt characters look distorted.

## Runtime library staging

`libghostty-vt-sys` links the vendored Ghostty core as `libghostty-vt.so.0`.
The package build script adds an rpath of `$ORIGIN/../lib/monochromatic-terminal` to `monochromatic-terminal`.
The `build`,
 `build:debug`,
 and `run` mise tasks copy `libghostty-vt.so*` from Cargo's build output into
`target/lib/monochromatic-terminal` so `target/release/monochromatic-terminal` can run from the package directory.
The `run` task also copies those shared libraries to `~/.local/lib/monochromatic-terminal`
before installing the binary in `~/.local/bin`.

## Layout

- `src/lib.rs`:
   module root.
- `src/scroll.rs`:
   pure pixel-to-row mapping and unit tests.
- `src/render.rs`:
   renderer-neutral RGB,
   cell,
   and snapshot models.
- `src/engine.rs`:
   `TerminalEngine`,
   VT feeding,
   resize,
   viewport mapping,
   and render extraction.
- `src/input.rs`:
   Slint key text to terminal byte encoding and unit tests.
- `src/pty.rs`:
   `portable-pty` shell spawning,
   PTY output events,
   input writes,
   resize,
   and tests.
- `src/stderr_filter.rs`:
   process stderr line filter for Ghostty's unimplemented OSC callback debug noise.
- `src/demo.rs`:
   deterministic demo VT stream kept for fixture content and future comparisons.
- `src/launcher.rs`:
   Wayland app-id hook,
   matching `monochromatic.terminal.desktop`.
- `src/main.rs`:
   Slint window wiring,
   PTY output draining,
   key input writes,
   and snapshot conversion.
- `ui/app.slint`:
   terminal viewport,
   `Flickable`,
   `FocusScope`,
   cell renderer,
   and resize or scroll callbacks.
- `Containerfile`:
   CachyOS build environment with Rust,
   Slint runtime libraries,
   and Zig 0.15.2.
- `Containerfile.release`:
   the Ubuntu 24.04 release build environment,
   kept so binaries that ship (the IDE release builds on it) run on the oldest glibc they promise.
- `mise.toml`:
   package-local build,
   lint,
   test,
   runtime-library staging,
   and run tasks.

## Known remaining work

- Encode mouse input back to the PTY with libghostty-vt mouse helpers.
- Add cursor rendering and selection rendering.
- Preserve scroll position when output arrives while the user is reading old scrollback.
- Replace fixed cell metrics with measured monospace font metrics.
- Move terminal ownership to a dedicated thread once PTY throughput matters.

## Commands

Cargo work runs on the host when Zig and the native dev libraries are present,
 and falls back to the CachyOS
container otherwise.
 Each task evaluates a `host_ok` predicate (cargo and zig on PATH plus
`pkg-config --exists fontconfig freetype2`);
 when all resolve it builds natively,
 and when any is missing it runs
the identical cargo command in podman.
 Zig 0.15.2 comes from the repo-wide mise `zig` tool.
 On an immutable-style
Fedora the host libraries are layered with:

```bash
rpm-ostree install fontconfig-devel freetype-devel
```

Building the container image is only needed for the container path;
 the host fallback also builds it
automatically when missing.

```bash
# package/terminal/app
mise run //package/terminal/app:image
```

Run compile checks,
 max-lines linting,
 clippy,
 and tests (host if Zig + dev libs present,
 else container):

```bash
# package/terminal/app
mise run //package/terminal/app:lint
mise run lint:monochromatic-lint -- package/terminal/app
mise run //package/terminal/app:lint:clippy
mise run //package/terminal/app:test
```

Build a release binary (and stage the libghostty-vt runtime libraries):

```bash
# package/terminal/app
mise run //package/terminal/app:build
```

Force the container path,
 asserting the container build still works on any host:

```bash
# package/terminal/app
mise run //package/terminal/app:verify:container
```

Build,
 install the `.desktop` file plus binary into the host user profile,
 and run the GUI on the host session:

```bash
# package/terminal/app
mise run //package/terminal/app:run
```
