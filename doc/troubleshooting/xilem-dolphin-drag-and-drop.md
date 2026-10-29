# Xilem 0.4.0 on Winit 0.30.13 cannot exchange native files with Dolphin

This note assesses bidirectional native file drag-and-drop between a Xilem/Masonry
application and KDE Dolphin.
 It separates the outgoing direction (app to Dolphin),
the incoming direction (Dolphin to app),
 the Linux backend each direction needs,
and Dolphin's own interoperability record.

This is not the first drag-and-drop investigation in this repository.
[winit-toolkits-no-wayland-drag-and-drop.md](winit-toolkits-no-wayland-drag-and-drop.md)
surveys the whole winit family and the native-toolkit alternatives,
[kwin-drag-only-first-data-device.md](kwin-drag-only-first-data-device.md) traces
KWin's single-data-device rule,
 and
[slint-drag-and-drop-file-lists.md](slint-drag-and-drop-file-lists.md) covers
in-process drag areas.
 What this note adds is the Xilem-specific trace,
 the state
of Winit's new data-transfer API,
 and the consequences of both for Dolphin.

Evidence is pinned to revisions checked on 2026-09-23:

- Xilem `271a27a6d4a930f7878d404f9014e3c50a3a9b88` (committed 2026-09-14).
- Winit stable `v0.30.13`,
   commit `e9809ef54b18499bb4f2cac945719ecc2a61061b`.
- Winit pre-release `v0.31.0-beta.3`,
   commit
  `7d20408d33210c93bf036335b88ee083ca563a90`.
- Winit development branch `2fd05d48b521846c5ae22c6f0a66e10bc0c91ca6`.
- `ui-events-winit` 0.3.0,
   tag `v0.3.0` of `endoli/ui-events`.

## Symptom

Dragging an item inside a Xilem window with pointer events is not the same as
starting an operating-system drag that Dolphin can receive.
 In the checked Xilem
revision there is no API to start a native file drag,
 so nothing can be offered to
Dolphin on any backend.

The incoming direction depends on the session type:

- On a native Wayland session,
   Winit `v0.30.13` emits no file events at all,
   so
  a Dolphin drop reaches neither the window layer nor a widget.
- On X11 or XWayland,
   Winit `v0.30.13` does emit `DroppedFile` and
  `HoveredFile`,
   but Xilem's event routing discards them,
   so a widget still never
  sees the drop.

This finding is about native file exchange with another desktop application.
 It
does not say in-app pointer dragging is impossible.

## Root cause

### Winit 0.30.13 has an incoming-only, X11-and-macOS-only file event surface

Winit `v0.30.13` declares the events in its shared enum at `src/event.rs:180`,
`src/event.rs:186`,
 and `src/event.rs:192`:

```rust
DroppedFile(PathBuf),
HoveredFile(PathBuf),
HoveredFileCancelled,
```

Counting the emitters per backend at that tag shows which platforms actually
produce them:

```text
v0.30.13:src/platform_impl/linux/x11/event_processor.rs:6
v0.30.13:src/platform_impl/macos/window_delegate.rs:3
```

`src/platform_impl/linux/wayland/` has no match,
 so the Wayland backend never
emits them.
 A search of the whole `src/` tree of that tag for `start_drag` or
`DataTransfer` returns no matches,
 so the release has no outgoing drag request and
no transfer-type negotiation on any platform.

### Xilem depends on that release and discards its file events

At the checked Xilem revision,
 the workspace dependency is:

```toml
# Cargo.toml:70
winit = "0.30.13"
```

`masonry_winit/src/event_loop_runner.rs:310` receives each Winit window event and
delegates at `:316` to `MasonryState::handle_window_event`,
 declared at `:808`.
That handler feeds `ui-events-winit`'s reducer,
 forwards the resulting keyboard and
pointer translations (pointer case at `:878`),
 then matches a fixed set of window
events at `:902` for rescale,
 redraw,
 close,
 resize,
 IME,
 and focus.
 The match ends
