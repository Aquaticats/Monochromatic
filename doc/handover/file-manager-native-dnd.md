# Handover: file-manager native drag-and-drop

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/nested-wayland-session` is now `package/nested-wayland-session/cli`;
> `package/desktop-app/file-manager` is now `package/file-manager/app`.

Cross-session state for building hand-written per-OS native drag-and-drop into the
file-manager prototype (`package/desktop-app/file-manager/`),
 so files can be
dragged between the app and the OS file manager.
Started 2026-07-05.
Update this doc after each milestone.

## Goal and hard constraint

Slint's in-process `DragArea`/`DropArea` cannot cross the application boundary,
 so
real file drag-and-drop needs native adapters per OS.
Ordering chosen with the user:
 inbound is cross-platform and comes first;
 outbound
is per-OS,
 starting with **Linux**,
 then macOS (`ssh m1`),
 then Windows
(`ssh x13-win`).

Hard constraint from the user:
the app must behave correctly on a **pure Wayland** session (no XWayland/XDND
shortcut).

## Pivotal finding that shapes everything

winit 0.30 (which Slint's winit backend wraps) has **zero** Wayland drag-and-drop:
no `DroppedFile`/`HoveredFile` events,
 no data-source API (checked winit 0.30.13
`src/platform_impl/linux/wayland/` directly;
 the only "drag" there is window
move/resize).
Those events exist only in winit's X11 and macOS backends.

Consequences:

- The easy cross-platform inbound path (`WinitWindowAccessor::on_winit_window_event`
  observing winit's `DroppedFile`) works on X11,
   macOS,
   and Windows,
   but NOT on
  pure Wayland.
- On pure Wayland,
   BOTH directions must be driven by hand over the `wl_data_device`
  protocol,
   on winit's own connection (a drag's `start_drag` needs the pointer
  press SERIAL the compositor delivered to this client,
   which only the app's own
  connection sees).

## Architecture (Linux / Wayland)

Share winit's Wayland connection and run a second event queue on a dedicated
thread (the exact pattern `smithay-clipboard` uses to add clipboard to winit
apps):

1. Get winit's `wl_display`/`wl_surface` from the Slint window via the winit raw
   handle (`dnd_native::wayland_handles`).
2. `Backend::from_foreign_display(display)` + `Connection::from_backend` wrap that
   existing connection without opening a new one.
3. `registry_queue_init` makes this thread's own event queue on the shared
   connection.
4. Co-bind our own `wl_seat` + `wl_pointer` (and later `wl_keyboard`) to observe
   the same button presses winit sees,
    tracking the latest press **serial**.
5. Drive `wl_data_device` via smithay-client-toolkit's `data_device_manager`:
   outbound `create_drag_and_drop_source(["text/uri-list"], Copy|Move)` +
   `start_drag(&data_device, &origin_surface, None, serial)`;
    inbound
   `DragOffer` accept/`set_actions`/`receive` then read the `text/uri-list` pipe.

The window only has a live winit window once the event loop is active,
 so the
adapter starts from a single-shot timer after `run()`,
 not at `AppWindow::new()`.

### The one flagged risk (spike early)

`start_drag` + a co-bound pointer + a shared/foreign winit connection is
reasoned-correct (per-seat implicit grab,
 global serials) but not shown in any
shipping crate;
 smithay-clipboard only proves the co-bound-pointer serial for
clipboard `set_selection`.
So milestone 1 validates just this:
 does a co-bound `wl_pointer` on winit's shared
connection receive button presses with serials?
If not,
 the whole shared-connection approach is dead and the fix moves into a winit
fork.

## Version pins (must stay unified with winit)

winit 0.30 already pulls these on Linux,
 so matching versions makes Cargo unify to
a single compiled `wayland-client`/`wayland-backend` and the foreign display is
genuinely shared:

- `smithay-client-toolkit` 0.19.2 (features `calloop`;
   re-exports `calloop` and
  `calloop_wayland_source`)
- `wayland-client` 0.31
- `wayland-backend` 0.3 (feature `client_system`,
   gives `from_foreign_display`)

Hard rule:
 never pull anything on `wayland-client` 0.30 / `wayland-backend` 0.2
(an old sctk ≤0.18),
 or the foreign display splits into two incompatible backends.
Reference:
 sctk 0.19.2 `examples/data_device.rs` is the complete both-directions
template.

## Verification infrastructure (this machine)

- Session:
   KDE Plasma,
   `kwin_wayland` compositor,
   native Wayland (`WAYLAND_DISPLAY`
  set;
   the app logs `backend=wayland`).
- Drop target:
   `dolphin` (the only installed file manager).
- Synthetic input for scripted drags:
   `ydotool` (present;
   needs its daemon +
  uinput).
   `wtype`/`wlrctl` absent.
- The user is present and can perform a real drag by hand when scripted input is
  impractical.

## RESOLVED (final): inbound DnD works on real KWin

Dragging `hello.txt` from Dolphin onto the file manager now works on the real KWin
6.7.1 Wayland desktop,
 verified end to end (`drag entered` -> `accepted` ->
`drop_performed` -> `inbound drop received count=1`,
 with KWin entering our own
`wl_data_device`).
 Commit `20a33cde9`.

Root cause (fully diagnosed with protocol traces,
 see
`doc/troubleshooting/kwin-drag-only-first-data-device.md`):
 the app bound TWO
`wl_data_device`s on one connection,
 Slint's clipboard device (via
`copypasta`/`smithay-clipboard`) first and our DnD adapter's second,
 and KWin
delivers a drag to only the FIRST data device a client binds
(`dropHandlerForSurface` returns `.first()`,
 KWin `seat.cpp`,
 their own TODO).
 So
KWin sent every drag to the clipboard device,
 which ignores drags,
 and our device
never heard about it.
 Every earlier hypothesis (seat-proxy routing,
 a
`calloop-wayland-source` read race) was refuted by `WAYLAND_DEBUG`.
 The Smithay
nested compositor could not reproduce it because Smithay delivers to ALL of a
client's data devices,
 which is why the automated `drop-file` test passed while the
real Dolphin drag failed.

Fix:
 our Slint fork adds `BackendBuilder::with_clipboard(bool)`;
 the file manager
calls `.with_clipboard(false)` so the winit backend binds NO clipboard data device,
leaving ours as the only one (hence KWin's first).
 Now only one `get_data_device`
appears at startup,
 and the drag lands on it.

- Fork branch:
   `Aquaticat/slint` `feat/winit-backend-clipboard-toggle` (based on the
  `v1.17.0` tag).
   Patch artifact:
  `doc/troubleshooting/slint-winit-clipboard-toggle.patch`.
- Consumed via `[patch.crates-io]` in the file manager's `Cargo.toml` (the whole
  Slint family resolves from that one monorepo,
   builds and lints clean).
- Maintenance note:
   drop the `[patch.crates-io]` once `with_clipboard` (or an
  equivalent) ships in an upstream Slint release.
   The change is prepared for upstream
  (branch pushed);
   opening the PR to `slint-ui/slint` is pending the user's go-ahead.
- The `with_clipboard(false)` opt-out means Slint's built-in clipboard is off.
   The
  spike has no text-input widgets,
   so nothing regresses today;
   when the app needs
  text editing,
   route clipboard through the app's own data device (or restore Slint's
  once upstream can deliver drags to a second device).

Toolkit note (`doc/troubleshooting/winit-toolkits-no-wayland-drag-and-drop.md`):
 the
hand-rolled `wl_data_device` adapter is inherent to EVERY winit-based toolkit (Slint,
Bevy,
 Iced,
 egui all lack Wayland DnD,
 winit#1881),
 not Slint-specific.
 Only native
toolkits (GTK4 via `gtk4-rs`,
 Qt) avoid it.
 Bevy would not have helped.

## Current state (2026-07-06)

Committed:

- Baseline plus the in-process drag-and-drop spike (`drag_drop.rs`) and its
  troubleshooting doc.
- `63f72cc81` native DnD foundation:
   `dnd_native.rs` extracts the Wayland handles;
  verified on a real Wayland session (`backend=wayland`).
- `0540a43a3` milestone 1 (serial probe):
   shares winit's connection on a dedicated
  thread,
   co-binds a `wl_pointer`.
   VALIDATED:
   the co-bound pointer receives real
  left-button presses with serials (the one flagged risk is resolved,
   so the
  shared-connection approach is sound).
- `51b79e537` inbound file drop:
   the `wl_data_device` adapter accepts a dragged
  `text/uri-list`,
   reads it off the receive pipe on drop,
   parses `file://` URIs to
  paths,
   and reports them to a HUD line (`hud-f`) via a callback.
   Row padding and a
  `MONOCHROMATIC_FM_NO_NATIVE_DND` escape hatch added.
   Parsing split into
  `dnd_wayland_parse`.

