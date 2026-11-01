# Building the real GTK4 file manager (Niri column-strip UI) — handover

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/nested-wayland-session` is now `package/nested-wayland-session/cli`;
> `package/desktop-app/file-manager` is now `package/file-manager/app`.

Live cross-session state for the implementation of the real file manager on GTK4,
 following the
approved plan.
 Kept updated as the build proceeds.
 The toolkit exploration that chose GTK4 is a
separate,
 finished doc (`file-manager-toolkit-exploration.md`);
 this one tracks the product build.

## Authoritative sources (do not re-derive these)

- Product UX + domain model + milestones:
   `doc/planning/file-manager.md` (product decisions
  resolved 2026-07-05).
   Everything there stands EXCEPT the stack.
- Stack decision (supersedes the planning doc's Slint sections):
   GTK4 (gtk4-rs).
   Evidence in
  `doc/handover/file-manager-toolkit-exploration.md`.
- Perf architecture:
   fixed-position panes on a canvas,
   GPU render-node cache,
   off-thread decode.
  Same handover's "Windows results";
   tile principle by analogy from `doc/decision/vector-design.md`
  (which is a DIFFERENT project's doc;
   only the tile/worker/eviction philosophy is borrowed).
- DnD reference implementations (verified):
   `doc/troubleshooting/gtk4-windows-outbound-file-drag.md`
  (Win32 OLE shim),
   `doc/troubleshooting/gtk4-macos-file-dnd.md` (objc2 AppKit shim + inbound
  macOS URI-unescape),
   `doc/troubleshooting/gtk4-windows-gvsbuild-directcomposition.md`
  (`GDK_DEBUG=dcomp` needed in-process for GPU rendering on Windows).

## Locked decisions

- Package:
   the GTK4 app reclaims the canonical,
   function-named `package/desktop-app/file-manager/`
  (planning doc mandates it so bundle IDs survive a stack fallback).
   The old Slint sources and their
  fork `[patch.crates-io]` are deleted outright (task #30);
   nothing salvaged from them.
   The
  `file-manager-gtk` spike package is retired into this one.
   `file-manager-qt` is kept as evidence.
- Package identity:
   crate `file-manager`,
   lib `file_manager`,
   bin `monochromatic-file-manager`
  (lib+bin split so the domain model unit-tests without GTK).
- Scope of this pass:
   the interactive shell (Foundation -> directory listing -> pane strip with
  spawn/dedup/focus + keyboard -> off-thread thumbnails + eviction -> DnD both directions).
  Deferred:
   session-restore persistence,
   single-instance IPC transport,
   fff-core search,
   the full
  FileOperation/UndoStack,
   native default-manager registration,
   packaging,
   and the AT milestone.

## Interaction model being implemented (from the planning doc)

Niri-like infinite horizontal strip of columns;
 each column stacks panes vertically.
 Horizontal =
lineage depth,
 vertical = panes spawned from the previous column.
 Single-click spawns a child in the
next column and auto-focuses it (directory -> listing pane;
 file -> select + preview pane;
Enter/double-click -> open with OS default).
 Dedup-first (clicking an entry whose pane exists focuses
it);
 Ctrl+click forces a duplicate;
 keyboard selection follows the same rules.
 Panes die only on
explicit close;
 off-screen panes evict content (snapshots,
 decoded bitmaps) but never identity;
bulk-close gestures early.
 Keyboard-primary throughout.

## Module map (as built)

`package/desktop-app/file-manager/src/`:
 `main.rs` (thin bin) + `lib.rs` (bootstrap,
 tracing,
`GDK_DEBUG=dcomp` on Windows,
 module wiring,
 controllers kept alive for the app lifetime);
`constants.rs`,
 `types.rs` (domain data),
 `model.rs` (`PaneStripState` spawn/dedup/close state
machine),
 `fs.rs` (reads/metadata/sort);
 `window.rs` (top-level window + inbound drop target);
`strip.rs` (fixed-canvas board,
 `reconcile`,
 spawn/close glue,
 scroll-to-reveal,
 Left/Right column
keyboard nav);
 `pane.rs` (listing pane with single-click-activate + Ctrl-duplicate tracking + close
button,
 plus the preview-pane builders);
 `thumbs.rs` (off-thread decode + byte-bounded evicting
texture cache);
 `dnd.rs` (inbound drop target + cfg-branched outbound drag);
 `win_drag.rs` (cfg
windows,
 OLE shim),
 `mac_drag.rs` (cfg macos,
 AppKit shim).
 Tests:
 `fs_tests.rs`,
 `model_tests.rs`
(linter-exempt).
 Spawn/dedup/focus and keyboard nav were folded into `strip.rs`/`pane.rs` rather than
separate `spawn.rs`/`keys.rs`.

## How to build / run

`mise run //package/desktop-app/file-manager:run` (debug build,
 runs on native Wayland via
