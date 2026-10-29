# Slint 1.17.0 Flickable: replacing a virtualized Repeater model mid-scroll caps the drag/wheel gesture

> Scratch-path note:
> `/tmp/agent` paths in this document are historical.
> Use `~/temp/agent` for current work.

A horizontally virtualized strip renders a windowed subset of columns inside a
`Flickable` whose `viewport-width` is the full strip width,
 and swaps the Slint
model when the visible window changes.
When the model is replaced wholesale (a new `ModelRc`,
 via the generated
`set_columns` setter) while the user is dragging or wheeling,
 the gesture stops
advancing and the strip appears to snap back toward the start.
Mutating one persistent `VecModel` incrementally instead (insert,
 remove,
set-row-data) leaves the gesture undisturbed and scrolling is smooth.

## Symptom

The prototype at `package/file-manager/app/` renders 1200 columns,
of which only the visible window (about 5 to 8) plus prefetch exist in the model
at once.
The columns sit at absolute x inside a `Flickable` with
`viewport-width: strip-width-px` and `viewport-x <=> h-scroll-px`.

With the model rebuilt-and-replaced on each visible-window change:

- Mouse wheel (horizontal):
  the strip scrolls only a fraction of a column,
   then will not advance further.
  The reporter described it as "can't scroll past half a pane."
