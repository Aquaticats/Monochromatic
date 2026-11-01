# File manager sticky flow: how two prototypes replaced a layout engine

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/desktop-app/file-manager-gtk-sticky` is now `package/file-manager/variant-gtk-sticky`;
> `package/desktop-app/file-manager-electron` is now `package/file-manager/variant-electron`;
> `package/cli/nested-wayland-session` is now `package/nested-wayland-session/cli`;
> `package/desktop-app/file-manager` is now `package/file-manager/app`.

This audit records how `package/desktop-app/file-manager-electron` and
`package/desktop-app/file-manager-gtk-sticky` were derived from the original GTK file manager
(`package/desktop-app/file-manager`),
 and exactly where their behavior differs from it.
It is self-contained:
 everything needed to understand the layout is in this document,
 and
every GTK concept is introduced through its web equivalent,
 so a reader who knows only HTML
and CSS can follow all of it.
An interactive companion with a live,
 scrollable demo of the same layout lives beside this
file as `file-manager-sticky-flow.html`;
 it is optional,
 this text stands alone.

Both prototypes are complete packages:
 they build,
 lint clean,
 carry unit tests for every pure
function,
 and pass an automated end-user test that runs the real app inside this repo's nested
Wayland compositor (`package/cli/nested-wayland-session`),
 drives it with synthetic keyboard
input,
 and asserts the app's observed state.
Both boundary tests use the same key sequences and the same state-file schema,
 so the two apps
are held to identical observable behavior.

## The product: a pane grid built from a browse tree

The file manager shows a horizontal strip of panes,
 like a multi-pane Finder.
Every pane is a fixed box,
 320 wide by 520 tall,
 with a 12px gap on both axes,
 so panes sit on
a grid whose horizontal stride is 332 and vertical stride is 532.
A pane's position on that grid is not chosen freely;
 it is derived from the browse tree:

- Column = lineage depth.
   Open a folder inside a pane and the new pane appears one column to
  the right of its parent.
- Row = assigned by a "tidy tree" layout with two rules:
   a parent aligns with its FIRST child,
  and a later sibling starts below the previous sibling's WHOLE subtree.

An example tree and the grid it produces (each box is one pane):

```txt
browse tree                          the resulting grid

root ─┬─ alpha ── nested                     column 0    column 1    column 2
      ├─ beta                              ┌──────────┬──────────┬──────────┐
      └─ gamma                       row 0 │ root     │ alpha    │ nested   │
                                           ├──────────┼──────────┼──────────┤
                                     row 1 │          │ beta     │          │
                                           ├──────────┼──────────┼──────────┤
                                     row 2 │          │ gamma    │          │
                                           └──────────┴──────────┴──────────┘
```

Root aligns with alpha (its first child);
 beta sits below alpha's whole subtree (alpha plus
nested,
 which both occupy row 0);
 gamma follows beta.
The two tidy-tree rules have a consequence this whole document leans on:
**every subtree owns a contiguous block of rows**,
 and within one column,
 panes from different
subtrees can never interleave.

One scroller owns the entire strip on both axes,
 the way a single `overflow: auto` div would.

## The behavior under audit: parents that follow the scroll

Scroll down and a parent pane is supposed to follow you:
 while any of its direct children is
on screen,
 the parent stays visible,
 riding inside a fixed region this project calls a rail or
band (the green boxes in debug screenshots,
 code name `Y6L` in the original).

A pane's band lives in its own column and spans from the pane's own row down to the bottom
edge of its DEEPEST DIRECT CHILD's row (children live one column right,
 but their rows define
how far the band reaches):

```txt
            column 0        column 1
          ┏━━━━━━━━━━┓    ┌──────────┐
    row 0 ┃ root     ┃    │ alpha    │      root's band (┏━┓ then ┇ ┇) spans
          ┃ (pane)   ┃    │          │      rows 0..1 because its deepest
          ┡━━━━━━━━━━┩    └──────────┘      direct child (beta) sits at row 1.
    row 1 ┇          ┇    ┌──────────┐
          ┇ empty    ┇    │ beta     │      alpha's and beta's bands are exactly
          ┇ band     ┇    │          │      their own boxes (no children), so
          ┗╍╍╍╍╍╍╍╍╍╍┛    └──────────┘      they never move.
```

As the app scrolls,
 the pane rides the viewport inside its band and stops at the band's end:

```txt
 scroll = 0                scroll = 300               scroll past band travel
