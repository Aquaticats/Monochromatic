# Plan: file manager

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

Status:
 accepted stack direction.
 Product decisions resolved in a grilling session on 2026-07-05.
 Column-strip virtualization spike built and passed on Linux (2026-07-05);
 see `package/file-manager/app/` and the spike result below.
 Row context-menu spike built and passed on Linux (2026-07-05);
 see the context-menu spike result below.
 Drag-and-drop spike built on Linux (2026-07-05):
 internal pane-to-pane drag passes;
 OS-native file drag-and-drop is not available in stock Slint on any backend and
 needs a per-OS native adapter (see the drag-and-drop spike result below).
 Native default-manager spike not yet run.
 Virtualization and context-menu spikes not yet run on macOS or Windows.
 Authored 2026-07-05.

## Goal

Build an infinite horizontal-scrolling file manager for desktop operating systems.
The interaction model is Niri-like:
the user moves through a horizontal strip of columns,
and each column stacks one or more panes vertically.

The first supported desktop targets are Windows,
macOS,
and Linux,
with default-manager registration on all three.
Electron and other bundled-browser stacks are out of scope for this project.

The v1 audience is personal use on the author's machines,
with distribution-shaped boundaries:
the packaging and recovery layer stays a designed interface,
but only the personal path is implemented,
and signing,
notarization,
installers,
and update channels are deferred.

## Source facts verified before this plan

- `doc/handover/slint-file-manager-assessment.md` records the prior Slint assessment:
  Slint is viable as a UI layer,
  while default-file-manager behavior needs native integration per operating system.
- The same handover records minimal Slint `cargo check` success on `m1` for macOS
  and `x13-win` for Windows.
- This repo already ships two Slint desktop apps:
  `package/terminal/app` and `package/music-player/desktop-app`,
  both on Slint 1.17.0 with the explicit winit backend and femtovg renderer.
  Their conventions (slint-build pipeline,
  `directories` for per-user paths,
  tracing for logs,
  std-thread service patterns with isolated tokio runtimes) are the local prior art.
- Slint desktop docs say Slint runs on Windows,
  macOS,
  and Linux distributions using Wayland or X-Windows,
  glibc,
  and d-bus.
- Slint Node docs mark the Node integration as Beta.
  The Rust integration is the safer default for a privileged desktop file manager.
- Slint `ListView` docs say only visible list items are instantiated,
  which gives stable performance for large vertical file lists.
- Slint source at `/tmp/agent/slint-file-manager-assessment-20260705/internal/core/model/repeater.rs`
  implements `ListView` virtualization around `viewport-y`,
  so row virtualization is vertical.
  The two-dimensional column-strip virtualization this product needs has no Slint building block
  and needs a project-level design and prototype.
- Slint source at `/tmp/agent/slint-file-manager-assessment-20260705/internal/core/data_transfer.rs`
  stores plain text,
  images,
  and application-local `user_data`.
  It has a TODO for custom binary data providers and MIME types.
- Slint source at `internal/backends/qt/qt_window.rs` implements `start_drag` with `QDrag` and `QMimeData`;
  the winit backend has no `start_drag` implementation in the audited checkout.
- Slint issue `#1967`,
  `better Drag 'n' Drop handling`,
  is still open.
  Current comments describe in-process drag and drop as almost complete,
  not complete native file-list drag and drop.
- `crabnebula-dev/drag-rs`,
  a crate for starting native drags from winit windows,
  is badly maintained per the author's review.
  Treat it as reference material for the native-adapter path,
  never as a dependency.
- Slint issue `#12354`,
  `ContextMenuArea: right-click on StandardTableView / ListView rows does not open menu`,
  is open and untriaged.
  Row context menus are a file-manager requirement,
  so this must be reproduced and worked around or fixed.
- `dmtrKovalenko/fff` is an MIT-licensed Rust file-search SDK
  with an embeddable `fff-core` crate,
  v0.9.6 released 2026-06-21 and active pushes as of 2026-07-05.
  It brings its own file watcher (`notify` 9.0.0-rc.3 plus a forked debouncer),
  an on-disk LMDB index via `heed`,
  and vendored libgit2 for ignore-awareness.
