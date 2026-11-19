# Key Helper

KWin script plus D-Bus daemon for KDE Plasma on Wayland.
The KWin script sees window and key events and owns the window objects;
the daemon does the actuation KWin scripts cannot:
spawning processes,
injecting keys via ydotool,
and talking to Neovim over msgpack-rpc.

The two halves talk over the session bus under `org.monochromatic.KeyHelper`.

## Features

- **Active window tracking**:
   the script reports the focused window class on every change,
   so the daemon knows whether Neovide is focused.
- **Double-shift to F20 (Neovide)**:
   the daemon watches evdev passively for two Shift taps within `DOUBLE_TAP_MS`,
   and sends `<F20>` to Neovim only while Neovide is focused.
   Reading evdev is passive,
   so it never consumes the Shift tap from other apps (e.g. VSCodium keeps its own double-shift).
- **Ctrl+F4 close tab**:
   remapped to Ctrl+W via ydotool for browsers,
   and sent as `<F16>` to Neovim for Neovide (which swallows Ctrl+F4).
- **Meta+N new instance**:
   launches another independent instance of the focused app and focuses the new window.
   Ghostty is launched with `--gtk-single-instance=false` so it is a fresh process,
   not a window in the running single-instance server;
   browsers get an explicit `--new-window`;
   everything else goes through `kstart`.

## Layout

- `src/`:
   the daemon,
   split into pure logic (`evdev-parse`,
   `keys`,
   `launch`) and I/O
   (`evdev`,
   `nvim`,
   `dbus-iface`,
   `index`).
- `kwin-script/`:
   the KWin KPackage (`metadata.json` plus `contents/code/main.js`).
   Its `DBUS_*` constants must match `src/constants.ts`;
   the Plasma JavaScript runtime cannot import TypeScript,
   so they are duplicated.

## Requirements

- KDE Plasma 6 with KWin scripting.
- [ydotool][] with `ydotoold` running (for key injection).
- Neovim (for the F16 and F20 RPC features);
   optional if you do not use Neovide.
- Node (for building);
   the shipped daemon is a self-contained Node SEA binary needing no Node at runtime.

## Build

```sh
# Build the daemon binary (ESM bundle, then embed into a Node SEA executable).
mise run //package/kwin/key-helper:binary
```

This produces `key-helper-service` in the package directory.
Rebuilding while the daemon runs is safe:
the binary is built under `dist/` and renamed over the old one,
so the running daemon keeps executing the old file until it is restarted,
and a failed build leaves the old binary in place.

## Install

```sh
# Symlink the KWin script into ~/.local/share/kwin/scripts, point autostart at
# the in-repo binary, and enable the script in kwinrc.
mise run //package/kwin/key-helper:install
```

The repo is the source of truth:
the KWin script is symlinked (edits are live after a script reload),
and autostart runs the in-repo `key-helper-service`.
After installing,
log out and back in,
or reload the KWin script and restart the service.

## Develop

```sh
# Run the daemon from source (no build step).
mise run //package/kwin/key-helper:run

# Run with file-watch auto-restart.
mise run //package/kwin/key-helper:run:dev

# Unit tests for the pure logic.
mise run //package/kwin/key-helper:test:unit
```

## D-Bus interface

Service `org.monochromatic.KeyHelper`,
path `/org/monochromatic/KeyHelper`,
interface `org.monochromatic.KeyHelper`:

- `SetActiveWindow(s windowClass)`:
   record the focused window class.
- `SendF20()`:
   send `<F20>` to the newest Neovim.
- `SendNvimKeys(s keys)`:
   send an arbitrary Neovim key sequence over RPC.
- `SendKeys(s keys)`:
   inject a `+`-joined combo via ydotool.
- `LaunchNewInstance(s desktopFileName, s resourceClass)`:
   launch a new instance of the focused app.

[ydotool]: https://github.com/ReimuNotMoe/ydotool
