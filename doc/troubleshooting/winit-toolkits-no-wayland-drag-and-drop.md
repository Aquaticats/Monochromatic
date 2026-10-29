# winit-based Rust GUI toolkits (Slint, Bevy, Iced, egui) have no native Wayland drag-and-drop; only native toolkits (GTK4, Qt) do

Choosing a Rust GUI toolkit for an app that needs OS drag-and-drop on a pure
Wayland session (the file-manager prototype,
 `package/file-manager/app/`)
runs into a gap that is easy to misattribute to one toolkit.
 It is not
Slint-specific:
 it is inherited from winit,
 which every one of the popular Rust
toolkits (Slint,
 Bevy,
 Iced,
 egui) uses for windowing.
 This doc records the gap,
which toolkits share it,
 and what actually avoids it,
 so the toolkit choice is
made once with the evidence in view.

## Status update (2026-09-23)

The winit gap this doc describes is closing upstream,
 and the framing shifts from
"winit has no Wayland DnD" to "toolkits pinned to winit 0.30.x have no Wayland DnD".

- Winit pull request
  [#4571](https://github.com/rust-windowing/winit/pull/4571) merged 2026-07-16 and
  shipped in the pre-release `v0.31.0-beta.3` (published 2026-09-04).
   It implements
  both receiving and initiating drags on Wayland,
   macOS,
   and Windows.
- X11 is still incoming-only:
   `winit-x11` overrides `data_transfer`,
  `fetch_data_transfer`,
   and `set_valid_dnd_actions`,
   but not `start_drag`,
   and its
  incoming path reports `Copy` only.
   Upstream called X11 outgoing out of scope for
  that pull request.
- The latest stable release is still `v0.30.13` (2026-03-02),
   so every toolkit in the
  survey inherits the gap until it migrates to a `0.31` release.
- [winit#1881](https://github.com/rust-windowing/winit/issues/1881) closed as completed
  2026-08-03;
   [#720](https://github.com/rust-windowing/winit/issues/720) and
  [#1499](https://github.com/rust-windowing/winit/issues/1499) remain open.

The hand-rolled `wl_data_device` workaround recorded here stays correct for winit
0.30.x builds,
 and the KWin first-device rule still applies to any client that binds
two devices.
 Once a toolkit migrates to winit 0.31,
 winit owns the data device,
 and a
second hand-rolled binding becomes the thing that breaks drops.

One path in this doc is stale.
 The hand-rolled adapter it names,
`package/file-manager/app/src/dnd_wayland.rs`,
 belonged to the retired Slint
implementation;
 it was added as `packages/desktop-app/file-manager/src/dnd_wayland.rs`
in commit `0540a43a3`,
 before the `packages/` to `package/` directory rename.
 The file
manager is now GTK4,
 and its drag-and-drop wiring lives in
`package/file-manager/app/src/dnd.rs`.

For the Xilem/Masonry side,
 including the event-routing trace,
 the removal history of
Masonry's file-drop events,
 and what adoption would require,
 see
[xilem-dolphin-drag-and-drop.md](xilem-dolphin-drag-and-drop.md).

## Symptom

- Dragging a file from a file manager onto the app on a native Wayland session
  does nothing:
   no drop event ever reaches the app.
- The same app on X11 (or XWayland) receives file drops fine,
   because the drop
  path there goes through a different code path.
- This looks like a bug in whichever toolkit is in use,
   but swapping to another
  winit-based toolkit does not fix it.

## Root cause

winit is the windowing layer under Slint,
 Bevy,
 Iced,
 and egui,
 and winit 0.30
has no Wayland drag-and-drop at all.
 Reading winit 0.30.13's Wayland backend
(`src/platform_impl/linux/wayland/`) directly:
 there are no `DroppedFile` or
`HoveredFile` window events and no data-source API;
 the only "drag" in that
backend is interactive window move/resize.
 winit's X11 and macOS backends do
emit `DroppedFile`/`HoveredFile`,
 which is why the same app works on X11 and
macOS but not Wayland.
 The upstream tracking issue is
[winit#1881](https://github.com/rust-windowing/winit/issues/1881) (open when this was
written;
 closed as completed 2026-08-03,
 per the "Status update (2026-09-23)" section),
 with
[winit#1499](https://github.com/rust-windowing/winit/issues/1499) as related
history.

Because the toolkits delegate windowing to winit,
 they inherit the gap:

- **Slint** (via `i-slint-backend-winit`):
   no OS file drag-and-drop on Wayland.
  Slint's in-process `DragArea`/`DropArea` cannot cross the application boundary
  (see [slint-drag-and-drop-file-lists.md](slint-drag-and-drop-file-lists.md)).
- **Bevy** (via `bevy_winit`):
   its own docs state "drag and drop window events
  are currently not supported on Wayland"
  ([Bevy Cheat Book](https://bevy-cheatbook.github.io/input/dnd.html)).
   Bevy's
  `FileDragAndDrop` event is winit's `DroppedFile`,
   so it is present on
  X11/Windows/macOS and absent on Wayland.
- **Iced** and **egui** (both winit-based,
   via `winit`/`eframe`):
   same gap;
  their file-drop events are winit's,
   which Wayland does not deliver.

The workaround every winit-based app must use is to drive the Wayland
`wl_data_device` protocol by hand on the app's own connection (the file manager
does this in `package/file-manager/app/src/dnd_wayland.rs`,
 sharing
winit's connection via `Backend::from_foreign_display`).
 That hand-rolled
adapter is inherent to the winit family,
 not to Slint.

Toolkits that are NOT winit-based do not have the gap,
 because they are
compositor-integrated widget toolkits with first-class drag-and-drop:

- **GTK4** via [`gtk4-rs`](https://lib.rs/crates/gtk4):
   native Wayland
  drag-and-drop through `GtkDropTarget` / `GtkDragSource`,
   plus native clipboard,
  and `GtkColumnView`/`GtkListView` for virtualized column lists (the widgets
  file managers such as the Nautilus family use).
   No winit,
   no co-bound data
  device.
- **Qt** via `cxx-qt` / `qmetaobject`:
   native Wayland drag-and-drop and
  clipboard;
   heavier bindings.

## Verification

- winit source:
   winit 0.30.13,
   `src/platform_impl/linux/wayland/` has no
  `DroppedFile`/`HoveredFile`;
   the X11 and macOS backends do.
   (Read during the
  native-DnD investigation;
   see `doc/handover/file-manager-native-dnd.md`.)
- Upstream tracking:
   [winit#1881](https://github.com/rust-windowing/winit/issues/1881)
  ("Support drag and drop on wayland"),
   open at time of writing;
   closed as
  completed 2026-08-03.
- Bevy:
   [Bevy Cheat Book, Drag-and-Drop (Files)](https://bevy-cheatbook.github.io/input/dnd.html)
  states Wayland is unsupported;
   Bevy windowing is `bevy_winit`.
- Reproduced directly for Slint:
   the file manager receives no OS drop on Wayland
  until the hand-rolled `wl_data_device` adapter is added,
   and that adapter works
  under a compositor that delivers drags correctly (verified via the Smithay
  nested compositor's `drop-file` command in `package/nested-wayland-session/cli`).

## Full framework survey: native Wayland OS file drag-and-drop

The determinant is the windowing/rendering engine,
 not the framework's language
or branding.
 Every framework with working native-Wayland cross-application file
drag-and-drop (without hand-rolling `wl_data_device`) sits on one of exactly
three engines:
 GTK/GDK,
 Qt/QtWayland,
 or FLTK's own Wayland backend.
 Everything
else sits on winit (no Wayland DnD at all),
 on Chromium (native Wayland runs,
but file drops from KDE's Dolphin are broken),
 or is X11-only (reaches Wayland
only through XWayland,
 which the pure-Wayland constraint forbids).
 This was
surveyed across the Rust,
 webview,
 and JVM/.NET ecosystems;
 verdicts are from
engine source,
 official docs,
 and issue trackers,
 not a live per-framework drop
test.

### Have it: GTK/GDK engine

- GTK4 (C,
   PyGObject,
   Vala):
   `GtkDropTarget` receives a `GdkFileList`;
   GDK
  implements DnD on `wl_data_device` in `gdkdnd-wayland.c`.
   Proven by Nautilus,
  a native Wayland GTK4 file manager.
- gtk4-rs and relm4 (Rust):
   bind `GtkDropTarget`/`GtkDragSource` unchanged;
   a
  real Nautilus-to-app Wayland drop is documented in ghostty#11175.
- wxWidgets (wxGTK backend):
   `wxFileDropTarget` accepts file-manager drops;
   a
  copy vs move fidelity bug on GTK3 Wayland is wx#17864,
   but the drop works.
- Flutter desktop (GTK embedder):
   native Wayland via a GTK subsurface
  (flutter/engine#20629);
   file DnD via `super_drag_and_drop` or `desktop_drop`,
  both GTK drag-dest.
   Still GTK3 with rough edges;
   verify on the KWin session.
- WebKitGTK webviews (Tauri v2,
   Wails,
   Neutralino,
   Dioxus desktop):
   all render
  through WebKitGTK,
   which is GTK,
   so file DnD rides GDK's Wayland backend.
  Dioxus desktop specifically uses `tao`+`wry` (GTK on Linux),
   not winit,
   so it
  is not part of the winit gap despite being a Rust GUI framework.

### Have it: Qt / QtWayland engine

- Qt C++,
   PySide6,
   PyQt6:
   `QWaylandDrag` (a `QPlatformDrag` over the data-device
  protocol);
   receive via QML `DropArea`/`drop.urls` or `QMimeData::urls()`.
  Proven by Dolphin,
   itself a Qt app on KWin Wayland.
- cxx-qt and qmetaobject (Rust):
   thin bindings over the same Qt DnD.

### Have it: FLTK's own Wayland backend

- fltk-rs with the `use-wayland` feature (FLTK 1.4+):
   native Wayland
  `wl_data_device` in `fl_wayland_clipboard_dnd.cxx`,
   surfaced as
  `Event::DndEnter` then `Event::Paste` with `app::event_text()`.
   Default
  fltk-rs is X11-only;
   the feature flag is required.

### Native Wayland, but no inbound cross-app file drop

- makepad (Rust,
   its own Wayland backend):
   implements clipboard and internal
  within-app drag only;
   its `wl_data_device` handler ignores inbound
  `Enter`/`Drop`.
   A near-miss,
   not usable for receiving a file from Dolphin.

### winit family: no Wayland DnD at all (winit#1881)

- Slint,
   Bevy,
   Iced,
   egui/eframe,
   floem (the floem-winit fork has no Wayland
  data-device module),
   xilem/masonry (glazier is deprecated,
   the stack is
  winit).
   All inherit winit#1881.

### Chromium / Ozone: native Wayland, but Dolphin drops broken

- Electron (native Wayland default since 38.2,
   roughly September 2025) and NW.js
  (opt-in via `--ozone-platform=wayland`).
   Native Wayland runs,
   but OS file
  drops from KDE's Dolphin into Chromium are broken (KDE#484018,
   REOPENED;
  Chromium#399587146),
   and internal DnD can freeze the app on Plasma 6
  (electron#49907).
   Disqualified for the exact KWin-plus-Dolphin scenario
  despite nominal Wayland support.
   Notably,
   the mirror of this app's Slint fight
  (the exact "Drop never fires on KDE Plasma 6 Wayland" signature) was hit and
  fixed in the WebKitGTK path:
   Tauri#11282,
   fixed by wry#1408,
   shipped in wry
  0.47.0 / Tauri 2.1.0,
   so Tauri v2 needs to be at least 2.1.0.

### X11 / XWayland-only: disqualified by the pure-Wayland constraint

- Compose Multiplatform (Skiko binds X11 directly,
   skiko#28),
   stock OpenJDK
  Swing/AWT and JavaFX (X11 only;
   the Wakefield pure-Wayland toolkit is still a
  prototype branch),
   Avalonia (X11-only in releases,
   native Wayland in preview
  with DnD unresolved),
   and .NET MAUI (no official Linux at all).
   All reach
  Wayland only through XWayland.
- Nuance:
   JetBrains Runtime now ships a native Wayland AWT (WLToolkit,
   default
  in 2026.1) with drag-and-drop,
   but Skiko/Compose does not ride it yet and
  file-manager drops on it are unverified.

## Verified workarounds

- **Stay winit-based and hand-roll `wl_data_device` (what the file manager does).**
  Share winit's Wayland connection on a second thread and drive the data-device
  protocol directly.
   Tradeoff:
   it is real protocol work,
   and it interacts badly
  with a toolkit that also binds a data device for its clipboard:
   KWin delivers a
  drag to only the first data device a client binds,
   so the app's device must be
  the only one (see
  [kwin-drag-only-first-data-device.md](kwin-drag-only-first-data-device.md) and
  the Slint `with_clipboard(false)` fork fix).
   This is the path taken because the
  toolkit was already Slint.
- **Switch to a native toolkit (GTK4 via `gtk4-rs`,
   or Qt).**
   Deletes the entire
  hand-rolled adapter and this whole class of problem:
   native DnD,
   native
  clipboard,
   and virtualized column widgets.
   Tradeoff:
   a UI rewrite away from
  Slint's markup and the work already invested in it.

## What does not work

- **Switching between winit-based toolkits** (for example Slint to Bevy,
   Iced,
   or
  egui) to get Wayland DnD.
   They all sit on winit and all inherit winit#1881.
  Bevy specifically is a game engine with no widget/list toolkit,
   so a file
  manager would also be a from-scratch UI,
   for no DnD benefit.
- **Relying on the toolkit's built-in file-drop event on Wayland.**
   It is winit's
  `DroppedFile`,
   which Wayland never delivers.

## Upstream filing decision

The upstream gap is winit's,
 and it already has an open tracking issue,
[winit#1881](https://github.com/rust-windowing/winit/issues/1881).
 Per the
duplicate-search rule,
 an existing issue means no new issue:
 this doc links it
rather than filing a second one.
 Adding Wayland drag-and-drop to winit is a large
backend feature,
 tracked upstream for years;
 the pragmatic client response is
already recorded here (hand-roll `wl_data_device`,
 or choose a native toolkit),
so there is nothing additive to post on the winit thread that it does not already
have.
 No fileable draft is kept.

`.out-of-scope/` was checked:
 no exemption covers winit,
 Wayland,
 or GUI-toolkit
drag-and-drop.

## Related docs

- [slint-drag-and-drop-file-lists.md](slint-drag-and-drop-file-lists.md):
   Slint's
  in-process `DragArea`/`DropArea` cannot cross the app boundary.
- [kwin-drag-only-first-data-device.md](kwin-drag-only-first-data-device.md):
   why
  the hand-rolled adapter still failed on KWin,
   and the Slint `with_clipboard`
  fork fix.
- `doc/handover/file-manager-native-dnd.md`:
   the native-DnD build state and the
  Smithay automated test.