┌─viewport────────┐       ┌─viewport────────┐        ┌─viewport────────┐
│ [root]  [alpha] │       │ [root]* [beta]  │        │ (root gone) ... │
│         [beta]  │       │   ^ pinned to   │        │ next content    │
└─────────────────┘       │     the top     │        └─────────────────┘
                          └─────────────────┘
```

If you know CSS you already know this behavior by name:
 it is exactly what
`position: sticky; top: 0` does to a section heading inside its section.

## The original, described for a web developer

GTK has no `position: sticky`.
The original app builds the strip out of `GtkFixed` canvases,
 which are `position: absolute`
containers:
 every child is placed at explicit x/y coordinates and nothing lays anything out
for you.
On top of that,
 in `package/desktop-app/file-manager/src/layout.rs` and `layout/lane.rs`,
 it
implements by hand:

- A scroll listener (`connect_value_changed` on the scroller's "adjustment",
   GTK's equivalent
  of a `scroll` event listener reading `scrollTop`).
- A per-lane offset store:
   a map from each parent pane to how far its group has slid,
  recomputed on every scroll tick.
- Hierarchical offset accumulation:
   a pane's final position sums the offsets of every ancestor
  lane above it,
   walking the parent chain.
- Rail clamping:
   a pane can belong to two rails at once (a child in its parent's rail,
   and
  itself the parent of another rail),
   so its position is clamped to the intersection of every
  rail containing it.
- A collision solver:
   after clamping,
   siblings could still be pushed onto each other,
   so a
  forward pass enforces minimum spacing going down and a backward pass going up,
   relaxing
  every column into a non-overlapping stack.
- Reveal machinery:
   after spawning a pane,
   scroll it into view.
  On the web this is one `element.scrollIntoView()` call;
   GTK updates scroll ranges one layout
  pass after content changes,
   so the original retries on an 8ms timer until geometry settles.

In web terms:
 a hand-rolled layout engine running in a `scroll` handler over a tree of
absolutely positioned boxes.
Measured over `layout.rs`,
 `layout/lane.rs`,
 `layout/lane/geometry.rs`,
 and
`layout/scroll.rs`,
 that engine is 549 lines of code (comments and blanks excluded).
It works,
 and a human approved its behavior as the product baseline (see
`doc/handover/file-manager-gtk-build.md`),
 but it took several recorded iterations of scroll
models,
 an off-by-one bug from clamping against stale scroll bounds,
 and the solver to get
there.

```txt
original engine, per scroll tick:

scrollTop ──> per-lane offsets ──> sum along parent chain ──> clamp into every
              (stored state)       (per pane)                 rail containing
                                                              the pane
                                                                  │
        rendered positions <── forward/backward spacing <────────┘
                               relaxation (the solver)
```

## The insight chain

The prototypes came out of a conversation that started with "how much simpler would this
layout be in HTML/CSS?
" and sharpened in three steps.

First:
 overlap prevention is not an intrinsic problem of this product.
HTML normal flow,
 the default block layout every web page starts with,
 cannot overlap
siblings.
The original needs a collision solver only because absolute positioning opts out of flow.
The solver is compensation for a representational choice,
 not essential complexity.

Second:
 `position: sticky` is the one CSS positioning scheme that adds scroll-driven movement
while STAYING in flow.
A sticky element keeps its layout slot,
 rides the viewport while its containing block passes,
and is released when the containing block's end pushes it off.
Written as a formula,
 a sticky element with `top: 0` sits at:

```txt
y = band_top + clamp(scroll − band_top, 0, band_height − element_height)
                     └──────┬─────────┘     └──────────┬─────────────┘
                     how far the viewport   the band's "travel": how far the
                     has entered the band   element can slide before hitting
                                            the band's end
```

Worked example with this app's constants (pane 520 tall,
 row stride 532),
 for a root whose
deepest direct child sits one row down:
 `band_top = 0`,
 `band_height = 532 + 520 = 1052`,
travel `= 1052 − 520 = 532`.

```txt
scroll     0     100    300    532    700
pane y     0     100    300    532    532   <- pinned to viewport top (y == scroll)
offset     0     100    300    532    532      until travel runs out at 532
```

Third,
 and decisively:
 the shared pane model already guarantees the structure sticky needs.
Recall the tidy-tree consequence:
 every subtree owns a contiguous row block.
Two results follow:

```txt
column c:  [P at row r]           P's direct children occupy rows r..d in column c+1,
            band of P: rows r..d  all inside P's contiguous subtree block.
           [Q at row q]           Q is the next pane in column c. Panes in one column
                                  are exactly the depth-c nodes, so Q starts a LATER
                                  subtree block: q > d, always.

