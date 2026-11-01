# Qt QML fast-scroll needs reuseItems + async incubation, or it collapses to 11 fps

The Qt/cxx-qt file-manager is a column-strip UI (a horizontal strip of columns,
 each
column a vertical stack of panes,
 each directory pane a virtualized row list,
 mixed
with 384x256 image-preview panes).
 Because that UI invites fast diagonal flinging
through very large directories,
 60 fps at high scroll velocity is a hard requirement,
not a nicety.
 This doc records how a naive QML implementation fails that requirement
(11 fps),
 how delegate recycling fixes it (60 fps),
 and the exact pattern the app must
use.
 It also records the Qt logging gotchas hit while measuring,
 so future QML
diagnostics here start from a working setup.

## The workload

Mirrors the Slint spike's scale so the numbers are comparable:

- `package/file-manager/app/src/strip.rs`:
   `COLUMN_COUNT = 1200`,
   a pane mix
  of roughly `400*26 + 800*5`,
   about 14400 panes total.
- `package/file-manager/app/src/rowmodel.rs`:
   directory panes address up to
  100000 rows each;
   only the visible handful is ever materialized.
- `package/file-manager/app/src/preview.rs`:
   preview panes decode 384x256
  bitmaps off-thread,
   show a placeholder,
   evict on window exit,
   and re-decode on
  scroll-back.
   Resident decoded bytes are bounded by the viewport.

The benchmark reproduces all of this:
 1200 columns,
 about 14400 panes,
 roughly half
of them preview panes decoding distinct 384x256 PNGs (`asynchronous`,
 `cache:false`
so scroll-back re-decodes exactly like `preview.rs`),
 the rest directory panes with up
to 100000 rows.
 It then flings the whole strip diagonally:
 columns sweep horizontally
across all 1200 while panes and rows sweep vertically across their full range at the
same time.

## Finding: recycling is the difference between 11 fps and 60 fps

Two implementations of the same scene,
 measured on the same machine at the same fast
diagonal scroll.
 FPS is counted from the window's `frameSwapped` signal and reported
once per second,
 so the measurement itself adds no per-frame cost.

Naive (`cacheBuffer: 0`,
 a `Loader` per pane,
 delegates destroyed and re-created at the
viewport edge every frame):

- Fast-scroll FPS collapsed to 11,
   ranging 20 to 36 once moving.
- GPU render work per frame was cheap (about 5.7 ms average),
   but the frame interval
  was 24 to 92 ms. The gap is GUI-thread time:
   destroying and instantiating delegates
  (each a `Rectangle` plus a `Loader` plus its component) synchronously every frame.
- Resident memory about 322 MB.

Optimized (`reuseItems: true` on every `ListView`,
 `Loader.asynchronous: true` for
pane content,
 `cacheBuffer` of one to two screens):

- Fast-scroll FPS held a steady 60.
   The only samples below 60 were the first three
  seconds (40,
   42,
   46) while the delegate pool and the decode pipeline warmed up.
- Resident memory about 226 MB (recycling keeps fewer live delegates and textures).

Virtualization stayed bounded in both cases:
 never more than about 54 panes,
 28
previews,
 and 459 rows realized at once,
 against 14400 panes and roughly 118 million
addressable directory rows.
 Virtualization was never the problem.
 Delegate churn on
the GUI thread was.

## The pattern the app must use

Build every list in the strip this way,
 and never the create-and-destroy default:

```qml
ListView {
    reuseItems: true            // recycle delegates on scroll, do not destroy/create
    cacheBuffer: someViewport   // prefetch about one screen of delegates
    // ...
}
```

- Put `reuseItems: true` on the outer column list,
   each column's pane list,
   and each
  directory pane's row list.
   This is the single highest-impact change (11 fps to 60).
- Select pane type (preview vs directory) with an asynchronous `Loader` or a
  `DelegateChooser`,
   never a synchronous `Loader`,
   so pane content incubates off the
  frame's hot path.
