# Slint 1.18.1 draws no row of a repeater model that the rendering notifier reset, and nothing asks for the frame that does

Tool:
Slint 1.18.1 from crates.io with the `renderer-femtovg-wgpu` feature and the winit backend.
The checksums are
`slint` `15e477d5ff6fec20909100ff8f7062432ebfbda7a62a0f37d0252aa126f38c83`,
`i-slint-core` `ed2ea15059d70bc31b1a8b1eb4c34f2c5df04cb7e53f36179b43c0261231d646`,
`i-slint-renderer-femtovg` `eb0de504d57cdadf80c2abf7bcb8b7135a3716aee1f41b6ebb90ea93e3649d7c`,
and `i-slint-backend-winit` `dae4fcf1b19e9cb109984d2af87d2b9485388c619265bff71dd523133888bee2`
(`package/desktop-app/ide/Cargo.lock`).
Slint 1.18.1 is the newest release (published 2026-09-21),
and the `master` branch of 2026-10-24 (`8be16937037c49ed2d7262a497399e704f65adab`) keeps the same order of steps.

## Symptom

The IDE draws its code view and its minimap picture into its own wgpu textures
inside the rendering notifier (`bind` in `package/desktop-app/ide/src/native/gpu_tile.rs`),
and the window shows them through `Image` elements in `for` repeaters
(`source-slices` in `ui/app.slint`,
`slices` in `ui/minimap.slint`).
When a tile or a picture needed textures of another size,
the notifier made them and replaced the slice list the repeater shows,
inside the frame that was about to draw it.
That frame drew no row of the new list,
no later frame was asked for,
and an idle window draws no frame by itself:
the code column stayed blank beside the gutter
(the "blank code column" of
`typescript-7-inlay-hints-outside-the-requested-range.md`),
and the first minimap picture of a file was a few frames late.

The same notifier code,
when the list changed in place with the same number of rows
(`VecModel::set_row_data`),
was shown by the frame that changed it.
Earlier notes called that an unexplained difference between the minimap and the code tile.
It is one rule that depends on the row count,
and the code tile crosses a count boundary far more often than the minimap picture.

## Root cause

The frame is made in this order,
and a repeater is only brought up to date at its start.

`WindowInner::draw_contents` makes the repeater instances first,
then starts the window's redraw tracker and runs the renderer's closure inside it
(`i-slint-core-1.18.1/window.rs:1846` and `:1855`):

```rust
crate::properties::evaluate_no_tracking(|| self.ensure_tree_instantiated());
...
Some(self.pinned_fields.as_ref().project_ref().redraw_tracker.evaluate_as_dependency_root(
```

`ensure_tree_instantiated` reaches `Repeater::ensure_updated`
(generated at `i-slint-compiler-1.18.1/generator/rust.rs:1493`),
which is the only place a repeater reads its model property again
(`Repeater::model`,
`i-slint-core-1.18.1/model/repeater.rs:663`)
and builds or removes instances (`:685`).

The femtovg renderer calls the notifier inside that closure,
before it visits the items
(`i-slint-renderer-femtovg-1.18.1/lib.rs:260` and `:301`):

```rust
notifier_fn.notify(RenderingState::BeforeRendering, &api)
...
i_slint_core::item_rendering::render_component_items(
```

A change the notifier makes to the model then lands between those two steps,
and what the frame shows depends on how the change reaches the repeater:

- A row replaced in the model the repeater already shows (`VecModel::set_row_data`) calls
  `RepeaterTracker::row_changed` (`model/repeater.rs:533`),
  which updates the instance on the spot when the model property is not dirty.
  The frame draws the new row.
- A model reset (`VecModel::set_vec`,
  `i-slint-core-1.18.1/model.rs:549`,
  which calls `reset`) runs `RepeaterTracker::reset` (`model/repeater.rs:612`):

  ```rust
  fn reset(self: Pin<&Self>) {
      self.is_dirty.set(true);
      self.inner.borrow_mut().instances.clear();
  }
  ```

  The instances are gone and `ensure_updated` has already run for this frame,
  so the frame visits no instance (`Repeater::visit`,
  `model/repeater.rs:814`,
  walks `inner.instances`).
  The repeater draws nothing.
- A new model set in the window's property leaves the old instances in place
  (`model/repeater.rs:663` is read only by `ensure_updated`),
  so the frame draws the old rows.

Nothing asks for the frame that draws the change.
`evaluate_as_dependency_root` throws away every dependency of the redraw tracker when the frame starts
(`i-slint-core-1.18.1/properties.rs:1361`),
and a dropped `DependencyNode` unlinks itself from the property
(`properties.rs:289`):

```rust
unsafe { *self.holder.dep_nodes.get() = Default::default() };
```

