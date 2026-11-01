# File manager toolkit exploration: Slint, then Qt, then GTK4 (handover)

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/desktop-app/file-manager-qt` is now `package/file-manager/variant-qt`.

Cross-session state for choosing the GUI toolkit for a native cross-platform file manager.
The hard constraint is native OS drag-and-drop on a pure Wayland session (KWin),
 no
XWayland.
 The fleet is Linux (primary,
 this KWin machine),
 macOS (`ssh m1`),
 Windows
(`ssh x13-win`).
 This doc is kept live;
 sections marked "measuring" or "not built" are
updated as work proceeds.

## Status at a glance

- Slint (winit):
   rejected for this app.
   No native Wayland DnD;
   needs a hand-rolled
  `wl_data_device` adapter plus a Slint fork.
   Painful,
   and we would own both.
- DECISION (2026-07-08):
   GTK4 (gtk4-rs) is chosen;
   Qt (cxx-qt) is out.
   GTK meets both tiered
  perf targets (60+ fps on the Linux box,
   30+ fps on the fanless M1 Air),
   teardown is clean on
  every platform,
   and it is the nicer toolkit to work with.
   Qt is out because its `reuseItems`
  teardown segfault (and `reuseItems` is required for fast scroll) surfaces as a user-visible
  "quit unexpectedly" CrashReporter dialog on macOS (unacceptable,
   unlike Linux where it exits
  silently),
   and it missed the 30-fps M1 bar.
   Decided on Linux + macOS evidence plus the crash;
  Windows is a later verification step for GTK,
   not a re-opening of the choice.
- Perf targets (set by the user):
   60+ fps on the most-performant machine (this Linux box),
   and
  30+ fps on the weakest (the fanless M1 Air).
   Not a flat 60-everywhere bar.
- Qt (cxx-qt):
   rejected.
   `reuseItems` (required for 60 fps fast scroll) segfaults on teardown;
  on Linux that was a silent on-exit crash we accepted,
   but on macOS it raises a CrashReporter
  dialog.
   Verified on m1:
   `EXC_BAD_ACCESS`/`SIGSEGV` (`KERN_INVALID_ADDRESS at 0x8`) in
  `QQmlReusableDelegateModelItemsPool::drain` during engine teardown,
   with a `.ips` crash
  report written (see `doc/troubleshooting/qt-qml-reuseitems-teardown-segfault.md`).
   Perf on
  the M1 Air (over RustDesk) oscillated 29-61 fps,
   dipping below the 30-fps bar.
   It does build
  and run (native Cocoa window,
   Rust bridge round-trips),
   but needs a per-OS `QMAKE` path,
   the
  Wayland QPA pin dropped,
   a C++ logging shim,
   and carries pre-1.0 API churn.
- GTK4 (gtk4-rs):
   chosen.
   Native Wayland window,
   native inbound DnD from Dolphin,
   virtualized
  list,
   clean teardown (rc 0,
   no crash dialog on any platform),
   all verified,
   owning nothing.
  Perf meets the targets:
   steady ~60 fps on the Linux box,
   and steady ~30-38 fps on the M1 Air
  even under RustDesk's screen-capture penalty (a real local number would be higher),
   as long
  as thumbnail decode is off the render path (synchronous per-frame decode drops it to ~4 fps).
  On macOS it builds and runs zero-config (no per-OS env),
   native Cocoa window with the
  virtualized list.
   On Windows it builds via gvsbuild and GPU-renders once DirectComposition is
  enabled (`GDK_DEBUG=dcomp`),
   holding a locked 60 fps on the real sparse-board workload on the
  x13-win (see "Windows (x13-win) results").

## Test machines (full specs)

All perf numbers in this doc were produced on these two machines.
 The tiered targets map to
them:
 60+ fps on the Linux desktop (most performant),
 30+ fps on the M1 Air (weakest).

Linux (primary,
 this KWin machine;
 perf target 60+ fps):

- CPU:
   AMD Ryzen 7 8700F,
   8 cores / 16 threads,
   boost to ~5.05 GHz.
- GPU:
   AMD Radeon RX 7600-class (Navi 33,
   RDNA3 discrete).
- RAM:
   64 GB.
- OS:
   Bazzite 44.20260629 (Kinoite,
   Fedora 44 atomic / rpm-ostree),
   kernel
  `7.0.9-ogc3.2.fc44.x86_64`.
- Desktop:
   KDE Plasma 6.7.1,
   KWin 6.7.1,
   Wayland session (no XWayland).
- Display:
   quad 4K,
   all four outputs (DP-3,
   HDMI-A-2,
   HDMI-A-1,
   DP-2) at 3840x2160@60 Hz,
   a
  heavy compositor load that the RX 7600-class GPU still drove at 60 fps in the benchmark.
  Measured on the local session (no remote-desktop overhead).

macOS (`ssh m1`;
 perf target 30+ fps):

- Model:
   MacBook Air (`MacBookAir10,1`),
   Apple M1,
   fanless.
- CPU:
   Apple M1,
   8 cores (4 performance + 4 efficiency).
- GPU:
   Apple M1 7-core (base M1 Air),
   Metal 4.
- RAM:
   16 GB.
- Storage:
   internal SSD 228 GiB (99 GiB free),
   fragile,
   so build writes were offloaded to the
  external MacData APFS volume (477 GiB,
   222 GiB free) via `CARGO_HOME`,
   `CARGO_TARGET_DIR`,
  `HOMEBREW_CACHE`;
   only Homebrew's prefix installs (unavoidable) touched the internal SSD.
- OS:
   macOS 26.5.2 (build 25F84),
   Darwin 25.5.0.
- Display:
   built-in Retina 2560x1600 @ 60 Hz.
   Measured over a RustDesk remote-desktop session,
  whose screen-capture depresses fps (a real local number would be higher);
   both toolkits ran
  under this same penalty,
   so the head-to-head stays fair.

Windows (`ssh x13-win`;
 no prior fps target,
 sits between the Linux box and the M1 Air):

- Model:
   Lenovo ThinkPad X13.
- CPU:
   AMD Ryzen 7 PRO 7840U,
   8 cores / 16 threads (Zen 4).
- GPU:
   AMD Radeon 780M (RDNA3 integrated);
   Vulkan 1.4.349,
   AMD proprietary driver 26.6.1;
  display driver 32.0.31019.2002.
- RAM:
   32 GB (30.6 GB visible).
- OS:
   Windows 10 21H2,
   build 19044.7417.
- Toolchain:
   Rust nightly `x86_64-pc-windows-msvc`,
   Visual Studio Community 2026 (gvsbuild needs
  `--vs-ver vs2026`),
   gvsbuild 2026.6.0 building GTK 4.22.4 from source,
   MSYS2 for gvsbuild's
  unix helper tools,
   Python 3.12 via mise.
   The app runs from a self-contained bundle (exe plus
  44 GTK DLLs plus glib schemas in one directory,
   no PATH needed).
- Display:
   physical display on at 1920x1200;
   measured over RustDesk.
   A WebGL/WebGPU browser GPU
  benchmark on the same box over the same RustDesk connection sustained 65 fps (WebGL) and 86 fps
  (WebGPU),
   1%-low 36/49,
   so RustDesk does not depress a well-behaved GPU app;
   bench dips here
  are the app's own cost,
   not the remote path.

## Decision criteria (corrected)

What does NOT decide it:
 styling and native look (out of scope;
 we style until it aligns);
which toolkit has a flagship file manager (both do,
 Nautilus is GTK and Dolphin is Qt);
macOS/Windows foreign look and packaging (solvable,
 complexity is not a blocker);
 raw
cross-platform reach (both reach all three targets).

What DOES decide it:
 developer experience,
 how nice each is to work with,
 plus passing the
build-and-run gate.
 Two more factors surfaced during the macOS gate and became decisive:
 the
teardown-crash behavior (silent on Linux,
 a user-visible crash dialog on macOS) and the tiered
perf targets (60+ on Linux,
 30+ on the M1 Air).

Developer-experience read (from building both spikes hands-on,
 preliminary):

- GTK4 (gtk4-rs):
   mature bindings (0.11,
   stable API),
   pure Rust,
   idiomatic (builder pattern
  and closures),
   UI in Rust with no separate markup language,
   no C++ shim.
   The spike was about
  90 lines and mostly worked first try.
   Nicer to work with so far.
- Qt (cxx-qt):
   pre-1.0 (0.9.1) with an API that churns across 0.x bumps;
   the UI lives in QML
  (a separate markup and JS language,
   a Rust/QML split);
   a hand-written C++ shim was needed
  just to route Qt's logging off-thread;
   the bridge macro forbids doc comments on extern
  blocks and has other sharp edges.
   More powerful cross-platform,
   but more friction.

This read is preliminary and must be confirmed by actually building each on macOS and Windows.

## Verified findings per toolkit

### Slint (winit) — rejected

- winit 0.30 has no Wayland DnD (winit#1881);
   inherited by every winit toolkit (Slint,
  Bevy,
   Iced,
   egui).
   See `doc/troubleshooting/winit-toolkits-no-wayland-drag-and-drop.md`.
- An earlier session got inbound Dolphin drops working by hand-rolling a `wl_data_device`
  adapter on winit's Wayland connection,
   but only after forking Slint to add
  `BackendBuilder::with_clipboard(false)`,
   because KWin delivers a drag to only a client's
  FIRST data device and Slint's clipboard device took that slot.
   See
  `doc/troubleshooting/kwin-drag-only-first-data-device.md` and
  `doc/handover/file-manager-native-dnd.md`.
   Roughly 1345 lines of DnD plumbing plus a
  maintained Slint fork.
   Ownership too high.

### Qt (cxx-qt) — works, with an accepted on-exit crash

- cxx-qt 0.9.1 against system Qt 6.11.1.
   Native Wayland QML window verified on KWin
  (`xdg_toplevel`,
   no XWayland).
   Package:
   `package/desktop-app/file-manager-qt`.
- Fast scroll needs `ListView { reuseItems: true }`:
   naive create/destroy delegates hit
  11 fps at fast scroll,
   `reuseItems` + async incubation + cacheBuffer holds 60 fps (after
  a ~3 s warmup).
   Measured with the standalone `qml` runtime,
   not yet the integrated
  cxx-qt app.
   See `doc/troubleshooting/qt-qml-listview-fast-scroll-recycling.md`.
- `reuseItems` segfaults on teardown (use-after-free in
  `QQmlReusableDelegateModelItemsPool::drain`),
   triggered by any `Loader` wrapping a nested
  reuseItems `ListView`,
   by three or more nested levels,
   or even a mixed 2-level layout.
  Qt 6.12 does not fix it (the crash-site change is a pure refactor).
   Resolution:
   accept
  the on-exit crash,
   or `std::process::exit(0)` after `exec()` for a clean exit;
   no Qt
  fork or backport (ownership).
   See
  `doc/troubleshooting/qt-qml-reuseitems-teardown-segfault.md`.
- Native DnD is a Qt capability (Dolphin itself is Qt on Wayland),
   but our Qt spike does
  not yet wire a `DropArea`,
   so DnD is not verified end to end in our code.
- Non-blocking logging is done:
   `tracing` on a `tracing-appender` NonBlocking writer,
   plus
  a `qInstallMessageHandler` C++ shim routing Qt's own logs into the same off-thread sink
  (`src/qt_log.rs`,
   `src/qt_log.cpp`).
- Cross-platform native feel is first-class,
   but that is not a deciding factor here (styling
  is out of scope).
   What counts against Qt on the deciding axis (developer experience):
   pre-1.0
  cxx-qt API churn,
   the UI split into QML (a separate markup and JS language),
   a hand-written
  C++ shim needed just to route logging,
   and the bridge-macro sharp edges.

### GTK4 (gtk4-rs) — cleanest so far

- gtk4-rs 0.11 (feature `v4_10`) against system GTK 4.22.
   Package:
  `package/desktop-app/file-manager-gtk`.
   About 90 lines.
- Verified end to end on this KWin session:
  - Native Wayland window (`xdg_toplevel`,
     114 protocol msgs,
     no xcb/XWayland).
  - Virtualized `ListView` over a 100000-row model (only ~28 rows realized;
     screenshot).
  - Clean teardown,
     `rc=0` (exactly where Qt segfaults).
  - Native inbound DnD:
     a real Dolphin drop of `~/Downloads/hello.txt` landed via a stock
    `GtkDropTarget` (`GdkFileList` over `wl_data_device`),
     logged as `inbound file drop`.
  - Non-blocking `tracing`,
     same as the Qt spike.
  - Fast scroll:
     the diagonal 2D grid of ~14400 mixed panes (previews and small lists,
    `src/main.rs` under FMGTK_BENCH) holds a steady ~60 fps with thumbnails decoded off the
    render path;
     naive synchronous per-frame decode drops it to ~4 fps.
     Clean teardown (rc 0)
    throughout,
     unlike Qt (no `reuseItems`-style teardown crash).
- Owns nothing:
   no fork,
   no hand-rolled protocol,
   no accepted crash.
- Correction:
   "Nautilus is GTK4" is not a differentiator over Qt,
   since Dolphin (a flagship
  Qt file manager) shows Qt is equally proven for this use case.
   Both are battle-tested;
   this
  is dropped as a reason.
- Not yet done:
   outbound drag (`GtkDragSource`);
   a real off-thread thumbnail decoder (the
  benchmark caches;
   the app needs a worker plus eviction like `preview.rs`).
   macOS and Windows
  build-and-run plus perf are now done (see the "macOS (m1) results" and "Windows (x13-win)
  results" sections);
   native DnD verification on macOS and Windows is still pending.

## macOS (m1) results

Machine:
 the MacBook Air M1 (full specs in "Test machines";
 fanless,
 weakest in the fleet),
viewed over RustDesk,
 with all heavy build writes offloaded to the MacData APFS volume.

Build-and-run developer experience:

- GTK4:
   `brew install gtk4`,
   then `cargo build`,
   43 s first build,
   zero config,
   zero env vars,
  zero warnings.
   Ran the binary directly;
   native Cocoa window with the virtualized list
  rendered (screenshot).
   GTK auto-selected the Quartz backend (no Wayland involved on macOS).
- Qt:
   `brew install qt` (6.11.1),
   then `cargo build`,
   1 m 34 s first build,
   but required
  `QMAKE=/opt/homebrew/opt/qt/bin/qmake` (macOS names it `qmake`,
   Linux `qmake6`,
   so the
  package's hardcoded `QMAKE=/usr/bin/qmake6` does not port),
   plus one benign linker warning
  (`duplicate -rpath ignored`);
   build.rs runs qmake/moc and compiles the C++ shim.
   Ran the
  binary directly (no `QT_QPA_PLATFORM=wayland`,
   which is Linux-only);
   native Cocoa window with
  the Rust-backed greeting rendered,
   proving the Rust/QML bridge round-trips.

Perf (diagonal fast scroll of the ~14400-pane grid with the shared 256-image 384x256 pool,
16 s runs,
 over RustDesk;
 the RustDesk screen-capture depresses both toolkits equally,
 so these
are a lower bound and the head-to-head stays fair):

- GTK4:
   debug ~30-38 fps;
   release + `caffeinate` also ~30-38 fps (release did not lift it,
  so the ceiling is the environment,
   not Rust debug overhead).
   Steady,
   always at or above the
  30-fps M1 bar.
   Clean teardown (`rc 0`) every run.
- Qt (`qml` runtime):
   oscillated 29-61 fps,
   briefly touching 60 (so the panel can present at 60
  with RustDesk connected) but dipping to 29 under image load,
   below the 30-fps bar.

The Qt teardown crash on macOS (decisive):
 the `qml` strip benchmark,
 which uses nested
`reuseItems` ListViews,
 crashed on exit and macOS wrote a CrashReporter `.ips` report
(`EXC_BAD_ACCESS`/`SIGSEGV`,
 `KERN_INVALID_ADDRESS at 0x8`,
 top frames
`QQmlDelegateModelItem::destroyObjectLater` -> `destroyCacheItem` ->
`QQmlReusableDelegateModelItemsPool::drain` -> `~QQmlDelegateModelPrivate`).
 Same use-after-free
as on Linux,
 but on macOS it is a user-visible "quit unexpectedly" dialog,
 which the user ruled
unacceptable.
 Detail in `doc/troubleshooting/qt-qml-reuseitems-teardown-segfault.md`.

GTK native DnD on macOS (verified on m1 over RustDesk;
 the drags reach the app,
 so this is GTK
handling,
 not remote input):
 inbound (Finder -> GTK) is received by the stock `GtkDropTarget`,
 but
`g_file_get_path` returns None because GDK's macOS backend percent-encodes the URI scheme colon
(`file%3A///...`);
 the full path is present and recovered by unescaping the URI,
 so inbound is