`GDK_BACKEND=wayland`).
 Tests:
 `//package/desktop-app/file-manager:test`.
 Rust linter (max-lines +
require-rustdoc):
 `//package/desktop-app/file-manager:lint:rust`.
 Types/clippy:
`//package/desktop-app/file-manager:lint:clippy`.

## Progress log

- 2026-07-08:
   Plan approved (`/home/user/.claude/plans/lively-zooming-pillow.md`).
   Handover started.
  Verified the two target packages are standalone crates with no workspace/package.json references
  (only doc-comments in `cli/nested-wayland-session`),
   so the reclaim needs no registration changes.
- 2026-07-08:
   Checkpoint 1 (reclaim + Foundation) DONE.
   Cleared the Slint deck (#30):
   deleted every
  old Slint source + the fork `[patch.crates-io]`;
   retired the `file-manager-gtk` spike.
   Scaffolded
  the GTK4 package at the canonical path (crate `file-manager`,
   bin `monochromatic-file-manager`,
  lib+bin split):
   `Cargo.toml` (gtk4 0.11 `v4_10` + tracing trio),
   `mise.toml` (build/lint/lint:rust/
  test/run,
   `GDK_BACKEND=wayland`),
   `README.md`.
   `src/`:
   `lib.rs` (`run`:
   non-blocking tracing +
  `Application` + `GDK_DEBUG=dcomp` on Windows before GDK init + `FM_QUIT_MS` self-quit),
   `window.rs`
  (`build_window`),
   `constants.rs`,
   `main.rs` (thin bin).
   Calibration:
   the repo rust linter flags
  every `use` for missing rustdoc (the retired spike never passed it) — code here documents each
  `use`.
   Verified:
   debug build clean (30s);
   `lint:rust` and `lint:clippy` both rc=0;
   ran the binary
  on native Wayland (`WAYLAND_DISPLAY=wayland-0`) -> `presented top-level window`,
   self-quit,
   exit 0.
- 2026-07-08:
   Checkpoint 2 (domain model + fs reads,
   #42) DONE.
   Pure,
   GTK-free,
   unit-tested.
  `types.rs` (`PaneId`,
   `EntryKind`,
   `PaneLocation`,
   `FileEntry`,
   `DirectorySnapshot`),
   `fs.rs`
  (`read_directory` -> sorted `DirectorySnapshot`;
   dirs-first case-insensitive sort;
   best-effort
  per-entry metadata;
   a bad single entry is skipped with a warning,
   only a failed dir-open
  propagates),
   `model.rs` (`PaneStripState`:
   `open_root`/`spawn_child` with dedup + force-duplicate,
  `focus`,
   `close`/`close_column`/`close_right_of`,
   `columns`/`active` accessors).
   Modules are `pub`
  so they unit-test without GTK and avoid `dead_code` on fields not yet rendered.
   11 tests
  (`fs_tests.rs`,
   `model_tests.rs`,
   both linter-exempt) cover sort,
   read+metadata,
   missing-dir error,
  dedup-and-focus,
   Ctrl-duplicate,
   close-clears-dedup,
   bulk close-column/close-right-of.
   lint:rust +
  lint:clippy rc=0 (calibration:
   the linter also requires rustdoc on tuple fields;
   clippy prefers a
  let-chain over a nested `if`).
   Next:
   directory-listing pane rendering a real directory (#43).
- 2026-07-08:
   Checkpoint 3 (directory-listing pane,
   #43) DONE.
   `pane.rs`:
   `build_listing_pane` builds
  a virtualized `GtkListView` over a `DirectorySnapshot` (`gio::ListStore` of `BoxedAnyObject`-wrapped
  `FileEntry`;
   `SignalListItemFactory` setup/bind;
   each row = themed icon by kind + name label) under
  a header showing the directory path.
   `window.rs` now reads `$HOME` (fallback cwd) via
  `fs::read_directory` and shows the listing pane,
   or an honest error label on failure.
   Verified:
   it
  listed real `$HOME` with `entries=108` (matches `ls -1A | wc -l`),
   presented the window,
   zero GTK
  errors,
   clean exit.
   A full-screen screenshot was intentionally skipped (it would capture personal
  desktop content,
   and single-window capture is not reliably targetable here);
   verification is the
  entry-count match plus the clean render log.
   Next:
   fixed-canvas pane strip + spawn/dedup/focus +
  keyboard (#44).
- 2026-07-08:
   Checkpoint 4a (fixed-canvas strip + spawn/dedup + close,
   part of #44) DONE.
  `constants.rs` adds `PANE_WIDTH/HEIGHT/GAP` + `FM_AUTOSPAWN`.
   `strip.rs`:
   `StripController` over a
  `GtkFixed` inside a `ScrolledWindow`;
   panes at `(column*(W+GAP), slot*(H+GAP))`,
   empty cells = no
  widget;
   `reconcile()` = `remove_stale` + `ensure_pane_widget` (move existing / build+put new,
   sized
  to the fixed pane box);
   `spawn_from` maps an activated `FileEntry` to Directory/Preview,
   calls
  `model.spawn_child` (dedup + column+1),
   reconciles,
   idle-defers a scroll-to-reveal,
   returns the new
  id;
   `close_pane` closes + reconciles.
   `pane.rs`:
   `single_click_activate` + `connect_activate` ->
  `on_activate`;
   header = ellipsized path label + a `window-close-symbolic` button -> `on_close`.
  `window.rs` returns the controller;
   `lib.rs` holds controllers for the app lifetime (pane closures
  hold only a `Weak`,
   so no reference cycle).
   Verified via `FM_AUTOSPAWN`:
   spawn -> `panes=2
  columns=2`,
   then close -> `panes=1`,
   presented,
   clean exit.
   Remaining for #44:
   Ctrl+click
  force-duplicate and Left/Right keyboard column focus.
- 2026-07-08:
   Checkpoint 4b (#44 DONE):
   Ctrl force-duplicate + keyboard column navigation.
   `pane.rs`:
  `install_force_duplicate_tracking` wires a capture-phase `GestureClick` + an `EventControllerKey`
  feeding a shared `Cell<bool>`;
   `connect_activate` reads-and-clears it and passes `force_dup` to
  `on_activate` (now `Fn(&FileEntry, bool)`);
   arrow keys only move selection,
   so browsing never spawns.
  `strip.rs`:
   `spawn_from` takes `force_duplicate` -> `model.spawn_child`;
   tracks `focused_column`;
  `install_column_nav` adds a capture-phase key controller on the scroller mapping Left/Right to
  `focus_relative_column` (grab_focus the adjacent column's first pane),
   leaving Up/Down to the
  focused list;
   a spawn grabs focus to the new pane in `scroll_to_pane`'s idle callback.
  Build/lint/test green;
   autospawn spawn+close unchanged (`panes 2 -> 1`),
   clean exit.
   The modifier
  and arrow paths compile and lint;
   their full boundary check (real Ctrl+click / Left-Right) is manual
  (headless input can't target the window precisely),
   but the spawn/dedup/force-duplicate/close LOGIC
  is unit-tested.
   Next:
   off-thread thumbnails + evicting cache + preview panes (#45).
- 2026-07-08:
   Checkpoint 5 (off-thread thumbnails + evicting cache + preview panes,
   #45) DONE.
   Deps:
  `image` 0.25 (decode) + `async-channel` 2 (worker->main delivery,
   since glib removed
  `MainContext::channel`).
   `thumbs.rs`:
   `Thumbnails::start` spawns a decode worker thread
  (`image::open` -> `thumbnail(THUMB_SIZE)` -> RGBA) that delivers `(path, Option<Decoded>)` over an
  async channel drained by `glib::spawn_future_local` on the main context;
   `deliver` builds a
  `MemoryTexture`,
   inserts into a byte-bounded LRU (`evict_to_budget` keeps the total under
  `THUMB_CACHE_BYTES` = 64 MB),
   and fulfils every waiting `Picture`;
   `request` deduplicates concurrent
  requests per path and serves cache hits immediately (`touch` bumps the LRU tick).
   `is_image` gates
  by extension.
   `pane.rs`:
   `build_preview_pane` (header + close) over `build_preview_body` (image ->
  off-thread `Picture`,
   else a large themed icon + name);
   the preview builders live here to keep
  `strip.rs` under max-lines (split when it hit 312).
   `strip.rs`'s Preview arm calls it with a close
  closure;
   `StripInner` owns the `Thumbnails`.
   Verified via `FM_START_DIR=/usr/share/pixmaps
  FM_AUTOPREVIEW`:
   preview spawned (`panes=2`),
   `thumbnail ready path=.../cupsprinter.png width=256
  height=256 cache_bytes=262144` (= 256*256*4),
   clean exit -> decode ran off-thread,
   texture built +
  cached with exact byte accounting,
   `Picture` updated.
   Full browse-many-images eviction is trusted
  from the LRU logic (only one thumbnail here,
   well under the cap).
   Next:
   DnD wiring (#46).
- 2026-07-08:
   Checkpoint 6 (DnD wiring,
   #46) DONE.
   `Cargo.toml`:
   target-gated `windows` 0.58 + `objc2`
  0.6 shim deps (compiled only on their OS).
   `dnd.rs`:
   inbound `install_drop_target` attaches a
  `GtkDropTarget` (`GdkFileList`,
   copy) to the window,
   logging each dropped file's recovered path +
  uri (`recover_path` falls back to `glib::Uri::unescape_string` for the macOS scheme-colon encoding).
  Outbound `install_file_drag` is cfg-branched:
   Wayland -> `GtkDragSource` with a `text/uri-list`
  content provider;
   windows -> `GtkGestureDrag` -> `win_drag::start_file_drag` (OLE shim);
   macos ->
  `GtkGestureDrag` -> `mac_drag::start_file_drag` (AppKit shim).
   `win_drag.rs` + `mac_drag.rs` ported
  verbatim from the verified troubleshooting-doc/scratchpad references,
   with rustdoc added on their
  `use`s/`impl` so the linter (which parses all `.rs` regardless of cfg) passes.
   Wired:
   the window
  gets the drop target;
   a preview pane's body (Picture / icon box) gets the outbound drag.
  Build/lint:rust (checks the win/mac files too)/clippy/test all green;
   ran clean with DnD installed
  (no errors,
   clean exit).
   Live drag verification (real Dolphin <-> app) is MANUAL:
   headless cannot
  synthesize a Wayland drag gesture,
   and these are the exact `GtkDropTarget`/`GtkDragSource` APIs the
  spike already verified with real Dolphin.
   Outbound is currently from preview panes (single-file,
   no
  click/virtualization conflict);
   dragging listing ROWS out is a documented refinement (needs
  drag-vs-activate arbitration + per-row path tracking).
   macOS/Windows shim at-boundary verification
  rides the m1/x13-win pass.
   Next:
   #47 final at-boundary verification + README/package completeness.
- 2026-07-08:
   Checkpoint 7 (#47,
   first-pass verification + completeness) DONE.
   The interactive shell
  is complete on Linux.
   Final green sweep:
   `lint`,
   `lint:rust`,
   `lint:clippy`,
   `test` (11) all rc=0;
  15 source files.
   Release artifact (`build`) verified at the boundary:
   (A) real `$HOME` autospawn ->
  `spawned panes=2 columns=2` then `closed panes=1`,
   presented,
   exit 0;
   (B) `/usr/share/pixmaps`
  autopreview -> off-thread `thumbnail ready 256x256 cache_bytes=262144`,
   presented,
   exit 0.
   Package
  completeness (PKG):
   `README.md` present,
   zero lint errors,
   unit tests cover the pure exported paths
  (model spawn/dedup/close,
   fs read+sort);
   GTK-coupled paths (strip reconcile,
   thumbnail decode/cache,
  DnD install) are runtime-verified via the autospawn/autopreview hooks and clean-run logs.

  What is verified automatically vs still manual:
  - Automatic (at boundary,
     this machine):
     build/lint/test;
     native Wayland window;
     real-directory
    listing (108-entry `$HOME` match);
     spawn -> next-column child + dedup+close reconcile;
    off-thread thumbnail decode + cache byte accounting;
     clean teardown (exit 0) on every run.
  - Manual (needs a human / another machine):
     live Ctrl+click duplicate,
     Left/Right column focus,
     and
    real Dolphin <-> app drag-and-drop (headless can't synthesize Wayland pointer/drag input);
     and the
    Windows OLE + macOS AppKit outbound shims (compiled behind cfg + linter-clean here,
     at-boundary
    on the x13-win/m1 pass).

  Deferred (designed,
   not built this pass):
   session-restore persistence,
   single-instance IPC,
  fff-core search,
   the full FileOperation/UndoStack (copy/move/rename/trash/delete + inverse undo),
  native default-manager registration,
   packaging/signing,
   the AT semantics milestone,
   dragging listing
  ROWS out (needs drag-vs-activate arbitration + per-row path tracking;
   outbound is preview-pane-only
  today),
   and drop-INTO-a-directory as a real file operation (inbound currently accepts + logs).
- 2026-07-08:
   Live-test fix (auto-scroll).
   Hands-on testing surfaced that a spawned pane did not
  scroll fully into view,
   and a stacked sibling looked "off by one" (the scroll revealed the pane
  ABOVE the new one).
   Root cause:
   the idle-based reveal retry spun through every attempt before GTK
  ran layout,
   so it clamped against stale scroll bounds.
   Fix (`strip.rs`):
   retry on an 8 ms timer
  (`REVEAL_INTERVAL_MS`) so the frame clock runs layout between attempts and the adjustment `upper`
  is current;
   `reveal` now returns whether the pane is fully visible so the retry stops on success.
  Also extracted the Left/Right column nav to a new `keys.rs` (made `StripInner` `pub(crate)`) to
  keep `strip.rs` under max-lines,
   and added the spawned entry path to the spawn log.
   Confirmed live:
  `revealed=true` on the first timed attempt with `v_value` 0/252/784 against `v_upper` 800/1052/1584
  for slots 0/1/2;
   `entry=` matches the clicked folder,
   so click->spawn had no real off-by-one.
- 2026-07-08:
   Live-test UX pass.
   (1) Row alignment:
   replaced the "append to column" pane model with
  an explicit pane TREE (each pane has a `parent`) laid out by an iterative pre-order tidy layout
  (`model.rs` `relayout`):
   a node's row = the next free leaf-row when the walk enters it,
   so a child
  aligns to its parent's row and a later sibling starts below the previous sibling's whole subtree.
  Every spawn/close re-lays-out and `reconcile` repositions existing panes as subtrees grow.
   Confirmed
  live (the `.dbus` case:
   a sibling pushes below a grown subtree) and by 13 model tests (added
  alignment + sibling-pushdown).
   (2) Black background:
   `style.rs` installs a `#000` `CssProvider` at
  application priority;
   confirmed.
   Open requirement:
   the clicked PARENT must stay fully in view and
  not jump when a child spawns (the user suggests "detach columns" = independent per-column vertical
  scroll);
   clarifying exact behavior before that scroll refactor.
- 2026-07-08:
   Detached-column strip,
   STAGE 1 (structure + independent scroll).
   Confirmed model with
  the user:
   each column scrolls vertically on its own,
   but a parent is tethered inside its children
  block's vertical span (top never above the block top,
   bottom never below the block bottom),
   and the
  columns scroll together at the tether boundary;
   also minimize partially-clipped panes.
   Stage 1
  re-architected `strip.rs` from one shared `GtkFixed` canvas to per-column vertical `ScrolledWindow`s
  (each over a `Fixed`,
   scrollbar hidden via `External` policy) inside one horizontal outer
  `ScrolledWindow`;
   panes placed at `row_y(row)` within their column;
   every column shares one content
  height so equal offsets align rows.
   Extracted scroll/reveal into `scroll.rs` (reveal reveals the
  column in the outer horizontal + the row in the column).
   Note:
   this session gives GtkApplication no
  D-Bus single-instance lock,
   so relaunches pile up windows -- always kill before relaunch.
   Confirmed
  live:
   one window,
   tree layout intact,
   each column wheel-scrolls independently.
   STAGE 2 next:
   the
  parent-within-children tether coupling;
   STAGE 3:
   snap to keep panes fully visible.
- 2026-07-08:
   Detached-column strip,
   STAGE 2 (tether) DONE.
   Each column's vadjustment
  `value-changed` calls `scroll.rs` `enforce_tether(initiator)`:
   with the initiator column's scroll
  fixed,
   neighbors clamp outward so the relative offset (child column minus parent column) stays in
  `[0, slack]`,
   where slack per pair = min over the left column's parents of `(deepest child row -
  parent row) * ROW_STRIDE`.
   The right pass keeps the child column in `[parent, parent+slack]`;
   the
  left pass keeps the parent column in `[child-slack, child]`;
   a re-entrancy guard
  (`StripInner.tethering`) ignores the `value-changed` the clamps themselves fire.
   Confirmed live:
   a
  child column can't scroll its contents above the parent,
   and the columns couple at the boundary.
  STAGE 3 next:
   snap offsets so as few panes as possible are partially clipped (the user sees a pane
  land not-fully-in-viewport on scroll-down).
- 2026-07-08:
   STAGE 3 (snap) is WIP and PAUSED for the day (session got long;
   the exact requirement
  wording outran a confident implementation).
   State:
  - `scroll.rs` `snap_columns` (debounced `SNAP_DELAY_MS` after the last scroll,
     via a
    `scroll_epoch`):
     rounds each column's vertical offset to its own nearest pane-height boundary,
    clamped to `[0, max]`,
     INDEPENDENTLY (an earlier version folded the tether in and dragged
    down-scrolled columns back to the top;
     that was fixed to independent rounding).
  - OPEN BUG:
     the user reports "scroll the rightmost column down to the end,
     it snaps back to the
    beginning soon after."
     Not yet root-caused.
     Debug offset logging is IN (`tracing::debug!`
    "tether pass" with per-column offsets,
     and "snap column" before/after/max/page/upper) -- reproduce
    with `RUST_LOG=file_manager=debug` and read the offsets to see which column resets and whether
    `max`/`upper` are as expected.
  - Likely-relevant facts to check next time:
     (a) there are TWO vertical scrolls -- the COLUMN scroll
    (moves whole panes;
     the tether+snap act here) and each pane's FILE-LIST scroll (independent,
    untouched);
     wheel routing between them may confuse which one moves.
     (b) every column's canvas is
    sized to the GLOBAL max content height (`set_content_height`) so equal offsets align rows,
     which
    means a short column can scroll DOWN into empty space below its panes -- reconsider whether that
    is desired.
     (c) panes are `PANE_HEIGHT=520` tall,
     so only ~1.7 fit per viewport -> at least one
    pane is always partly clipped;
     a smaller `PANE_HEIGHT` may make the snap moot.
  - UNRESOLVED REQUIREMENT:
     "tint each pane's available scroll region" / what exactly the snap should
    do -- the phrasing wasn't fully understood this session;
     clarify before more snap changes.
  - DEBUG SCAFFOLDING (gated,
     safe to keep or delete):
     `FM_DEBUG_TINT=1` loads `style.rs` `DEBUG_CSS`
    tinting `.fm-column` (per-column hue by `nth-child`),
     `.fm-canvas` (orange,
     the scrollable
    canvas),
     `.fm-pane` (pink border),
     `.fm-header` (purple),
     `.fm-list` (teal,
     the file-list scroll
    area).
     Widgets carry those classes (`strip.rs` columns;
     `pane.rs` container/header/list).
  - RESUME:
     `pkill -9 -f /monochromatic-file-manager` (this session has no D-Bus single-instance
    lock,
     so kill before relaunch),
     rebuild,
     run with `FM_DEBUG_TINT=1 RUST_LOG=file_manager=debug`,
    reproduce the snap-back,
     read the offset log.
     Committed up to STAGE 2 (tether) as working;
     STAGE 3
    (snap) + debug scaffolding committed as WIP.
- 2026-07-08:
   Session paused (touch grass).
   Open regression left IN PLACE (per user "keep the
  change") as a repro to investigate next session:
   adding `.fm-canvas { background-color: ... }` to
  `DEBUG_CSS` (tinting the scrollable `GtkFixed` inside each column -- the "available scroll region")
  makes the whole window render PURE BLACK,
   and ALL the other debug tints vanish too.
   So a background
  on the scroll-content `GtkFixed` appears to break the entire `DEBUG_CSS` provider (a GTK CSS
  parse/load failure would drop every rule -> back to the base black;
   likely candidates:
   the Fixed
  overdrawing its children,
   or `load_from_data` rejecting the sheet).
   First thing to try next time:
  run with `FM_DEBUG_TINT=1` and read stderr for a Gtk-CSS/GLib `WARNING`/`CRITICAL` about the
  stylesheet;
   if the sheet is being dropped wholesale,
   split `.fm-canvas` into its own provider to
  isolate it,
   or move the tint onto a child overlay instead of the Fixed.
   Everything else stands;
  resume the snap (STAGE 3) with a fresh head.
- 2026-07-08:
   Layout seam refactor.
   `strip.rs` now owns only the controller duties:
   mutate
  `PaneStripState`,
   build pane widgets,
   and hand `PanePlacement` snapshots to `layout.rs`.
  New `layout.rs` owns the GTK strip adapter:
   outer horizontal scroller,
   per-column vertical
  scrollers,
   `GtkFixed` canvases,
   widget map,
   focused-column state,
   and reconciliation.
   The former
  `scroll.rs` logic moved under `layout/scroll.rs`,
   still private to the layout seam,
   so reveal,
  tether,
   and snap no longer reach through `StripInner` for model or widget fields.
   This preserves
  current behavior while making the remaining snap bug a layout-adapter problem instead of a
  controller-plus-scroll cross-cutting problem.
- 2026-07-08:
   Tinted live-debug fix.
   User screenshot showed two separate problems:
   short columns
  inherited another column's deeper content height,
   producing huge tinted blank scroll regions,
   and
  snapping rounded `252 / 532` to row 0 before clamping,
   so a short `0..252` scroll range always
  snapped back to the top.
   Fix:
   `layout.rs` now sizes each column canvas to that column's deepest
  pane row (while keeping global row coordinates for pane placement),
   and `layout/scroll.rs`
  chooses the nearest reachable snap candidate by clamping both neighboring row-boundaries first,
   so
  bottom-of-range is a valid snap point.
   Verified lint/check/clippy/tests;
   reopened tinted debug run
  for live user verification.
- 2026-07-08:
   Labeled debug tint pass.
   User clarified that the green pane bodies were misleading and
  asked for every debug-relevant region to be tinted/labeled with visible descriptions plus short
  codes.
   Added `debug_tint.rs`;
   currently enabled visible labels are `V6C` column fixed pane canvas,
  `Y6L` immediate-child shared lane,
   and `B6P` preview body.
   Removed the green list/body fill;
  outer-strip,
   column-scroller,
   header,
   list-viewport,
   realized-row,
   and pane-shell labels are
  intentionally disabled while lane behavior is the focus.
   Immediate-child lanes are explicit
  debug-only rounded green boxes keyed by parent pane id and labeled with parent,
   column span,
   child
  count,
   and row extent.
   Verified lint/check/rust-lint/clippy/tests before reopening.
- 2026-07-09:
   App-owned lane scrolling.
   User clarified that full columns should not be scrollers,
  and refined the scroll model after earlier lane-capture versions made some states unscrollable and
  swallowed wheel events:
   vertical wheel/trackpad input belongs to the whole app first,
   while each
  sibling-group lane independently tries to stay visible within its own green-box limit.
   A lane is
  still a parent plus direct children.
   `layout.rs` now uses static per-column `GtkFixed` canvases
  inside one horizontal+vertical outer `ScrolledWindow`;
   per-column vertical `ScrolledWindow`s were
  removed.
   `layout/lane.rs` owns lane sticky offsets driven by the outer vertical adjustment,
  hierarchical offset application,
   vertical reveal,
   and rounded green `Y6L` overlays.
   `Y6L` boxes
  are fixed rails in app-layout coordinates;
   only panes receive sticky offsets inside those rails.
  Pane positioning resolves per column with forward/backward spacing passes,
   so panes keep
  `PANE_HEIGHT + PANE_GAP` separation while staying inside every `Y6L` rail they belong to.
   Lane
  offsets apply through the parent lane for non-root panes;
   root panes have no parent lane,
   so they
  use their own root lane.
   This prevents parent panes with their own children from double-moving into
  siblings while still letting the root stick inside its root rail.
  `layout/scroll.rs` now only handles horizontal reveal and the shared row-pixel helper.
  `debug_tint::wrap` mirrors wrapped-child expand flags so wrappers do not create fake debug
  geometry.
- 2026-07-09:
   Layout approved.
   Human verification on the live `file-manager-lane-tint` run approved
  the app-owned vertical scroll model with static columns,
   fixed `Y6L` rails,
   non-overlapping panes,
  and root-pane stickiness.
   Treat this layout behavior as the accepted baseline for the next file
  manager iteration.