- `dd6f05319` fix:
   create the data device in `SeatHandler::new_capability`,
   NOT
  `new_seat` (`new_seat` does not fire for a seat already present at startup,
   so the
  device was never bound).
   Startup now logs `data device bound on shared seat`.
- `0cbd72f2e` diagnostics:
   log the drag `enter` (with offered mime types),
  accept/not-accept,
   `leave`,
   and `drop_performed`.
- `28e9bb00a` (in `nested-wayland-session`) the automated inbound test:
   a `drop-file`
  control command that makes the compositor originate a drag onto the hosted app.
   This
  drove the full inbound sequence to success (see RESOLVED),
   verifying the co-bound
  data-device path.

### RESOLVED: the co-bound wl_data_device inbound path works (automated test)

The automated nested-compositor test (below) settles the open question.
 Driving a
scripted,
 compositor-originated drag onto the hosted file-manager produces the full
inbound sequence,
 end to end,
 on a pure-Wayland co-bound `wl_data_device`:

```txt
[compositor] drop-file: drag hovering over the app; releasing in 200ms
[app]        native DnD: drag entered mimes=["text/uri-list"]
[app]        native DnD: accepted text/uri-list drag
[compositor] drop-file: server drag dropped onto the app (accepted)
[compositor] drop-file: released the drag; drop delivered to the app
[app]        native DnD: drop_performed fired
[compositor] drop-file: app requested drag data for mime text/uri-list
[app]        native DnD: inbound drop received count=1
[compositor] drop-file: app finished the drag; clearing the pending uri-list
```

