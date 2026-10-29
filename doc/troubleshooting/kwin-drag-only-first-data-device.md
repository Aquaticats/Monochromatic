# KWin 6.7.1 delivers a Wayland drag only to a client's first `wl_data_device`,
so a winit/Slint app's separately-bound drag-and-drop device never receives file drops

A file dragged from Dolphin onto the Slint file-manager prototype
(`package/file-manager/app/`) on a native KWin Wayland session does
nothing:
 no drop,
 and the drag cursor shows the "forbidden" (no-drop) icon while
hovering the app.
 The identical app receives the same drag correctly under the
project's Smithay-based nested compositor
(`package/nested-wayland-session/cli`,
 its `drop-file` command).
 The difference
is not the app:
 it is how KWin selects which of a client's `wl_data_device`
objects receives a drag.

## Symptom

- Dragging `hello.txt` from Dolphin onto a file-manager pane:
   nothing happens.
  The app's log shows only its three startup lines and never
  `native DnD: drag entered`.
- While the drag hovers over the app window,
   the pointer shows the **"forbidden"
  / no-drop cursor**.
   That is the drag source (Dolphin) reporting that no target
  accepted the offer,
   because the app never sent `wl_data_offer.accept`.
- The same app under the Smithay nested compositor logs the full,
   working
  sequence and updates its HUD (`native DnD: drag entered` ->
  `accepted text/uri-list drag` -> `drop_performed fired` ->
  `inbound drop received count=1`).

The app drives Wayland drag-and-drop itself because winit 0.30 has no Wayland
DnD (documented in
[slint-drag-and-drop-file-lists.md](slint-drag-and-drop-file-lists.md)).
 It
shares winit's connection on a second thread
(`Backend::from_foreign_display`) and co-binds its own `wl_data_device`,
 the same
technique `smithay-clipboard` uses.
 That adapter is correct;
 the failure is
upstream of it.

## Root cause

A single Wayland client can bind `wl_data_device` more than once.
 This app's
process ends up owning **two** on one connection:

- One bound by Slint's clipboard.
   Slint's winit backend uses `copypasta`,
   which
  on Wayland uses `smithay-clipboard`,
   which shares winit's connection and binds
  a `wl_data_device` (plus `zwp_primary_selection_device_v1`) during Slint
  initialization.
   This is bound **first**.
- One bound by the app's own DnD adapter (`dnd_wayland.rs`),
   started from a
  single-shot timer after the window is realized.
   This is bound **second**.

KWin selects the target of a drag by taking the **first** of the target client's
data devices,
 and never reconsiders.
 Verified against the KWin source at commit
`4303c6b42` (current master) and re-checked at tag `v6.7.1` (the running
version);
 the subagent investigation also confirmed the same shape at `v6.0.0`,
`v6.2.0`,
 and `v6.3.0`.

The drop handler is chosen by `dropHandlerForSurface`,
 which returns
`list.first()` (`src/wayland/seat.cpp:181`):

```cpp
// src/wayland/seat.cpp
AbstractDropHandler *SeatInterface::dropHandlerForSurface(SurfaceInterface *surface) const
{
    auto list = d->dataDevicesForSurface(surface);
    if (list.isEmpty()) {
        return nullptr;
    };
    return list.first();
}
```

`dataDevicesForSurface` walks the seat's flat `dataDevices` list in
registration order,
 keeping those whose native `wl_client` matches the target
surface's client (`src/wayland/seat.cpp:131`):

```cpp
// src/wayland/seat.cpp
QList<DataDeviceInterface *> SeatInterfacePrivate::dataDevicesForSurface(SurfaceInterface *surface) const
{
    if (!surface) {
        return {};
    }
    QList<DataDeviceInterface *> primarySelectionDevices;
    for (auto it = dataDevices.constBegin(); it != dataDevices.constEnd(); ++it) {
        if ((*it)->client() == *surface->client()) {
            primarySelectionDevices << *it;
        }
    }
    return primarySelectionDevices;
}
```

The drag-start path is the same shape,
 taking `[0]`
(`src/wayland/seat.cpp:1297`).
 KWin's own comment (`src/wayland/seat.cpp:479`)
documents this as a deliberate simplification,
 and names the exact case that
breaks us:

```cpp
// src/wayland/seat.cpp
// TODO: technically we can have multiple data devices
// and we should send the drag to all of them, but that seems overly complicated
// in practice so far the only case for multiple data devices is for clipboard overriding
```

So the drag enter goes to the client's first device (Slint's clipboard),
 which
