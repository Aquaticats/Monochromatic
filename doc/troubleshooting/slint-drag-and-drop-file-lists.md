# Slint 1.17.0 drag-and-drop: in-process DragArea/DropArea works, but no OS-native file-list drag-and-drop on any backend

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Slint 1.17.0 exposes a working in-process drag-and-drop surface (`DragArea` and
`DropArea`,
 made public in this release),
 so a Slint app can drag items between
its own panes with a typed app-local payload and a copy/move/link action.
It has no OS-native file drag-and-drop:
a file dragged from the OS file manager into the window is not delivered to Slint,
and a drag started from the window advertises no file-list (`text/uri-list`) to
other applications.
This holds on both the winit and the Qt backend,
 so switching backends does not
unlock file drag-and-drop for a file manager.

This blocks two of the file-manager plan's drag-and-drop spike pass criteria
("external drops expose file paths or file URLs to Rust",
 "outbound drags
advertise OS-native file-list payloads") while the other two ("internal drags
carry structured app-local data",
 "copy and move are distinguishable") pass.

## Symptom

In the prototype at `package/file-manager/app/`,
 dragging a directory
row onto another pane works end to end:
the row's `DragArea` payload (its `(pane, row)` identity) reaches the target
pane's `DropArea`,
 and the negotiated move/copy action is recorded.

What does not work,
 for a file manager:

- Dragging a file from the OS file manager (Files,
   Finder,
   Explorer) onto the
  window delivers nothing to Rust:
  the built-in `DropArea` never fires for an external drag,
   and no
  `WindowEvent` carries the dropped path.
- Dragging a row out of the window onto the OS file manager advertises no file
  to the OS:
  the drag carries only the app-local payload (and,
   in later Slint,
   plain text
  or an image),
   never a `text/uri-list`,
   so a file manager receiving the drop
  has no file to copy or move.

## Root cause

### The transfer type has no file-list representation

Slint's drag payload is `DataTransfer`,
 which carries exactly three
representations:
plain text,
 an image,
 and application-internal `user_data`
(`~/.cargo/registry/src/index.crates.io-*/i-slint-core-1.17.0/data_transfer.rs:72`):

```rust
struct DataTransferInner {
    // ...
    user_data: Option<Rc<dyn Any>>,
}
```

`user_data` is an `Rc<dyn Any>`:
a process-local pointer whose `PartialEq` compares by `Rc::as_ptr`,
 so it is
meaningful only inside the same address space and cannot cross to another
application.
There is no `text/uri-list`,
 file-path,
 or file-list variant.
This matches the plan's earlier source fact that `data_transfer.rs` has a TODO
for custom binary data providers and MIME types.

### Released 1.17.0 has no native drag path at all

In the version the prototype pins,
 a drag never reaches the platform.
The core `StartDrag` handler arms an in-window drag directly and calls no backend
hook
(`i-slint-core-1.17.0/input.rs:1381-1396`):

```rust
InputEventResult::StartDrag => {
    mouse_input_state.grabbed = false;
    let drag_area_item = grabber.downcast::<crate::items::DragArea>().unwrap();
    let drag_area = drag_area_item.as_pin_ref();
    let (mut drop_event, allowed) = drag_area.initial_drop_event();
    // ...
    mouse_input_state.drag_data = Some(DragData { event: drop_event, allowed });
    mouse_input_state.drag_source = Some(grabber.downgrade());
    drag_area.dragging.set(true);
    MouseGrabResult { event: None, accepted: true }
}
```

The backend native-drag hook (`start_drag`) is invoked nowhere in 1.17.0 core:
a search of `i-slint-core-1.17.0/` for `start_drag` finds only an unrelated local
variable in `items/drag_n_drop.rs:166` (`let start_drag = dx > threshold || ...`).
So in 1.17.0,
 drag-and-drop is in-process on every backend,
 and no external file
drop is intercepted:
the winit backend crate has no `DroppedFile`/`HoveredFile`/`start_drag` handling
(a search of `i-slint-backend-winit-1.17.0/` for those returns nothing;
its only "drag" code is window-resize dragging).

