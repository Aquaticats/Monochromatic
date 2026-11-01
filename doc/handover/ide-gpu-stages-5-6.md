# IDE GPU migration, stages 5 and 6: state at the stop

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/cli/nested-wayland-session` is now `package/nested-wayland-session/cli`;
> `package/desktop-app/ide` is now `package/ide/app`.

Branch `worktree-agent-a0880539282a3162b`, base `63991e9923`.
Work was stopped on the coordinator's request so the machine could be rebooted.
Measurements and design are in `doc/planning/ide-gpu-snapshot-harness.md`, section "Stages 5 and 6".

## Done

- The minimap is a Vello scene drawn by the code view's one renderer, in chunks of at most 512 rows,
  copied into a slice texture; the CPU painting code and the `swash` dependency are deleted.
- All 45 `minimap-*` source guards fail when removed and pass restored (disposable copy).
- A missing redraw request after a slice was found by screenshots and fixed (`src/native/minimap/picture.rs`).
- README of the package and `doc/planning/ide-vello-text.md` describe the new minimap.

## Gates

- Passed on the snapshot before the last two test commits: `lint` (0 findings), `test` (790),
  `test:minimap`, `test:annotations`, `test:bin`, `build`, `inspect:software-renderer`.
- `test:native`: passed 265 of 265 on one snapshot; failed once on that snapshot's predecessor
  (`an_external_edit_of_fifty_thousand_lines_shapes_only_the_line_it_touches`).
- Never ran on the final tip: `lint`, `test:native`, `test`, `test:bin`, `build`, `inspect:software-renderer`
  (the run was stopped).
- The owner's rule: one gate run per committed snapshot, no repeated full-suite batches, no load generation.

## Not done

- The final-tip gates above.
- Chunk seam check in a screenshot and a window taller than the compositor's 3,704 px
  (closed on 2026-10-22: see the section "Scale 2, the light scheme and the seams, measured in the compositor").
- A quiet-host cadence run of `big.rs` after the redraw fix.
- The external edit test is a timing test that failed in about 4 of 32 full runs before its last fix
  (lines that never wrap); that fix passed alone 4 of 4 and is not proven in full runs.

## Next steps

1. `mise run //package/desktop-app/ide:lint`, then `test:native` once, then `test`, `test:bin`, `build`,
   `inspect:software-renderer`.
2. If the external edit test fails again, read the `EDIT before ... after ...` line it prints
   (picture window and statistics before and after the edit).
3. Look at a 4x crop of a chunk boundary in a window of about 3,700 px.
4. Owner questions are listed at the end of the "Stages 5 and 6" section.
5. The baseline worktree `.claude/worktrees/ide-gpu56-baseline` (detached at `origin/main` `625bcecb61`) can be removed
   with `git worktree remove` after the cadence comparison is no longer wanted.

## Stacked textures

Owner decision of 2026-10-21: a window taller than the device's texture limit stacks several textures,
so the minimap covers the whole window.
Built on branch `worktree-agent-a4d54af4e63faea6b` (off `a7f8523f0c`).

### What changed

- `src/minimap/plan.rs`: `texture_rows` (the most rows of one texture: the device's side, or fewer when its bytes
  would pass the 64 MiB rule of the source tile) and `texture_runs` (groups the chunks of a picture into textures of
  whole chunks, none taller than that).
- `src/native/minimap/request.rs`: the key no longer caps the bands at one texture (`most_bands` is kept and tested,
  and no longer used where the key is made); the key now holds the texture limit, so a changed limit posts a picture.
- `src/native/gpu_tile/slice.rs` (`draw_stack`), `gpu_tile.rs`, `gpu_tile/types.rs`: the picture is rendered chunk by
  chunk and copied on the GPU into as many textures as `texture_runs` names, each with its own `Image`;
  `Picture::Minimap` is gone from the single-texture targets.
  A recorded limit below one chunk (512 rows), as a test may set, gives one chunk per texture;
  a device whose own side is below one chunk is refused with the existing "does not fit a texture" message.
- `ui/minimap.slint`, `ui/app.slint`: `minimap-image` is replaced by a `[MinimapSlice]` model (image, first row,
  height, all in physical pixels) drawn by a `for` loop at whole-pixel rows, so the textures abut without a gap.
  `minimap-picture-width`, `-height` (now the textures together) and `-top` are kept for the tests and pointer code.
- Tests: `tall_window_minimap_tests.rs`, `minimap_seam_tests.rs` (new), `tests/minimap_scene.rs`.
  The old test `a_picture_taller_than_the_device_limit_is_cut_to_the_limit` asserted the defect
  and became `a_picture_taller_than_the_device_limit_covers_the_whole_track`.
  `the_minimap_of_a_tall_window_is_a_picture_of_glyphs_not_blocks` now bounds each texture by the limit
  instead of the whole picture.

### Evidence

- Failing before the fix (commit `10fb0e26ef`, filter `test(/tall_window_minimap/)`):
  `the picture is 8064 rows of a track of 11968`.
  The 12,000 px window over the harness device capped to 8,192 gave one texture that ended at row 8064.
- Passing after (commit `27c8877907`): the same test passes with two textures that are each at most 8,192 rows,
  together as tall as the track, with no error and no blank chunk.
- Seam and three or more textures, `textures_that_stack_show_the_picture_one_texture_shows`:
  the picture of a 3,500 px window (one texture) is drawn again at a texture limit of 1,536 (three textures),
  1,024 (four) and 512 (seven).
  Every texel read back from the stack equals the texel of the one texture to within the renderer's noise of one level,
  the rows on both sides of each seam are inked,
  and the window's own frame, read whole, shows the same column of minimap pixels.
  Positive controls: the stack shifted by one row is not equal, in the textures and in the frame.
- The guard fails when broken (rule GFP, committed test, fault applied to `slice.rs`, then restored):
  a placement of every texture after the first one row too low fails on
  `the composited column differs at limit 1536`;
  a copy of the chunks one row early inside a texture fails on
  `the stack of 3 textures differs from the one texture at limit 1536`.
  A first version of the frame check read only the 660 rows of the harness frame and a one-row placement fault
  passed it; the harness frame is as big as the size the adapter was told (`Window::set_size`), not the size a
  resize event gives, and the test now sets it (commit `d2c7e4aa5c`).
- `minimap_seam_tests.rs`: the chunks of a picture of about 1,500 rows (three chunks, two seams), stacked,
  equal one unsplit render of the same rows at scales 1 and 1.5.
  Positive control at scale 1.5, where a seam falls through a line:
  chunks that keep only the items inside their rows differ from the unsplit render.
  At scale 1 a 512 row seam is a code row edge and the control would lose nothing, so it is not asserted there.
- Unit tests of the grouping in `tests/minimap_scene.rs`: limits that are and are not multiples of a chunk, a limit
  of exactly one chunk, one under a chunk, an empty picture, and the byte rule for a wide picture.

### Not exercised

- A device whose texture limit is above 8,192 (16,384 is common) or below it: the harness device is capped to 8,192,
  and three or more textures were made by lowering the recorded limit (`set_texture_dimension`) after the first frame,
  not by a window of 17,000 px or more.
- A composited frame at a fractional display scale or at scale 2 with a stack of textures:
  the placement is whole physical pixels by construction, but only scale 1 was read back
  (closed on 2026-10-22 at scale 2, with two textures in a composited frame of the real device:
  see the section "Scale 2, the light scheme and the seams, measured in the compositor"; fractional scales not tried).
- The cost of a very tall window: a 12,000 px window renders about 24 chunks in the frame the picture arrives in,
  on the window thread, as the cut picture rendered about 16 before.
  Its frame time was not measured.
- The densest text (`minimap_dense_tests.rs`) was not stacked.
- No look at the minimap of a tall window in a real or nested compositor window
  (closed on 2026-10-22: 1100x3700 at scales 1 and 2 and 1100x8100 at scale 2, headless).
- The recorded limit is read when a picture is requested and when it is drawn;
  a device change at run time (a new device with another limit) is posted again through the key,
  but a window that moves to another GPU was not tried.

### The scroll hitch the first version of the stack caused

The orchestrating session's cadence A/B (release, 1100 by 660, 50,000 line `big.rs`, scroll scenario, recorder at 60 Hz)
showed one identical (missed) recorded frame at slot 35
in 11 of 13 runs of the stacked tree and in 6 of 27 of the baseline.

#### Cause

- A picture of the first stacked version replaced the list of textures that the markup draws
  (`set_minimap_slices` with a new `VecModel`) every time one arrived, for a picture of the same shape as well.
  The markup's `for` loop then dropped and made every `Image` item again inside the frame of a scroll step.
  The old code set one `Image` once and later only changed numbers.