has no drag handling and ignores it.
 The app's own device,
 bound second,
 never
receives `enter`,
 so the app never calls `accept`,
 and the source shows the
no-drop cursor.
 Client identity is not the problem:
 KWin filters by native
`wl_client`,
 and the shared-connection design keeps both devices on one client
(this was independently proven by the Smithay test,
 whose routing has the same
`same_client_as` filter and delivered the drag).

Smithay does not have this limitation:
 both its server DnD grab and its client
DnD grab iterate **every** matching data device
(`smithay-0.7.0 src/wayland/selection/data_device/server_dnd_grab.rs:154` and
`dnd_grab.rs:170`,
 `for device in ... .filter(|d| d.id().same_client_as(...))`),
so the app's second device receives the drag there.
 That is why the automated
`drop-file` test passed and the real Dolphin drag did not.

### Earlier wrong readings (do not re-derive)

- "KWin routes drags per `wl_seat` proxy,
   and the app binds its device on a
  different proxy than the one with pointer focus."
   Refuted:
   KWin keys on the
  native `wl_client`,
   not the `wl_seat` protocol object
  (`dataDevicesForSurface` above;
   `SeatInterface::get` resolves every proxy of a
  seat global to the one logical seat,
   `src/wayland/seat.cpp`).
- "KWin does deliver to the app's device but our second event queue never
  dispatches it (a `calloop-wayland-source` read race with winit)."
   Refuted by
  `WAYLAND_DEBUG`:
   the drag `enter` never lands on the app's device at all.
   It
  lands on the clipboard's device (a different object id).
   The app's pointer,
  co-bound on the same second queue,
   dispatches fine,
   so the queue works;
   the
  data-device events simply never arrive because KWin sent them elsewhere.

## Verification

Under test:

- KWin / Plasma `6.7.1`,
   native Wayland session (not XWayland).
- KWin source:
   clone of `KDE/kwin`,
   HEAD `4303c6b42` (master),
   key logic
  re-confirmed at tag `v6.7.1`.
- App:
   `package/file-manager/app` debug build;
   Slint 1.17.0 via
  `copypasta` -> `smithay-clipboard` 0.7.3;
   `smithay-client-toolkit` 0.19.2.
- Drag source:
   Dolphin (KDE `kioworker` file manager).

Harness (drive it yourself;
 a real human drag is more reliable than synthetic
input on a multi-monitor session):

```sh
# 1. A throwaway file to drag.
mkdir -p /tmp/dnd-src && printf 'hello\n' > /tmp/dnd-src/hello.txt

# 2. Run the app with Wayland protocol tracing + info logs.
FM=package/file-manager/app/target/debug/monochromatic-file-manager
WAYLAND_DEBUG=1 RUST_LOG=info "$FM" > /tmp/fm.log 2>&1 &

# 3. Open Dolphin on the folder, then drag hello.txt onto a file-manager pane.
dolphin /tmp/dnd-src &

# 4. Inspect: which wl_data_device did KWin enter, vs which is the app's own?
grep -aE "wl_data_device#[0-9]+\.enter|get_data_device\(new id" /tmp/fm.log
grep -aE "native DnD:" /tmp/fm.log
```

Fails on KWin (this is the whole bug):

- `-> wl_data_device_manager#A.get_data_device(new id wl_data_device#40, ...)`
  then `... #B.get_data_device(new id wl_data_device#61, ...)`:
   two devices,
   the
  app's own (logged by the app as its bound device) is the second (`#61`).
- `wl_data_device#40.enter(..., wl_data_offer#N)` with
  `wl_data_offer#N.offer("text/uri-list")`:
   KWin enters the **first** device,
  `#40` (the clipboard).
   The app's `#61` receives no `enter`/`motion`/`drop`.
- The app log shows no `native DnD: drag entered`.

Works on the Smithay nested compositor (same app binary):

```sh
printf 'ping\n'                       | socat - UNIX-CONNECT:/tmp/nws.sock  # ok
printf 'drop-file /tmp/hello.txt\n'   | socat - UNIX-CONNECT:/tmp/nws.sock  # ok
# app log: native DnD: drag entered -> accepted -> drop_performed -> inbound drop received count=1
```

## Verified workarounds