### Post-1.17.0 adds a native offer, still without a file-list

A later commit on the 1.17 line (the assessment clone at commit `2447c69`) adds a
native-drag offer,
 but it still carries no file-list.
The offer is made only when the payload has serializable text or an image,
 and
otherwise falls back to the in-window drag
(`internal/core/input.rs:1324-1354`):

```rust
fn offer_native_drag(/* ... */) {
    let data = drag_area.data();
    // A native drag only carries serializable data, so offer it only when there's some.
    if data.has_plain_text() || data.has_image() {
        let request = crate::window::DragRequest { data: data.clone(), /* ... */ };
        if window_adapter.internal(crate::InternalToken).is_some_and(|i| i.start_drag(&request)) {
            // ... native took over ...
            return;
        }
    }
    // No backend took over: fall back to the in-window drag.
    state.arm_in_window_drag(drag_area, source, seed_position);
}
```

Only the Qt backend implements the `start_drag` hook,
 and it builds a `QMimeData`
with only text and image,
 never `setUrls`
(`internal/backends/qt/qt_window.rs:2464-2543`,
 clone `2447c69`):

```cpp
QMimeData *mime = new QMimeData();
if (has_text) { mime->setText(text); }
if (has_image) { mime->setImageData(payload_pixmap.toImage()); }
QDrag *qdrag = new QDrag(widget_ptr);
qdrag->setMimeData(mime);
// ... no mime->setUrls(...) anywhere ...
```

The Qt inbound bridge is symmetric:
it reads only text and image from the incoming `QMimeData`,
 never `hasUrls()`
(`internal/backends/qt/qt_window.rs:288-313`):

```cpp
const QMimeData *mime = event->mimeData();
QString text = mime->hasText() ? mime->text() : QString();
QImage image = mime->hasImage() ? qvariant_cast<QImage>(mime->imageData()) : QImage();
// ... no mime->hasUrls() / mime->urls() ...
```

So even the post-1.17.0 Qt backend carries no file-list in either direction,
 and
the winit backend still has no native drag at all.
The `start_drag`-on-Qt-only fact from the plan describes this post-1.17.0 code;
it does not give the file manager OS file drag-and-drop,
 because the payload it
transfers has no file representation.

### DataTransfer is only reachable through a doc-hidden path in 1.17.0

Constructing a `DataTransfer` from Rust to set the app-local payload is awkward on
1.17.0:
the crate-root `slint::DataTransfer` alias landed after the release,
 and the bare
`slint::private_unstable_api::DataTransfer` is a private glob import (rustc
`E0603`).
The only reachable public path is inside the doc-hidden module
(`slint-1.17.0/private_unstable_api.rs:158,182`):

```rust
pub mod re_exports {
    // ...
    pub use i_slint_core::data_transfer::DataTransfer;
}
```

so the prototype imports `slint::private_unstable_api::re_exports::DataTransfer`,
even though `private_unstable_api` is `#[doc(hidden)]` and documents that
"compatibility is not guaranteed".
The in-process `DropEvent`/`DragAction`/`DragArea`/`DropArea` surface itself is
public in 1.17.0:
`DropEvent` is `pub struct DropEvent` in
`i-slint-common-1.17.0/builtin_structs.rs:91`,
 whereas in
`i-slint-common-1.16.1/builtin_structs.rs:110` it was a private
`BuiltinPrivateStruct::DropEvent`,
 so the in-process DnD widgets became public in
1.17.0.

## Verification

Version under test:
the prototype depends on crates.io Slint `1.17.0` (`slint`,
`i-slint-backend-winit`,
 `slint-build`);
source for the version under test is the installed crates under
`~/.cargo/registry/src/index.crates.io-*/` (`slint-1.17.0`,
 `i-slint-core-1.17.0`,
`i-slint-common-1.17.0`,
 `i-slint-compiler-1.17.0`,
 `i-slint-backend-winit-1.17.0`).
Post-1.17.0 source (the native-drag offer,
 the Qt backend) is the assessment clone