The HUD line then reads `last inbound drop (from OS): /tmp/hello.txt`.
 So the leading
hypothesis (that a co-bound `wl_data_device` might not receive server-delivered drag
`enter`/`drop`) is DISPROVEN:
 it receives them,
 and the app's accept -> drop ->
receive -> parse -> HUD path is correct and complete.

That means the earlier "dragging `hello.txt` from dolphin does nothing" symptom is NOT
an architecture flaw in the shared-connection co-bound data device.
 It is specific to
the real kwin desktop + dolphin (routing / focus / the manual drag itself),
 not the
app's code.
 The app's inbound implementation is verified working;
 a real-desktop
dolphin repro can be revisited separately,
 but it no longer blocks the inbound path.

Note:
 `MONOCHROMATIC_FM_NO_NATIVE_DND=1` disables the whole native adapter,
 and the
app behaves the same,
 so the adapter is not breaking anything.

## Automated inbound DnD test in the nested compositor (BUILT, PASSING)

Manual drags on the real kwin desktop gave no clean signal,
 so DnD testing is now
automatic in `package/cli/nested-wayland-session` (a Smithay nested compositor that
hosts one app,
 injects input via its own seat over a Unix-socket control API,
 and
screenshots).
 Committed as `28e9bb00a`.

Key enabler:
 **Smithay 0.7.0 has server-originated drag-and-drop**,
 so the compositor
IS the drag peer and no second app (dolphin) is needed:

- `smithay::wayland::selection::data_device::start_dnd(dh, seat, data, serial,
  pointer_start_data, touch_start_data, metadata: SourceMetadata)` installs a server
  pointer grab.
   `SourceMetadata { mime_types: ["text/uri-list"], dnd_action: Copy }`.
- `ServerDndGrabHandler::send(mime_type, fd, seat)` (on `Compositor` in `handlers.rs`)
  writes the `file://PATH` uri-list to the app's receive fd.
   `dropped`/`finished`/
  `cancelled` trace the outcome.
