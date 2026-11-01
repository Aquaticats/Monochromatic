# file-manager-qt

Qt/cxx-qt desktop file-manager.
 Successor to the Slint prototype,
 chosen because a
real Qt/QtQuick UI provides native OS drag-and-drop on every target platform,
including native Wayland,
 which the winit-based toolkits (Slint,
 Bevy,
 Iced,
 egui)
lack.
 Rationale and the full cross-platform survey live in
[winit-toolkits-no-wayland-drag-and-drop.md](../../../doc/troubleshooting/winit-toolkits-no-wayland-drag-and-drop.md).

## Status

Spike in progress.
 Verified working:
 a native Wayland QML window on KWin 6.7.1,
driven by a Rust `#[cxx_qt::bridge]` QObject (a `wl_compositor` / `xdg_toplevel`
surface,
 no XWayland).
 Next milestones:
 a Rust `QAbstractListModel` feeding a QML
`ListView`,
 then inbound (Dolphin drop) and outbound native file drag-and-drop.

## Toolchain

Builds against the system Qt6 (6.11.1).
 On this Bazzite/Fedora atomic host the Qt6
`-devel` packages were layered live,
 no reboot:

```sh
sudo rpm-ostree install --apply-live --idempotent \
  qt6-qtbase-devel qt6-qtdeclarative-devel qt6-qtshadertools-devel
```

cxx-qt-build locates Qt by running `qmake`,
 but Fedora ships only `qmake6` (there is
no bare `qmake` for Qt6),
 so the mise `[env]` sets `QMAKE=/usr/bin/qmake6`.
 The
already-layered clang and cmake supply the C++ toolchain cxx-qt needs.

## Tasks

- `build`:
   release build.
- `build:debug`:
   debug build.
- `run`:
   debug build,
   then run the GUI on native Wayland (`QT_QPA_PLATFORM=wayland`).
- The root `mise run lint` checks `rust/require-rustdoc` plus `rust/max-lines` here
  (this package is a cxx-qt file,
  so `rust/require-rustdoc` exempts its `use` imports and trait-impl methods).
- `lint:clippy`:
   clippy with warnings denied.
- `test`:
   `cargo nextest run`.

Run any task with `mise run //package/file-manager/variant-qt:<task>`.

## Layout

- `src/main.rs`:
   boots `QGuiApplication` + `QQmlApplicationEngine`,
   loads the QML.
- `src/cxxqt_object.rs`:
   the `#[cxx_qt::bridge]` root QObject exposed to QML.
- `qml/main.qml`:
   the QtQuick UI;
   its module URI and version match `build.rs`.
- `build.rs`:
   registers the QML module and compiles the bridge (pure Cargo,
   no CMake).