The notifier's change is therefore made while the tracker depends on nothing,
so it marks no tracker dirty and the window never calls `request_redraw` on the adapter
(`WindowRedrawTracker::notify`,
`window.rs:503`).
`Repeater::visit` registers the model and dirty flag as dependencies again
(`track_model_changes`,
`model/repeater.rs:650`,
called at `:819`),
but after the change,
and the end of `evaluate_as_dependency_root` marks the tracker clean (`properties.rs:1365`).
The winit adapter clears its own `pending_redraw` at the start of `draw`
(`i-slint-backend-winit-1.18.1/winitwindowadapter.rs:846`),
so the next draw happens only when some other event asks for it.
The next frame's `ensure_tree_instantiated` builds the instances,
which is why one more frame always shows the change.

An earlier note of this repository said that the test harness draws every change the notifier made in the same
snapshot and so cannot show the blank.
That was wrong:
`Window::take_snapshot` calls the same `draw_contents` and the same renderer
(`i-slint-renderer-femtovg-1.18.1/lib.rs:420`),
and the harness shows exactly the three behaviors above.
Those notes had measured only changes of the `set_row_data` kind,
which the frame does show.

## Verification

The probe is `src/native/repeater_frame_tests.rs` in `package/desktop-app/ide`:
a window with `for shade in shades : Rectangle { background: shade }`
whose model holds one red color,
a rendering notifier that changes the model to blue inside the first frame after it is armed,
and the center pixel of that frame,
of the window's ask for another frame (`take_frame_wanted`,
the harness adapter's `request_redraw`),
and of the next frame.
It runs on the hardware Vulkan device through
`mise run //package/desktop-app/ide:test:native 'test(/repeater_frame_tests/)'`.

Changes that work (the frame that holds them draws them):

- `set_row_data(0, blue)` inside the frame:
  that frame draws blue.
- Any change made before the frame starts (new model,
  `set_vec` of the same or another length,
  `set_row_data`):
  the first frame draws blue.
  This is what `GpuTile::submit_tile` and `GpuTile::submit_minimap` do.

Changes that fail inside the frame:

- `set_vec` to a list of the same length or another:
  the frame draws black (no row),
  the window asked for no frame,
  and the next frame draws blue.
- A new model set in the property:
  the frame draws red (the old row),
  the window asked for no frame,
  and the next frame draws blue.

The same holds in the application's own markup with `Image` rows backed by Vello textures
(`tile_slice_frame_tests` and `minimap_slice_frame_tests`):
with the slices set inside the frame again,
the very next frame draws 0 of the tile's pixels after a hand-over that changes the number of textures,
and all of them when only the heights change.
The shapes tried are in the test
`the_frame_that_follows_a_hand_over_shows_the_tile_whatever_the_number_of_textures`
(two textures,
then three,
then one,
then two,
then two of other heights).

## The real window

The harness and the real window differ only in the adapter around `draw_contents`:
the winit adapter calls it from `WinitWindowAdapter::draw`
(`i-slint-backend-winit-1.18.1/winitwindowadapter.rs:841`)
and decides by `pending_redraw` (`:846`,
`:1849`) whether another frame follows.
The renderer is the same `FemtoVGRenderer::render`.

Measured on 2026-10-24 with a debug build of the IDE in the nested compositor (headless,
hardware Vulkan device,
window 1080 by 1880 logical px at scale 2) on a disposable plain text.
The build logged each change of a slice list
(`show_slices` in `src/native/gpu_tile/types.rs`,
the debug lines `gave the window a new list of slices`,
`reset the window's list of slices to another number of rows`,
and `replaced rows of the window's list of slices`),
and an environment switch in a scratch build (not committed) handed the tile and the picture over inside the frame
again,
with or without the ask for the next frame.

- With the hand-over outside the frame (the shipped code),
  the first tile of a 1500-line text logged the reset
  (`before=1`) before the frame that drew it and the code column was drawn;
  presses on the minimap afterwards replaced one row of a list (`replaced=1`) when the tile or the picture changed
  height,
  outside the frame.
- With the hand-over inside the frame (scratch build,
  2 alternating pairs of runs,
  12 presses each,
  1500 lines),
  every list change the presses made was a replaced row
  (the tile and the picture kept their number of textures),
  and every press was drawn in the screenshot taken 1.5 s later.
  In a 287-line text,
  6 presses (bottom,
  top,
  middle,
  bottom,
  top,
  middle) recorded at 20 frames per second with the ask for the next
  frame switched off made 40 frames each,
  and the mean grey of the code column's crop stayed between 0.5425 and 0.5435 in all 240 frames
  (a value for a blank column was not calibrated in this setup,
  so this measures no change of the picture and does not count blank frames);
  each tile replacement in them was a replaced row (`replaced=1`).
  This is the same-count half of the rule,
  in the real window.