- The grab's `update_focus` sends the app's `wl_data_device` a `data_offer` + `enter`
  as the pointer moves over its surface (`server_dnd_grab.rs`,
   filtered to the
  surface's own client),
   then `drop` on button release.

What was built:

- `drop-file PATH [X Y]` control command (`src/dnd.rs`,
   `src/protocol.rs`,
  `src/control.rs`,
   `src/handlers.rs`,
   `src/state.rs`):
   move the pointer over the app,
  press left,
   `start_dnd`,
   nudge to emit the offer/enter,
   then release on a 200ms
  dwell timer.
   The dwell matters:
   Wayland DnD needs the target to `accept` a mime type
  and `set_actions` (a Copy action) BETWEEN enter and drop,
   or the drop is cancelled
  as unvalidated;
   releasing synchronously in the same call fails.
   The payload lives in
  `Compositor::pending_dnd_uri_list` so `send` (which only has `&mut Compositor`) can
  reach it.
- Build in the Fedora container (`mise run //package/cli/nested-wayland-session:...`;
  host lacks `wayland-server` headers).
   `cargo check`,
   rust linter,
   15 tests
  (incl.
   the `drop-file` parse test),
   and clippy (`-D warnings`) all pass.

How to run the test (both binaries built already;
 compositor runs on the HOST as a
nested winit client,
 per its README):

```sh
printf 'hello\n' > /tmp/hello.txt
RUST_LOG=info \
  package/cli/nested-wayland-session/target/debug/monochromatic-nested-wayland-session \
  --socket /tmp/nws.sock --size 1280x720 -- \
  package/desktop-app/file-manager/target/debug/monochromatic-file-manager \
  > /tmp/nws.log 2>&1 &
# wait for: native DnD: data device bound on shared seat
printf 'drop-file /tmp/hello.txt\n' | socat - UNIX-CONNECT:/tmp/nws.sock   # => ok
printf 'screenshot /tmp/after.png\n' | socat - UNIX-CONNECT:/tmp/nws.sock
grep -E 'drop-file:|native DnD:' /tmp/nws.log   # the full success sequence
printf 'quit\n' | socat - UNIX-CONNECT:/tmp/nws.sock
```

Result:
 the full sequence in the RESOLVED section above,
 and the HUD showing the path.
`socat` (not `nc`) is the socket tool present on this machine.

Not yet built:
 `drag X1 Y1 X2 Y2` (outbound observation via `ClientDndGrabHandler`).
That pairs with the app OUTBOUND work below;
 add it when building outbound.

Smithay clone for reference (throwaway,
 re-clone if gone):
`gh repo clone Smithay/smithay <dir> -- --depth 1 --branch v0.7.0`.

## Remaining milestones

1. DONE:
    automated inbound test in the nested compositor (`drop-file`,
    `28e9bb00a`).
   The inbound co-bound `wl_data_device` path is verified working end to end;
    the real
   dolphin symptom is environment-specific,
    not an app-code defect (see RESOLVED).
2. App OUTBOUND drag:
    `create_drag_and_drop_source` + `start_drag(latest_serial)` +
   fill the `DataSourceHandler` send/finish/cancelled stubs;
    verify with a new
   compositor `drag X1 Y1 X2 Y2` command (observe via `ClientDndGrabHandler`),
    or drop
   onto dolphin.
3. macOS (`ssh m1`):
    outbound `NSDraggingSource`/`NSPasteboard` file URLs via objc2;
   inbound via winit `on_winit_window_event` + `DroppedFile` (winit HAS DnD on
   macOS,
    unlike Wayland).
4. Windows (`ssh x13-win`):
    outbound `DoDragDrop`/`IDataObject` `CF_HDROP` via the
   `windows` crate;
    inbound via winit `DroppedFile`.

## Key files and commands

- `package/desktop-app/file-manager/src/dnd_native.rs`:
   raw-handle extraction +
  per-OS `start` dispatch.
- `package/desktop-app/file-manager/src/dnd_wayland.rs`:
   the Wayland
  `wl_data_device` adapter (Linux only).
- `package/desktop-app/file-manager/src/app.rs`:
   single-shot timer calls
  `dnd_native::start(app.window())` once the window is realized.
- Build/lint/test:
   `mise run //package/desktop-app/file-manager:{lint:clippy,lint:rust,test}`.
- Run the GUI (needs the Wayland session):
   `mise run //package/desktop-app/file-manager:run`.
  The binary is `monochromatic-file-manager`;
   stop it with
  `pkill -f monochromatic-file-manager`.

## Related docs

- `doc/troubleshooting/slint-drag-and-drop-file-lists.md`:
   why stock Slint has no
  OS file drag-and-drop (the source trace + upstream `#1967` decision).
- `doc/planning/file-manager.md`:
   the plan and the drag-and-drop spike result.
- `doc/handover/slint-file-manager-assessment.md`:
   the earlier stack assessment.