- Decode previews with `Image { asynchronous: true; sourceSize: 384x256 }`.
   The
  memory-vs-rework tradeoff is `cache`:
   `cache: false` matches `preview.rs`
  (evict and re-decode on scroll-back,
   lowest resident memory) and still held 60 fps
  here;
   `cache: true` trades memory for fewer re-decodes if a target machine needs it.
- Keep row and pane delegates shallow.
   Every extra nested item and every per-delegate
  binding is paid once per recycled delegate per fling.

## Logging gotchas hit while measuring

QML `console.log` and `QSG_RENDER_TIMING` output vanished until three things were set,
each worth knowing for any future QML debugging in this repo:

- Fedora ships `/usr/share/qt6/qtlogging.ini` with `*.debug=false`,
   which suppresses
  `console.log` (it logs at debug level).
   Re-enable with
  `QT_LOGGING_RULES="*.debug=false;qml.debug=true"` (enable only what you need;
   a bare
  `*.debug=true` floods with millions of `qt.text.emojisegmenter` lines).
- Qt routes messages to the systemd journal when stderr is not a tty,
   so redirecting to
  a file yields an empty file.
   Force stderr with `QT_FORCE_STDERR_LOGGING=1`
  (`QT_LOGGING_TO_CONSOLE` is deprecated).
- Qt's default logging is synchronous on the emitting thread;
   there is no background
  logging thread.
   `QSG_RENDER_TIMING` emits several `qCDebug` lines per frame from the
  render and GUI threads,
   each an inline blocking write,
   which inflates frame-interval
  numbers.
   Measure FPS from `frameSwapped` counted once per second instead,
   and give the
  real app non-blocking logging (see
  [the file-manager-qt README](../../package/file-manager/variant-qt/README.md)
  and the non-blocking-logging work):
   `tracing` with a `tracing-appender` non-blocking
  writer,
   plus a `qInstallMessageHandler` that forwards Qt's own messages into that same
  off-thread sink.
   Production never enables `QSG_RENDER_TIMING`.

## Still to verify

- These numbers are from the standalone `qml` runtime driving pure-QML integer models,
  not the real app.
   The integrated cxx-qt app adds a Rust `QAbstractListModel` (per-row
  data cost) and the Rust decode worker for previews.
   The integrated version must be
  re-measured,
   not assumed from this.
- The three-second warmup dip to about 40 fps is smoothable by pre-warming the delegate
  pool and the decode cache;
   not yet done.
- Measured on the strongest available machine:
   AMD Radeon RX 7600 (Navi 33),
   KWin 6.7.1
  Wayland,
   Qt 6.11.1,
   60 Hz (so 60 is the cap).
   The macOS (m1) and Windows (x13-win)
  targets,
   and any weaker hardware,
   each need their own run before this is settled.

## Reproduce

The optimized harness is
`package/file-manager/variant-qt/bench/strip-virtualization.qml`.
 It needs a pool
of 256 distinct 384x256 thumbnails beside it,
 generated with ffmpeg (kept out of git):

```sh
cd package/file-manager/variant-qt/bench
mkdir -p imgs
ffmpeg -y -loglevel error -f lavfi -i "testsrc2=size=384x256:rate=1" \
  -frames:v 256 "imgs/img_%03d.png"
QT_QPA_PLATFORM=wayland QT_FORCE_STDERR_LOGGING=1 \
  QT_LOGGING_RULES="*.debug=false;qml.debug=true" \
  /usr/lib64/qt6/bin/qml strip-virtualization.qml
```

Read the once-per-second `[bench] fps=...` lines;
 the HUD shows live vs addressable
delegate counts.
 Toggle `reuseItems` and `cacheBuffer` to reproduce the 11 fps naive
result.

## Related

- [winit-toolkits-no-wayland-drag-and-drop.md](winit-toolkits-no-wayland-drag-and-drop.md):
  why the app moved from Slint to Qt in the first place.
- `package/file-manager/app/src/strip.rs`,
   `rowmodel.rs`,
   `preview.rs`:
   the
  Slint spike's two-axis virtualization and off-thread preview cache that this mirrors.