- The count-changing half was not reproduced in the real window on a plain text.
  The only reset inside a frame was the first tile of the window,
  which the notifier follows by asking for the next frame,
  and a second frame followed it in every run
  (the minimap picture's hand-over a few milliseconds later draws one).
  A tile's number of textures follows its height
  (`tile_texture_rows` in `src/shaped_text/limit.rs`:
  at a code width of 1300 px,
  3226 rows by the byte rule and 10752 by the bin rule,
  so a full three-view tile of a 3760 px view is 4 textures and a tile clipped by the start of the text fewer),
  and in the 287-line and 1500-line plain texts every press kept its number.
  The blank code column that the verification agent saw came with TypeScript files whose hint rows change a
  tile's height much more than plain text does,
  and the harness reproduces the count change with painted stacks of chosen heights instead.

So the evidence for the real window is the equality of the code path
(`draw_contents` runs the same steps in both),
the harness repeating the symptom,
and the real-window logs showing that the lists the notifier replaced in the frame had the same number of rows.
What remains unshown is a recorded blank frame with a reset list in the real window.

## Verified workarounds

- Make the textures and the list outside any frame,
  in the call that hands the tile or picture over
  (`Inner::prepare_tile`,
  `Inner::prepare_minimap`),
  as the IDE does.
  The frame that follows draws the scenes into those textures before it draws the items,
  and the list is up to date when the frame starts.
  Tradeoff:
  the device is held only after the first frame,
  so the very first tile of a window cannot use it.
- For that first tile the notifier keeps the in-frame step and calls `Window::request_redraw`
  after it replaced the list,
  so the next frame draws the tile.
  Tradeoff:
  one frame without the tile.
- A change of the same row count made with `set_row_data` is drawn by its own frame,
  so a list that keeps its row count needs neither.
  Tradeoff:
  the count of a code tile's textures follows its height,
  so it cannot be held fixed.

## What does not work

- Asking for one more frame after every in-frame replacement
  (the design of commit `e0b6493f30` in the troubleshooting doc of the blank code column):
  it draws the tile one frame late and the user sees the blank frame.
- Reading the harness's snapshot to prove the fix:
  it does show the blank frame,
  but only a test that counts the pixels of the very frame after the hand-over does
  (`the_frame_that_follows_a_hand_over_shows_the_tile_whatever_the_number_of_textures`).

## Upstream filing decision

`.out-of-scope/` holds no entry for Slint rendering or models
(`cargo-workspace.md` is the only file that names Slint,
for the workspace).
Searches of `slint-ui/slint` issues and pull requests on 2026-10-24 for
`rendering notifier repeater`,
`set_rendering_notifier property`,
and `BeforeRendering` found no report of this behavior.
Issue #1322 (a closed experiment that reported repeater changes to the window for accessibility) and
#9239 (a closed report about `take_snapshot` lagging properties set outside a frame) are the nearest
and say nothing about a change made inside the notifier.
Nothing is added to them.

1. **Is it really upstream's fault?**
   Not as a defect.
   `set_rendering_notifier` documents only that it "allows custom rendering on top or below of the scene"
   (`i-slint-core` `api.rs`),
   and the dependency model above follows from how the tracker is built.
   The undocumented part is that models and properties must not change inside the callback.
2. **Can upstream fix it?**
   Yes.
   The renderer could run `ensure_tree_instantiated` again after the notifier returned,
   or the docs could state the rule.
3. **Are they supporting this use case?**
   The notifier exists for custom rendering,
   and the femtovg wgpu renderer's notifier support was added in issue #10640.
   Driving model changes from it is not shown in any documented example.
4. **Would the repo welcome our contribution?**
   Not checked in `CONTRIBUTING.md`,
   because the audit stops at constraint 1.
5. **Will they likely fix it?**
   No signal either way.
6. **Have we prototyped a minimal fix?**
   No.

Decision:
do not file.
The IDE works around the behavior by design,
and the finding is a documentation gap,
not a defect that the audit shows is upstream's;
constraint 1 holds only as a documentation request and constraint 6 is not met,
so nothing was prototyped.
The draft below is kept for a future session that sees upstream signal change,
and it is **do not file as-is**:

~~~md
Title: Document that changing a model or a property inside the rendering notifier is not drawn by that frame

Slint 1.18.1, renderer-femtovg-wgpu. In `BeforeRendering`, `VecModel::set_vec` on the model of a `for` repeater makes
that frame draw no row, and a new model set on its property makes it draw the old rows. No frame is asked for,
because `WindowInner::draw_contents` clears the redraw tracker's dependencies before the renderer's closure
(`properties.rs` `evaluate_as_dependency_root`) and the repeater is brought up to date only in
`ensure_tree_instantiated`, which runs before the notifier. `set_row_data` of an existing row is drawn. A note in the
`set_rendering_notifier` docs would save embedders (custom textures shown through `Image` rows) a day of measuring.
~~~

This draft is local and has not been posted.