- **Disable the toolkit's clipboard data device so the app's DnD device is the
  only one (chosen,
   verified on KWin 6.7.1).**
   Stop Slint's `copypasta` from
  binding its clipboard `wl_data_device`,
   so the app's own DnD device is the only
  one and thus KWin's "first".
   Implemented by a Slint fork that adds
  `BackendBuilder::with_clipboard(bool)`;
   the file manager calls
  `.with_clipboard(false)` (fork branch `Aquaticat/slint`
  `feat/winit-backend-clipboard-toggle` off the `v1.17.0` tag,
   consumed via
  `[patch.crates-io]`;
   patch:
   `slint-winit-clipboard-toggle.patch`).
   Verified end
  to end:
   a real Dolphin drag now reaches the app (`inbound drop received
  count=1`),
   and only one `get_data_device` appears at startup.
   Tradeoff:
   Slint's
  built-in clipboard is off while disabled;
   the spike has no text input,
   so nothing
  regresses.
   A fuller variant routes clipboard through the app's single data device
  too,
   avoiding any clipboard loss;
   not needed yet.
   Robust across compositors and
  removes the two-devices condition entirely.

- **Bind the app's data device first (rejected as fragile).**
   If the app's device
  registered before Slint's clipboard,
   KWin's `first()` would pick it.
   Rejected:
  Slint binds `copypasta`'s device during its own initialization,
   before the app
  gets the winit window handle it needs to share the connection,
   so the app
  cannot reliably win the ordering without a Slint-internal hook.
   Even if won
  once,
   the order is not contractual and could change with Slint or winit
  versions.

- **XWayland / XDND (rejected by project constraint).**
   The app must behave
  correctly on a pure Wayland session,
   so routing DnD through XWayland is out of
  scope.

## What does not work

- Changing the app's second event queue dispatch (swapping
  `calloop-wayland-source`'s `WaylandSource` for `dispatch_pending` polling,
   or
  for `roundtrip`).
   None help:
   the events never reach the app's device,
   so no
  amount of dispatching on that device surfaces them.
   These were dead ends from
  the "our queue misses the events" wrong reading above.
- Making the app's data device on the same `wl_seat` proxy winit uses.
   Irrelevant:
  KWin keys on `wl_client`,
   not the proxy.
- Reproducing the failure in the Smithay nested compositor.
   It cannot reproduce
  it:
   Smithay delivers a drag to all of a client's data devices,
   so the second
  device always receives it there.
   Reproduction requires a real KWin session (or
  another compositor that also delivers to only one device).

## Upstream filing artifact

`.out-of-scope/` was checked:
 no exemption covers KWin,
 Plasma,
 Wayland,
 or
data-device behavior.
 The six-constraint check follows;
 all six hold,
 so the
draft below is fileable.

### Six-constraint check

1. **Is it really upstream's fault?**
    Yes.
    `dropHandlerForSurface` returning
   `first()` (and the drag-start `[0]`) is the direct cause.
    The two-data-device
   condition is Wayland-legal and arises in any app that adds its own data device
   alongside a toolkit clipboard;
    KWin is the component that drops the drag.