- The window thread cost does not show it: the frame that takes in a picture lasts 5 to 8 ms in the stacked tree and in
  the baseline alike, and the frames after it 2.4 to 3.5 ms in both, so the loss is outside the window thread's own work
  (the toolkit's image items, the compositor), and it was not located further.
- Intervention, not inference: the same release binary with only this change (commit `3247690ff8`, `show_slices` in
  `src/native/gpu_tile/types.rs`) brings the count down to the baseline's.
- Not the cause: the request cadence (ids and posts are the same in both trees over the first 2 s, 50 ms after the
  scroll starts for the first one), the texture limit in the key (no re-post), the grouping (one texture here),
  and a new texture or `Image` per picture (the textures are reused while their sizes stay).

#### Counts (identical recorded frames out of 119 in the input window, one run each)

Runs whose highlighting failed (`syntax parse failed ... configured timeout was exceeded`, an error banner of 30 px
shows and the first picture request comes at 183 ms instead of 50 ms, so the hitch moves from slot 35 to slot 43) are
counted apart: they are a load-dependent change of the fixture, not of the tree.
The coordinator's earlier counts include such runs on both sides.

- Question 1, baseline (`origin/main`) against the first stacked tip, 6 pairs interleaved:
  baseline 0 of 6 runs with an identical frame;
  stacked 6 of 6 (4 of the 4 whose highlighting worked, at slot 35 in all four; the two that failed highlighting at 43).
- Question 2, baseline against the fix, 10 pairs interleaved:
  baseline, highlighting worked in 9 runs: 2 with an identical frame (slot 35 once, slot 99 once);
  fix, highlighting worked in 8 runs: 2 with an identical frame (slots 67 and 130, and 67; none at 35).
  The floor is the baseline's own rate, 2 of 9 runs (no run of the baseline against itself was made).
  The longest identical run is 1 in every run.
- The extra frame that the baseline has too: `picture.rs` asks for a redraw when it hands a picture to the GPU,
  so a frame lands between two scroll steps; it is on `origin/main` and was not changed
  (single instrumented runs of the baseline with highlighting show the same timeline as the stack: request at 50 ms,
  picture at 70 ms, a 5.5 ms frame, then a hitch at slot 35 in that run).

#### Guard

`a_picture_of_the_same_shape_changes_the_rows_of_the_list_and_never_replaces_it` counts the lists the window is given
(`SLICE_LISTS_SET`) across a picture of the same bands, forced by a resize of one pixel.
It failed while every picture made a new list (commit `ee59d9630f`, `the window was given a new list of textures`)
and passes with the fix.
The source guard `minimap-slices-in-place` removes the fix in a disposable copy.

#### Not done here

- The remaining recorded identical frames in both trees (slot 35 once, slots 67, 99, 130) were not explained;
  the baseline has them too, at the same rate as the fix.
- The cadence was measured with the nested compositor only, on a host that other sessions load;
  the pressure stall values are in `~/temp/agent/a4d5-q1.jsonl` and `a4d5-q2.jsonl`.

#### Gates of the fix

Run once each on tip `105686effa`:
`lint` 0 findings; `test:native` 274 passed, 5 skipped; `test` 797 passed, 1 skipped; `test:bin` 60 passed;
`build` finished; `inspect:software-renderer` not linked, as before.
`inspect:source-guards` for `minimap-slices-in-place` only (1 of 148 guards): baseline passes, removed fails with
`the window was given a new list of textures`, restored passes.

### Gates of the stacked textures

Run once each, in this order, on tip `89924c2a12` (the tree that this section's commit follows):

- `lint`: 0 findings (clippy, rustfmt, monochromatic-lint including the README and the Rust doc comments, slint).
  Earlier runs on the way found shadowed bindings, avoidable closures, a file past the 300 line limit
  (the test-only readers moved to `src/native/gpu_tile/probe.rs`), and semantic line breaks; all fixed.
- `test:native`: 273 passed, 5 skipped (not selected by the task), 0 failed.
- `test`: 797 passed, 1 skipped, 0 failed.
- `test:bin`: 60 passed, 0 failed.
- `build` (release): finished.
- `inspect:software-renderer`: the software renderer is not linked, and both positive controls reach it.

Not run: `inspect:source-guards` (it builds disposable copies of the package for every guard).
Its entries for the minimap were updated by hand:
`minimap-bands-fit-the-device` became `minimap-bands-uncapped`,
`minimap-chunk-copy-row` follows the new test and indentation,
and `minimap-texture-limit-key`, `minimap-texture-runs` and `minimap-texture-top` are new.
None of the five was executed, so each is unproven until that task runs.
The two faults tried by hand (see Evidence) are the same as `minimap-texture-top`
and a variant of `minimap-chunk-copy-row`.

## The extra frame at a picture hand-over

Session of 2026-10-21 on branch `worktree-agent-a3a543d4c80040d25`, base `e2f7ee81f2`.
It answers the open item of "The scroll hitch the first version of the stack caused":
why one scripted-scroll frame is a missed frame at slots 35, 67, 99 or 130 in both trees,
a period of about 32 slots.

### Cause

- `src/native/minimap/picture.rs` called `request_redraw()` the moment it handed a picture to the GPU.
  In a scroll the window already draws a frame at every wheel step, and the GPU draws a waiting picture inside the next
  frame, so the request added a second frame between two steps.
- The period is the picture cadence, not a beat with the recorder.
  In the instrumented run `ide-cadence-RjmA8F` the hand-overs fall at slots 34, 66, 94, 98, 125, 129, 156 and 160,
  and the stall follows the first one at slot 35.
  Not every hand-over stalls (that run stalls after 34 only); where in the tick the extra frame lands decides it.
- What the second frame does in the recording, measured with the nested compositor's recorder
  (it captures at each tick and sends the frame callbacks right after;
  `package/cli/nested-wayland-session/src/recorder.rs`):
  the text area does not move for one tick (a shift of 0 px where every other tick shifts 25 px),
  and the later ticks go on shifting 25 px, never 50.
  The window log of the same run shows one wheel event per tick, and at the hand-over a
  "set a minimap picture" line, a "winit asked for a redraw" in the same millisecond, and a 5.4 ms frame that no wheel
  event started.
- Why the compositor shows the next scroll frame one tick later is a reading, not a measurement:
  the extra frame takes the present slot of the tick, so the following scroll frames queue one behind.
  The intervention below shows that the extra frame is what produces the stall; the queueing is not shown.

### The cadence tool undercounts this

The hash of the whole frame (`identical` in `inspect:cadence`) hid the stall in most runs.
In `ide-cadence-RjmA8F` slot 35 is "changed" by two pixels (x 1015 to 1081, y 323 to 618) and the text area is
unchanged; in `ide-cadence-y1JxNU` slot 35 is zero pixels different.
The new minimap picture is drawn into the extra frame, so the frame differs by a few pixels or by none,
depending on whether the new picture equals the old one.
The measurement therefore also counts a stalled tick: a tick of the input window in which the text area
(x 330 to 900, y 160 to 480) moved 0 rows against the frame before, found by matching row hashes at shifts of
-150 to 150 (scratch `mm-stall.mjs`, one 25 px step per tick is expected).
Positive control: the runs of the old behavior stall at the first hand-over in 20 of 20 runs (slot 35 in 16, slot 43 in
4), and the hash tool saw an identical frame there in 7 of those 20.

### Fix

The window no longer asks for a frame at the hand-over.
It starts a single-shot timer of 20 ms (`FRAME_WAIT`, longer than a frame at 50 Hz) and asks for a frame only if the
picture is still waiting then (`GpuTile::minimap_waiting`).
A scroll draws the picture in its next step and nothing is asked for; an idle window gets its frame 20 ms later.

- Guards: `a_picture_that_a_frame_draws_asks_for_no_frame_of_its_own` (commit `727242fa73`) counts the frames asked for
  (`PICTURE_FRAMES_ASKED`) across a hand-over and a drawn frame and expects none;
  `a_picture_that_no_frame_draws_asks_for_a_frame` expects exactly one in an idle window.
  Source guards `minimap-picture-waits-for-frame` and `minimap-picture-asks-when-idle` remove each half.

### Counts

Release build, `big.rs`, 1100x660, scroll scenario, recorder at 60 Hz, one run at a time, alternating pairs.
The measured binary carried a temporary environment switch (`IDE_REDRAW_NOW` restored the immediate request);
it is removed in the final tip, which was measured again below.
"First hand-over stall" is a stalled tick at slots 33 to 37, or 41 to 45 when highlighting timed out under load
(a 30 px banner moves the first hand-over, as in the section above).

- A against A (5 pairs, both with the immediate request): first hand-over stall in 5 of 5 and 5 of 5 runs (slot 35
  in 9 of 10, slot 43 in 1); stalls per run 2, 2, 3, 2, 2 and 2, 2, 3, 2, 2 (11 and 11);
  runs with a hash-identical frame 2 of 5 and 1 of 5.
  This is the floor: the stall is the same in every run, and the hash count alone moves between 1 and 2 of 5.
- Immediate request (A) against the fix (B), 10 pairs:
  first hand-over stall A 10 of 10 (slot 35 in 7, slot 43 in 3), B 0 of 10;
  runs with a hash-identical frame A 6 of 10 (7 frames), B 0 of 10;
  stalls per run A 1, 2, 2, 2, 2, 1, 3, 1, 3, 2 (19), B 2, 2, 1, 1, 1, 2, 1, 2, 2, 1 (15).
  Both differences are far past the floor.

The final tip, without the switch (3 pairs of the same binary, 6 runs):
no stalled tick at the first hand-over in 6 of 6, and no hash-identical frame in 6 of 6.

### Stalled ticks that remain

Stalls per run fall from 1.9 to 1.5 and do not reach zero.
The ones left (slots 50 and 98 in most runs, sometimes 74, 120 or 142) are not picture hand-overs.
A run logged at debug level (`ide-cadence-vQjf3g`, the final binary) shows the same pattern at them:
"window thread applied a finished tile" and a "winit asked for a redraw" that no wheel event started, then a 5 ms frame
(+320 ms and +1117 ms after the first wheel event, the stalls of that run at slots 50 and 98).
`src/native/render/present.rs` (the `request_redraw()` after `submit_tile` and after `submit_sign`)
asks for a frame at the tile hand-over the way `picture.rs` did.
That the stall at slot 50 is this one is read from the log, not shown by an intervention; it was not changed.
(The later session in "The extra frame at a tile hand-over" ran the interventions: the redraw request named here is
not the cause, the swap of the tile is.)

### Limits

- Measured only in the nested compositor, with its recorder as the clock, on a host that other sessions load
  (pressure stall average over 10 s between 0 and 16 during the runs).
  A real compositor (KWin, Mutter) was not tried, so the visible effect there is not shown.
- Scroll scenario only. The drag scenario was not run; a held drag also hands pictures over.
- A picture that arrives in an idle window is drawn up to 20 ms later than before. Not measured as a latency.
- On a display slower than 50 Hz the wait ends before the next frame, and the old extra frame comes back.
- The first picture of a file, a resize and a theme change hand a picture over in an idle window;
  the second test covers the request, and a look at the screen after each was not made.

### Gates

Run once each on the tip that follows the last code commit (`style(ide): rustfmt the picture frame code`):
`lint` 0 findings (a first run found rustfmt diffs, fixed with `format:rust`);
`test:native` 277 passed, 5 skipped (171 s);
`test` 797 passed, 1 skipped;
`test:bin` 60 passed;
`build` finished;
`inspect:no-software-renderer` passes (the software renderer is not linked;
`slint/renderer-software` and `slint/mcp` do reach it);
`inspect:source-guards` for the two new guards (2 of 150): baseline passes, removed fails, restored passes, for both.
The other 148 guards were not run.

## The extra frame at a tile hand-over

Session of 2026-10-21 on branch `worktree-agent-a0b5a3cdf2d733095`, base `b4e9063c3a`.
It answers the open item of the section "The extra frame at a picture hand-over":
the stall at slot 50 of the scripted scroll, which the previous session read from a log and attributed to the
`request_redraw()` calls after `submit_tile` and `submit_sign` in `src/native/render/present.rs`.

### Method

One release binary carried temporary environment switches (commit `ddca86e067`, removed in `d8ab9f62f2`):
`IDE_X_PICTURE=now` restored the immediate picture request (the old behavior, tree A), `IDE_X_TILE=none` removed the
`request_redraw()` after `submit_tile`, and `IDE_X_HOLD=1` was the fix below.
The measure is the frame-shift probe (ticks of the input window in which the text area moved 0 rows), one app run at a
time, alternating order, release, `big.rs`, 1100x660, scroll scenario, frames deleted after each run.
The probe sees the stall that the whole-frame hash misses; its positive control is the restored picture request,
which stalls at slot 35 in 6 of 6 runs.

### Interventions

- A against A, the floor (6 pairs, both with the switch unset): slot 50 in 6 of 6 and 6 of 6 runs;
  stalls in total 10 and 12 (other slots 74, 98, 120, 141, 142).
  The stall at slot 50 does not move between runs of one binary.
- Picture request restored (tree A, 6 runs): slot 35 in 6 of 6, slot 50 in 0 of 6, 13 stalls in total.
  The stall does not move to the next extra frame; the extra frame at slot 35 changes the timing of the whole run.
  In the debug timeline of such a run (`~/temp/agent/tile-dbg-p.timeline`)
  the tile is applied 1 ms after a wheel event, inside the frame the wheel started, so it adds no frame.
  Why the picture's frame moves the tile to such a moment is a reading from that one run (the 20 ms poll of the tile
  worker against the 16.7 ms steps drifts), not a measurement.
- The request after `submit_tile` removed (`IDE_X_TILE=none`, 6 runs): slot 50 in 6 of 6, 10 stalls in total.
  The explicit request is not the cause.
  The timeline of such a run (`~/temp/agent/tile-dbg-t0b.timeline`) still shows "winit asked for a redraw"
  4 ms after the wheel event,
  at the moment of "window thread applied a finished tile", and a 5 ms frame that draws the tile; the next wheel event
  follows 17 ms after the previous one.
  Something in the swap asks for that frame; the setters that `show` runs for the tile (the image's size and place,
  the selection rows as a new model, the caret) are the candidates, and which one does it was not isolated.
  What the intervention shows is that leaving the whole swap for the next step removes the frame and the stall.

### Cause

The poll timer of the tile worker (`src/native/tile.rs`, 20 ms) swaps a finished tile in at a moment unrelated to the
scroll steps (16.7 ms apart).
The swap makes the toolkit ask for a frame of its own, so between two steps there is a second frame,
which is the same effect as the picture hand-over's.
The text area then does not move for one tick.

### Fix

A finished tile is left for the next scroll step (`step_due` in `src/native/tile.rs`).
The timer does not poll while the last step is less than `STEP_WAIT` (20 ms) old; the step itself polls, at the end of
`on_viewport_changed` (`src/native/viewport.rs`), so the setters run in the frame that step draws.
With no step coming, the timer takes the tile 20 ms later, so an idle window does not keep the old tile.
The explicit `request_redraw()` calls stay: they draw a tile that changed no property the window watches.

The first version polled at the start of `on_viewport_changed`, before the step had placed the view.
It made the native test `an_external_edit_of_fifty_thousand_lines_shapes_only_the_line_it_touches` fail 20 of 20 in the
container (the baseline passed 20 of 20): the picture of the file before the edit had 190 lines where the baseline's
has 194, because the finished tile and the pass it carries were taken against a state that the step was about to
change.
Polling after the step's own work restores the order the timer has, and the test passes.

- Guards: `a_tile_that_finishes_between_two_steps_is_left_for_the_next_step` and
  `a_tile_that_finishes_with_no_step_due_is_taken_by_the_poll` (`src/native/tile_step_tests.rs`).
  Source guards `tile-left-for-next-step`, `tile-taken-by-the-step` and `tile-taken-by-the-poll-when-idle`.

### Counts

- The fix with the poll before the step (`IDE_X_HOLD=1`, 8 pairs against the unset binary): slot 50 in 8 of 8 against
  0 of 8; stalls in total 11 against 0.
- The fix with the poll before the step as the final binary against the baseline `b4e9063c3a` (8 pairs): 0 against 13
  stalls in total, slot 50 in 0 of 8 against 8 of 8.
  This binary failed the native test above and is not the shipped one.
- The final tip (`2fba9c567c`, poll after the step) against the baseline (8 pairs): 1 against 13 stalls in total;
  slot 50 in 0 of 8 against 8 of 8.
  The one stall is at slot 33 in a run of the fix (the baseline shows none there in 36 runs of this session);
  one run of the fix and one of the baseline also show a row shift of 50 px in a tick, a late frame followed by a
  catch-up.
  Whether the stall at 33 is the host or the fix was not decided: no more pairs were run.

### What remains

- Stalls at slots 74, 98, 120 and 142 of the baseline are gone in the fix runs (one stall, at slot 33, in 24 runs
  of the three passes).
  They were not examined one by one; in the timeline of one baseline run the stall at 74 follows a tile applied 1 ms
  after a wheel event, so not every stall of the baseline is an extra frame.
  The fix removes them together with slot 50 in these runs, which is a count, not a cause.
- An input slower than 50 Hz (a step later than `STEP_WAIT`) takes the old behavior: the poll swaps the tile in
  between two steps.
- A tile that finishes within `STEP_WAIT` after the last step of a scroll waits up to one more poll period (20 ms);
  this latency was not measured, and the second test covers only that the poll takes the tile.
- The drag scenario, display scales other than 1, a real compositor and the wrap indicator's own redraw
  (`submit_sign`, once per display scale, before the input) were not tried
  (scale 2 closed on 2026-10-22 for the picture, the input and the scroll step, not for the stall probe:
  see the section "Scale 2, the light scheme and the seams, measured in the compositor").
- The pair budget of ten per question was exceeded for the fix (three passes, 8 pairs each: the first pass with the
  switch, the final binary before and after the order change), because the order change was a code change.

### Gates

Run once each on tip `2fba9c567c` (documentation commits follow it):
`lint` 0 findings (a first run found rustfmt diffs in the module order and four semantic line break findings, fixed);
`test:native` 279 passed, 5 skipped (the earlier run on the previous tip failed the edit test above);
`test` 797 passed, 1 skipped; `test:bin` 60 passed; `build` finished;
`inspect:no-software-renderer` passes (not linked; the two features reach it);
`inspect:source-guards` for the three new guards (3 of 153): baseline passes, removed fails, restored passes, for all
(`tile-taken-by-the-step` shares the baseline of `tile-left-for-next-step`).
The other 150 guards were not run.
The counts above come from `~/temp/agent/tile-ab-1.jsonl`, `tile-ab-2.jsonl`, `tile-ab-3.jsonl` and `tile-ab-4.jsonl`.

### Idle latency of the 20 ms wait and rare stalls elsewhere

Session of 2026-10-22 on branch `worktree-agent-a0d7b8bed3990e6c8`, base `b6727b7b86` (the merged fix).
It takes the two items of "What remains" that were open: the added latency of the wait, and the stalls away from
slot 50.
No product code changed (the temporary switches of commit `0eb036e425` were reverted in `52d043318a`), so no gate ran.

#### Method of the session

One release binary carried the temporary switch `IDE_X_STEP_WAIT_MS`: unset is the merged 20 ms wait,
`0` turns the wait off and also the poll at the end of `on_viewport_changed`
(the code before the fix; it stalls at slot 50 in 16 of 16 runs below, which is its positive control),
`150` is a wait that must move the result by about 130 ms.
The binary also logged "source tile worker took a job" and "finished a build" at debug level.
Every number comes from one app run at a time in the nested compositor (1100x660, release, `big.rs`), each run a
capped unit (`CPUQuota=600%`, `MemoryMax=40G`), arms alternating per pair, artifact directories deleted after each run.
Logs were at debug level for the frame, tile, tile build and wheel targets only, in every arm.
The host load average stayed between 7 and 10 from other sessions; the counts below are over runs, not single runs.
The driver and the analysis are `~/temp/agent/lat-run.mjs`, `lat-stats.mjs`, `lat-extra.mjs` and `lat-stall-stats.mjs`;
the rows are in `~/temp/agent/lat-q1-s16.jsonl`, `lat-q1-sweep.jsonl`, `lat-explore2.jsonl`, `lat-q2a.jsonl` and
`lat-q2b.jsonl`.

#### Idle latency of the wait

The scenario is a scroll of 2 wheel steps with 1500 px each (`inspect:cadence scroll ... 2` with
`IDE_CADENCE_SCROLL=3000`), then nothing.
The view leaves the tile at once, so the picture shows no text until the new tile is drawn; the time asked for is the
last wheel event (compositor log) to the end of the first frame the toolkit finishes after the tile is applied
(app log).
A gentle scroll that stays inside the old tile's band changes no visible pixel at a hand-over, so this latency is only
visible when the view has outrun the tile.
A scroll of a single wheel step cannot be made with the compositor's command: 20 of 20 pairs with `steps=1` built no
tile at all, so that case is not measured.
Slower scrolls that end at a band exit produced chains of tiles (a discarded tile, then a re-center 33 ms later, then
the settle of the row measurement about 350 ms after the last step; 8 pairs in `~/temp/agent/lat-explore.jsonl`),
and the 20 ms question did not separate from them;
the 2-step scenario has one tile (`appliedAll` 1) in every run.

- The positive control, wait 150 ms (4 runs, all in `lat-explore2.jsonl`): last wheel to frame 165.1, 165.8, 165.9 and
  166.8 ms, against about 30 ms with the wait off; the measure sees a 136 ms move.
- The spread of unchanged input, wait off (47 runs, step 16 ms in 20 and 8 to 24 ms in 27): last wheel to frame median
  29.7 ms, quartiles 28.8 and 30.3, range 25.8 to 33.4.
  A difference of more than about 3 ms is not noise.
- The merged binary (47 runs, same inputs): median 30.2 ms, quartiles 29.5 and 49.3, range 28.3 to 51.0.
  The distribution has two modes: 28 runs at 28.3 to 32.9 ms, and 19 runs at 38.8 to 51.0 ms.
  In 18 of the 19 the poll logged "leaving a finished tile for the next scroll step" after the last wheel event;
  in none of the 47 runs with the wait off.
  Paired by order, the merged binary minus the wait off has a median of +2.2 ms with step 16 ms (20 pairs, quartiles
  -0.2 and +19.5) and +1.4 ms in the sweep of step lengths (27 pairs, quartiles -0.1 and +19.2); the slow runs add
  about 20 ms (22.3 at most).
- The recorder agrees in the 20 runs with step 16 ms: the first frame that differs from the one before it, counted
  from the last input slot, is slot 1 in 11 merged runs and slot 3 in 9; with the wait off it is slot 1 in 18 and
  slot 2 in 2 (whole-frame hash, which the minimap's picture also changes, and a slot is 16.7 ms).

The median is not above one display frame (17 ms): the added latency is about 1 ms at the median, so it is not a cost
by that measure and no fix was made.
It is a cost in a minority: in about 4 of 10 hand-overs of this kind the tile waits one more poll period (+20 ms, a
little more than one frame).
The mechanism is a reading from the timeline of one run, not an intervention: the two last wheel events arrive in the
same instant, the first requests the tile and starts the poll timer, the second is handled a few milliseconds later,
so the poll 20 ms after the timer start finds the last step less than 20 ms old and leaves the tile, and the next
poll is a full period later.
A poll that leaves a tile could ask for one more look when the wait ends (a single-shot timer for the rest of
`STEP_WAIT`), which would cut the tail to about 20 ms after the last step in every case; this was not built, because
the median criterion of this question was not met, and a one-shot timer would be a second timer path next to the
poll.

#### Stalls away from slot 50

Scroll scenario, 120 steps, frame-shift probe (`mm-stall.mjs`), alternating order, debug log kept per run.

- Fix switched off (16 runs): 32 stalls in total, per run 2,2,2,2,1,2,2,3,2,2,2,2,2,2,2,2;
  slot 50 in 16 of 16 runs (positive control), slot 74 in 11, slot 98 in 3, slots 120 and 142 in one each;
  one run shows a row shift of 50 px in a tick (slot 131).
- Merged binary (16 runs in the alternating pairs, then 40 more merged runs, 56 in all): no stall in any run, and no
  row shift other than one step.
- A detector at the application level, from the debug logs: redraw requests that no wheel event started, inside the
  window of the input.
  With the fix off, all 16 runs have one (the tile taken by the timer, about 320 ms after the first wheel event,
  which is slot 50); three runs have more, one of them 10 in all, six in a row spaced
  16.7 ms apart (1069 to 1152 ms after the first wheel event) that follow no tile hand-over;
  bursts like that look like late delivery of input, which was not examined further.
  With the merged binary, 0 in 56 runs, and all 560 tiles applied in the window were applied inside a scroll step
  (within 2 ms after a wheel event); the poll left a tile 280 times and the next step took it.
  The positive control of this detector is the same 16 fix-off runs.
- The earlier stalls at slots 33 and 72 (one run each of 16 merged runs, as reported to this session, and slot 33 in
  one of 24 runs in the subsection "Counts") did not occur in these 56 runs,
  so no timeline of a stalled run of the merged binary exists, and the question
  "the host or the fix" is not decided by an intervention.
  What the 56 runs show is that the mechanism of the fix, a tile taken between two steps, did not occur once, which
  does not support the fix as the cause; with no stall to read, the host is not shown either.
  This session's 0 in 56 puts the upper bound of the per-run rate near 5% (rule of three), below the 2 of 16 seen
  in the other session, so the rate itself is not steady between sessions, which fits the host but does not show it.
  Synthetic host load, which could raise the rate of a host stall on purpose, is forbidden in this repository, so it
  was not tried.

#### Not run

- Neither the drag scenario, a display scale other than 1, a real compositor, nor a slower input than 50 Hz.
- No stalled run of the merged binary was read, so no fix for a stall away from slot 50 was made or tested.
- The code before the fix was reproduced by the switch (wait and step poll off), not by a build of `b4e9063c3a`;
  the stall at slot 50 in 16 of 16 runs is the check that the two agree.
- Gates (`lint`, `test:native`, `test`, `test:bin`, `build`, `inspect:no-software-renderer`) were not run: no product
  code and no guard changed.

### A wake-up for the end of the 20 ms wait

Session of 2026-10-22 on branch `worktree-agent-a52afe09d606ed6ed`, base `f5de574fdd` (the merged fix and the
measurements of the subsection "Idle latency of the 20 ms wait and rare stalls elsewhere").
It builds the fix that subsection proposed and decides it by measurement: the tile that waited a whole poll period
more (+20 ms) after a poll that left it.

#### Change

- The poll of the tile worker (`src/native/tile.rs`) now runs one function, `poll_unless_step_due`, from both of its
  timers.
  The function, `STEP_WAIT`, `step_wait_left` (the former `step_due`) and the wake-up code are in the new sibling
  module `src/native/tile/wait.rs`, because `tile.rs` passed the limit of 300 code lines with them (306).
  When the last scroll step is under `STEP_WAIT` old, the function logs "leaving a finished tile for the next scroll
  step" as before and also sets `tile_wake`, a single-shot `slint::Timer` in `State`, for the rest of the wait
  (`wake_when_wait_ends`).
  When the wake-up fires it runs the same function, so `poll` stays the only place a tile is taken, and a step that
  came meanwhile only makes the wake-up find the wait running again and set itself once more.
- Every `poll` stops `tile_wake` first, so the step that takes a tile (the call at the end of `on_viewport_changed`)
  cancels the wake-up that is waiting.
  There is one wake-up timer, and `start` on it replaces a pending one.
  Both timers hold weak handles of the window and its state, as the repeated poll does, so a closed window is not kept
  alive and a wake-up that fires after the close finds nothing to do.
- The first build set the wake-up for the exact rest of the wait, and a run of it logged 207 sets in a row, from
  864 microseconds left down to none, within one millisecond (`left_us` in the log
  `~/temp/agent/lat2-spin-l2-5-B.run.log`): the toolkit's timer clock counts milliseconds, so a wake-up for under one
  millisecond fires at once, finds a little of the wait left and sets itself again, a spin on the window thread of
  about a millisecond.
  `wake_delay` rounds the delay up to a whole millisecond.
  The first Q1 pass (`lat2-q1-main.jsonl`) and the first stall pass (`lat2-q2.jsonl`) are of the spinning build, and
  are marked so below; the final Q1 pass and the final stall pass are of the build with `wake_delay`.
  A third pair of passes (`lat2-q1-tip.jsonl`, `lat2-q2c.jsonl`) is of the tip after the move into `tile/wait.rs`
  (a move of the code, and the one change of the tip over the final passes, besides an import order).
- Tests (`src/native/tile_step_tests.rs`):
  `a_tile_left_for_a_step_that_never_comes_is_taken_when_the_wait_ends` (the poll that leaves a tile sets a wake-up,
  and with the repeated poll stopped the wake-up alone takes the tile once the wait is over),
  `a_scroll_step_that_takes_a_tile_cancels_the_wake_up_for_the_end_of_the_wait` and
  `a_wake_up_is_set_for_a_whole_number_of_milliseconds_that_covers_the_wait`.
  The first one uses the clock of the existing tests (`stepped_at` set to a time in the future keeps the step due),
  with no timing threshold, so it does not depend on the load of the host.
  A first version of it measured the time from a step to the swap against a threshold between 21 and 40 ms; it was
  replaced because a poll that comes one millisecond late finds the wait over and takes the tile with or without the
  wake-up, so under load the guard would have passed with its line removed.
- Source guards (`bin/inspect-source-guards.mjs`): `tile-wakes-when-the-wait-ends`, `tile-wake-cancelled-by-the-poll`
  and `tile-wake-covers-whole-milliseconds` are new; `tile-left-for-next-step` and
  `tile-taken-by-the-poll-when-idle` moved their anchors to `step_wait_left` in `src/native/tile/wait.rs`, which
  replaced `step_due`.

#### Method of this session

Two release binaries from two worktrees: tree A is `f5de574fdd` (origin/main), tree B is this branch.
Each worktree has a reflink copy of the package `target` directory, and the release build is the one the
`inspect:cadence` task makes.
Every number is from one app run at a time in the nested compositor (1100x660, release, `big.rs`, the `scroll`
scenario of `inspect:cadence`), each run a capped unit (`CPUQuota=600%`, `MemoryMax=40G`), arms alternating per pair,
the artifact directory deleted after each run (stall runs keep their log for a while).
The log level, the last wheel event to the first finished frame after the tile is applied, and the frame-shift probe
are those of the previous subsection.
The load average of the host moved between 3 and 21 from other sessions and from the builds of this one.
The scripts are `~/temp/agent/lat2-run.mjs`, `lat2-stats.mjs`, `lat2-modes.mjs`, `lat2-tail.mjs`, `lat2-apply.mjs`,
`lat2-extra.mjs`, `lat2-overlap.mjs` and `lat2-timeline.mjs` (copies and extensions of the `lat-*` ones).
The rows are `lat2-q1-main.jsonl` (spinning build), `lat2-q1-spread.jsonl`, `lat2-q1-final.jsonl`,
`lat2-q1-tip.jsonl`, `lat2-q2.jsonl` (spinning build), `lat2-q2b.jsonl` and `lat2-q2c.jsonl`.
The three Q1 passes of B and the three stall passes are of three builds: the spinning build, the build with
`wake_delay`, and the tip after the move into `tile/wait.rs`.

#### Idle latency, counts

The scenario is the Q1 of the previous subsection: 2 wheel steps of 1500 px, then nothing.
The tile is "late" when it is applied more than 30 ms after the last wheel event; the two modes of the earlier
sessions sit at about 20 ms and about 40 ms, and no run lies between them.

- A, all runs of the session (main pass 40, spread pass 20 and 20, final pass 40, tip pass 20): 70 of 140 runs have
  the tile applied late, which is 50%.
  The earlier session saw about 4 of 10 (19 of 47).
- B, the spinning build (40 runs): 0 late; applied 20.2 to 21.0 ms (median 20.4); first frame after it 26.9 to
  35.7 ms, median 29.6 (quartiles 29.0 and 30.3); A in the same pass: 22 late, median frame 48.1.
- B, the build with `wake_delay` (40 runs, 40 of A alternating with it): 0 late; applied 20.4 to 21.3 ms (median
  20.9); first frame after it median 30.6 ms (quartiles 29.7 and 32.3, range 28.7 to 40).
  A in the same pass: 20 late of 40 (applied 20.2 to 40.8 ms); median frame 42.9 (quartiles 30.4 and 49.5).
  A's fast mode alone (19 runs) has a median of 30.3 ms (range 29 to 32.8), so B is +0.3 ms above it at the median.
  Two B runs have a frame at 36.8 and 40 ms although their tile was applied at 21 ms or less: the frame ended late,
  which is not the wait.
  The paired difference B minus A has a median of -10.5 ms (quartiles -19.0 and +0.6, range -23.7 to +10.5).
- B, the tip (20 runs, 20 of A): 0 late; applied 20.5 to 21.1 ms (median 20.9); first frame after it median 30.1 ms
  (quartiles 29.6 and 30.9, range 28.7 to 37.6; two frames over 36 ms).
  A in the same pass: 8 late of 20; median frame 31.1 (quartiles 30.0 and 48.8).
- The slow mode is gone: 0 of 100 B runs against 70 of 140 of A.
  The median does not rise: 30.6 ms and 30.1 ms against the fast mode of A at 30.3 ms and the wait-off value of the
  earlier session at 29.7 ms.
  A tile that the repeated poll leaves is now taken about 21 ms after the last step, whichever way the poll fell.
- The spread of unchanged input, A against A with the same binary in two arm labels (20 pairs):
  late tiles in 11 of 20 and 9 of 20, medians 48.6 and 35.8 ms.
  Inside each mode the spread is small (the fast mode of A over the 80 runs of the first two passes lies at 28.6 to
  34.4 ms, about 6 ms), and the effect is the removal of the second mode, 19 ms away from the first (30.3 and
  49.4 ms medians in the final pass).
  Pair differences between two runs of one binary are not small (A2 minus A: median -2.5 ms, quartiles -15.4 and
  +4.2, range -26.3 to +24.0), because a pair can fall in different modes; the claim rests on the share of runs in the
  slow mode, which is 55% and 45% for the two labels of one binary and 0% for B.
- Positive control: the wait of 150 ms of the previous subsection (a 136 ms move) was not rebuilt, since the branch
  has no switch for it.
  The measure's reach in this session is shown by A itself: its two modes differ by about 19 ms, and every run
  falls in one of them.

#### Stalls

Scroll scenario, 120 steps, frame-shift probe, A and B alternating.
The probe fails on some noisy runs ("probe status 1": the frames could not be matched; 1 of 32 and 1 of 60 runs, both
of B, both with many wheel gaps over 20 ms; none in the 32 of the tip pass); those runs are not counted below.

- First pass, B the spinning build (16 pairs): A 16 runs, 2 stalls (slots 35 and 40), none at slot 50;
  B 15 runs, 1 stall, at slot 50.
- Second pass, B with `wake_delay` (30 pairs): A 30 runs, 7 stalls in 7 runs (slots 50, 56, 94, 125, 128 twice,
  140), slot 50 in 1; B 29 runs, 12 stalls in 10 runs (slots 35, 53, 72, 76, 97, 98, 119 twice, 128, 139 twice, 141),
  slot 50 in 0.
- Third pass, B the tip (16 pairs): A 16 runs, 1 stall (slot 96); B 16 runs, 3 stalls (slots 66, 72, 140);
  slot 50 in 0 and 0.
- Slot 50, three passes: B 1 of 60 runs, A 1 of 62; B after the spinning build 0 of 45.
  The stall of the previous session's merged binary was 0 of 56 runs, and the unfixed code 16 of 16; neither was
  rerun here, and the host is noisier than then (wheel gaps over 20 ms between two injected events in 34 of 62 A runs
  and 33 of 62 B runs).
  A stall count of zero at slot 50 is not shown by A on this host (1 of 62); B has 1 of 60 with the spinning build and
  0 of 45 with the two later builds.
- The stall at slot 50 of the spinning build (pair 5 of the first pass, log `~/temp/agent/lat2-spin-l2-5-B.run.log`)
  has a readable timeline and fits a cost
  of the wake-up: the wheel event at 316.7 ms (counted from the first one) was followed by the next one 22.6 ms
  later, 2.6 ms after the end of the wait; the wake-up applied the tile at 336.9 ms, and the frame the next wheel event
  drew followed at 341.6 ms.
  Any input gap longer than `STEP_WAIT` with a tile waiting ends this way with the wake-up, where the repeated poll
  only does when one of its ticks falls into the gap.
- Counting the tiles applied outside a step (more than 2 ms after a wheel event, inside the input): A 22 tiles in 6
  runs of 62, B 23 tiles in 11 runs of 62, so the number of such tiles is the same and they are found in more runs for
  B.
  Runs with a stall: A 10 of 62, B 14 of 60, which is not a difference these counts can show.
  Of the runs with a stall, 7 of 14 of B and 2 of 10 of A also have a tile applied outside a step; the other stalls
  are not tied to a tile hand-over in their run, and are taken to be the host's, which was not shown.
  If the cost is real, it is a small rate that these 62 runs per arm cannot resolve; a longer run of the stall pass
  would decide it.

#### Gates

Run once each on the tip `9f21309e6d` (documentation commits follow it):
`lint` 0 findings (a first run found the code line limit of `tile.rs` and two semantic line break findings in a
test's comment, a second a rustfmt import order; each fix was a new snapshot, and the third run passes);
`test:native` 282 passed, 5 skipped (the five tests of `tile_step_tests.rs` among them);
`test` 797 passed, 1 skipped; `test:bin` 60 passed; `build` finished;
`inspect:no-software-renderer` passes (not linked; the two features reach it);
`inspect:source-guards` for the six guards of the tile hand-over (6 of 156): baseline passes, removed fails, restored
passes, for all (`tile-taken-by-the-step` shares the baseline of `tile-left-for-next-step`).
The other 150 guards were not run.
A first guard run stopped at its baseline because the copied cache lacked `crate-licenses.json` (an input of the
build script, not a guard result); it was repeated after copying that file.

#### Limits

- The cost found above (an input gap over 20 ms with a tile waiting) is a property of the wait's length: a longer
  `STEP_WAIT` covers longer gaps and adds the same amount to the idle latency.
  It was not varied.
- Not run: the drag scenario, display scales other than 1, a real compositor, an input slower than 50 Hz, and any
  step length other than 16 ms (the sweep of the earlier subsection).
- The `probe status 1` runs are excluded from the stall counts and were not examined.
- The text of this subsection has the same kind of `markdown/semantic-line-breaks` findings as the older text of the
  file (about 200 in the new text, 480 in the older; the six named gates do not lint this file), and was not reflowed.
- The stall passes keep only the spin log; the other kept run logs were deleted, and the rows in the `lat2-*.jsonl`
  files are what remains.
- The wait of 150 ms (the positive control of the previous subsection) and the unfixed code were not rebuilt.

### Stalls away from slot 50 and the cost of the wake-up, by label rotation and tick class

Session of 2026-10-22 on branch `worktree-agent-ae05d7039f46336dc`, base `cd1e9ec340` (origin/main with the wake-up).
It takes the two questions that the subsections "Idle latency of the 20 ms wait and rare stalls elsewhere" and
"A wake-up for the end of the 20 ms wait" left open:
are the stalls away from slot 50 the host or the fix, and does the wake-up cost a frame.
No product code and no guard changed, so no gate ran.

#### Method

Tree A is `3f978311f0`, the first parent of the wake-up merge `401170b566`; `src/native/tile/wait.rs` does not exist in
it, and `git diff --stat 3f978311f0 cd1e9ec340 -- package/desktop-app/ide` lists six files, all of the wake-up (the new
`tile/wait.rs`, `tile.rs`, `native.rs`, `state.rs`, the tests and the source guards).
Tree B is `cd1e9ec340`.
Both were built with `mise run //package/desktop-app/ide:build` in their own worktree under `~/temp/agent`.
That directory is on another filesystem than the checkout, so the copy of the release part of `target` was a real copy
(3.6 GB each), not a reflink; each build took about 5 minutes under `CPUQuota=600%`.
The runs are the scroll scenario of `inspect:cadence` (120 steps of 25 px, 16 ms apart, release, `big.rs`, 1100x660),
the frame-shift probe `mm-stall.mjs`, one app run at a time, each run a capped unit
(`CPUWeight=10000`, `MemoryMax=40G`, `CPUQuota=600%`), the driver itself a unit with `CPUQuota=100%`.
Logs were at debug level for the frame, tile, tile build, gpu tile and wheel targets in every label.

The labels are A, B and A2, where A2 is the binary of A under a second name.
Each round runs the three labels once, in a rotating order (A B A2, then B A2 A, then A2 A B), so one round holds a
pair of A against B, a pair of A against A2, and a pair of B against A2 that share the host conditions of that minute.
This replaces the brief's separate 60 pairs of A against B and 30 pairs of A against A.
Two passes ran: 60 rounds, then 25 more
because 28 runs of the first pass could not be read (see "Counts" in this subsection).
255 runs in all, 85 per label; 226 give a probe result (A 74, A2 79, B 73), which makes 70 rounds with A and B both
read and 72 with A and A2 both read, more than the 60 and 30 asked for.
A throwaway round of three runs, not counted, came first.

The driver (`~/temp/agent/g56-run.mjs`) keeps the debug log of every run, the hash of every recorded frame, and the
modification time of every frame file, which puts each recorded slot on the clock of the logs.
The recorder is a steady 60 Hz timer and the file clock steps by 4 ms, so the slot clock is a least squares line through
the 190 to 250 modification times of a run; its worst residual is 5 to 8 ms in the runs looked at, so a tick is placed
within about one half of a tick.
A draw that finishes in the application is seen in the recorded frame some milliseconds later.
Over the whole-frame hash, which changes whenever a draw happens, one delay explains every tick of the input window in
161 of the 226 readable runs, and the delays that do form a plateau about 11 to 13 ms wide at the median in each label;
a delay of 6 ms was used throughout.

A stalled tick (the text moved by 0 rows between slot N-1 and N) is put in one class, in this order:
the tile applied outside a scroll step (more than 3 ms after the last wheel event) in the 17 ms before the tick ends,
split into with and without an input gap (two wheel events more than 20 ms apart touching that range);
a late wheel (a gap over 20 ms in that range, or no wheel event in the tick);
a tile applied inside a step; a redraw that no wheel event or tile started; none of these.
The same classes are computed for all 119 ticks of every run, so the rate of a stall in each class is known.
Scripts: `~/temp/agent/g56-run.mjs`, `g56-lib.mjs`, `g56-analyze.mjs`, `g56-more.mjs`, `g56-early.mjs`,
`g56-early2.mjs`, `g56-show.mjs`, `g56-dump.mjs`; rows `g56-rows.jsonl` and `g56-rows2.jsonl`,
per-tick table `g56-ticks.json`, the timeline of every stalled tick (46) `g56-stall-timelines.txt`.
The run directories were deleted after the analysis (3.2 GB), except `run.log`, `slots.json`, `cadence.json` and
`hashes.json` of the 39 runs that have a stall, kept in `~/temp/agent/g56-stall-runs` (8 MB).

#### Counts

- Runs unreadable: 28 of 255 (A 10, A2 6, B 12), plus one run of A that made no artifact (`status 1 dirs 0`).
  In each of the 28 the recorder captured fewer frames than the input needed (14 to 46 frames per second against a
  median of 60 and a minimum of 48 in the readable runs), the input took 2.2 to 5.6 seconds against 2.0, and the probe
  could not read the missing frames.
  Rounds 22 to 26 of the first pass hold 14 of them: the compositor was starved there, which is host load.
  These runs are not in any other count of this subsection.
- Stalls (A, A2, B): 13, 19 and 14 in 74, 79 and 73 runs; runs with a stall 11, 15 and 13 (15%, 19%, 18%);
  runs with two or more stalls 2, 4 and 1.
- Slot 50: 0 of 74, 0 of 79 and 0 of 73 runs.
  The stall of the unfixed code did not come back in any label, with the wake-up or without.
- Histogram of the stall slots (slot, count).
  A: 31 2, 32 1, 34 1, 35 3, 68 1, 91 1, 96 1, 124 1, 129 1, 130 1.
  A2: 31 2, 35 1, 41 1, 48 1, 66 2, 69 1, 76 1, 80 1, 81 1, 82 1, 83 1, 91 1, 96 1, 116 1, 142 3.
  B: 31 2, 32 1, 33 2, 34 2, 35 2, 57 1, 61 1, 72 1, 88 1, 125 1.
- The spread, A against A2 (one binary), paired inside the 72 rounds that have both, bootstrap over rounds, mean and 95%
  interval of the difference in stalls per run: A2 minus A +0.042 [-0.083, +0.181];
  runs with a stall +0.028 [-0.083, +0.139]; stalls after slot 36 +0.097 [-0.014, +0.208].
- B against A, 70 rounds: stalls per run +0.043 [-0.071, +0.157]; runs with a stall +0.043 [-0.043, +0.143];
  stalls after slot 36 0.000 [-0.071, +0.086].
  B against A2, 73 rounds: -0.027 [-0.178, +0.123]; 0.000 [-0.123, +0.123]; -0.110 [-0.247, +0.014].
  Each interval of B against A has about the width of, and nearly the same place as, that of A2 against A, so B is not
  worse than A beyond the spread.
  The upper end, +0.157 stalls per run, is nearly the rate of A itself (0.176 per run): a cost up to that size is not
  excluded by 70 rounds, and halving the interval takes about four times the runs.
- The host indicators agree between the labels: input gaps over 20 ms per run 2.66, 2.91 and 2.42 (A, A2, B);
  tiles applied outside a step in the input window 98 in 17 runs, 108 in 19 runs and 85 in 16 runs.
  The paired difference of the tiles outside a step, B minus A, is -0.086 per run [-0.443, +0.257]: the wake-up does not
  add tiles taken in a gap, which it could only do if the repeated poll missed gaps that it catches.

#### Classes of the stalled ticks

- Ticks and stalled ticks per class, A then A2 then B.
  Tile outside a step in an input gap: 42 and 1, 43 and 0, 31 and 1.
  Tile outside a step, no gap: 121 and 0, 143 and 0, 112 and 0.
  Late wheel: 799 and 10, 1008 and 13, 728 and 7.
  Tile applied in a step: 901 and 1, 927 and 2, 887 and 0.
  Other redraw: 833 and 0, 1027 and 0, 781 and 0.
  None of these: 6181 and 1, 6343 and 4, 6084 and 6.
- The class that the wake-up could add to, a tile outside a step in an input gap, holds 2 of the 46 stalled ticks:
  slot 96 of A (round 2, no wake-up code in that binary: the repeated poll took the tile 0.6 ms before the wheel event
  that ended an input gap of 21 ms) and slot 72 of B (round 2: tile applied 4.5 ms after a wheel event, two frames
  of 9 and 4 ms, the next wheel event 21 ms after the last).
  Its stall rate is 2 of 116 ticks, against 30 of 2535 in the late-wheel class and 11 of 18608 in the last class.
  Whether the tile of the B stall was taken by the wake-up or by a poll tick is not in the log; the log has the same
  line for both.
- Runs by the number of input gaps over 20 ms in the run (a count from the compositor's log, that involves no tile):
  none: 133 runs with 3 stalls (A 0 in 45, A2 1 in 45, B 2 in 43);
  one to two: 31 runs, 13 stalls; three to five: 9 runs, 4 stalls; six or more: 53 runs, 26 stalls.
  So 43 of the 46 stalls fall in the 93 runs that have at least one gap (A 13 of 13, A2 18 of 19, B 12 of 14).
  This is partly built in, since a missing step is a gap, but the count is taken from the compositor's events and not
  from the picture.

#### Decisions per stall slot

- Slots 31 to 35, the first six ticks of the input (A 7 stalls, A2 3, B 9; 18 runs, 19 stalled ticks):
  11 late wheel, 8 none of the classes; none of the 18 runs has a tile event, a tile left or a tile worker job
  between the first wheel event and slot 36 (in the 226 readable runs two have one, both with no stall).
  The first draw after the first wheel event falls between slot 29 and slot 30 in 175 of 225 runs, so these are the
  first ticks of the input, with the compositor's injection and the first frames starting up.
  Decision: not a tile hand-over, so not the wake-up; host or the start of the input.
  B has 9 against A 7 and A2 3, which is inside the spread of one binary (A against A2 differs by 4).
- Slots 41, 66 (second one), 88: none of the classes (A2, A2, B). Decision: neither, so the host.
- Slots 57, 61, 66, 68, 69, 76, 80 to 83, 91 (A2), 96 (A2), 116, 124, 125, 129, 130, 142 (twice): late wheel,
  in A, A2 and B alike (the gaps are 20.1 to 365 ms, median 26.5; the late wheel stalls of the first six ticks have
  the same gaps). Decision: a late wheel, so the input, host side.
- Slots 48 (A2), 91 (A), 142 (A2): a tile applied inside a step, with the explicit redraw after it that the fix leaves
  in place (the throwaway round showed the same shape in A at slot 96). Decision: a tile in a step, which the fix
  leaves; 3 of 226 runs.
- Slots 72 (B) and 96 (A): a tile outside a step in an input gap. Decision: the case the wake-up could cost;
  one in B and one in A, so not beyond the spread, and the A one cannot come from the wake-up.

#### Answers

- Q1, are the stalls away from slot 50 the host or the fix: 41 of the 46 stalls are a late wheel or none of the classes,
  and neither depends on the tile hand-over; 3 are a tile in a step and 2 a tile outside a step in a gap.
  The rate of a stall is the same in the binary without the wake-up (0.18 per run) as in the one with it (0.19) and in
  the second name of the first (0.24).
  The stalls are the host's input delivery in the sense that the stall follows a late wheel event, not in the sense that
  the load was shown to cause them: no synthetic load was made (forbidden), and the cause of the late wheel events
  (the compositor's injection timer under host load, or the application's own threads sharing the 600% cap) was not
  separated.
  The stall at slot 50 did not occur in 226 runs.
- Q2, does the wake-up cost a frame: not shown. B against A is +0.043 stalls per run [-0.071, +0.157], the same as
  A against A2 (+0.042 [-0.083, +0.181]), and the tile-outside-a-step class does not grow.
  Runs with a stall in the subsection "A wake-up for the end of the 20 ms wait" (B 14 of 60, A 10 of 62) and here
  (B 13 of 73, A 11 of 74) are of one size; the owner session's 5 of 20 for A against 2 of 20 for B is in B's favour
  and is not repeated here. No fix was made and no gate ran.

#### Limits

- The slot clock is a fit through coarse modification times; a stalled tick is placed within about half a tick, and the
  17 ms reach of the classes overlaps neighbouring ticks. The classes are windows, not exact causes.
- Positive control: the unfixed code was not rebuilt, so the link between a tile outside a step and a stall is not
  re-shown here; it rests on the subsection "Idle latency of the 20 ms wait and rare stalls elsewhere"
  (16 of 16 runs at slot 50), and on 2 stalls of this one.
- The lat2 run `lat2-spin-l2-5-B.run.log` has no frame clock, so the classes could not be validated on it.
- The 28 unreadable runs are host-starved runs; their share per label (A 10, A2 6, B 12 of 85) is not evidence of a
  difference, and the stall counts are of the readable ones.
- Not run: the drag scenario, display scales other than 1, a real compositor, an input slower than 50 Hz, steps other
  than 16 ms, the latency scenario (no product code changed, so it was not re-measured), and any gate.

## Scale 2, the light scheme and the seams, measured in the compositor

Session of 2026-10-22 on branch `worktree-agent-a1612e9c563c279ed`, base `cd1e9ec340`.
It closes three checks that were never done: display scale 2, the light color scheme, and the seams of the minimap
chunks and of the code tile at scales 1 and 2 in both schemes.
No product code changed (a temporary fault in `src/native/gpu_tile/slice.rs` served as a positive control and was
reverted, `git status` is clean), so no gate ran.

### Method

- The binary is the installed release `~/.local/bin/monochromatic-ide` (sha256 prefix `27b877741a056ecf`, the build of
  `origin/main`), in `monochromatic-nested-wayland-session` 0.6.1.
  The compositor already had everything the checks need: `--scale`, `--color-scheme` (private portal, no host
  setting touched), `--accessibility` (bounds in physical pixels), `--headless`, `record`, `wheel`, `scroll`, `key`.
  Nothing was missing, so the compositor was not changed.
- The runs are headless (`--headless --gpu /dev/dri/renderD128`, the RX 7600 that the application's Vulkan loader
  order picks, texture limit 16,384).
  The nested window is clamped to the host screen: a request for 1100x3700 at scale 2 gave a 2160x3704 capture,
  where headless gives 2200x7400.
  Control: at 1100x660 and scale 1 the nested and the headless capture differ in about 50 pixels of the code and minimap
  columns and in the sidebar columns (pointer hover state of the nested window); the code view is the same.
- Fixtures below a disposable `/tmp` directory: a file of 600 lines of the harness's `code.rs` under three lines that
  make characters countable (60 `=` signs, then 62 distinct characters), and a file of 30,000 identical lines
  (`let value = compute(alpha, beta); // note`), whose minimap picture and code view repeat every line pitch exactly.
  A repeat that breaks is a gap, an overlap or a color jump, wherever it is.
- One app run at a time, each in a systemd user unit with `CPUQuota=600%` and `MemoryMax=40G`.
  Measured under the load there was (load average 2 to 14 from other sessions; no wait for quiet).
  Frames were read with Pillow and numpy; the scripts are `~/temp/agent/sss/` (`session.mjs`, `run.mjs`,
  `scenarios.mjs`, `a1.py` to `a10.py`), the images `~/temp/agent/sss-shots/`.
  The artifact directories, the worktree `target` directory and the fixtures were removed afterwards.

### Check 1: display scale 2

Everything is compared in logical pixels, scale 1 against scale 2, 1100x660, dark.

- Sharpness of the code view (Laplacian energy over ink energy; share of 2x2 blocks that are constant):
  native scale 2 reads 1.18 and 3 percent; a nearest-neighbour upscale of the scale 1 capture, the control, reads 1.57
  and 100 percent; a bilinear upscale reads 0.25 and 0.5 percent.
  The glyphs are drawn at physical resolution (4.7 times the bilinear value, and no 2x2 stair).
  The light scheme reads the same (1.18 and 3 percent).
- Layout: the line pitch is 24 logical px and the character advance 9.0 at both scales (autocorrelation of the `=`
  row); line numbers sit at the same logical y; the language banner wraps at the same words.
- Minimap: the accessibility bounds of the track and the picture are exactly twice those at scale 1
  (track 990,32,110,628 against 1980,64,220,1256; picture 110x768 against 220x1536).
- Caret: 2.0 logical px wide (2 and 4 physical), 21 logical px tall, at x 333 to 335 and y 188 to 211 on both.
- Selection: a double click on `compute` selects the rectangle 423,186 to 486,210 (logical) at both scales, and the
  clipboard holds `compute` at both; select all covers 315,150 to 975,660 at both.
  Selected glyphs are white on `#0078D4` (4.53 to 1) at both scales and both schemes.
- Hit testing: 10 clicks per run (a quarter and three quarters into the cell of characters 0, 3, 17, 40 and 58 of the
  line of distinct characters), then shift+End and ctrl+c, and the pasted text read through `wl-paste` on the
  compositor's display.
  A quarter in gives the character, three quarters in gives the next one.
  Scale 1: 10 of 10 in all 5 runs.
  Scale 2: 10 of 10 in 6 of 7 runs whose pastes were all checked, and another run of 8 repeated clicks on one cell
  was correct 8 of 8 (a further batch of 4 runs was seen only in its first 5 pastes, all correct).
  The one run that failed (the first run of this session, while a release build of this session ran in the
  background) had no effect for its first 5 clicks and then all correct: the sequence is what an application that
  is not yet taking input would show (the compositor answers `ping` before the window is ready).
  It did not recur in the 6 later checked scale 2 runs, no frame or log of it exists, and it is not attributed.
- Scroll step: one wheel notch moves the text about 60 logical px at both scales (the best whole-pixel match is 60 and
  120 physical, with the same residual of 7.6 levels at both, so the step is not a whole number of pixels), a touchpad
  scroll of 100 logical px moves 100 (100 and 200 physical, residual 0).
- Texture rules at physical size, from the accessibility bounds in a 1100x3700 window (scale 2: 2200x7400 physical):
  the code tile is 1358x12288 (66,748,416 bytes, under the 64 MiB rule, under 16,384 rows) and covers the view of
  7,336 rows with 2,476 rows to spare above and below; the minimap picture is 220x7936 (15.5 chunks, one texture).
  At scale 1 the tile is 679x11064 and the picture 110x3968.
  Because a window cannot be taller than the device's texture limit (see below), a picture of more than 16,384 rows
  needs a window of about 16,000 physical rows.
  A 1100x8100 window at scale 2 (2200x16200 physical) gave the stack of the real device: the first texture 220x16384
  and a second one of 512 rows, abutting, with no gap.

### Check 2: the light scheme

- The scheme comes from the compositor's private portal (`--color-scheme light`, and `color-scheme` on the control
  socket to switch); the application follows it with no setting of its own.
- Search for dark-only constants in the Vello drawing code: the only literal colors in `src/` outside tests are the
  twelve syntax hues of `src/code_colors.rs` (each with a dark and a light value), the white and black that
  `src/selection_ink.rs` picks by contrast, and the tint of `src/native/render.rs` that follows the selected ink.
  The base color of every Vello render is transparent.
  Every other ink is a window property from the markup (`Palette`, `AnnotationInk`, `Surface`).
- Measured contrast of ink against its background in light (background `#FAFAFA`), at scale 1 and scale 2:
  code comment ink `#555C65` 6.48; syntax `#14567F` 7.54; gutter numbers `#737373` 4.54 (0.6 opacity by design);
  information message `#005FB8` 6.04; error `#C42B1C` 5.42; warning `#9D5D00` 5.03; hint `#616161` 5.93;
  the marks in the gutter (`E`, `W`) the same inks; the find box text `#005FB8` on white 6.31;
  the other find matches, text `#555C65` on a `#D7D7D7` fill 4.70;
  selected row and selected text, white on `#0078D4`, 4.53.
  Dark for comparison: comment `#A0A8B0` 8.72, error `#FF99A4` 10.34, warning `#FCE100` 15.91, information 11.67.
  Every ink that is text is at least 4.5 in both schemes.
- The minimap's strongest glyph pixel reads 3.89 in light and 5.24 in dark; the picture is a graphic, not text, and
  its colors come from the same function as the code's (`ink(role, colors)`).
- Looked at (images below): light code view, find box with 166 matches, select all with the underlines of diagnostics
  under the selection, the viewport box, the sidebar and the tooltip of a diagnostic are all readable;
  nothing is dark on dark or light on light.
- Diagnostics marks were checked with the debug build and its `IDE_INSPECT_ANNOTATIONS` file (error, warning,
  information and hint messages, wavy, dashed and dotted underlines, the `E` and `W` marks, parameter and type hints),
  at both scales in both schemes (`a1d`, `a1l`, `a2d`, `a2l`).
  Hint labels at scale 1 never reach their full ink (thin unhinted strokes, 4.4 to 4.9 against 7.6 to 8.2 at scale 2).

### Check 3: seams

- Chunk seams of the minimap, 1100x3700, picture rows repeat every 4 rows at scale 1 and 8 at scale 2.
  The measure: for every row of the track, the largest difference over the picture columns between that row and the
  row one pitch below, rows of the viewport box's top and bottom border and of tooltips left out.
  The seams are the picture's top (from the accessibility bounds) plus multiples of 512.
  After a click on the track (no fling, 4 s settle), three positions per run.
- Result: the largest difference over all unmasked rows is 1 level of 255 in every shot but two, over 4 configurations
  (scale 1 and 2, dark and light) with 3 shots each, 128 chunk seams in all (7 per shot at scale 1, 14 or 15 at
  scale 2), and 3 shots of the 16,200 row window with 32 seams each, one of them the cut between two textures
  (at row 16,384 of the picture).
  The two other shots (scale 1 and 2, light, first position) flag 24 and 45 rows that a 4x crop shows to be the
  hover tooltip of the banner's diagnostic drawn over the minimap, 50 to 300 rows away from any seam.
  The first rows of the picture at scroll 0 hold the language banner, which is not periodic, and are not seams.
- Positive control: a release build with the chunk copy one row early for every chunk after the first
  (`origin: Origin3d { x: 0, y: row - 1, z: 0 }` in `draw_stack`, temporary, reverted).
  Every shot (4 at scale 1, 4 at scale 2) shows a break at the first seam, with differences of 103 (scale 1) and 156
  (scale 2) levels in 4 to 7 rows just above it;
  the other seams do not break because every later chunk is moved by the same row.
  Against 0 flagged rows in all clean shots.
  What this measure cannot see: the pitch (4 or 8 rows) divides 512, so no seam cuts through a glyph, and a chunk
  shifted by a whole line leaves the picture identical.
  Those two cases are covered only by `src/native/minimap_seam_tests.rs` (scale 1.5, equal to an unsplit render),
  not by these captures.
  The positive control of the tile measure is the synthetic 1 px roll, not a code fault.
  The fault is visible in a 6x crop as a row of squashed glyphs
  (`seam-compare-scale2-clean-left-fault-right.png`).
- Tile seam: the code tile is one texture, so it has no seam inside it
  (true of an ordinary window; a tile taller than a texture has seams since 2026-10-22, measured in the section
  "The code tile stacked across textures").
  What can go wrong at a tile is a swap that places the text one pixel off.
  The measure is the phase of the text rows modulo the line pitch, against the gutter numbers' phase
  (which the markup places).
  Over 18 shots (scales 1 and 2, dark and light, before and after a tile swap by a minimap click or a page jump)
  the text phase is constant to 0.05 physical px, and text minus number is +0.9 px (0.87 to 0.99) at scale 1 and
  +1.9 px (1.76 to 1.99) at scale 2, the same 0.9 logical px.
  Positive control: the same capture rolled by 1 px reads +1.00.
  No blank run longer than a line in any settled frame.
  Frames recorded during two page jumps back to back (37 frames at 24 per second at scale 1, 12 at 7 per second at
  scale 2): no blank run, text phase constant.
- A frame that does show the tile's edge: a touchpad scroll of 1100 logical px at scale 2 in a 1100x3700 window
  (recorded at 10 frames per second) had one frame with 647 blank rows at the bottom of the code view, the next
  frame full again.
  The view had moved 600 physical rows in one frame, past the 2,476 rows of overscan.
  It is the tile outrun that the earlier cadence work knows, not a seam.

### Findings that are not defects of these three checks

- Symptom first: in a 7680x4320 window at scale 1 the top of the file is not drawn.
  The code tile does not cover a view larger than the 64 MiB rule allows.
  The tile is cut to the rule around its middle, so a view of more than about 16.7 million pixels (the area of text
  columns by rows) has rows that carry no text.
  Measured: 7680x4320 at scale 1: the tile is 7259x2280 and rows 21 to 1016 and 3062 to 4320 of the 4,320 are blank
  in the code view (52 percent of the view; the top of the file is not drawn).
  2200x16200 physical (1100x8100 at scale 2): the tile is 1358x12288 and the first 1,894 and last 1,970 rows are blank.
  Not a problem at 3,700 px at scales 1 and 2 (tile 1358x12288 covers 7,336 rows).
  Computed from the rule, not measured: a 5K window at scale 1 (about 4,600 text columns, cap 3,640 rows against
  2,880) is covered; a 6K window at scale 1 is not (cap about 2,940 against 3,384).
  This is the rule recorded in the section "Stacked textures" (the minimap stacks, the code tile does not).
  Options for the owner: stack the code tile as the minimap does, ranked first (covers every window, needs the scene
  split and the clipping of the selection rows per texture); lower the scale of a view that does not fit (cheap, soft
  text); leave it (affects only windows of more than 16.7 million physical pixels of text).
  Not fixed here: it is a new feature, not a defect of the checked paths.
  Closed on 2026-10-22 by stacking the code tile: see the section "The code tile stacked across textures".
- A physical window taller or wider than the device's texture limit panics the application:
  1100x8400 at scale 2 (2200x16800) ended with `wgpu error: Validation Error ... In Surface::configure ... Requested was
  (2200, 16800), maximum extent for either dimension is 16384`.
  The panic is raised in the toolkit's surface configuration.
  Real displays are far below that, so a stacked minimap is reachable only in a window of about 16,000 physical rows.
  Closed on 2026-10-22 at the cause (the window is held to the texture side); the panic could not be reproduced end to
  end in the nested compositor, whose own output surface panics first at that size:
  see the section "The code tile stacked across textures".
- At every scale 2 window of 3,700 px the application logs `vello::render: Trying to paint too large image.
  1358x12288` at warn level (243 lines in 9 runs; 2 for 1376x12144).
  It is the Vello warning for more than 256 coarse workgroups
  (`doc/troubleshooting/vello-silent-blank-or-corrupt-frames-at-fixed-buffer-limits.md`: "I did not see the warning
  fire" with no logger).
  It fires for the 64 MiB tile at 288 workgroups (by my count of 256 by 256 pixel bins),
  below the corruption limit that file measures, and no wrong pixel
  was found in the view in these runs; the rows of the tile outside the view were not read back.
  The 64 MiB rule is 16,777,216 pixels, exactly 256 bins of 256 by 256, so a tile at the cap with edges off the
  256 pixel grid counts more than 256 bins: it fires at scale 2, where the tile is at the cap, not at scale 1.
  The only evidence that such a tile renders correctly outside the view: in the recorded scroll through the overscan
  at scale 2 (`sr2d`, frames 7 to 10) the view moved through those rows with 0 periodicity violations.
  Closed on 2026-10-22: each texture of the tile is cut to at most 256 bins, and no run of the new build logged the
  warning (see the section "The code tile stacked across textures").
- During a fling the minimap picture lags the box: in one run the track had 91 to 94 blank rows at its bottom 3.5 s
  after a 3,777 px touchpad scroll (the fling went on for more than 10 s), and 0 blank rows once the scroll had
  stopped.
  A transient of fast scrolling, in two runs; not examined further.

### Images (under `~/temp/agent/sss-shots/`)

- `e1/plain.png` and `e2/plain.png`: the harness fixture at scale 1 and 2, dark.
- `l1-int/find.png`, `l1-int/all.png`, `l2-int/word.png`: light, find box, select all, selected word.
- `a1l/plain.png`, `a1d/plain.png`, `a2l/sel.png`: diagnostics marks in light and dark, selection over underlines.
- `seam-compare-scale2-clean-left-fault-right.png`: the positive control of the seam measure.
- `u2d3/`, `u2l3/`, `st3d/`: the tall windows with the pictures; `w8k/plain.png`: the 8K window with the blank bands.

### Not done

- Fractional scales (the stack is whole physical pixels by construction), a real compositor, the drag scenario.
- The stall probe of the cadence work at scale 2.
- A readback of the rows of the 64 MiB tile outside the view
  (closed on 2026-10-22 for the stacked tile: the native tests read every texture back and compare it with one
  texture; see the section "The code tile stacked across textures").
- Gates, because no product code changed.

## The code tile stacked across textures

Session of 2026-10-22 on branch `worktree-agent-a074735d16c6106a5`, base `8864316eb2` (origin/main).
Owner decision (2026-10-22): the code tile stacks textures as the minimap does, and a window whose tile does not fit a
texture never shows wrong output, soft text or a notice instead.
It closes the three findings of the section "Scale 2, the light scheme and the seams, measured in the compositor":
the blank bands in a 7680x4320 window, the surface panic of a window past the texture limit, and the Vello warning
"Trying to paint too large image".

### What changed

- `src/shaped_text/limit.rs`: a texture holds at most `tile_texture_rows(columns)` rows:
  the device's side, and `256 * floor(256 / ceil(columns / 256))` rows, where Vello counts at most 256 bins of 256 by
  256 pixels (this is what its coarse stage counts before the warning, and 256 bins are exactly 64 MiB, so the old byte
  rule is implied).
  The tile of 1358 columns is 10,752 rows (42 bins) and not 12,288 (288 bins).
  `stack_rows` is the most rows of a stack: four textures, or the side plus 1,024 rows when that is more, so the view of
  the tallest window the device can show is always held.
  `texture_heights` cuts a tile into whole textures from the top, the rest last.
  `tile_cap(columns, scale, wanted)` is the larger of what one texture holds and `wanted` plus six code rows, within the
  stack: an ordinary window still gets one texture, and a line wrapped over thousands of rows is clipped as before.
- `src/tile_scene.rs`, `tile_scene/item.rs`, `tile_scene/encode.rs`: `paint_stack` builds one scene for each texture
  (`tile_items_in`).
  A scene holds the rows, hints, messages and underlines that can reach its rows, placed from its first row by a whole
  number of pixels, so a glyph that two textures share is drawn at the same fractional position in both;
  selection and find fills of the tile are clip bands that `encode` cuts to the texture.
  The caret, the find boxes and the wrap indicator are drawn by the markup and need no split.
- `src/native/gpu_tile/source.rs` (new), `types.rs`, `probe.rs`, `readback.rs`: the textures of a tile are made once
  for a shape and reused for a tile of the same shape; the window is given a `[SourceSlice]` list (image, first row,
  height) that is changed in place (`show_slices`, the lesson of the hitch of the minimap's stack).
- `ui/app.slint`, `src/native/start.rs`, `gpu_tile.rs`: `source-slices` is drawn by a `for` loop at whole physical rows
  from the tile's top; `texture-limit` (told at start and again when the device arrives) bounds `max-width` and
  `max-height` of the window in physical pixels.
- `src/native/viewport.rs`, `render/paint.rs`, `tile_build/job.rs`, `render/present.rs`: the cap and the re-center edge
  use the stack and the view; the re-center keeps its rule (a view must stay a quarter of a view clear of the tile's
  edges) over the whole stack, so a window of one texture behaves as before.

### Evidence

- New or changed native tests in `src/native/tile_stack_tests.rs`, `tall_window_tests.rs`, `wrap_clip_tests.rs` and
  `wrap_resize_tests.rs`, and in `tests/raster_limit.rs`, `tile_vello_limit.rs` and `wrap_rows.rs`.
  A stack drawn at lowered texture limits, read back and put one below the other, equals the one texture at every texel
  (selected and plain rows; hints, messages and underlines of all severities with a cut through them at every code
  row); the same in the window's composited frame, with a cut inside the view; a stack one row lower is not equal
  (a positive control in each).
  At one padding a row of an underline's wave differs by an alpha of 4 of 255 at a cut (32 texels):
  the test allows a faint fringe of at most 64 texels of alpha 8 or less, and nothing else.
  The window frame is compared below the rows that show the path of the project, which a test names at random.
- Bin rule: no accepted texture covers more than 256 bins by Vello's own formula, at sides 8192, 16,384 and 32,768 and
  every width (`no_accepted_texture_covers_more_bins_than_vello_counts_before_it_warns`).
- The window's `texture-limit` is the device's side after a frame, not the default of the markup (16,384, made
  different from the 8,192 of the test device so that a test can tell them apart).
- Source guards (`bin/inspect-source-guards.mjs`), each shown with a baseline pass, a fail with the guard removed and a
  pass restored, in a disposable copy: `stack-bin-rule`, `clip-texture-side`, `stack-own-top`, `stack-band-cut`,
  `stack-slices-abut`, `stack-textures-reused`, `window-texture-limit-told`, `window-max-width`, `window-max-height`,
  and `minimap-slices-in-place` with its anchor moved to the generic `show_slices`.
  `clip-cap` is gone (the byte rule is implied by the bin rule, so its mutation changes nothing).
  Three older guards, `failed-job-freed`, `overscan-early-return` and `minimap-scroll-notes-bands`, already had absent
  anchors on origin/main (indentation); they were not run and not repaired here.

### Measurements

Release builds of origin/main (A, `~/temp/agent/cs-a`) and of this branch (B, the branch worktree itself), the nested
compositor headless on the RX 7600 (texture limit 16,384), `uni.rs` (30,000 identical lines) for the periodic measures.
Measured under the load of the host (load average 8 to 14 from other sessions, no wait for quiet).

#### Windows past the old limit (rows of ink, no blank band)

The scenario settles, clicks the minimap, jumps two pages and waits (shots `s0` to `s3`).
The code columns only; rows with any ink out of the window's rows; the longest blank run.

- 7680x4320 at scale 1, dark: A has 1,405 to 1,553 inked rows of 4,320, the last inked row is 3,061 or 3,287 and the
  blank runs are 160 to 1008 (or 21 to 1016) and the rest below the last inked row (1,258 and 1,032 px).
  B has 2,879 to 2,897 inked rows (about 66 percent, text rows at a 24 px pitch), the last inked row is 4,319 and no
  blank run is over 30 px (the longest is 24 px at the banner of `s0` and 11 px after).
  Light, the same counts.
- 3840x2160 at scale 2 (7680x4320 physical): A has 1,580 to 1,629 inked rows and a blank band of 936 to 959 px at the
  bottom; B has 2,782 to 2,826 inked rows (64.5 percent) in dark and in light, the last inked row 4,319.
- 1100x8100 at scale 2 (2200x16200): A has 7,966 to 8,014 inked rows and a blank band of 1,955 to 1,969 px at the
  bottom (last inked row 14,244); B has 10,454 to 10,492 inked rows (64.5 percent), the last 16,199, dark and light.
- Looked at with the image viewer (images below): the window downscaled and 4x crops at cuts, scale 1 and 2, dark and
  light: every line is drawn, the line numbers run on without a gap, the caret and the tooltip are where they were.
- The tile of the 7680x4320 window is, by the rule and not by a readback, 7 textures of 2,048 rows (7,259 columns are
  29 bins across and 8 bins down), 12,960 rows in all; cuts fall inside the 4,320 rows of the view.

#### Seams (text phase, no gap, no overlap)

`~/temp/agent/cstack/seam.py`: the text columns of a row equal those of the row one line pitch below (24 px times the
scale) over the whole window below the banner, and the distance from the top of each line's text to the top of its
line number, and between consecutive lines, is constant.

- B, 24 shots (6 captures, 4 shots each): 0 rows differ in 16 shots; the line pitch is 24 (scale 1) or 48 (scale 2) on
  every line and the text to number distance is 2 (scale 1) or 3 (scale 2) on every line.
  The other 8 hold the language banner (`s0` of every capture, 6 shots) or the tooltip of a diagnostic (`s1` of the
  two tall captures, 144 rows, read in a crop); they are not seams.
- Positive control: a release build with the slices placed one row too far apart (`top += held.target.height + 1`,
  temporary, reverted, `git status` clean): 35 rows differ in 3 runs at 7680x4320 (rows 1858 to 1874, 3903 to 3915 and
  3923 to 3927), line pitches 5, 20 and 25 appear, and the text to number distance takes 4 values; at 1100x8100 and
  scale 2, 32 rows differ at row 8119 and a pitch of 49 appears.
  Against 0 rows in every settled shot of the clean build.

#### Vello warning

- The build before the change logged `vello::render: Trying to paint too large image. 6838x2400.` 5 times in each of 3
  runs (the 3840x2160 and the 1100x8100 windows at scale 2, and one at 7680x4320).
  The 6 runs of the new build logged none.
  The same runs, the same log level: the old build is the positive control.

#### Window past the texture limit

- A window of 1100x8400 at scale 2 (2200x16800) could not be tried end to end: the nested compositor's own output
  surface panics first, in `Surface::configure` ("Requested was (2200, 16800), maximum extent for either dimension is
  16384"), before the application starts, with A and with B alike.
  So the panic named in the earlier finding was probably the compositor's and not the application's.
  What is verified is the bound the owner asked for: the markup holds the window to the device's side
  (`the_window_is_never_larger_than_the_devices_texture_side` and the three guards above).
  A real compositor that grants a larger window was not available.

#### Scroll cadence

120 steps of 25 px every 16 ms (the `scroll` scenario of `inspect:cadence`, release, 1100x660, `big.rs`), the frame
shift probe of `mm-stall.mjs`, arms alternating per pair.
A tick in which the text did not move is a stall.
At this size the stack is one texture, so these runs measure the one texture path.

- A against B, 22 pairs (23 runs of B, as one pair was repeated after another session removed an artifact directory):
  B had 4 stalls in 3 of 23 runs (slots 45, 49 and 50, 139); A had 1 in 1 of 22 (slot 127).
- A against A, the same binary, 15 pairs (28 usable runs of 30, the frames of 2 were gone): 7 stalls in 7 runs
  (25 percent) at slots 61 to 128.
  The spread of one binary (1 of 22 and 7 of 28 runs) covers B (3 of 23).
- Other odd ticks (a tick that moved two steps, or the end of the scroll): one run of B had 11 and 27, two runs of A
  had 2 and 6 in all; none in the same-binary control.

#### Idle latency of the tile hand-over

Two wheel steps of 1500 px, 16 ms apart, nothing after; the tile is late when it is applied more than 30 ms after the
last wheel event.
40 pairs, twice.

- First pass: A 0 late of 40 (applied 1.4 to 21.6 ms, median 20.9); B 2 late of 40, in pairs 29 and 30 (applied at
  52.6 and 238.6 ms, first frame at 86.7 and 311.3 ms); the other 38 runs of B applied at 20.4 to 21.5 ms.
- Second pass: A 0 late of 40 (applied 20.3 to 21.1, median 20.8); B 0 late of 40 (applied 20.3 to 21.5, median 20.8).
- So B was late in 2 of 80 runs against 0 of 80 for A: two runs in a row of one pass, none in the other.
  The artifact directories were removed after each run, so the two late runs were not examined and are not attributed.
  A burst of load is likely (A, measured in the same minutes, was on time), but a count of 2 in 80 cannot separate that
  from an effect of the change.
  The criterion of 0 late in 40 was met by the second pass and not by the first.

### Limits that the change brings

- A tile placed by a map that still counts unmeasured lines as one row can be taller than three views:
  the frame guard (`fit_clip`) cuts it to the bound of the stack (four textures, 28,608 rows at 2226 columns, 255 MB)
  and not to the three views of the window, because the guard does not know the view.
  Measured by holding the measurement pass of a 2000 px window over `mixed(6000)`: 19,368 rows in 3 textures at scale 1
  where the settled tile is 6,576 rows in one.
  Before the change the same case was cut to one texture.
  The tile is correct and bounded, and the next placement after the pass replaces it;
  a view past it is not shown wrong.
  The window test of the textures (`a_2000_px_window_gets_a_tile_of_textures_...`) failed once in the full run because
  the pass was late under 21 parallel tests (4 textures at scale 2); it now waits for the pass.
- A tile of another shape (a different number or size of textures) is shown by the frame after the one that drew it:
  the first frame of a window, and a re-center to a tile of another shape.
  `source_typography_repaints_without_changing_selection` compared its first frame, taken before that, with a later one
  and failed in every run alone; it now takes a frame first.
  A tile of the same shape is drawn in place and shows in the frame that draws it (`a_tile_of_the_same_shape_...`).

### Not run

- A window past the limit in a real compositor; fractional scales; the drag scenario; a stack at scale 1.5.
- The cadence at sizes where the stack is more than one texture (7680x4320): those windows were captured, their scroll
  was not measured.
- A readback of the tile's textures in the running application
  (the tests read them in the harness; the captures read the composited window).

### Images (under `~/temp/agent/cs-shots/` and `~/temp/agent/cstack/`)

- `cs-shots/B-w8-s1-dark`, `B-w8-s1-light`, `B-w4-s2-dark`, `B-w4-s2-light`, `B-t-s2-dark`, `B-t-s2-light`:
  shots `s0` to `s3` of the stack scenario; `A-*` the same of origin/main;
  `F-*` the positive control of the seam measure.
- `cstack/v-b8-s1.png`, `v-b8l-s1.png`, `v-b8-s1-crop.png`, `v-tl-s3.png`, `v-w4l-s2.png`:
  the views that were looked at.

### Gates

Product snapshot `628fb36f39` (later commits change documents only), one run each of `build`,
`inspect:no-software-renderer`, `test` (801 passed, 1 skipped), `test:bin` and `test:native -- 'all()'`
(288 passed, 5 skipped); `lint` ran on the same tree before the commit (0 findings).
The first snapshot, `9250c1981a`, failed two native tests, so the gates ran again on a new snapshot (no rerun):

- `source_typography_repaints_without_changing_selection` failed every time it ran alone:
  the frame it kept first was taken before the window showed the first tile (a tile of a new shape is shown by the
  frame after the one that drew it), so its comparison with a later frame of the same weight failed;
  the test now takes a frame first.
- `a_2000_px_window_gets_a_tile_of_textures_within_the_devices_limit_at_both_scales` got 4 textures where 2 were
  expected, once in the full run and not in 3 runs alone: the measurement pass was late under the load of 21 parallel
  tests.
  Holding the pass reproduced a tile past three views (see "Limits that the change brings"), the test now waits for
  the pass.
- Lint found 89 breaks of the documentation line rule in new comments, a file of 332 code lines (limit 300; the
  underline frame moved to `src/tile_scene/frame.rs`), and a closure that `show_slices` took (it now returns the new
  list); all fixed on the new snapshot.
- The source guards were run on the earlier snapshot; the refactor moved no anchor they use.
### The minimap picture in a fling

Session of 2026-10-22 on branch `worktree-agent-afaeefdb803099e40`, base `212d45cdfb`.
It settles the finding "During a fling the minimap picture lags the box" of the previous subsection
(there "not examined further").
The lag is a defect of ours, with a measured cause and a fix; the code tile outrun that the same subsection
mentions (647 blank rows at the bottom of the code view) is out of scope and was not touched.

#### Measure and its positive control

- Release build of `origin/main` (`212d45cdfb`; sha256 prefix `27b877741a056ecf`, the same binary as the installed one),
  `monochromatic-nested-wayland-session` 0.6.1 with `--headless --gpu /dev/dri/renderD128`, 1100x660, scale 1, dark.
  The file is 50,000 identical lines (`let value = compute(alpha, beta); // note`), so every row of the picture repeats.
  The input is the previous subsection's: `scroll 600 330 0 3777 steps=8 step-ms=16`,
  recorded with `record DIR 30 bmp` from 0.5 s before the command to 5 s after it (about 28 to 30 frames per second).
- A row of the minimap track (logical rows 32 to 660, columns 995 to 1095) is blank when all its pixels are one colour.
  Per frame the script reads the longest blank run from the bottom (`trail`) and from the top (`lead`) of the track.
  A frame is "blank" when either run is at least 8 rows (two code lines).
  Scripts and results are in `~/temp/agent/mmlag/` (`mm-lag2.mjs`, `ab.mjs`, `sum.mjs`).
- Positive control: a file of 100 lines (the picture ends above the bottom of the track).
  The metric reads 163 blank rows at the bottom in 60 of 60 frames, and a screenshot of the same run shows the
  picture ending 163 rows above the bottom of the track.
  Frames before the input and a settled track read 0 to 2 rows.
- The spread: runs of one binary against itself, A against A2, alternating, in the same sessions as the other arms.

#### Counts before the fix

Share of the recorded frames from the command on that have at least 8 blank rows (20 runs per arm,
arms rotating A, B, A2 per round, one app run at a time, under the load there was):

- A: mean 0.449 (sd 0.037); A2 (the same binary): mean 0.433 (sd 0.101).
  Frames per run 153; frames per run with a blank track A 46 to 78 (median 70).
  Largest blank run per run: A mean 309 rows (260 to 370) of the 628 rows of the track.
  The last blank frame is frame 152 of 153 in A and in A2: the lag does not end within the 5 s that were recorded.
- An earlier batch with the first script version, 16 alternating pairs: A mean 0.464 (sd 0.123), A2 0.412 (sd 0.167).
  Those runs were loaded harder and their spread is wider; the difference between the arms stays inside it.
- 1100x3700 (the previous subsection's window), 10 frames per second, 7 rounds: A 0.678 of frames (sd 0.033),
  A2 0.689 (sd 0.038), largest run 304 and 285 rows of the 3,668 of the track.
- Slower flings: 1000 px in 8 steps: 0 blank frames in 5 of 5 runs of A; 2000 px in 8 steps: A 0.021 of frames
  (largest run 16 rows), 3777 px: 0.45.
  The lag starts at flings of about 15,000 px per second of input and grows with the speed;
  the 120-step scroll of the cadence tool (3,000 px in about 2 s) does not show it.

#### Cause

Interventions on a build of `origin/main` with two temporary environment switches (`IDE_X_MARGIN` for the bands kept on
each side of the track, `IDE_X_SLIDE_MS` for the period of requests during a scroll; removed, never committed),
8 rounds of the four arms alternating, share of blank frames:

- Control, margin 1 (the shipped value): 0.390 (sd 0.233; these runs recorded 19 to 22 frames per second under a heavier
  load, so the spread is wide).
- Margin 8 on both sides: 0.010.
- Request period 16 ms instead of 60 ms: 0.102.
- Both: 0.067 (one run in the arm lost its whole track for four recorded frames, as one run of the margin arm did;
  such frames also appear in 4 of 4,650 frames of the first batch at low frame rates
  and in 0 of 9,158 in the later batch;
  they are not examined here).

The coverage of the picture is the cause; the pace of the requests is a smaller part.
The debug log of a fling of A (`monochromatic_ide::native::minimap=debug`) gives the chain:

- A picture is requested for the bands the track shows at that moment and one band on each side
  (`Geometry::bands_for`, one band is 768 px of text or 128 rows of the picture).
- The worker builds it in 3.0 ms on average (3.8 ms at most), but it is handed to the GPU 33 ms after the request
  (the 20 ms poll of replies plus the frame in which the window thread takes it), and the next request follows
  66 ms after the last one (the 60 ms `SLIDE` period plus the poll).
- In the fling the top of the picture moves 256 to 384 picture rows from one picture to the next
  (about 3,900 rows per second, 23,000 px of text per second, still so after 5 s).
  A picture that holds 128 rows beyond the track is exhausted about 33 ms after it is on screen, and the next one
  arrives about 66 ms after that.
  The rows beyond the end of the picture are blank, and each picture lands further down than the last,
  which is the flicker between a short and a full picture in the recorded frames.
- Not tried: keeping the old picture under the new one.
  The new picture covers more in the direction of travel than the old one did, so it would help only after a turn.

#### Fix

The window of bands a request names holds `LEAD` (8) bands on the side the view is heading toward and one on the other,
and one on each side at rest (`Heading`, `Geometry::bands_toward` in `src/minimap/`).
`Heading` is a state of the window (`MinimapState::heading`), updated by `minimap::note` from the top of the track:
a move of a whole band from where the heading was decided sets it, a smaller creep never flips it.
That keeps the key stable between requests: the pace and the number of requests are those of before
(77 requests in the 5 s of a fling in both, from the debug logs of one run each, a gap of 66.6 and 66.5 ms
between pictures, no frame asked for by the hand-over wait).
What changes is the size of a picture: 14.8 bands and 477 lines against 7.9 and 257, and 5.0 ms of worker time against
3.0 (7.7 ms at most against 3.8).
Not done: a reversal still has one band behind the track until the heading turns (a band of movement and a request),
and the first 100 to 150 ms of a fling still have the old picture, since the heading needs a whole band to be set.

Counts after, same conditions, 20 rounds of A, B and A2:

- 1100x660, 3777 px: B has 0.012 of frames with a blank track (sd 0.003; two frames per run, the first two or three
  frames of the fling; the largest run per run 122 rows (1 to 203)), against A 0.449 and A2 0.433.
  The last blank frame of B is frame 4 after the command at the latest.
  Blank rows summed over the frames of a run: A 5,526, A2 5,438, B 137.
- 1100x3700, 7 rounds: B 0.020 (one frame per run, the largest run 76 rows) against A 0.678 and A2 0.689.
  In the first round a settled track at the end reads 1 blank row in B and 22 to 27 in A and A2.
- 2000 px in 8 steps: B 0 blank frames in 5 of 5 runs (A 0.021).

Stalls (the frame-shift probe of the 120-step scroll, `mm-stall.mjs`, release builds, nested window, slot 50 is the
tile hand-over):

- A against B, 20 pairs: A 1 stalled tick in 19 usable runs (slot 31), B 4 in 20 (slots 140, 39, 94 and 118);
  slot 50: 0 of 19 and 0 of 20.
- A against A (A2), 15 pairs: A 2 in 14 usable runs (slots 66 and 31), A2 5 in 14 (slots 97, 73, 121, 142, 125);
  slot 50: 0 in both.
  The same binary differs by 3 stalls in 14 runs, so B's 4 in 20 is inside the spread.
  One run per batch lost its probe (a failed frame read) and is left out.
- No stall at slot 50 in any of the 67 usable runs.

#### Tests and guards

- `the_window_of_a_fling_reaches_ahead_on_the_side_it_heads_toward` and `a_heading_follows_the_track_by_whole_bands`
  (`tests/minimap.rs`) hold the window and the heading rule without a window of the application.
- `a_fling_is_given_a_picture_that_reaches_the_bands_it_is_heading_toward`
  (`src/native/minimap_tests.rs`) flings down and up through the real window and checks that the shown picture holds
  the text the track reaches `LEAD - 1` bands further on, in the direction of the fling.
- The existing guards `minimap-bands-uncapped` and `minimap-window-margin` were re-anchored to the new code.
  New guards in `bin/inspect-source-guards.mjs`: `minimap-lead-down`, `minimap-lead-up`, `minimap-heading-needs-a-band`,
  `minimap-heading-followed` and `minimap-window-has-heading`.
  All seven (the two re-anchored and the five new) ran in a disposable copy,
  each as baseline passes, guard removed fails (the named test, with the named message), restored passes:
  7 of 161 guards; the other 154 were not run.
  The guard run used the tip `8880ec6752`.

#### Gates

Run once each on the tip `8880ec6752` (the last code commit; a first `lint` run found five shadowed names, five line
breaks and a missing impl doc in the new code, fixed in that commit, and `lint` passes on it):
`lint` 0 findings; `test` 799 passed, 1 skipped; `test:native -- 'all()'` 283 passed, 5 skipped (224 s);
`test:bin` 60 passed; `build` finished; `inspect:no-software-renderer` passes.
Commits: `d6668b1aa9` (code, unbuilt), `b2b4cfd851` (rustfmt and guards), `8164722cbb` (README and docs),
`8880ec6752` (lint fixes).

#### Limits

- Headless compositor on the GPU, its recorder as the clock, a synthetic touchpad fling; no real touchpad,
  no real compositor, no drag scenario.
- The metric counts rows of one colour in the track; a picture that is there but wrong is not seen.
- The load of the host was not controlled; the arms alternated and a same-binary arm gave the spread.
- The measured B binary was built from commit `b2b4cfd851` plus documentation; the final tip differs from it only in
  comments and the README.
- The window the previous subsection saw at 3.5 s (91 to 94 blank rows) was reproduced in kind (22 to 27 blank rows at
  5 s in A) but not in number; fling speed varies from run to run with the load.