in a catch-all at `:936`:

```rust
// masonry_winit/src/event_loop_runner.rs:936
_ => (),
```

`WinitWindowEvent::DroppedFile` and `WinitWindowEvent::HoveredFile` fall into that
catch-all.
 The reducer cannot pick them up either:
 `ui-events-winit` 0.3.0 handles
modifiers,
 keyboard,
 cursor enter/leave/move,
 mouse input,
 wheel,
 pinch,
 rotation,
and touch in `ui-events-winit/src/lib.rs:105` through `:252`,
 and returns `None`
for everything else at `:253`.
 Its `reduce` method is declared at `:85`.
 A search of
that file for `DroppedFile`,
 `HoveredFile`,
 or `FileDrop` returns no matches.

A tree-wide search of the checked Xilem revision for `DroppedFile`,
 `HoveredFile`,
`DragEntered`,
 `DragDropped`,
 `DataTransfer`,
 `start_drag`,
 and
`fetch_data_transfer` across `masonry/src`,
 `masonry_core/src`,
`masonry_winit/src`,
 `xilem/src`,
 and `xilem_masonry/src` returns no matches.

### Masonry removed its file-drop event variants deliberately

Xilem pull request [#950][xilem-pr-950],
 merged 2025-05-08,
 removed the
file-transfer variants from Masonry's own `PointerEvent`.
 At its base commit
`4ebfeed68a14b968260dca19fee45ed0822b9fdf`,
`masonry_core/src/core/events.rs` declared them at `:211`,
 `:213`,
 and `:215`:

```rust
HoverFile(PathBuf, PointerState),
DropFile(PathBuf, PointerState),
HoverFileCancel(PointerState),
```

The pull request body states the rationale:

> Remove file hover/drop events from `PointerEvent` because they were never mapped
> and could never be mapped (since they relied on pointer positions which are not
> available in file hover on winit).

The "never mapped" half is independently verifiable:
 at that same base commit,
 a
search of every `*.rs` file for `DroppedFile`,
 `HoveredFile`,
 or
`HoveredFileCancelled` returns no matches,
 so no code ever converted Winit's file
events into those variants.

The consequence is design-level,
 not just a missing case statement.
 Modeling a file
drop as a pointer event was tried and rejected,
 so incoming drops need their own
event type carrying a transfer identifier and a position,
 which is the shape Winit
`v0.31.0-beta.3` now provides.

### Winit merged a bidirectional API, with an X11 gap

Winit pull request [#4571][winit-pr-4571] merged on 2026-07-16 as commit
`156433eb912a62a1a07da0f9bbaa2d775270c788`,
 and shipped in the pre-release
`v0.31.0-beta.3` published 2026-09-04.
 The latest stable release is still
`v0.30.13` (published 2026-03-02),
 because `v0.31.0-beta.3` is marked as a
pre-release.
 Winit issue [#1881][winit-1881],
 "Support drag and drop on wayland",
closed as completed on 2026-08-03.
 Issues [#720][winit-720] and
[#1499][winit-1499] remain open as the general drag-and-drop tracking stubs.

In `v0.31.0-beta.3`:

- `winit-core/src/event.rs:88`,
   `:106`,
   `:125`,
   `:139`,
   `:152`,
   `:163`,
   and
  `:174` declare `DragEntered`,
   `DragPosition`,
   `DragDropped`,
   `DragLeft`,
`DataTransferReceived`,
   `OutgoingDragDropped`,
   and `OutgoingDragCanceled`.
- `winit-core/src/event_loop/mod.rs` declares `fetch_data_transfer` at `:221`,
`data_transfer` at `:237`,
   `set_valid_dnd_actions` at `:263`,
   and `start_drag` at
  `:317`.
   The default `start_drag` body returns `RequestError::NotSupported` at
  `:329`,
   using the message constant at `:334`.
- `winit-wayland/src/event_loop/mod.rs` overrides all four:
   `:786`,
   `:851`,
   `:864`,
  and `:891`.
- `winit-x11/src/event_loop.rs` overrides only the incoming side:
   `:817`,
   `:831`,
  and `:881`.
   It has no `start_drag` override,
   so an X11 backend falls through to
  the unsupported default.
- `winit-x11/src/event_processor.rs:531` and `:561` carry comments stating that
  non-copy drag actions are not implemented on X11,
   and both sites report
  `DndAction::Copy`.
- `winit-core/src/event_loop/mod.rs:376` declares `DndAction`;
   the `Move` variant
  documents Wayland,
   macOS,
   and Windows at `:377` through `:384`,
   and `Link`
  documents macOS and Windows only.
- `winit-core/src/data_transfer.rs:143` declares the `UriList` type hint,
   which is
  the cross-platform representation of a file list.

The development branch at `2fd05d48b521846c5ae22c6f0a66e10bc0c91ca6` has the same
shape:
 `fn start_drag` appears in the core default and in the Wayland backend only.
Upstream stated the X11 scope explicitly in the pull request description:

> Both receiving and initiating a drag operation are implemented on Windows,
> Wayland and macOS.
> X11 supports receiving dropped data,
> but initiating a drag on
> X11 is not planned as part of this PR.

So the API route exists and is published as a pre-release,
 but two things stand
between it and Dolphin:
 Xilem has not adopted it,
 and X11 has no outgoing
implementation in either the pre-release or the development branch.

### Dolphin and KWin add two hazards of their own

Two KDE reports describe Dolphin failing to deliver drag data to native non-Qt
Wayland clients:

- [Bug 519492][dolphin-519492],
   "Plasma 6.6.4 + Wayland:
   drag offer to non-Qt
  clients has empty MIME types".
   Status `REPORTED`,
   last modified 2026-07-15 when
  fetched.
   The report says the drop event arrives with zero MIME types and no file
  payload in Brave and Firefox running natively on Wayland,
   while drags from Thunar
  and from the Plasma desktop succeed on the same system.
- [Bug 484018][dolphin-484018],
   "KDE Plasma in Wayland does not support Drag and
  Drop operations from Dolphin to Chromium (and Firefox)".
   Status `REOPENED`,
   last
  modified 2026-09-06 when fetched,
   with a comment dated 2026-09-06 confirming the
  behavior on Fedora 44 with Plasma 6.7.4 on Wayland.
   It lists 519492 under "See
  Also".

Neither report was reproduced with a Xilem application,
 and neither mentions Xilem,
Masonry,
 or Winit.
 They are compatibility risk evidence for the incoming direction,
not proof that a Xilem drop target would fail.
 They also fix the failure mode:
 the
offer arrives and the type list is empty.
 A Xilem drop target that requests only
`text/uri-list` would see exactly that symptom,
 so a test must record the offered
type list rather than only pass or fail.

The second hazard is compositor-side and specific to KWin,
 which is what Dolphin
sessions run.
[kwin-drag-only-first-data-device.md](kwin-drag-only-first-data-device.md)
traces KWin's `dropHandlerForSurface` taking `list.first()`
(`src/wayland/seat.cpp:181` in KWin,
 re-verified there at `v6.7.1` and at several
earlier tags):
 when one client binds more than one `wl_data_device`,
 KWin delivers
the drag to the first and never reconsiders.
 That is what broke the Slint file
manager's hand-rolled adapter,
 because Slint's clipboard had already bound a device.

Whether that hazard applies to a future Xilem on Winit 0.31 is an inference,
 not a
measurement.
 The relevant fact is that Masonry's clipboard context is allowed to
fail on Linux and then does nothing,
 at
`masonry_winit/src/event_loop_runner.rs:404` through `:412`:

```rust
let clipboard_cx = if cfg!(target_os = "linux") {
    // If we're running on Linux, we might fail to get the clipboard context because
    // we're using Wayland, so we fall back to NopClipboardContext to be safe.
    clipboard_cx.unwrap_or_else(|_| Box::new(NopClipboardContext))
```

If that fallback is what happens on a Wayland session,
 a Xilem app binds no
competing data device,
 and Winit's own device would be the first one KWin sees.
 Upstream hit the same interaction from the other side:
 a comment on pull request
#4571 dated 2026-08-03 reports that Winit's data device stops `smithay-clipboard`
based crates from receiving events,
 and a maintainer replied that Wayland permits
multiple data devices and called it a compositor bug.
 Both readings point at the
same test:
 on KWin,
 count the client's data devices and confirm which one receives
`wl_data_device.enter`.

## Verification

Revisions and the commands that produced each finding.
A shallow clone has neither the cited tags nor the pull request #950 base commit,
so fetch each revision first.
These setup commands and every search in this section
were re-run in fresh clones on 2026-09-23 and produced the results quoted here.

```bash
# Xilem: checked revision, plus the pre-removal base of pull request #950.
gh repo clone linebender/xilem "$HOME/temp/agent/xilem-dnd" -- --depth 1
git -C "$HOME/temp/agent/xilem-dnd" fetch --depth 1 origin \
  271a27a6d4a930f7878d404f9014e3c50a3a9b88 \
  4ebfeed68a14b968260dca19fee45ed0822b9fdf

# Winit: stable tag and pre-release tag.
gh repo clone rust-windowing/winit "$HOME/temp/agent/winit-dnd" -- --depth 1
git -C "$HOME/temp/agent/winit-dnd" fetch --depth 1 origin \
  refs/tags/v0.30.13:refs/tags/v0.30.13 \
  refs/tags/v0.31.0-beta.3:refs/tags/v0.31.0-beta.3
```

Run each search from inside the matching clone.

```bash
# Winit stable: which backends emit file events, and how often.
git grep -c -E 'DroppedFile|HoveredFile' v0.30.13 -- \
  src/platform_impl/linux/wayland src/platform_impl/linux/x11 src/platform_impl/macos

# Winit stable has no outgoing API. Expected: no matches, exit status 1.
git grep -n -E 'start_drag|DataTransfer' v0.30.13 -- src

# Winit pre-release: which backends implement start_drag.
git grep -n 'fn start_drag' v0.31.0-beta.3 -- \
  winit-core/src/event_loop/mod.rs winit-wayland/src/event_loop/mod.rs winit-x11/src/event_loop.rs

# Winit pre-release: X11 copy-only limitation.
git grep -n 'do not implement non-copy drag' v0.31.0-beta.3 -- winit-x11/src/event_processor.rs

# Xilem dependency at the checked revision.
git grep -n '^winit = ' 271a27a6d4a930f7878d404f9014e3c50a3a9b88 -- Cargo.toml

# Xilem routing: catch-all arm and pointer-only translation.
git grep -n -E 'fn window_event\(|pub fn handle_window_event\(|WindowEventTranslation::Pointer|_ => \(\),' \
  271a27a6d4a930f7878d404f9014e3c50a3a9b88 -- masonry_winit/src/event_loop_runner.rs

# Xilem clipboard fallback on Linux.
git grep -n 'NopClipboardContext' \
  271a27a6d4a930f7878d404f9014e3c50a3a9b88 -- masonry_winit/src/event_loop_runner.rs

# Xilem has no native DnD surface. Expected: no matches, exit status 1.
git grep -n -E \
  'FileDrop|FileHover|DroppedFile|HoveredFile|DragEntered|DragDropped|DataTransfer|start_drag|fetch_data_transfer' \
  271a27a6d4a930f7878d404f9014e3c50a3a9b88 -- \
  masonry/src masonry_core/src masonry_winit/src xilem/src xilem_masonry/src

# Removed variants existed before pull request 950.
git grep -n -E 'HoverFile|DropFile' \
  4ebfeed68a14b968260dca19fee45ed0822b9fdf -- masonry_core/src/core/events.rs

# Those variants were never fed from Winit. Expected: no matches, exit status 1.
git grep -n -E 'DroppedFile|HoveredFile|HoveredFileCancelled' \
  4ebfeed68a14b968260dca19fee45ed0822b9fdf -- '*.rs'
```

The `ui-events-winit` findings come from the published crate source rather than
from a Xilem clone:

```bash
curl --silent --location \
  'https://raw.githubusercontent.com/endoli/ui-events/v0.3.0/ui-events-winit/src/lib.rs' \
  -o "$HOME/temp/agent/ui-events-winit-lib.rs"
grep -n -E 'pub fn reduce|WindowEvent::|_ => None' "$HOME/temp/agent/ui-events-winit-lib.rs"

# Expected: no matches, exit status 1.
grep -n -E 'DroppedFile|HoveredFile|FileDrop' "$HOME/temp/agent/ui-events-winit-lib.rs"
```

Each command returned the matches quoted in the Root cause section,
 or the stated
no-match result with exit status 1.
 The no-match results were sanity-checked against
a broader pattern in the same tree,
 so a wrong path or a bad pattern could not
masquerade as an absent feature.
 The backend count is a positive control of that
kind:
 the same pattern matches in the X11 and macOS backends while returning
nothing for Wayland.

What was not verified:

- No Xilem application was built and no live Dolphin transfer was attempted,
   so
  neither direction is demonstrated end to end.
- The two KDE reports were read,
   not reproduced.
- No X11 or XWayland session behavior was measured.
- The KWin data-device ordering for a Xilem process was not measured;
   the
  reasoning in the Root cause section is inference from source.

### Present in the checked source

- Winit `v0.30.13` incoming file-path events on X11 and macOS.
- Winit `v0.31.0-beta.3` cross-platform drag data types,
   asynchronous fetch,
  action negotiation,
   and a Wayland `start_drag` implementation.
- Masonry pointer-event translation for in-app dragging.
- A working bidirectional GTK4 implementation in this repository,
   described in the
  Verified workarounds section.

### Absent or incomplete

- Winit `v0.30.13` file events on Wayland.
- Any Xilem or Masonry API for starting a native drag.
- Any routing of Winit file-drop or drag events into the Masonry widget tree.
- An X11 `start_drag` implementation in Winit `v0.31.0-beta.3` or in its
  development branch.
- Non-copy drag actions on X11 incoming transfers.
- Evidence of Dolphin interoperability in either direction.

## Verified workarounds

No workaround provides bidirectional native file exchange between Xilem 0.4.0 and
Dolphin.

The repository already contains a route that works,
 and it is not Xilem.
`package/file-manager/app/` is a GTK4 application whose dependency comment
at `Cargo.toml:20` through `:26` records the reason for the choice:
`GtkDropTarget` and `GtkDragSource` give native Wayland drag-and-drop,
 "the
capability the winit/Slint stack lacked".
 Its `src/dnd.rs` module comment states
that inbound uses a native `GtkDropTarget` over `GdkFileList` and outbound is
native on Wayland via `GtkDragSource`,
 with Windows and macOS needing separate
shims.
 Tradeoff:
 that is a different toolkit,
 so it is an alternative to Xilem
rather than a fix inside it.

Implementing an in-app drag from pointer events works for app-owned content and
needs no upstream change,
 but it never registers an operating-system drag source.
Dolphin cannot receive it,
 and it cannot accept a Dolphin offer either.

Depending on Winit `v0.31.0-beta.3` directly is not a consumer-side workaround.
Xilem's window and event plumbing is written against `0.30.13`,
 its reducer crate
`ui-events-winit` 0.3.0 matches Winit 0.30 event names such as `CursorMoved` and
`MouseInput`,
 and the new requests live on `ActiveEventLoop` rather than on a window
event that an application could intercept from outside Masonry.

## What does not work

- Treating Winit `v0.30.13`'s `DroppedFile` as a bidirectional API,
   or as available
  on Wayland.
   It is an incoming event,
   the X11 and macOS backends are its only
  emitters,
   and Xilem discards it.
- Treating in-app pointer dragging as system drag-and-drop.
- Treating Winit pull request #4571 as a finished Xilem feature.
   The API is in a
  pre-release and Xilem has not adopted it.
- Assuming X11 parity from the Wayland implementation.
   The X11 backend has no
  `start_drag` override and reports `Copy` only on incoming transfers.
- Modeling a file drop as a `PointerEvent` variant.
   Xilem pull request #950 removed
  exactly that modeling,
   with the reason recorded in its description.
- Switching to another winit-based toolkit to escape the gap.
   The survey in
  [winit-toolkits-no-wayland-drag-and-drop.md](winit-toolkits-no-wayland-drag-and-drop.md)
  records that Slint,
   Bevy,
   Iced,
   egui,
   floem,
   and Xilem all inherit it.
- Binding a second `wl_data_device` beside a toolkit's clipboard device on KWin.
  [kwin-drag-only-first-data-device.md](kwin-drag-only-first-data-device.md) records
  the drag going to the first device only.
- Treating framework support as proof of Dolphin compatibility,
   given
  [Bug 519492][dolphin-519492] and [Bug 484018][dolphin-484018].

## What would make it possible

The route with the most existing plumbing is native Wayland.
 It needs four pieces:

1. A Winit release Xilem can depend on that carries the data-transfer API,
   plus the
  Xilem-side dependency and event-name migration (`CursorMoved` to `PointerMoved`,
`MouseInput` to `PointerButton`,
   and the reducer crate that matches them).
2. Masonry routing for the new events:
   `DragEntered`,
   `DragPosition`,
`DragDropped`,
   `DragLeft`,
   `DataTransferReceived`,
   `OutgoingDragDropped`,
   and
  `OutgoingDragCanceled`,
   delivered as their own widget event type rather than as
  pointer events,
   per the pull request #950 rationale.
3. A widget-level API in both directions.
   Outgoing:
   declare offered types
  (`UriList` for files),
   supply bytes lazily,
   choose permitted `DndAction` values,
  and provide a drag icon.
   Incoming:
   accept or reject with
  `set_valid_dnd_actions`,
   request a type with `fetch_data_transfer`,
   and read the
  result asynchronously.
4. Dolphin testing that records Plasma and Dolphin versions,
   session type,
   the
  offered MIME type list,
   the negotiated action,
   the number of `wl_data_device`
  objects the process binds,
   and cancellation behavior.

X11 bidirectional support additionally needs an outgoing XDND implementation in
Winit itself,
 which neither `v0.31.0-beta.3` nor the development branch contains,
and which upstream described as out of scope for the merged pull request.
 Full
copy/move/link negotiation on X11 needs more than that,
 since the backend reports
`Copy` only.

Two ways to reduce the risk before committing to the integration:

- Test the incoming direction without Dolphin first.
  `package/nested-wayland-session/cli`
  implements a `drop-file` command (`src/protocol.rs:303`,
   parsed at `:455` through
  `:480`) that drives a real `wl_data_device` drop against a nested client,
   which is
  how the Slint file manager's inbound path was verified.
   A Smithay compositor does
  not reproduce KWin's first-device rule or Dolphin's offer contents,
   so it settles
  plumbing,
   not interoperability.
- Then run the same build against Dolphin on KWin and record the offered type list,
  which is the datum that distinguishes a Xilem bug from
  [Bug 519492][dolphin-519492].

This is engineering work,
 not a funding question.
 Developer time is required for the
migration,
 routing,
 widget API,
 and Dolphin testing;
 money can pay for that time and
can pay for an X11 outgoing backend in Winit if X11 matters.
 Money cannot shorten
Winit's release schedule,
 cannot make Xilem adopt the API,
 and cannot fix a Dolphin
drag offer that advertises no MIME types.
 No duration or cost estimate is supportable
from this source audit,
 because nothing here has been built or measured.

Two decisions have to be made before anyone can scope the work:

- Which session types must work:
   native Wayland only,
   or X11 and XWayland too.
- Which semantics are required:
   file URIs only,
   or negotiated copy/move/link with
  completion and cancellation reporting.

## Upstream filing artifact

### Upstream filing decision

Nothing was filed and no draft is kept.
 The request was an assessment of what would
make the feature possible,
 and the evidence describes an unimplemented feature plus an
API adoption boundary,
 not a demonstrated Xilem defect.

The six checks are recorded so later work does not mistake the capability gap for a
confirmed upstream bug:

1. **Is it really upstream's fault?**
    Not established as a defect.
    Xilem does not
   expose the feature,
    and its pinned Winit release lacks an outgoing API and lacks
   Wayland file events entirely.
    Winit has since merged both for Wayland.
2. **Can upstream fix it?**
    Yes in principle.
    Wayland outgoing support exists in
   `v0.31.0-beta.3`;
    Xilem can bridge the events and requests.
    X11 outgoing support
   is separate,
    unbuilt work in Winit that upstream called out of scope for the merged
   pull request.
3. **Are they supporting this use case?**
    No documented signal in Xilem.
    No doc,
   example,
    or test covers native file exchange.
    Open issue
   [#1768][xilem-issue-1768] lists missing core features and does not mention
   drag-and-drop;
    open issue [#1836][xilem-issue-1836] proposes refactoring
   `masonry_winit` and does not mention it either.
    Winit,
    by contrast,
    now documents
   the data-transfer API as a supported feature.
4. **Would the repo welcome a contribution?**
    `README.md:150` states "Contributions
   are welcome by pull request."
    The checked revision has no `CONTRIBUTING.md`,
    no
   issue or pull request template,
    and no AI-assistance restriction;
    `.github/`
   contains only `copyright.sh`,
    `debug_assertions.sh`,
    and `workflows/ci.yml`.
5. **Will they likely fix it?**
    No signal either way for Xilem.
    Searches of the Xilem
   tracker for `drag and drop`,
    `drag`,
    `dnd`,
    `drop file`,
    `drop`,
    and `winit 0.31`
   across open and closed state,
    plus a title search for `drag drop file`,
    found no
   native drag-and-drop tracking issue or pull request.
    Closed issue
   [#312][xilem-issue-312] is about unifying pointer and touch events,
    not file
   transfer.
    On the Winit side,
    [#1881][winit-1881] is closed as completed and
   [#720][winit-720] and [#1499][winit-1499] remain open,
    so an X11 outgoing gap has
   a live tracking home but no filed issue.
    Absence of signal is not a refusal.
6. **Have we prototyped a minimal fix?**
    No.
    The required semantics and the Dolphin
   interoperability outcome are both unknown,
    so a prototype would be guessing at the
   target.
    Because this is a capability assessment rather than an accepted feature
   request,
    the auto-prototype step was not started.

`.out-of-scope/` was checked for an Xilem,
 Winit,
 Dolphin,
 or drag-and-drop
exemption;
 the only matches for those words are unrelated `lightningcss.md` and
`jsr.md` text,
 so no exemption applies.

The duplicate search also covers Winit:
 [#1881][winit-1881] is closed,
 so the Wayland
gap it tracked is not fileable,
 and the X11 outgoing gap is stated as out of scope in
the merged pull request rather than tracked as an issue.
 If X11 support later becomes
a requirement,
 the additive content for [#720][winit-720] would be the
`winit-x11/src/event_loop.rs` finding that the backend overrides the three incoming
methods and none of the outgoing ones.

If a Xilem feature request is wanted later,
 the additive content is the pull request
#950 finding:
 Masonry's file-drop variants were removed as unmappable pointer events,
and Winit `v0.31.0-beta.3` now supplies the non-pointer shape that makes them
implementable.
 That belongs in a feature request,
 not a bug report,
 and it needs the
Wayland prototype result attached to be worth a maintainer's time.

## Related docs

- [winit-toolkits-no-wayland-drag-and-drop.md](winit-toolkits-no-wayland-drag-and-drop.md):
  the winit-family survey,
   the native-toolkit alternatives,
   and the hand-rolled
  `wl_data_device` history.
- [kwin-drag-only-first-data-device.md](kwin-drag-only-first-data-device.md):
   KWin
  delivers a drag to a client's first data device only.
- [slint-drag-and-drop-file-lists.md](slint-drag-and-drop-file-lists.md):
   in-process
  drag areas cannot cross the application boundary.
- [gtk4-macos-file-dnd.md](gtk4-macos-file-dnd.md) and
  [gtk4-windows-outbound-file-drag.md](gtk4-windows-outbound-file-drag.md):
   the
  outbound shims the GTK4 file manager needs off Linux.
- `doc/handover/file-manager-native-dnd.md`:
   the native-DnD build state and the
  nested-compositor drop test.

## References

- Xilem revision [`271a27a`][xilem-commit]:
   `Cargo.toml:70`,
`masonry_winit/src/event_loop_runner.rs:310,316,404-412,808,878,902,936`.
- Xilem pull request [#950][xilem-pr-950] and its base revision
  [`4ebfeed`][xilem-base-commit]:
   `masonry_core/src/core/events.rs:211,213,215`.
- Winit [`v0.30.13`][winit-stable]:
   `src/event.rs:180,186,192`,
`src/platform_impl/linux/x11/event_processor.rs`,
`src/platform_impl/macos/window_delegate.rs`.
- Winit [`v0.31.0-beta.3`][winit-beta]:
   `winit-core/src/event.rs:88-174`,
`winit-core/src/event_loop/mod.rs:221,237,263,317,329,334,376-384`,
`winit-core/src/data_transfer.rs:143`,
`winit-wayland/src/event_loop/mod.rs:786,851,864,891`,
`winit-x11/src/event_loop.rs:817,831,881`,
`winit-x11/src/event_processor.rs:531,561`.
- Winit pull request [#4571][winit-pr-4571],
   merged 2026-07-16;
   issues
  [#1881][winit-1881],
   [#720][winit-720],
   and [#1499][winit-1499].
- `ui-events-winit` 0.3.0:
   [`ui-events-winit/src/lib.rs`][ui-events-lib] at `:85`,
  `:105-252`,
   `:253`.
- KDE [Bug 519492][dolphin-519492] and [Bug 484018][dolphin-484018].
- Xilem issues [#1768][xilem-issue-1768],
   [#1836][xilem-issue-1836],
   and
  [#312][xilem-issue-312].
- This repository:
   `package/file-manager/app/Cargo.toml:20-26`,
`package/file-manager/app/src/dnd.rs:1-8`,
`package/nested-wayland-session/cli/src/protocol.rs:303,455-480`.

[xilem-commit]: https://github.com/linebender/xilem/commit/271a27a6d4a930f7878d404f9014e3c50a3a9b88
[xilem-base-commit]: https://github.com/linebender/xilem/commit/4ebfeed68a14b968260dca19fee45ed0822b9fdf
[xilem-pr-950]: https://github.com/linebender/xilem/pull/950
[xilem-issue-312]: https://github.com/linebender/xilem/issues/312
[xilem-issue-1768]: https://github.com/linebender/xilem/issues/1768
[xilem-issue-1836]: https://github.com/linebender/xilem/issues/1836
[winit-stable]: https://github.com/rust-windowing/winit/releases/tag/v0.30.13
[winit-beta]: https://github.com/rust-windowing/winit/releases/tag/v0.31.0-beta.3
[winit-pr-4571]: https://github.com/rust-windowing/winit/pull/4571
[winit-1881]: https://github.com/rust-windowing/winit/issues/1881
[winit-720]: https://github.com/rust-windowing/winit/issues/720
[winit-1499]: https://github.com/rust-windowing/winit/issues/1499
[ui-events-lib]: https://github.com/endoli/ui-events/blob/v0.3.0/ui-events-winit/src/lib.rs
[dolphin-519492]: https://bugs.kde.org/show_bug.cgi?id=519492
[dolphin-484018]: https://bugs.kde.org/show_bug.cgi?id=484018