2. **Can upstream fix it?**
    Yes,
    and their own TODO names the fix ("we should send
   the drag to all of them").
3. **Are they supporting this use case?**
    A client binding `wl_data_device` more
   than once is allowed by the protocol,
    and KWin already tracks a list of them
   (`dataDevices`);
    it just picks one for drags.
4. **Would the repo welcome our contribution?**
    KWin takes merge requests on
   `invent.kde.org` (`CONTRIBUTING.md`);
    a test is "appreciated,
    but not expected"
   for a bug fix.
    No ban on AI-assisted contributions appears in `CONTRIBUTING.md`.
   The draft discloses AI assistance and the human-verification status.
5. **Will they likely fix it?**
    Their in-code TODO agrees the fix is correct ("we
   should send the drag to all of them") and only deferred it as unprioritized
   ("seems overly complicated in practice so far the only case ... is for clipboard
   overriding").
    That is a deferral,
    not a stated non-goal or won't-fix,
    and this
   report supplies the missing second case.
    Not a fail.
6. **Have we prototyped a minimal fix?**
    Yes:
   [kwin-drag-only-first-data-device.patch](kwin-drag-only-first-data-device.patch)
   beside this doc.

### Prototype and its verification

The patch changes `SeatInterfacePrivate::Drag::target`
(`QPointer<AbstractDropHandler>`) to `targets`
(`QList<QPointer<AbstractDropHandler>>`) and updates the five sites that used it
(`setDragTarget`,
 `notifyDragMotion`,
 `endDrag`,
 `cancelDrag`,
 and the
`startDrag` origin path).
 Correctness argument:

- Single-device case (every app today):
   `dataDevicesForSurface` returns one
  device,
   `targets` holds one element,
   every loop runs once,
   so behavior is
  identical to the old `first()` path.
- Multi-device case (this bug):
   every data device of the surface's client
  receives `updateDragTarget` (enter/leave),
   `motion`,
   and `drop`.
   A device that
  ignores drags (the clipboard) is unaffected;
   the device that wants them (the
  app's DnD device) now receives them.
   Sending `drop` to a device whose client
  never accepted is harmless:
   that client ignores it.
- Xwayland / no-`wl_data_device` case:
   `dataDevicesForSurface` is empty,
   so the
  code falls back to the single passed `dropTarget` (the Xwayland drop handler),
  preserving the existing path.
- `std::as_const` is already used seven times in `seat.cpp`,
   so no new include is
  needed.
   The patch applies cleanly to `KDE/kwin` HEAD `4303c6b42`.

Verification not run:
 exercising it needs a full KWin + KF6 6.27.0 build plus its
integration DnD test (`autotests/integration/dnd_test.cpp` spins up a whole
compositor).
 That build is disproportionate for this session (bleeding-edge KF6,
large disk footprint,
 and this machine has hit disk limits).
 The failure itself
is already reproduced above on the running KWin 6.7.1;
 a maintainer,
 or a future
session with a KWin build,
 should confirm the post-patch success and,
 per
`CONTRIBUTING.md`,
 may add a two-data-device case to the DnD autotest.

### Duplicate search

KDE uses `bugs.kde.org` and `invent.kde.org`,
 not GitHub.
 Searched for KWin
drag-and-drop delivered to only the first / multiple data devices / clipboard
override.
 No existing report matches this specific behavior;
 nearest hits are
unrelated (XWayland interop,
 a Firefox missing-`leave` fix,
 a GTK
drag-motion-on-activate issue).
 Maintainer awareness is recorded only in their own
TODO;
 no tracked bug or merge request addresses it.

### Draft merge request (fileable)

~~~md
Title: Deliver a drag-and-drop to all of a client's data devices, not just the first

## Problem

A Wayland client may bind `wl_data_device` more than once on one connection, for
example one device for clipboard and a separate one for drag-and-drop. KWin
delivers a drag to only the first of a client's data devices, so the drag is
silently dropped for the others.

`SeatInterface::dropHandlerForSurface` returns
`dataDevicesForSurface(surface).first()` (`src/wayland/seat.cpp`), and the
drag-start path uses `[0]`. The in-code TODO already notes this: "we should send
the drag to all of them, but that seems overly complicated in practice so far the
only case for multiple data devices is for clipboard overriding." This is a real
second case: an application that adds its own `wl_data_device` for drag-and-drop
alongside a toolkit's clipboard `wl_data_device`.

## Reproduction

- A GUI app whose toolkit binds a clipboard `wl_data_device` (for example via
  smithay-clipboard / copypasta) and that also binds a second `wl_data_device`
  for drag-and-drop. On KWin (6.7.1 tested), dragging a file from Dolphin onto the
  app shows the no-drop cursor and never delivers the drop; `WAYLAND_DEBUG` shows
  `wl_data_device.enter` sent only to the client's first (clipboard) device,
  never the second.
- Compositors that deliver to all of a client's data devices (for example
  Smithay's `known_data_devices` iteration) deliver the drop correctly to the
  same app.

## Fix

Make the drag target a list of all the surface client's data devices and iterate
on enter, motion, drop, and leave, falling back to the single passed handler
(Xwayland) when the surface has no `wl_data_device`. The single-device case is
unchanged (one-element list). Patch attached.

## Note

This change and its analysis were prepared with AI assistance. The reproduction on
KWin 6.7.1, the source trace, and the patch's clean application to HEAD
`4303c6b42` were verified; a full KWin + KF6 build and the integration DnD test
were not run here and should be confirmed before merge. A two-data-device case
could be added to `autotests/integration/dnd_test.cpp`.
~~~

Decision:
 **fileable** as a merge request on `invent.kde.org/plasma/kwin` with the
patch attached.
 The consumer-side fix (a single data device in the app) is
implemented regardless,
 because it resolves the user's problem without waiting on
upstream.

## Related docs

- [slint-drag-and-drop-file-lists.md](slint-drag-and-drop-file-lists.md):
   why
  stock Slint (and winit 0.30) has no OS file drag-and-drop on Wayland.
- `doc/handover/file-manager-native-dnd.md`:
   the native-DnD build state,
   the
  Smithay `drop-file` automated test,
   and the single-data-device fix plan.