- Pointer drag (reproduced with the embedded MCP server's `drag_element`):
  each drag scrolls a little,
   then the strip returns to the first column.
  After several drags the strip is still showing column 0.

Setting the scroll position directly (a slider bound to `h-scroll-px`) instead of
gesturing does NOT show the problem:
the strip jumps to a far column and stays there.
Slow-scroll stutter and fast-scroll stale frames also appeared while the model
was rebuilt on every scroll event (before the rebuild was gated to window
changes);
 both are the same wholesale-rebuild cost.

## Root cause

Slint's `Flickable` computes a pointer-drag step from the LIVE viewport position
each move event,
 and warns in its own source that this position is unstable when
inner scrollables are involved.

The drag `MouseEvent::Moved` arm reads the current `viewport_x`,
 adds the mouse
delta,
 clamps,
 and writes it back
(`internal/core/items/flickable.rs:835-872`,
 clone commit `2447c69`):

```rust
// Important constraint: The viewport_y might not be stable, and might jump around
// wildly!
// This is especially the case if a ListView is involved, which will continuously
// update its own viewport_y to keep the current item visible, which can cause the
// viewport_y to jump.
// ...
let current_viewport_position =
    LogicalPoint::from_lengths(viewport_x.get(), viewport_y.get());
// ...
// Do not rely on the existing viewport position to be stable, as e.g. the
// ListView will continuously update it.
// So we cannot calculate the delta in viewport coordinates.
let new_viewport_position = current_viewport_position + mouse_delta;
let new_viewport_position = ensure_in_bound(flick, new_viewport_position, flick_rc);
viewport_x.set(new_viewport_position.x_length());
```

Replacing the whole columns model (a new `ModelRc` handed to the generated
`set_columns`) makes the `Repeater` discard every column instance and build new
ones,
 and each rebuilt column contains a `ListView` (itself a `Flickable`) that
resets to the top.
That is exactly the "inner scrollables make the viewport jump" case the comment
warns about,
 happening every frame the model is replaced.
Because the drag delta is `viewport_x.get() + mouse_delta`,
 a destabilized
`viewport_x` between move events corrupts the accumulated scroll,
 so the gesture
cannot make net progress.

`ensure_in_bound` clamps the position to the content bounds
(`internal/core/items/flickable.rs:929-937`):

```rust
fn ensure_in_bound(flick: Pin<&Flickable>, p: LogicalPoint, flick_rc: &ItemRc) -> LogicalPoint {
    let geo = Flickable::geometry_without_virtual_keyboard(flick_rc);
    let w = geo.width_length();
    let vw = (Flickable::FIELD_OFFSETS.viewport_width()).apply_pin(flick).get();
    // ...
    let min = LogicalPoint::from_lengths(w - vw, h - vh);
    let max = LogicalPoint::default();
    p.max(min).min(max)
}
```

The clamp uses the explicit `viewport-width` (the full strip width),
 so the bound
is huge and is NOT what caps the gesture.

### Earlier hypothesis that was wrong

The first hypothesis was "`set_columns` resets `viewport-x` to 0."
It is wrong.
Slint's out-of-bounds change handler only writes `viewport-x` back when the
property has NO binding
(`internal/core/items/flickable.rs:104-121`):

```rust
let x = (Flickable::FIELD_OFFSETS.viewport_x()).apply_pin(flick);
if *x_out_of_bounds && !x.has_binding() {
    x.set(p.x_length());
}
```

The prototype two-way binds `viewport-x <=> h-scroll-px`,
 so `x.has_binding()` is
true and this reset never fires.
Two independent facts disprove the reset hypothesis:
this binding guard,
 and the observation that a slider setting `h-scroll-px`
directly holds a far position through the same model replacements.
The cap is gesture-position corruption during the rebuild,
 not a viewport reset.

## Verification

Version under test:
the app depends on crates.io Slint `1.17.0`
(`slint`,
 `i-slint-backend-winit`,
 `slint-build`).
Source traced in the Slint clone at
`/tmp/agent/slint-file-manager-assessment-20260705`,
 commit `2447c69`
(1.17 line;
 the `Flickable` gesture architecture cited here is unchanged in the
1.17.0 release,
 whose `flickable.rs` carries the same `Moved`-arm delta logic and
`ensure_in_bound`).

Reproduction harness:
build the prototype with the embedded Slint MCP server and drive it headless.

```bash
# package/file-manager/app
mise run //package/file-manager/app:mcp   # binds 127.0.0.1:9317
```

Then `drag_element` the `AppWindow::strip-flick` element leftward repeatedly and
`take_screenshot`.

Fails (model replaced on window change):
after eight drags the screenshot still shows column 0 and the horizontal slider
thumb sits at zero.

Works (persistent model mutated incrementally):
after eight drags the screenshot shows columns near index 130 and the strip holds
that position;
 column build count over the whole session is about a dozen instead
of thousands.
The author also confirmed real mouse-wheel scrolling is smooth and holds far
positions.

## Verified workarounds

Keep ONE persistent `VecModel<ColumnView>`,
 set on the window once,
 and never
replace it.
Mutate it through `Repeater`/`ModelNotify` instead:

- a horizontal scroll slides only the delta columns in and out
  (`VecModel::insert` / `VecModel::remove`),
  so staying columns and their `ListView`s are never rebuilt;
- a vertical scroll or active-item change rewrites the in-window rows in place
  (`VecModel::set_row_data`);
- a landed background decode refreshes only its owning column,
   flushed once
  scrolling settles.

The implementation is `package/file-manager/app/src/model_sync.rs`
(the `sync_horizontal`,
 `refresh_all_in_window`,
 and `refresh_column` methods),
driven from `src/controller.rs`.

Tradeoffs:

- The controller must diff the desired window against the current one and emit
  minimal insert/remove/set-row-data calls,
   which is more code than handing over a
  freshly built model.
  It is isolated in `model_sync.rs`.
- A disjoint jump (the new window shares no columns with the old,
   for example a
  slider slammed across the strip) still falls back to `VecModel::set_vec`,
   a full
  reset.
  That is acceptable because a slider jump is not an in-progress gesture,
   so there
  is no gesture position to corrupt,
   and the slider path was already shown to hold.

## What does not work

- Rebuilding and replacing the model on every `changed viewport-x`
  (the first attempt).
  It churns the `Repeater` every frame (slow-scroll stutter) and swaps the model
  mid-render (fast-scroll stale frames),
   on top of the gesture cap.
- Rebuilding and replacing the model only when the visible window changes,
   from a
  16 ms frame timer (a "reconcile").
  It removes the per-pixel churn but still replaces the model mid-gesture at each
  column boundary,
   so the drag still returns to the start.
- Capturing `h-offset` before `set_columns` and restoring `h-scroll-px` after it.
  The gesture still loses its accumulated position,
   because the corruption is in
  the drag delta read from the live `viewport_x` during the rebuild,
   not a single
  post-swap value to restore.
- Adding a full-width invisible spacer child to pin the Flickable's content
  extent.
  It does not help,
   because the bound already uses the explicit `viewport-width`
  (`ensure_in_bound`,
   above),
   so the content extent was never the cap.

## Upstream filing decision

`.out-of-scope/` has no Slint exemption (the directory lists no `slint` entry).
Continuing to the 6-constraint check:

1. Is it really upstream's fault?
   No.
   Slint's documented way to virtualize is a model whose changes are delivered
   incrementally through `ModelNotify`;
   `ListView` and `Repeater` are built around that.
   Replacing the whole model on every scroll event is a misuse,
    and Slint's own
   `Flickable` source already documents and defends against viewport instability
   from inner scrollables.
   The framework behaves as designed.

Constraint 1 fails,
 so the remaining constraints are not evaluated and nothing is
filed upstream.
There is no Slint bug here;
the fix is to use the incremental-model pattern Slint intends,
 recorded above.