workable and it is not a permission/TCC issue.
 Outbound (GTK -> Finder) is rejected even with a
correctly-encoded `text/uri-list`,
 so it is done via a native `objc2` AppKit drag shim (a
`MainThreadOnly` `NSDraggingSource` beginning an `NSDraggingSession` with the file `NSURL`),
 built
and verified:
 dragging the handle to a Finder folder copies the file.
 Analogous to the Windows OLE
shim.
 Reference implementation in `doc/troubleshooting/gtk4-macos-file-dnd.md`.

## Windows (x13-win) results

Machine:
 the ThinkPad X13 (full specs in "Test machines"),
 driven over RustDesk.
 GTK4 builds,
runs,
 and GPU-renders natively on Windows,
 and holds a locked 60 fps on the real UX workload.

Build-and-run developer experience:

- The toolchain is a one-time setup:
   MSYS2 (gvsbuild's helper tools),
   Python via mise,
   gvsbuild
  2026.6.0 building GTK 4.22.4 with `--vs-ver vs2026` (gvsbuild defaults to VS 2022 and skips
  VS 2026 otherwise).
   The GTK source build took ~21 min on this 8-core part;
   the Rust crate then
  built against it with zero source changes (same `main.rs` and `Cargo.toml` as Linux and
  macOS),
   1 m 24 s release.
- The app is shippable as a self-contained folder:
   exe plus the 44 GTK DLLs plus glib schemas in
  one directory,
   no external PATH.
   Verified it loads standalone.

The DirectComposition gotcha (cost hours,
 worth recording):
 gvsbuild ships a patch
(`0001-remove-direct-composition.patch`) that turns GTK's DirectComposition from a default-on
feature into an opt-in debug flag,
 and GTK's GL renderer hard-requires a DirectComposition
device.
 Without opting in,
 GL fails to realize ("OpenGL requires Direct Composition") and GTK
falls back to the Cairo software renderer,
 which rasterizes the whole scene at 3-6 fps.
 gvsbuild
also builds GTK with Vulkan disabled (`-Dvulkan=disabled`),
 so there is no Vulkan renderer to
fall back to.
 The fix is one env var,
 `GDK_DEBUG=dcomp`,
 which enables the DirectComposition
device and lets the GL renderer run on the GPU.
 Full detail and evidence in
`doc/troubleshooting/gtk4-windows-gvsbuild-directcomposition.md`.

Perf (diagonal pan,
 over RustDesk,
 `GDK_DEBUG=dcomp`):

- The GPU is not the limit:
   uncapped (`GDK_DEBUG=dcomp,no-vsync`) the 780M peaked at 68 fps for
  the heaviest scene,
   and the browser GPU benchmark hit 86.
   Software fallback was never taken
  with dcomp on (zero `GSK_DEBUG=fallback` lines).
   Textures were not the limit either:
   a
  text-only run was slower than the mixed run,
   since a text pane's Pango layout costs more than
  an image pane's single cached quad.
- The bottleneck under load is CPU per-frame widget work (item realize/bind plus Pango layout),
  and it scales with the number of populated panes animating at once.
   A single scrolling pane
  holds a locked 60;
   a dense grid with every cell populated and all columns scrolling at once
  (~72 populated panes in view) collapses to 3-37.
   That dense case is a synthetic worst case.
- The real UX (a sparse 100x100 board,
   ~20% of cells populated,
   ~2000 panes split 50/50 image
  and text,
   panned diagonally) holds a locked 60-61 fps,
   p99 frame time ~16.7 ms,
   after a
  one-second first-paint warmup.
   This is the number that reflects the product.

Why the sparse board hits 60 while the dense grid does not,
 and the architecture lesson:
 the
winning structure places the populated panes at fixed positions on a canvas (empty cells are no
widget),
 so a pan does not re-run virtualization or relayout;
 GTK caches each pane's render node
and the pan just re-composites those cached quads on the GPU.
 That is the `doc/decision/
vector-design.md` tile principle realized inside GTK.
 For the real UI (see "Open items"):
 build
the pane board as fixed-position panes on a canvas rather than nested virtualized `ListView`s
(which churn widgets during a pan);
 decode thumbnails off-thread into a bounded,
 evicting texture
cache;
 keep the pan path allocation-free.
 One scaling boundary:
 realize-all works because 100x100
(~2000 populated) is bounded;
 if the board grows an order of magnitude,
 move to 2D virtualization
or per-pane texture tiles.

Native DnD on Windows (both directions verified):
 inbound (Explorer -> GTK) works through the
stock `GtkDropTarget` (`GdkFileList`) over GDK's Win32 OLE backend.
 Outbound (GTK -> Explorer)
does NOT work through GTK:
 GDK's Win32 drag source leaves the file-to-"Shell IDList Array"
conversion unimplemented,
 so Explorer rejects the drop.
 The resolution is a small native Win32
OLE drag source in Rust (the `windows` crate:
 a shell `IDataObject` via `BHID_DataObject` plus
`SHDoDragDrop`),
 triggered from a `GtkGestureDrag`;
 with it,
 dragging a file to an Explorer folder
copies it (`effect=1`).
 Windows-only;
 Wayland and macOS use the native `GtkDragSource`.
 Detail and
a verified reference implementation in `doc/troubleshooting/gtk4-windows-outbound-file-drag.md`.

Verdict:
 Windows passes the perf gate (locked 60 for the real workload),
 which confirms GTK4
across the fleet.
 This does not re-open the toolkit choice;
 GTK4 stands,
 and it remains the only
option satisfying the Wayland DnD constraint (winit-based stacks,
 including the `vector-design`
engine stack,
 have no Wayland DnD).

## Branches and worktrees

- `feat/file-manager-qt` at `/var/home/user/worktrees/file-manager-qt`:
   the Qt spike,
   the
  three Qt/winit troubleshooting docs,
   and a require-rustdoc linter change (cxx-qt files
  exempt `use` and trait-impl items).
   Pushed to `Aquaticat/Monochromatic`.
- `feat/file-manager-gtk` at `/var/home/user/worktrees/file-manager-gtk`:
   the GTK spike and
  this handover.
   Pushed.
- `main`:
   the earlier Slint native-DnD work and the winit-toolkit survey doc.

## Key decisions made

- Leave Slint (its winit base cannot do Wayland DnD without a fork we own).
- Choose GTK4 (gtk4-rs);
   reject Qt (cxx-qt).
   See the DECISION bullet in "Status at a glance"
  and the "macOS (m1) results" section for the full evidence.
- Qt was rejected because its required `reuseItems` teardown crash is a user-visible crash
  dialog on macOS,
   and it missed the 30-fps M1 bar;
   the earlier "accept the on-exit crash"
  stance held only on Linux and does not survive macOS.
- Log off-thread in both spikes (tracing NonBlocking;
   Qt logs bridged in).
- Toolchains layered live via `rpm-ostree install --apply-live` (Qt6 devel;
   gtk4-devel) on
  Linux;
   `brew install gtk4` / `brew install qt` on macOS.

## Open items and next steps

- DECISION MADE:
   GTK4.
   No further toolkit comparison is needed;
   Windows is now a verification
  step for GTK,
   not a re-open of the choice.
- Build the real GTK column/pane UI:
   `GtkColumnView` or nested lists,
   an off-thread thumbnail
  decoder with eviction (like `preview.rs`,
   not the benchmark's decode-everything cache),
   and
  wire outbound `GtkDragSource` plus clipboard.
   Inbound `GtkDropTarget` is already proven.
- Windows (`x13-win`):
   build/run,
   perf,
   and native DnD (both directions) are done and passed
  (locked 60 on the real sparse-board workload;
   inbound via stock `GtkDropTarget`;
   outbound via a
  native Win32 OLE shim,
   since GTK's own drag source cannot deliver files to Explorer).
   In the
  real app on Windows:
   set `GDK_DEBUG=dcomp` in-process for GPU rendering,
   and port the OLE drag
  shim from `doc/troubleshooting/gtk4-windows-outbound-file-drag.md`.
- macOS DnD is done (see "macOS (m1) results" and `doc/troubleshooting/gtk4-macos-file-dnd.md`):
  inbound works with an in-app URI-unescape (GDK's macOS backend encodes the URI scheme colon);
  outbound is done via a native `objc2` AppKit drag shim (built and verified).
   Port both into the
  real app,
   alongside the Windows `GDK_DEBUG=dcomp` + OLE shim;
   the earlier assumption that macOS
  DnD "just works via Cocoa" was wrong.
- The Qt spike,
   its troubleshooting docs,
   and the cxx-qt linter carve-out can stay as recorded
  evidence;
   no further Qt spike work is planned.

## How to run the spikes

- Qt:
   `mise run //package/desktop-app/file-manager-qt:run` (forces `QT_QPA_PLATFORM=wayland`).
- GTK:
   `mise run //package/desktop-app/file-manager-gtk:run` (forces `GDK_BACKEND=wayland`).