- Alternative stack docs were checked before choosing Slint plus Rust:
  Qt,
  Flutter,
  Tauri,
  Avalonia,
  Iced,
  and GPUI.
  The rejected alternatives and reasons are recorded in `doc/decision/slint-file-manager-stack.md`.

## Interaction model

The strip is a sequence of columns;
each column stacks panes vertically,
matching Niri's column model.
The layout is two-dimensional:
horizontal position is lineage depth,
vertical position within a column accumulates panes spawned from the previous column.

Single-click on anything spawns its child representation in the next column,
then auto-scrolls focus there:

- single-click on a directory spawns a listing pane,
- single-click on a file selects it and spawns a preview pane,
- Enter and double-click on a file open it with the OS default application.

Spawn rules:

- Dedup first:
  clicking an entry whose pane already exists focuses the existing pane instead of spawning.
- A modifier (for example Ctrl+click) forces a fresh duplicate pane.
- Preview panes are symmetric with directory panes:
  each previewed file gets its own persistent pane,
  dedup-and-focus on revisit,
  explicit close.
  Browsing many photos deliberately mints many panes;
  culling (see spike gates) must keep that cheap.
- Keyboard selection changes follow the same spawn and dedup rules as clicks.
- A pane opened by the OS (default-manager launch paths) has no parent pane;
  it spawns as a new column immediately right of the focused column,
  after dedup.

Lifecycle rules:

- Panes die only on explicit close (Niri-style).
- No automatic pruning and no pane-count caps;
  pane identity (location,
  selection,
  scroll position) costs bytes,
  and web browsers demonstrate thousands of tabs are fine.
- Off-screen panes may evict content freely:
  `DirectorySnapshot`s and decoded preview bitmaps drop and re-materialize on scroll-back;
  identity never drops.
- Bulk-close gestures (close column,
  close everything right of here) are required early,
  because spawn-on-descent accumulates panes as a side effect of browsing.

Session rules:

- The strip restores by default on launch:
  pane locations,
  order,
  columns,
  selection,
  and scroll targets,
  in a versioned on-disk format.
- Restore cost must scale with the viewport,
  not the strip:
  snapshots and previews load lazily through the same bounded-window mechanism scrolling uses.
- Missing or unreadable directories restore as error panes in place.

Instance rules:

- Single instance.
  OS open requests route to the running instance over platform IPC
  (Unix socket or D-Bus on Linux,
  named pipe on Windows,
  Apple Events on macOS)
  and raise the window.

## Chosen architecture

Use Slint plus Rust.
The stack is not a vague "core" plus UI split.
It is a set of responsibility boundaries:

- **Slint UI layer**:
  windows,
  pane strip visuals,
  file-list row delegates,
  settings screens,
  dialogs,
  and custom widgets.
- **Rust application state layer**:
  pane graph,
  active selection,
  navigation history,
  view options,
  command routing,
  and undoable file-operation state.
- **Rust filesystem runtime layer**:
  directory reads,
  metadata enrichment,
  file watching,
  search (delegated to `fff-core`),
  previews,
  and cancellable background work.
- **Rust platform integration layer**:
  default-folder-viewer registration,
  shell open paths,
  single-instance IPC,
  platform-native icon pipelines,
  notifications,
  system menus,
  package hooks,
  and OS-specific drag and drop adapters.
- **Packaging and recovery layer**:
  installer metadata,
  signing,
  user-visible repair commands,
  uninstall cleanup,
  and restore-to-system-default behavior.
  Designed as an interface in v1;
  only the personal-machine path is implemented.

The package lives at `package/file-manager/app/`,
function-named and stack-agnostic like its sibling `package/terminal/app`,
because bundle identifiers and registry restore values baked in by the native spikes must survive a stack fallback.
A product display name can be layered on later without moving anything.

Concurrency:
tokio and rayon are both trusted and may be used freely wherever they simplify our own code,
for example rayon for parallel directory walks
and tokio for watcher streams,
cancellation,
and the single-instance IPC server.
Owned-code simplicity outranks runtime minimalism.

File watching uses the same `notify` major that `fff` pins (9.x),
so the dependency tree never carries two incompatible notify versions.
Trash operations go through the `trash` crate,
verified per OS at the user boundary.

## Slint usage rules