at `/tmp/agent/slint-file-manager-assessment-20260705`,
 commit `2447c69` (1.17
line,
 ahead of the 1.17.0 release).

Reproduction harness:
build the prototype with the embedded Slint MCP server and drive it headless.

```bash
# package/file-manager/app
mise run //package/file-manager/app:mcp   # binds 127.0.0.1:9317
```

Works (internal pane-to-pane drag,
 this prototype):
`find_elements_by_id` the row `DragArea`s / `AppWindow::touch` and the pane
`AppWindow::drop` areas,
 then `drag_element` from a row's center to a different
pane's center.
The drop fires and Rust records the identity and action,
 read back on
`AppWindow::hud-e` and logged.
A move drag (default mode) logs
`internal pane-to-pane drop source_pane_id=0 source_row=0 target_pane_id=1 action="move"`;
after clicking `AppWindow::btn-dragcopy` (copy mode),
 the same drag logs
`action="copy"`.
A plain left-click on a row still selects and does not drop,
 so the `DragArea`
wrap leaves selection intact.
The drop target center lands over the destination pane's `ListView` rows,
 and the
drop still fires,
 confirming the `Flickable` forwards `DragMove`/`Drop`
(`i-slint-core-1.17.0` `items/flickable.rs` returns `ForwardAndIgnore` for those)
rather than swallowing them.

Cannot be exercised headless (both fail through the built-in path):

- External file drop from the OS file manager into the window:
  requires a real pointer drag from a separate application,
   which the MCP driver
  cannot synthesize,
   and 1.17.0 wires no `WindowEvent` for it regardless.
  Source-conclusive:
   no external-drop delivery exists in 1.17.0 (no backend
  `start_drag` hook is called;
   the winit backend has no `DroppedFile` handling).
- Outbound native drag from the window to the OS file manager:
  1.17.0 has no native drag path,
   and the post-1.17.0 Qt `start_drag` advertises
  only text/image,
   so no file-list is ever offered.

## Verified workarounds

### Internal drag-and-drop: in-window DragArea/DropArea with a user_data payload

Carry the app-local identity in `DataTransfer::user_data` and keep the payload
free of text and image,
 so the drag stays in-window on every backend and behaves
deterministically:

```rust
// package/file-manager/app/src/drag_drop.rs
let mut transfer = DataTransfer::default();
transfer.set_user_data(Rc::new(DragIdentity { source_pane_id, source_row }));
// no set_plain_text / set_image
transfer
```

```slint
// package/file-manager/app/ui/app.slint
drag := DragArea {
    data: root.make-drag-data(pane.pane-id, row.index);
    allow-copy: true;
    allow-move: !root.drag-force-copy;
    // ... row content ...
}
// ... and per pane ...
drop := DropArea {
    can-drop(event) => { return root.pane-can-drop(event, pane.pane-id); }
    dropped(event) => { return root.pane-dropped(event, pane.pane-id); }
    // ... pane content ...
}
```

Tradeoffs:

- The payload is process-local:
  `user_data` is an `Rc<dyn Any>` and cannot cross to another application,
   which
  is exactly why it never triggers a native drag and stays deterministic,
   but it
  is useless for cross-application transfer.
- With no modifier the proposed action is the first allowed of move,
   copy,
   link,
  so `allow-move` on top of `allow-copy` makes the default a move;
  the prototype flips `allow-move` off with a `drag-force-copy` toggle to make a
  copy drag drivable without injecting a Ctrl modifier.
- Constructing `DataTransfer` on 1.17.0 reaches through the doc-hidden
  `slint::private_unstable_api::re_exports::DataTransfer`;
  a production build should pin a Slint that re-exports it at the crate root.

### OS-native file drag-and-drop: a hand-written per-OS adapter