therefore: P's lowest possible bottom = row_y(d) + 520
           Q's highest possible top   = row_y(q) ≥ row_y(d) + 532
           gap ≥ 12px at every scroll position, for every pair, in every column.
```

Bands within a column are disjoint,
 so if each pane is clamped inside its own band,
 no two
panes can ever collide,
 at any scroll offset.
Non-overlap stops being something you solve and becomes something the structure guarantees.
Once the rule and the guarantee were both on the table,
 each prototype is just one way of
saying "apply the sticky rule per pane":
 the Electron app says it declaratively and lets the
browser execute it;
 the GTK variant says it as one arithmetic expression per scroll tick.

## Prototype 1: file-manager-electron

The renderer builds the strip as ordinary HTML.
Rails are real elements;
 their heights and top margins are plain numbers computed once per
model change (never during scrolling):

```html
<div class="strip">          <!-- overflow: auto; display: flex; the one scroller -->
  <div class="column">       <!-- 320px wide, plain block layout -->
    <div class="rail">       <!-- height = band height; margin-top places it at its row -->
      <section class="pane"> <!-- position: sticky; top: 0; height 520px -->
```

The whole scroll-time behavior is two declarations in
`package/desktop-app/file-manager-electron/src/styles.css`:

```css
.strip { overflow: auto; }
.pane  { position: sticky; top: 0; }
```

The only geometry the app computes is in `src/bands.ts` (119 code lines):
 each rail's height
and the margin above it.
Because rails are normal-flow siblings and the margins derive from the disjoint-bands
guarantee,
 the margins are provably non-negative (the unit tests assert this),
 which is the
structural non-overlap guarantee in code.
During scrolling,
 zero application script runs;
 the browser's layout engine,
 which executes
sticky positioning off the main thread,
 does everything.

What JavaScript remains is not layout:
 the pane model (`src/strip.ts`,
 a line-for-line port of
the original's Rust model so both ecosystems share semantics,
 with the Rust unit tests ported
case for case),
 DOM reconciliation (creating and removing pane elements when the model
changes),
 keyboard handling,
 and the sandboxed IPC bridge that lets the renderer list
directories without filesystem access of its own.

Revealing a newly spawned pane is a single `scrollIntoView()` call,
 because the browser
reflows synchronously before scrolling.
 No retries,
 no timers.

## Prototype 2: file-manager-gtk-sticky

The second prototype answers the obvious objection:
 "fine,
 but we ship GTK,
 not a browser.
"
It ports the RULE back to GTK rather than the technology.

It reuses the original crate's public model,
 filesystem,
 and type modules verbatim
(`file-manager` is a path dependency),
 so the two GTK apps differ only in the layout engine.
The engine is `src/band.rs`:
 75 code lines of pure functions,
 no GTK types anywhere,
 and the
load-bearing part is exactly the sticky formula:

```rust
pub fn sticky_y(band: Band, scroll: f64) -> f64 {
    let travel = (band.height - f64::from(PANE_HEIGHT)).max(0.0);
    band.top + (scroll - band.top).clamp(0.0, travel)
}
```

The GTK adapter (`src/layout.rs`,
 227 code lines) is only plumbing:
 one absolute canvas (a
`GtkFixed`,
 the `position: absolute` container),
 a widget map,
 and a scroll listener whose
entire body is "re-place every pane at `band::positions(placements, scroll)`".
Gone relative to the original:
 the per-lane offset store,
 the parent-chain accumulation,
 the
rail-intersection clamping,
 the two-pass collision solver,
 and the offset pruning on close.

```txt
sticky variant, per scroll tick:            (compare with the original's diagram above)

scrollTop ──> for each pane: y = band_top + clamp(scroll − band_top, 0, travel)
              (no stored state, no chain, no intersection, no solver)
```

There is no layout state at all;
 position is a pure function of the model snapshot and the
scroll offset,
 which is why `band.rs` can be unit-tested by sweeping scroll values and
asserting the overlap count is zero everywhere.

Because GTK has no native sticky,
 this variant still pays two costs the browser build does
not:
 the scroll listener runs on the UI thread on every scroll tick (like a JS `scroll`
handler,
 where the browser's sticky runs in its compositor),
 and the reveal-after-spawn retry
timer survives,
 because GTK settles scroll ranges one layout pass after content changes.

## How the claims were verified

- The pure math is unit-tested in both languages
  (`file-manager-electron/src/bands.unit.test.ts`,
  `file-manager-gtk-sticky/src/band_tests.rs`):
   leaf bands never travel,
   parent bands stretch
  to the deepest direct child and ignore grandchildren,
   the clamp pins then releases,
   and a
  scroll sweep over the shared fixture tree reports zero overlapping pairs at every sample.
- Both apps mirror a shallow JSON state file (the Electron main process writes it;
   the GTK app
  writes it when `FM_STICKY_STATE_PATH` is set) with identical keys,
   including two facts
  computed from geometry:
   `rootPinned` (the first root pane sits exactly at the viewport top
  while the app is scrolled) and `overlapCount`.
- Both boundary tests host the real app inside the nested Wayland compositor with a throwaway
  fixture directory and drive it with the same key sequence (enter;
   left;
   enter again for
  dedup;
   left;
   down;
   enter;
   backspace),
   asserting the same states,
   decisively
  `rootPinned: true` with `scrolledDown: true` and `overlapCount: 0`.
- Screenshots with the debug tint were pixel-measured to confirm the rendered rails and panes
  coincide;
   that measuring caught a real silent bug (see the footguns section).

## Behavioral differences against the original

The prototypes are deliberately not pixel-identical to the approved baseline.

### Release point at the end of a rail

The original stops a lane's movement when the lane's bottom edge reaches the viewport bottom
(`lane_max_offset` in `layout/lane.rs`:
 maximum offset is `lane_bottom − viewport_height`).
Sticky releases a pane when the band's end pushes the pane off (travel is
`band_height − pane_height`).
Concretely,
 for a band spanning rows 0 through 3 (2116px tall) in a 600px viewport:

```txt
                     original                      sticky (both prototypes)
follows until        scroll = 2116 − 600 = 1516    scroll = 2116 − 520 = 1596
i.e. stops when      the BAND's bottom edge         the PANE reaches the band's
                     reaches the viewport bottom    own end
```

Sticky's variant is the standard web semantics ("the heading stays until its section is
gone");
 the original releases earlier,
 and its release point depends on the viewport height
while sticky's does not.

### Every parent sticks, not only roots

In the original,
 a pane's movement comes from its parent's lane,
 so the visibly sticking pane
is primarily the root riding its own root lane.
In the sticky model every pane owns a band:
 leaves get a band exactly one pane tall (zero
travel,
 so they are static grid boxes),
 and every pane with children sticks within its own
band,
 the way nested section headings all stick in a long document.
A mid-lineage parent therefore follows the scroll in the prototypes in situations where the
original would let it slide away while its own children remain visible.

### The parent-chain accumulation and the solver are gone

No pane's position depends on any other pane's position;
 each clamps independently inside its
own fixed band,
 and the disjoint-bands guarantee makes conflicts impossible.
One observable consequence:
 the original's relaxation passes could compress the gap between
siblings when clamping pushed them together (spacing was a minimum,
 not a fixed grid).
In the prototypes,
 gaps are fixed flow margins and can never compress.
If a future design wants gap compression back,
 that is exactly where a solver,
 or CSS beyond
sticky,
 re-enters.

### Deliberate scope cuts, not layout differences

The prototypes do not implement thumbnails,
 drag-and-drop,
 the Windows/macOS drag shims,
 or
the original's bulk close gestures;
 the Electron port also omits the snapshot generation
counter.
These are orthogonal to the layout comparison;
 the GTK variant reuses the original's model,
 so
those semantics remain available to it unchanged.

## What the swap actually bought, measured

- Original lane engine:
   549 code lines of stateful scroll-driven positioning.
- GTK sticky variant:
   302 code lines total (`band.rs` 75,
   pure and unit-tested;
  `layout.rs` 227 of GTK plumbing),
   zero layout state,
   the policy being one clamp.
- Electron variant:
   119 code lines of one-shot flow arithmetic plus two CSS declarations;
  zero code of any kind runs during scrolling.

The honest conclusion,
 and the reason this audit exists rather than just a port:
 most of the
win was available WITHOUT leaving GTK,
 because it came from the model (flow semantics plus the
tidy tree's disjoint-bands guarantee),
 not from the browser.
What only the browser provides is executing that model declaratively:
 sticky off the main
thread,
 synchronous reflow that makes `scrollIntoView` retry-free,
 and a layout engine with
twenty years of debugging behind it.

## The integration footguns, recorded in full

The sticky CSS needed no debugging.
Reimplementing the same rule in GTK,
 and packaging the Electron app to this repo's standards,
surfaced a series of footguns;
 every one now has a full troubleshooting write-up with a
source-cited root cause,
 a reproduction,
 and verified workarounds:

- The silent one,
   and this audit's own corrected misdiagnosis:
   the per-column canvases
  stretched ~74px off the grid because an ellipsized `GtkLabel` still requests its FULL text
  width as its natural size,
   and GTK grows children toward naturals before expand flags are
  consulted;
   `set_hexpand(false)` was a red herring (a minimal probe proves the flag itself
  works),
   and an earlier commit message here wrongly blamed expand propagation.
  Proven by clean-worktree reproduction and a `max-width-chars` isolation:
  [gtk4-label-ellipsize-natural-width](../troubleshooting/gtk4-label-ellipsize-natural-width.md).
- First Enter closed the pane (focus lands on the header's close button,
   the first focusable
  widget),
   and after that fix Enter did nothing (a `GtkListView` row only activates when a ROW
  has focus,
   which no bare `grab_focus` provides):
  [gtk4-listview-keyboard-activation](../troubleshooting/gtk4-listview-keyboard-activation.md).
- The `ListView::scroll_to` fix required GTK's 4.12 API level,
   which deprecates
  `CssProvider::load_from_data` and breaks warnings-as-errors builds:
  [gtk4-cssprovider-load-from-data-deprecation](../troubleshooting/gtk4-cssprovider-load-from-data-deprecation.md).
- `serde_json::json!` expands to a banned `unwrap` under this repo's clippy policy:
  [serde-json-macro-clippy-disallowed-unwrap](../troubleshooting/serde-json-macro-clippy-disallowed-unwrap.md).
- Keystrokes sent before the app's window maps are silently dropped by the test compositor,
  so the observed `ready` fact is gated on GTK's map signal;
   plus two operational traps
  (AF_UNIX path length,
   `pkill -f` self-matching):
  [nested-wayland-gui-test-footguns](../troubleshooting/nested-wayland-gui-test-footguns.md).
- On the Electron side:
   pointing a second tsdown config at an outDir another config owns lets each run's default
  clean delete the other's output;
   self-inflicted,
   since the repo's one-`dist/`-subdir-per-config convention exists to prevent exactly this,
   and the preload bundle now builds into its own `dist/preload`
  ([tsdown-shared-outdir-clean](../troubleshooting/tsdown-shared-outdir-clean.md)),
  sandboxed preloads must be CommonJS
  ([electron-sandboxed-preload-cjs](../troubleshooting/electron-sandboxed-preload-cjs.md)),
  `isolatedDeclarations` demands annotations on computed exported consts
  ([typescript-isolated-declarations-computed-const](../troubleshooting/typescript-isolated-declarations-computed-const.md)),
  and quitting the compositor under Electron prints an alarming but benign broken-pipe FATAL
  ([electron-nested-wayland-shutdown-broken-pipe](../troubleshooting/electron-nested-wayland-shutdown-broken-pipe.md)).

The pattern across the GTK entries:
 each is documented or documentable toolkit behavior,
 but
the browser either provides the guarantee by default (widths mean what they say,
 focus and
activation conventions match user intuition) or fails loudly where GTK fails silently.

## Where the pieces live

- Original:
   `package/desktop-app/file-manager` (lane engine under `src/layout*`).
- Electron prototype:
   `package/desktop-app/file-manager-electron` (`src/bands.ts`,
  `src/styles.css`,
   model port in `src/strip.ts`).
- GTK sticky prototype:
   `package/desktop-app/file-manager-gtk-sticky` (`src/band.rs`,
  adapter in `src/layout.rs`,
   shared model via the `file-manager` crate).
- Compositor test host:
   `package/cli/nested-wayland-session`.
- Approved-baseline history:
   `doc/handover/file-manager-gtk-build.md`.
- Interactive companion:
   `file-manager-sticky-flow.html` beside this file.

## Decision this audit does not make

Whether the sticky release semantics (and every-parent stickiness) should replace the approved
baseline is a product call:
 the baseline was approved by a human on a live run,
 and these
prototypes change its feel in the two ways described above.
The next step,
 if the direction is wanted,
 is a side-by-side live run of `file-manager` and
`file-manager-gtk-sticky` on the same fixture tree.