Use `.slint` files compiled from Rust with `slint-build` and `slint::include_modules!()`.
Avoid inline `slint!` for production UI because file-backed `.slint` sources have better editor,
LSP,
and build-script behavior.

Use Slint `ListView` for vertical file lists when possible.
Do not rely on `ScrollView` for long file lists,
because `ListView` is the widget with documented visible-item instantiation.

Do not assume Slint provides column-strip virtualization.
The strip needs bounded instantiation at three nesting levels:
columns virtualize horizontally,
panes within a column virtualize vertically,
and each pane's `ListView` virtualizes its rows.
Rust state exposes only a bounded window of columns and panes to Slint.
The UI may render:

- visible columns and panes,
- one prefetch column before the viewport,
- one prefetch column after the viewport,
- lightweight placeholders for transition edges when needed.

The exact budget is an implementation detail,
but it must be measured with instrumentation before the stack choice is considered validated.

The backend is a cargo feature switch during spikes:
winit is the lean default (in-repo prior art,
AccessKit),
and the Qt backend is measured alongside it in the drag and drop spike
because it is the only backend with `start_drag` in the audited source.

Avoid `StandardListView` for the main file list until issue `#12354` is understood.
A custom `ListView` delegate gives more control over pointer events,
context menus,
selection visuals,
and drag handles.

## Domain model

The application state layer should use explicit concepts instead of a single global app state object:

- `PaneId`:
  stable identity for one pane instance;
  survives deliberate duplicates,
  so it is never merely a location.
- `PaneLocation`:
  directory path,
  file preview,
  virtual search result,
  recent files view,
  or future non-directory source.
  Locations are the dedup lookup key.
- `DirectorySnapshot`:
  immutable view of entries at one read generation;
  evictable and re-readable.
- `FileEntry`:
  name,
  path,
  kind,
  metadata summary,
  icon key,
  preview status,
  and capability flags.
  The icon key indirection is what lets icon pipelines change without touching the model.
- `SelectionState`:
  anchor,
  focused row,
  selected entry set,
  and range-selection mode.
- `PaneStripState`:
  ordered columns,
  panes per column,
  active pane id,
  scroll target,
  and visible window;
  serialized in a versioned format for session restore.
- `FileOperation`:
  copy,
  move,
  rename,
  delete,
  trash,
  restore,
  and conflict resolution,
  each recorded with enough state to derive its inverse.
- `UndoStack`:
  inverse-operation undo and redo at Dolphin parity:
  rename undoes by renaming back,
  move by moving back,
  trash by restore,
  copy by deleting the copy behind a confirmation.
  Overwrites are never undoable in v1;
  they always require explicit confirmation,
  and the UI never claims recoverability it does not have.
  Inverse-operation failure (target changed since) surfaces as an honest error state.
- `PlatformRegistrationState`:
  current default-manager registration status,
  saved prior values,
  verification status,
  and available revert actions.

Snapshots should be immutable.
Filesystem updates create new snapshots and publish model diffs to Slint.
This keeps file watching,
sorting,
filtering,
and view refreshes deterministic.

## Search

Search ships in v1 and delegates its logic to `fff-core`:

- search panes (fuzzy file search as a `PaneLocation`),
- recent-files panes (candidate source:
   fff's frecency data;
  confirm during implementation),