There is no stock-Slint path;
the file manager needs a native adapter per OS around Slint pointer events (the
plan's failure action).
Inbound needs the platform's dropped-file event
(winit's `DroppedFile`/`HoveredFile`/`HoveredFileCancelled` where available,
 or a
platform hook),
 surfaced into Rust as file paths.
Outbound needs the platform's native drag with a `text/uri-list` payload
(`QMimeData::setUrls` on Qt,
 `NSFilePromiseProvider`/`NSPasteboard` on macOS,
`IDataObject`/`DoDragDrop` on Windows,
 XDND/`text/uri-list` on X11/Wayland).
`crabnebula-dev/drag-rs` is reference material only,
 per the plan.
This adapter belongs in the native-integration milestone,
 at the consumer
boundary,
 so it solves the user-facing need regardless of upstream movement.

## What does not work

- Relying on stock Slint 1.17.0 for OS file drag-and-drop in either direction.
  1.17.0 has no native drag path at all;
  no backend `start_drag` hook is invoked (`i-slint-core-1.17.0/input.rs:1381-1396`).
- Switching to the Qt backend to get file drag-and-drop.
  Even the post-1.17.0 Qt `start_drag` and drop bridge carry only text and image,
  never a file-list (`qt_window.rs:2464-2543` and `:288-313`,
   clone `2447c69`),
  so the winit-vs-Qt choice does not resolve file drag-and-drop.
- Putting file paths in the `user_data` payload to reach another app.
  `user_data` is a process-local `Rc<dyn Any>` and never crosses the application
  boundary.

## Upstream filing decision

`.out-of-scope/` has no Slint exemption for this bug class
(the only Slint mention is `.out-of-scope/cargo-workspace.md:18`,
 about not
installing Slint on the host,
 unrelated).

Duplicate search:
`gh search issues --repo slint-ui/slint "drag drop file"` and
`"start_drag native drag"` return the tracking issue
[`slint-ui/slint#1967`][issue-1967],
 "better Drag 'n' Drop handling" (open,
labelled `a:language-slint`,
 `api`).
No separate file-list / `text/uri-list` / `start_drag` issue exists.

Diff of our findings against the thread:
issue `#1967` is the maintainers' own DnD design issue,
 and a maintainer
(`ogoffart`) has already enumerated both of our gaps as known future work:

- external drop into Slint:
  "Drag&Drop files,
   images,
   ... from external application to Slint:
   We need a drop
  area that can get a callback to accept and get notified from a drag&drop,
   and we
  need to extend our platform::WindowEvent so that we can intercept the drag (eg,
  from winit DroppedFile,
   HoveredFile,
   HoveredFileCancelled)";
- outbound drag to another app:
  "Dragging from a Slint application to another application.
   So when dragging,
  we're not moving a Slint element,
   but assign an image to the cursor,
   with a
  mimetype and a filename or some other clipboard content."

Both gaps we found are named there,
 in the maintainers' own words,
 with the exact
winit events and the mimetype+filename requirement.
Our source trace adds precision (the specific 1.17.0 lines and the Qt
text/image-only bridge) but nothing the maintainers do not already know about
their own code.

Six-constraint check:

1. Upstream's fault:
   yes,
    it is a Slint feature gap (no file-list in `DataTransfer`,
    no native drag
   path in 1.17.0).
2. Can upstream fix it:
   yes,
    and they have scoped it in `#1967` (extend `WindowEvent` for inbound,
    a
   mimetype+filename payload for outbound).
3. Supporting this use case:
   not yet;
   it is explicitly planned future work,
    not shipped.
4. Would the repo welcome our contribution:
   not applicable;
   there is nothing to contribute that is not already in `#1967`.
5. Will they likely fix it:
   plausibly;
   `#1967` is open and maintainer-authored with no won't-fix signal.
6. Prototyped a minimal fix:
   not applicable;
   a full file-list DnD implementation spans core plus every backend and is a
   large feature the maintainers are designing in `#1967`,
    and there is nothing
   additive to file,
    so no fix draft is warranted.

Decision:
do not open a new issue (`#1967` is the duplicate),
 and post no comment
(`#1967` already documents both gaps in the maintainers' own words,
 so nothing we
have advances the thread).
There is no fileable draft;
the fix is the consumer-side per-OS native adapter recorded above,
 deferred to the
native-integration milestone,
 which solves the user-facing problem regardless of
upstream movement.

[issue-1967]: https://github.com/slint-ui/slint/issues/1967