- in-pane type-ahead filtering (fff's matcher over the current snapshot).

Integration risks to track:

- `fff` is pre-1.0 (0.9.x) and fast-moving;
  a version bump can shift APIs under a v1 feature.
- `fff` maintains its own watches and LMDB index;
  our runtime layer and fff will both watch overlapping trees,
  and index placement must follow per-user data directory conventions.

## Icons and previews

v1 renders platform-native icons plus image thumbnails:

- icons come from each OS
  (XDG icon theme on Linux,
  `SHGetFileInfo` on Windows,
  `NSWorkspace` on macOS),
  behind the `FileEntry` icon-key indirection,
  with caching and DPI variants per platform;
- image thumbnails decode in-process with memory-safe Rust (`image` crate),
  reusing the freedesktop thumbnail cache on Linux where present;
- previewers never execute files;
- richer previews (PDF,
  video) are deferred.

## Native integration scope

Registration is symmetric across Windows,
macOS,
and Linux in v1,
including Win+E interception on Windows
(a separate registration with its own saved restore value and revert,
following the OneCommander precedent).

Do not describe the feature as "replace Finder" or "replace Explorer" without subcases.
The native integration layer must track observable launch paths separately:

- opening a folder from another app,
- revealing a file or folder from another app,
- opening drives,
- desktop interactions,
- file open and save dialogs,
- keyboard shortcuts such as Win+E,
- direct Explorer or Finder launches from shell UI,
- uninstall and revert behavior.

Windows work needs native code for Directory and Drive shell commands,
Win+E handling,
registry value backup,
argument quoting,
installer integration,
and a safe revert path.

macOS work needs native code for `NSFileViewer`,
LaunchServices handlers for `public.folder`,
bundle identifiers,
restart messaging,
Full Disk Access guidance,
verification,
and Finder restore commands.

Linux work needs XDG MIME defaults,
`mimeapps.list` behavior,
desktop-file installation,
portal behavior where relevant,
and desktop-environment compatibility checks.

Every registration prototype must run against disposable state or saved restore values,
not against irreversible user defaults.

## Spike gates

Implementation should not start by building the full file manager.
Run these spikes first and record results in this document or a follow-up planning note.

Sequencing:
one shared in-repo Slint prototype hosts the virtualization spike first
(identity risk resolves before anything else),
with the context-menu spike folded into the same app,
then the drag and drop spike extends it with cargo-feature backend switching.
The native default-manager probes are independent of Slint
and run on `m1` and `x13-win` whenever convenient.
Spike crates live in-repo,
following the `terminal/app` prototype precedent,
so results stay reproducible.

### Column-strip virtualization spike

Build a Slint plus Rust prototype with the two-dimensional column strip:
columns virtualize horizontally,
panes within a column virtualize vertically,
each visible pane contains a `ListView` with a large model,
and preview panes hold decoded images.
Instrument the number of instantiated columns,
panes,
and row delegates,
plus decoded-image memory.

Pass criteria:

- off-screen columns and panes are not fully instantiated,
- vertical row delegates remain bounded by viewport size,
- memory stays bounded while jumping across far-apart strip positions,
- browsing many image files mints many preview panes while decoded-image memory stays bounded by the viewport,
  and evicted previews re-decode on scroll-back,
- keyboard focus survives pane recycling,
- prototype runs on Linux,
  macOS,
  and Windows.

Failure action:
compare Qt and GPUI before continuing with Slint.

Result (2026-07-05):
PASSED on Linux.

- Prototype path:
  `package/file-manager/app/`
  (function-named package,
  built as the plan's designated home rather than a throwaway crate).
- Operating systems tested:
  Linux only (Wayland,
   winit backend,
   femtovg renderer);
  macOS `m1` and Windows `x13-win` still pending.
- Steps run:
  a synthetic strip of 1200 columns,
  about 14400 panes,
  and about 121 million addressable rows was driven headless through the
  embedded Slint MCP server (release build) and live on the host GPU.
  Horizontal and vertical scrolling,
  far jumps across the strip,
  keyboard navigation,
  and column close were exercised while reading the instrumentation HUD.
- Observed numbers:
  columns instantiated 5 to 7 of 1200;
  panes instantiated 17 to 28 of about 14400;
  distinct rows materialized under 2000 of about 121 million;
  decoded image memory 1.5 to 4.2 MiB resident,
  bounded by the viewport,
  with decode count rising on scroll-back (re-decode confirmed);
  keyboard focus on the active pane survives pane recycling.
- Smoothness:
  preview decode was moved off the UI thread to a background worker during the spike,
  so publish is now pure windowing at 8 to 16 microseconds per scroll step,
  constant regardless of the 14400-pane strip size,
  with no decode time in the publish path and no dropped frame on a hard jump.
  An earlier synchronous-decode version peaked at one 14.2 ms frame on a hard jump;
  backgrounding the decode removed it.
- Vertical scrolling:
  resolved as a product decision during the spike to move every column at once
  through one shared vertical offset (the tallest column sets the range),
  rather than scrolling only the focused column.
- Horizontal scroll smoothness:
  the first cut rebuilt the whole columns model on every scroll event,
   which
  stuttered on slow scroll,
   showed stale frames on fast scroll,
   and (once the
  rebuild moved to a frame timer) capped a mousewheel gesture at the first column
  because replacing the model mid-scroll fought the Flickable's own viewport-x.
  The fix is a persistent Slint `VecModel` mutated incrementally through
  `Repeater`/`ModelNotify`:
  a horizontal scroll slides only the delta columns in and out (insert/remove),
  vertical scroll and active changes rewrite in-window rows in place
  (`set_row_data`),
  and a landed decode refreshes only its owning column.
  The model is set on the window once and never replaced,
  so the Flickable's scroll is never disturbed.
  Column build churn dropped from thousands of rebuilds per session to about a
  dozen,
   and the author confirmed horizontal scrolling is smooth and holds far
  positions.
  The Slint source trace and the rejected approaches are recorded in
  `doc/troubleshooting/slint-flickable-windowed-model-scroll.md`.
- Chosen action:
  continue with Slint;
  no fallback triggered.
  The background-decode worker built here is the spike-scale seed of the
  file-watching-and-async milestone's cancellable preview jobs,
  and the incremental-model approach is the pattern the production strip should use.
- Not covered by this spike:
  the context-menu spike was not folded in yet,
  and the drag-and-drop and native default-manager spikes are still pending.

### File drag and drop spike

Prototype file drag and drop in both directions,
on both the winit and Qt backends via a cargo feature switch,
on all three operating systems:

- drag files from the OS file manager into the Slint app,
- drag files from the Slint app into the OS file manager,
- drag entries between panes inside the app.

Pass criteria:

- external drops expose file paths or file URLs to Rust,
- outbound drags advertise OS-native file-list payloads,
- internal drags carry structured app-local data,
- copy and move actions are distinguishable.

The backend comparison is part of the spike record:
if only the Qt backend passes,
that result feeds the fallback decision as evidence,
not speculation.

Failure action:
either design a native adapter around Slint pointer events
(hand-written per OS;
`drag-rs` is reference material only),
contribute upstream Slint file-list MIME support,
or compare Qt before committing to Slint.

#### Result: internal drag passes, OS file drag-and-drop unavailable (2026-07-05)

Folded into the same prototype (`package/file-manager/app/`).
Full source trace,
 verification,
 and the upstream do-not-file decision are in
`doc/troubleshooting/slint-drag-and-drop-file-lists.md`.

- Prototype path:
  `package/file-manager/app/`
  (`src/drag_drop.rs`,
  `ui/app.slint` `DragArea`/`DropArea`,
  `src/drag_drop_tests.rs`).
- Operating systems tested:
  Linux only (winit backend,
   headless MCP);
  macOS `m1` and Windows `x13-win` still pending,
   riding along with the
  virtualization spike's cross-platform pass.
- Steps run:
  a row is dragged to a different pane through the embedded Slint MCP server's
  `drag_element`,
   and the recorded identity plus action are read back on the HUD
  (`AppWindow::hud-e`) and the log.

Pass criteria,
 split:

- internal drags carry structured app-local data:
  PASSED.
  A dragged row packs its `(pane, row)` identity into the transfer's `user_data`;
  the target pane reads it back on drop.
  A move drag logs
  `source_pane_id=0 source_row=0 target_pane_id=1 action="move"`;
  a left-click still selects without dropping,
   so the drag wrap leaves selection
  intact.
- copy and move actions are distinguishable:
  PASSED.
  A drag-mode toggle (standing in for the copy modifier) flips the drop to
  `action="copy"`.
- external drops expose file paths or file URLs to Rust:
  FAILED (blocked by Slint).
  Stock Slint 1.17.0 wires no external file-drop delivery on any backend.
- outbound drags advertise OS-native file-list payloads:
  FAILED (blocked by Slint).
  Slint 1.17.0 has no native drag path at all;
  the post-1.17.0 Qt `start_drag` advertises only text and image,
   never a
  file-list.

Backend comparison (the spike record):
the winit-vs-Qt choice does NOT resolve file drag-and-drop.
Released 1.17.0 core invokes no backend native-drag hook,
 so drag is in-process
on both backends.
Even in post-1.17.0 code,
 the Qt backend's `start_drag` and drop bridge carry
only text and image (never `setUrls`/`hasUrls`),
 and the winit backend has no
native drag-and-drop at all.
So switching to Qt buys no file drag-and-drop;
this is evidence from the audited Slint source,
 not speculation.
The Qt backend was not built here (Slint's Qt backend needs `qmake`,
 which is
absent;
Qt runtime libs are present but the dev tooling is not),
 and building it would
not change the conclusion because Qt carries no file-list.

Chosen action:
continue with Slint;
no fallback triggered.
The internal `DragArea`/`DropArea` drag is the seed for the plan's
"internal pane-to-pane drag first" milestone.
OS inbound and outbound file drag-and-drop take the plan's failure action,
 a
hand-written per-OS native adapter (winit `DroppedFile`/`HoveredFile` inbound;
`QMimeData::setUrls` / `NSPasteboard` / `IDataObject` / XDND `text/uri-list`
outbound),
 deferred to the native-integration milestone at the consumer boundary.
Upstream issue `#1967` already documents both gaps in the maintainers' own words,
so nothing was filed.

### Row context menu spike

Reproduce Slint issue `#12354` with the intended custom row delegate.
Test right-click,
keyboard menu key,
Shift+F10,
and long-press where the platform supports it.

Pass criteria:

- row context menu opens for mouse and keyboard paths,
- focused row and clicked row are deterministic,
- menu commands receive the correct `FileEntry` identity,
- workaround does not break row selection or drag initiation.

Failure action:
avoid affected Slint widgets or patch Slint before building production file-list interactions.

#### Result: passed on Linux (2026-07-05)

Folded into the same prototype (`package/file-manager/app/`) on the
custom `ListView` row delegate,
 not `StandardTableView`.
Issue `#12354` reproduces exactly as filed:
the row's own `TouchArea` grabs the right-press,
 so the wrapping
`ContextMenuArea` never opens on a row.
Source-traced into Slint `1.17.0` and recorded with the workaround in
`doc/troubleshooting/slint-contextmenuarea-listview-rows.md`.

The verified workaround is to forward the right-press from the row's
`pointer-event` to `context-menu.show(...)`,
 translating the click into the
area's coordinate space with the same `absolute-position` maths Slint's own
`listview.slint` uses.
Rust records the (pane,
 row) the click targeted so a chosen command carries a
deterministic identity,
 mirrored to the HUD and logged.

Pass criteria,
 each met and driven headless through the embedded MCP server:

- opens for mouse and keyboard paths:
 a right-click on a row opens the menu (screenshot),
   and both the keyboard menu
 key (`Key.Menu`) and the MCP-drivable menu button open it;
 the built-in only wires `Shift+F10` on Windows,
   so the prototype adds it in the
 pane `FocusScope` for cross-platform parity,
- focused and clicked row deterministic:
 the right-clicked row highlights as the target,
   matching the menu position,
- commands receive the correct identity:
 activating `Open`/`Rename`/`Delete` logs and shows the exact (pane,
   row) the
 click targeted (for example right-clicking `dir #20` row 0 logs
 `pane_id=20 row=0`);
 the keyboard path targets the active pane's active row,
- selection and drag intact:
 a left-click still selects the row (and records it) without opening the menu,
 and the right-button branch is additive,
   so left-button press/drag is unchanged.

Long-press is Android-only in Slint's `ContextMenu` item,
 so it is not a desktop
path here.
The spike does not yet re-run on macOS or Windows;
that rides along with the virtualization spike's cross-platform pass.

### Native default-manager spike

Prototype registration and revert behavior on each operating system.
Use the launch-path list in this plan as the test matrix.
On Windows the matrix includes Win+E.

Pass criteria:

- registration changes are visible to the OS,
- revert restores previous values,
- uninstall behavior is specified,
- user-visible warnings match the real platform limits,
- verification opens folders through the OS launch path,
  not only through direct process invocation.

Failure action:
ship the file manager without default-manager claims,
or scope default-manager support to the operating systems whose prototypes pass.

## Implementation milestones

Keyboard interaction is core in every milestone,
not a final milestone:
each feature ships keyboard-operable,
because a file manager is keyboard-primary software.

### Foundation

Create the Rust package at `package/file-manager/app/`,
Slint build pipeline,
logging,
settings storage,
single-instance IPC,
and a single empty window.
Run the app on Linux,
macOS,
and Windows at the user boundary.

### Directory listing

Implement directory reads,
metadata loading,
error states,
sort order,
in-pane filtering,
and refresh.
Render one pane with a custom Slint `ListView` delegate,
keyboard-navigable.

### Pane strip

Implement `PaneStripState`,
column layout,
spawn and dedup rules,
pane recycling,
active-pane focus,
horizontal and vertical navigation,
bulk-close gestures,
and session restore.
Keep the virtualization instrumentation in the codebase until the behavior is stable.

### Icons and previews

Implement platform-native icon pipelines per OS behind the icon-key indirection,
image thumbnail decoding,
and preview panes with the pane-per-file lifecycle and eviction.

### Selection and commands

Implement focused row,
range selection,
multi-selection,
open,
reveal,
copy path,
rename,
trash,
delete,
copy,
move,
and the inverse-operation undo stack.
Destructive operations must use reversible fixtures in tests and clear confirmation paths in the UI.
Overwrites always confirm and never claim undoability.

### File watching and async work

Add filesystem watchers,
cancellable directory refreshes,
preview jobs,
and stale-result suppression.
A slow directory read must not block pointer or keyboard interaction.

### Search

Integrate `fff-core`:
search panes,
recent-files panes,
and index lifecycle,
with cancellation and streaming results into the pane model.

### Drag and drop

Implement internal pane-to-pane drag first.
Add OS inbound and outbound file drag only after the file drag and drop spike passes.

### Native shell integration

Implement the platform adapters independently.
Each adapter needs query,
register,
verify,
revert,
and uninstall hooks.
Do not share behavior through stringly commands when a typed adapter can expose platform-specific state.

### Packaging

Add platform packages only after the app launches and reads directories on each target.
v1 implements only the personal-machine path;
signing,
notarization,
installer or MSIX decisions,
Linux package formats,
and update strategy stay designed-but-deferred.

### Assistive technology

Add accessible roles,
labels,
and screen-reader checks for panes,
rows,
menus,
and dialogs.
Keyboard coverage already shipped inside each milestone;
this milestone covers the assistive-technology semantics on top of it.

## Testing and verification

Use unit tests for pure Rust state transitions:
pane graph,
spawn and dedup rules,
selection,
sorting,
undo stack,
operation planning,
and platform-registration state machines.

Use integration tests for filesystem behavior against throwaway directories:
watch events,
rename conflicts,
copy and move plans,
trash fallbacks,
permission errors,
and symlink handling.

Use UI boundary tests for Slint behavior:
window launch,
row rendering,
keyboard navigation,
context menus,
pane scrolling,
and drag initiation.

Use platform boundary tests for native integration:
folders opened from another app,
folder reveal behavior,
default-manager verification,
and revert behavior.

Do not count compilation alone as verification.
The app must be launched and exercised as an end user would use it on every target operating system.

## Security posture

No renderer process should receive ambient filesystem authority through a WebView IPC bridge.
That is why Electron and Tauri are not the chosen stack.

The Rust application must still treat filesystem operations as privileged:

- previewers must not execute files,
- image decoding stays in memory-safe Rust,
- paths crossing syntax boundaries must be encoded for the destination grammar,
- delete and overwrite operations need explicit command states,
- background tasks must be cancellable,
- native registration changes must preserve restore data,
- logs must avoid leaking file contents unless the user explicitly requests diagnostic detail.

## Fallback conditions

Reopen the stack decision if any of these happen:

- Slint cannot support bounded column-strip virtualization without rewriting major runtime pieces.
- Slint cannot support native file-list drag and drop through a maintainable adapter or upstream patch.
- Row context menus cannot be made reliable for the main file list.
- Accessibility requirements require platform hooks Slint cannot expose.
- Native default-manager integration dominates the project enough
  that Qt's mature platform layer outweighs Slint's smaller stack.
- `fff` churn under the v1 search feature becomes unsustainable;
  the fallback there is pinning plus vendoring,
  not a stack change.

The first fallback candidate is Qt.
GPUI is the second fallback if custom pane rendering becomes more important than mature widgets.

## Documentation updates required after spikes

When a spike passes or fails,
update this planning document with:

- exact prototype path,
- operating systems tested,
- command or UI steps run,
- observed failures,
- chosen workaround or stack-change trigger.

If the stack changes,
update `doc/decision/slint-file-manager-stack.md` in the same commit.
