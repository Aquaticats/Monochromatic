# Native suite flakes: state after the third flake session

> Moved 2026-10-29 by the generic package category elimination
> (`doc/planning/generic-category-elimination.md`);
> paths and names below this note are historical:
> `package/desktop-app/ide` is now `package/ide/app`.

The section "Third session" below is the third session (branch `worktree-agent-a4671ba160cf3feec`,
2026-10-22);
the sections after it are the second session (branch `worktree-agent-a5d7faec36a652c16`),
kept as they were
where the third session did not change them.
Package `package/desktop-app/ide`.
No test runs under retries,
and `.config/nextest.toml` of the package is unchanged.
The owner rule of 2026-10-20 (recorded in `doc/handover/slint-ide-0x.md`) holds:
"proven unfixably flaky" is a very high bar,
and a retry override needs a report that lists every alternative tried or ruled out with evidence.
No test met that bar,
so there is no override.

## Third session (2026-10-22): conditions and results

### Conditions

Every count below was taken with the permitted tool under the load the host had:
the host's load average peaked at 8 to 56 across runs (the tool samples it),
video memory sat at 6.0 to 7.98 of 7.98 GiB in every run,
and my own real jobs ran beside the hunts (`test:native`,
`test`,
`build`,
a 21-thread subset run).
No synthetic load was started.
Contention was raised only by the tool's own parameters:
up to 8 copies with 4 test threads each,
32 test processes sharing the 2 CPUs.
The build container used 8 CPUs (`IDE_CONTAINER_CPUS=8`) for the tool's build.

A filtered `test:native` (the gate form `'all()'` included) runs one test at a time:
the sum of the 283 test durations of the gate run is 198.5 s and its wall time 198.6 s (mean concurrency 1.0);
nextest's own help says so:
"--no-capture:
Run tests serially and do not capture output".
Only an unfiltered run uses the thread count of the bound.

### Fixed in this session

#### Task `test:native:stress` prepared nothing

On a fresh worktree the task failed at its first build (`build.rs`:
`target/debug/runtime/manifest.json` cannot be read),
because `test:native` depends on `runtime` and `notices` and the stress task did not.
It depends on both now (`package/desktop-app/ide/mise.toml`).

#### The picture test of fifty thousand lines (`minimap_tests::fifty_thousand_lines_give_a_bounded_picture_without_blocking_the_window`)

New flake,
found by a 21-thread subset run in the tool (1 failure in 127 tests),
then reproduced at will.
Cause:
the window thread applies every finished picture in one poll,
so a loaded host hands the test the third picture of the open as the first one it sees.
Each open asks for several builds (the first width,
a highlighting reply,
a settled extent);
a later build finds its lines in the worker's row cache and shapes none,
and the test read that build's statistics as the cold build's
("a cold build shapes its lines",
left 0,
right 97,
"pictures applied 3,
lines shaped by all of them 291").
The running total of applied pictures cannot stand in for the cold build either,
because a dropped picture also fills the cache (3 applied,
65 shaped by them,
97 reused by the last).
Fix:
the test asserts what the last picture can show (every line shaped or reused,
none skipped);
a cold build shaping every line stays asserted on an empty cache in `tests/minimap_scene.rs`.
Counts under the same conditions (tool,
8 copies x 50 iterations,
1 thread each,
2 CPUs):
before 317 of 400 failed,
after the first version of the fix 1 of 400 failed (the cumulative check above),
after the final version 0 of 400.

### Open item 1: the annotation stability tests

Not reproduced;
the cause is not found;
they stay open and instrumented,
and are not retried.
Counts of this session
(rows_arriving_in_view = `rows_arriving_in_view_move_only_lines_beneath_them`,
rigid = `scrolling_across_annotated_lines_is_rigid`),
all with 0 failures:

- 6 copies x 60 iterations,
  2 threads each,
  load average peak 56.1,
  video memory 7.98 of 7.98:
  360 runs each of rows_arriving_in_view,
  `rows_arriving_above_the_view_move_no_visible_pixel`,
  `a_reload_holds_row_space_until_annotations_return`,
  `rows_above_the_view_wait_until_scrolling_has_stopped` and `the_noise_tolerance_tells_a_shifted_row_from_a_re_render`.
- 4 copies x 8 iterations,
  2 threads each,
  beside my gate runs,
  build and test runs,
  load average peak 18.7:
  32 runs each of all six tests of the file including rigid,
  and two disposable probes in the same process model:
  32 runs of 150 inject-and-remove cycles (9,600 pairs of window frames,
  134,400 line comparisons)
  and 32 runs of six passes of 300 one-row scroll steps of the rigid test (57,600 steps).
- 8 copies x 40 iterations,
  4 threads each (32 test processes on 2 CPUs),
  load average peak 34.7:
  320 runs each of the five tests above and of the new stall guard (1,920 runs).
- 6 copies x 10 iterations,
  3 threads each:
  60 runs of rigid.

This session ran rows_arriving_in_view 737 times and rigid 92 times;
with the 1,310 runs of the two in the earlier sessions that is no failure in about 2,100 runs of the two.
No frame was kept,
because no comparison failed.

What the records of the one failure say (`~/temp/agent/stress-lV2AMP`,
banned heavy-load tool,
8 copies x 60 iterations):

- The failures came in a burst,
  not at random.
  `copy-7.log` has rigid (iteration 57 of 60,
  8.3 s) and rows_arriving_in_view (iteration 57,
  9.0 s) 0.7 s apart;
  `copy-2.log` has rigid at iteration 56;
  `copy-4.log` has `wrap_tests::opening_fifty_thousand_lines_never_blocks_the_window_thread` failing at iteration 55
  in `fs::write` of its fixture with `StorageFull` ("No space left on device").
  The container's `/tmp` is an overlay on the host disk (checked:
  1.9 T,
  `df` in a container),
  so the host disk was full at that time for every process that wrote to it.
  Whether the full disk touched pixels is unknown.
- The rows_arriving_in_view message names line 1,
  row 8,
  and 4 of 696 texels that differ by up to 104 levels,
  the first at compared index 1 (146 against 250).
  `letter_room(1)` starts the compared part at the left edge of the one digit of line 1's number,
  so index 1 is the digit's second column,
  and a one-pixel shift of that digit's stem explains four texels in a row.
  The digit is drawn by the toolkit's FemtoVG renderer (a `Text` with `opacity: 0.6` in `ui/app.slint`,
  glyphs from `femtovg-0.27.0` `text.rs`,
  placed at `glyph.x.trunc()` with a sub-pixel bin of `glyph.x.fract()` in
  tenths),
  not by the Vello tile.
  A sub-pixel bin that flips for a tiny difference in x would produce exactly this,
  and it is not shown;
  the rigid failure of the same burst (45 non-contiguous rows,
  largest difference 135 levels) is also a glyph-sized
  difference.
- Both frames of that failure were not kept (the frame keeping was added afterwards),
  so the hypothesis cannot be tested.

What is now in the tests for the next failure:

- Every comparison that fails keeps both frames in `target/flake-frames/` (as before).
- `shot()` takes a second frame of the same state and the facts of a failed comparison say whether it was equal to
  the first (`second frame of the same state equal: yes` or the first differing pixel),
  and,
  when the tile counters moved between the two frames,
  the counters before them.
  A frame that is not repeatable is a renderer glitch;
  a repeatable frame that differs from the other moment is a state difference.
- Next step on a failure:
  open `target/flake-frames/*-first.ppm` and `-second.ppm`,
  find the texels,
  and compare with the repeatability fact.

### Open item 2: the two incomplete entries

#### The scroll quiet-time test

Complete.
Cause:
the scrolling rule asked the real clock whether 200 ms had passed since the last scroll step,
so a host that did not run the window thread for that long showed rows early
(commit `3bcdc17603`,
2026-10-12,
made the clock one a test can hold).
Evidence now:
the new test `a_reader_that_stalls_past_the_quiet_time_still_has_rows_wait` holds the clock,
scrolls,
sleeps 300 ms of real time (the stand-in for the stalled host),
and requires the rows to wait.
Guard proof:
with `quiet_now` in `src/native/rows/quiet.rs` made to ignore the held clock (disposable edit,
restored),
the guard failed 20 of 20;
the same body written against the real clock (a disposable copy) failed 10 of 10 with
"rows above the view moved the offset while the reader was scrolling" (2433 against 2407);
with the clock held the guard passed 10 of 10,
then 320 of 320 and 32 of 32 under the load above.
The original test itself passed 20 of 20 with the clock ignored,
and a copy of it with the real clock and no stall passed 600 of 600 under the tool,
so the natural failure is not reproducible with the tool and the stall is the control.
`rows_above_the_view_wait_until_scrolling_has_stopped` after the fix:
600 of 600,
360 of 360,
32 of 32,
320 of 320,
60 of 60.

#### `the_view_follows_the_caret_through_every_visual_row`

Cause (inferred,
not reproduced):
`measured()` in `wrap_tests.rs` said the measurement had finished while a newer pass was still running.
`TileWorker::take_pass` closed the pass on the finishing output of a replaced pass
(commit `0e390da7ca`,
2026-10-19,
found through `wrap_clip_tests`);
the caret test counts the rows of the map right after `settled`,
and the recorded failure (162 rows counted,
121 in the map after the Down presses) is rows counted before the
newer pass shrank the map.
The failures were recorded on the minimap branch (before 2026-10-15) and on `main`;
no later failure of this test is recorded.
Evidence:

- The guard of that fix,
  `tile_build::pass_tests::the_finish_of_a_replaced_pass_leaves_the_newer_pass_open`,
  failed 20 of 20 with the fix reverted (disposable edit,
  restored) and passes with it.
- With the fix reverted and the highlighting-reply wait of `reader()` disabled (both disposable),
  the caret test still passed 600 of 600 and the known wrap clip test `revealing_a_late_row_of_an_unmeasured_last_line...`
  600 of 600 under the tool (it failed about 1 in 100 at one CPU when it was found).
  The tool does not reproduce the natural failure of either,
  so the link between this test and the fix rests on the mechanism and on the dates,
  not on a count.
- A before count at the old commit is impossible:
  `2255008ede` runs on the winit backend,
  which needs a display (`Error initializing winit event loop`),
  and the GPU snapshot harness replaced it.
- After:
  600 of 600 (6 copies x 100,
  load average peak 40.4),
  600 of 600 (the reverted arm),
  60 of 60
  (6 copies x 10 with 3 threads each),
  10 of 10 beside the guard runs.
- The test now names the rows counted,
  the wrap width and the viewport width at that moment,
  the rows in the map
  after the Down presses,
  and the facts of both moments if its last assertion fails.

An undiagnosed recurrence stays open:
this entry is "fixed by inference".

### Open item 3: video memory

Measured,
no change made.
Method:
a sampler (a short read-only Node script of the session,
not kept,
a loop over `/proc` with the fdinfo and sysfs reads named here,
2 s interval,
2,765 s,
1,374 samples)
read the video memory (`drm-memory-vram`) and the system memory mapped for the card (`drm-memory-gtt`) of every
test process from `/proc/<pid>/fdinfo`,
and the card's counters from sysfs,
while the runs above went on.
Results:

- One test process holds 0.28 GiB typically and 0.52 GiB at most.
- 21 test processes at once (the tool with 21 threads,
  a 127-test subset):
  2.37 GiB in all.
  32 at once (8 copies x 4 threads):
  3.42 GiB.
  All test processes of all sessions at the busiest moment (32 of them):
  3.62 GiB.
- The card sat at 98 percent or more in 156 of 1,374 samples,
  with other programs (desktop,
  browsers,
  other sessions) holding up to 7.39 GiB of it.
  So the card is full without the tests,
  and halving their parallelism would free well under a third of it.
- The card spilled into system memory (card GTT peak 12.66 GiB,
  of which the tests 4.63 GiB),
  and no test failed or slowed in the way a lack of video memory would show:
  every failure of this session has another cause (the picture test above,
  stall control,
  guard proofs).
- The full-run comparison (default threads against a cap) was not made,
  because it needs a second full run of one snapshot.
  The subset runs above stand in for it:
  the 21-thread subset (127 tests,
  126 passed,
  the one failure being the picture test above,
  reproduced at 8 processes) and
  the 32-process hunts passed at a card as full as the full runs.
- Whether the textures could be freed or shared was not examined further:
  nothing shows that video memory matters,
  and the smallest correct change is none.
  `IDE_CONTAINER_TEST_THREADS` stays the override if a failure is ever attributed to video memory.

## The stress tool

Run it with `mise run //package/desktop-app/ide:test:native:stress --filter '<nextest expression>' --count N --copies K --threads T`.
The tool builds once in a container with the normal build bound.
It then runs the filtered tests inside a second GPU container limited to 2 CPUs and 2 GiB,
and prints failures per test.
The tests contend with each other inside that cap;
the tool starts no host load.
Its logs are under `~/temp/agent/stress-*`.

What the settings measure (observed in this session):

- A filter that selects one test,
  with `--copies 3 --threads 2`,
  runs about three test processes at once on the 2 CPUs.
  That is little contention:
  `rows_arriving_in_view_move_only_lines_beneath_them` passed 300 of 300 that way
  in 39 s.
- `--copies 8 --threads 1` raised the contention (800 runs of the same test in 104 s,
  video memory peak 6.75 GiB)
  and still passed 800 of 800.
- The failures this suite had were seen under a host load of 10 to 12 CPUs on the first version of the tool,
  which is banned (rule `NLG`).
  A rate seen there cannot be re-measured with the permitted tool,
  which is why deterministic controls
  (a chosen hold time,
  an injected state) are used below where the cause is known.
- The build container uses 75 percent of the host's CPUs unless `IDE_CONTAINER_CPUS` is set;
  this session set it to 8 for every build and every task run.

## Fixed in this session

### Find box context menu (`query_input_tests::find_box_context_menu_runs_each_entry`)

Cause,
found by reading the toolkit source and confirmed by a control:

1.  The find bar sits at the bottom of the 660 px window.
   The test right-clicked at `find_clear_x - 60` and `find_clear_y + 24` (y = 632,
   x = 951 or 964).
   `place_popup` (`window/popup.rs` of `i-slint-core` 1.18.1) clamps the menu into the window,
   so the menu is pushed up and left,
   and the pointer rests on one of its entries
   (which entry was not measured).
2.  A menu entry runs on a pointer release when 300 ms have passed since the entry was created
   (`widgets/common/menu-base.slint` of `i-slint-compiler` 1.18.1,
   the `up` branch of `pointer-event`:
   `animation-tick() - root.open-time > 300ms`,
   inside the entry's rectangle).
3.  `process_mouse_input` (`window.rs` of `i-slint-core`) refreshes the animation clock from the real clock
   on every pointer event,
   so the 300 ms are real time between the right press and the right release.
   Under host load the test thread was descheduled for that long,
   the release ran the entry under the pointer,
   and the entry closed the menu.
   The keys then went to the find box:
   Return stepped the find match (status "2/2"),
   and a typed character was appended ("needlex" against "x",
   the older message).

The earlier reading,
a release that closes the popup by a second Slint path,
was wrong:
the popup counts "before the press 0,
after it 1,
after the release 0" are exactly this path.

Fix (commit `39065b2c14`):
the test right-clicks at the left part of the text area,
`sidebar width + divider + 36 px`.
The clamp leaves the menu's x at the pointer's x there,
so the pointer rests on the menu's frame,
outside every entry.
The helper `open_menu` takes a hold time between press and release.
The new test `a_slow_right_click_leaves_the_find_menu_open` holds 400 ms.

Guard proof (GFP) and counts,
4 copies x 150 iterations on the 2-CPU tool,
one thread per copy:

- Old click position,
  hold 400 ms:
  600 of 600 failed,
  with the natural signature
  ("popups before the press 0,
  after it 1,
  after the release 0,
  after settling 0;
  find status 1/2").
- Old click position,
  `find_box_context_menu_runs_each_entry` itself:
  0 of 600 failed.
  The natural rate on the permitted tool is below that;
  the heavy-load rates were 2 of 240 and 5 of 480.
- New click position:
  hold test 0 of 600 failed,
  the original test 0 of 600 failed.

The deterministic hold stands in for the load-dependent rate;
the 2 of 240 and 5 of 480 are not re-measurable.

### Highlighting-reply wait in the other fixtures that open files

`wait_for_highlighting` in `src/native/find_tests.rs` (extracted from `reader()`) is now also called by
`watch_tests::open`,
`refresh_latency_tests::measure`,
the first fixture of `navigation_tests`,
and `language/test_support.rs` (`reader_with`).
The second `navigation_tests` fixture (`native_failed_open_keeps_source_and_does_not_promote_history`)
opens no file at start (`State::new("", None)`),
so there is no reply to wait for and it is unchanged.
`minimap_tests::open` and `wrap_tests::open` go through `reader()` and already waited.

This removes an exposure;
no failure of those fixtures was ever measured,
so there is no before count.
After:
42 tests of `watch_tests`,
`navigation_tests` and `native::language`,
3 copies x 8 iterations,
0 failures
(the only guard of the wait is `a_reader_is_opened_with_its_highlighting_applied`,
which covers `reader()`).

### Frames kept by every stability comparison

`kept_if_different` in `annotation_stability_tests.rs` writes both frames to `target/flake-frames/`
for every frame comparison of that file that fails (before,
only `moved` and the rigid scroll did).

## Open, not reproduced

### Annotation stability tests

The recorded failure (log `stress-lV2AMP/copy-7.log`,
heavy-load tool,
480-run hunt):
`rows_arriving_in_view_move_only_lines_beneath_them`,
"line 1 changed its pixels in row 8 of its code row
(window rows 64 and 64);
4 of 696 texels differ,
by 104 levels at most,
the first at index 1 (146 against 250)";
facts before:
`renders 4 tiles_rendered 1`,
after:
`renders 5 tiles_rendered 2`.
So the window frames straddle one tile re-render,
and 4 texels at the start of the compared part of the row
(which begins at the line's number) differ by more than the measured render noise of one level.
The rigid scroll failure of the same hunt:
31 rows differ from row 101 to 511,
largest channel difference 198.

Tried this session,
with the permitted tool:

- With the highlighting-reply wait:
  `rows_arriving_in_view...` 1,145 passed of 1,145 (45 + 300 + 800 runs);
  `scrolling_across_annotated_lines_is_rigid` 165 of 165 (45 + 120);
  the other four tests of the file 45 of 45 each.
- With the wait disabled in a disposable edit (so the late reply can re-render):
  every test of the file 90 of 90.
  So the late reply is not shown to cause these failures,
  and its removal did not make them reproducible.
- The tile render itself is stable:
  `repeated_renders_of_one_tile_spread` passed 400 of 400 runs
  (16,000 renders),
  none above one level.
  The window frame (Slint's own text and gutter) is not covered by that probe.

Not reproducible with the permitted tool,
and the cause is not found.
The next failure now keeps its frames (`target/flake-frames/*-first.ppm` and `-second.ppm`),
and the message names the texels and the facts of both moments.
Next step when a failure exists:
open both frames,
find the 4 texels,
and decide between a sub-pixel glyph shift
of the gutter number and a renderer outlier.
This stays an open,
instrumented flake and is not retried.

### Not reproduced, no instrumentation added

`write_wait_tests` (one failure in 19 parallel runs of an earlier head,
in `doc/planning/ide-gpu-snapshot-harness.md`),
the two `watch_tests` with 400 ms budgets (failed together once in the same 19),
`wrap_tests::rows_above_the_view_changing_height_move_no_visible_pixel`,
and the nine-level pixel difference of `rows_above_the_view_wait_until_scrolling_has_stopped`
(0 of 135 in this session:
45 with the wait and 90 without).
The 400 ms budgets state how soon the product must follow a file change,
so a longer budget would weaken them;
no change was made without a reproduction.

### Environmental

A full disk (`No space left on device`) in `opening_fifty_thousand_lines_never_blocks_the_window_thread`
and a wgpu `vello.scene` buffer validation error in `the_extent_settles_as_chunks_arrive` were seen once each.
Video memory sits at the card's limit in full runs (next section),
which is one candidate for the second;
unproven.

## The runtime task is safe for concurrent callers

`build` and `test` both depend on `runtime`, so starting them together ran the task twice in one tree.
One caller failed with `ENOENT: no such file or directory, rename 'target/debug/runtime/grammars/awk.so.new'`,
because both copied to the same staging name and the first rename took the file away from the second.

The staging name was one of several shared writes.
Both callers also ran `ide-runtime fetch` and `build` in the one directory `target/runtime-config`;
the first paired start into a cold tree failed in both callers with 22 failed `git fetch` runs
(`unable to create temporary file`, `cannot open '.git/FETCH_HEAD'`).
Unique staging names alone would therefore not have been enough.

The fix is in `package/desktop-app/ide/bin/runtime-publish.ts` and in the `runtime` task of
`package/desktop-app/ide/mise.toml`:
- A kernel `flock` on `target/runtime.lock` serializes everything from the `ide-runtime` build to the last publish.
  The kernel drops it when its holder dies, so a killed run leaves no stale lock.
- Each file is staged as `<name>.<pid>.<random>.new` beside its destination, renamed into place,
  and left alone when the destination already holds the same bytes.
- `manifest.json`, which the build script reads first and which makes the directory valid, is written last.
- Staging files left by a killed run are swept under the lock, because the build script embeds every file under
  `queries` and `licenses`.

Measured in one worktree with the grammars built,
deleting `target/debug/runtime` and `target/release/runtime` before each pair
(2 CPUs and 4 GiB per caller, two `mise run //package/desktop-app/ide:runtime` started at the same moment):
- Before the fix, 20 paired starts: 10 pairs had one failed caller (10 of 40 runs), every one the `awk.so.new` ENOENT.
  This is the positive control: the harness sees the failure.
- After the fix, 20 paired starts: 0 failed callers (0 of 40 runs); a further 20 on the final code: 0 of 40.
  One pair took 307 s because a cargo process of another session held the package cache lock of the shared
  `ide-cargo` volume (`Blocking waiting for file lock on package cache`); the others took 4 to 10 s.
- A SHA-256 over every path, file type and file content of `target/debug/runtime` (1622 entries, no links)
  is `671506250b7c783e868b239ef2e61bd02e30827aad7ee24ac2ec7b7f4ad51e37` before the fix,
  after a single caller with the fix, and after every paired start that ended in a tree
  (20 before the fix, 64 after; `release/runtime` equals `debug/runtime`).
  The last 20 after-fix pairs ran on the final lock code (waits through `events.once` and async iteration);
  the first 44 ran on an earlier version of it with the same behavior.
- A reader polling every 2 ms during 8 further pairs, like the build script, found a readable manifest 4045 times
  and an incomplete tree 0 times.
  Two earlier probes reported 136 and then 3 incomplete states while the harness deleted the tree between pairs.
  The third probe separated those cases and saw none of either kind, so the 3 are explained by reading the task,
  not shown by the probe: once a manifest exists the task removes no listed grammar
  (`removeStaging` matches only the `.<pid>.<random>.new` names and a rename leaves no gap),
  so only the harness's own `rmSync` can make a listed file disappear.
  The harness now removes the manifest first.

`bin/runtime-publish.test.ts` (part of `test:bin`) holds the unit checks.
Its concurrent-publishers test fails with the same `ENOENT ... .so.new` when the staging name is made fixed again
(shown on the commit that added it, then restored), and a second test checks that a lock whose holder is killed is
released.
Gates on the final code, once each, at 8 CPUs and 16 GiB (not the 21-CPU default): `lint` passed,
`test:bin` 71 passed, and the unfiltered `test:native` 283 passed and 5 skipped (45.3 s).
A caller that waits for the lock prints
`Waiting for another runtime preparation to release target/runtime.lock` after one second.

## Test thread count against video memory

The third session measured this per process and decided:
no cap (section "Open item 3:
video memory" above).
The second session's text follows.

The card reports 7.98 GiB.
One reading with no run of mine active:
0.5 GiB (a desktop session and other agents share the card).
Peaks the stress tool sampled in this session (all programs on the card):
5.57 GiB with about 3 test processes,
6.75 GiB with 8,
and 7.3 to 7.5 GiB with 6 of the mixed annotation tests.
Use per process varies with the window size of the test,
so the count alone does not predict the peak.
Not measured:
a full default run (21 threads on this host) against a capped run,
because that needs a second
full run of one snapshot,
which the rules forbid.
No failure in this session is attributed to video memory,
so no default was changed.
The override exists already:
`IDE_CONTAINER_TEST_THREADS` (`bin/container-bounds.ts`).
A cap should be proposed only with a failure it removes;
the `vello.scene` error above is the only candidate.

## Gates on the final tip

Third session,
final code tip `7529ea3e1a` (the commits after it change only this document):
`lint` passes (once,
at `b103fed788`,
which differs from the code tip only in this document),
`test:native 'all()'` 283 passed and 5 skipped with no FLAKY,
FAIL or SLOW line (174.7 s,
one test at a time),
`test` 797 passed and 1 skipped,
`test:bin` 60 passed,
`build` finished the release profile (once more after an earlier attempt of this session collided with a concurrent
`runtime` task of the same tree and failed before compiling),
`inspect:no-software-renderer` reports the software renderer is not linked.
An earlier snapshot of this session (`51c1aed3ab`) passed the same gates (283 passed and 5 skipped in 198.6 s,
797 passed and 1 skipped,
60 passed);
the later commits changed one test file (the picture test) and this document.
`inspect:*-guards` were not run:
no guard changed.

Second session:
`lint` passes (rustfmt diffs fixed with `format:rust`,
semantic-line-break findings fixed),
`test:native -- 'all()'` 275 passed and 5 skipped with no FLAKY or FAIL line (177 s),
`test` 797 passed and 1 skipped,
`test:bin` 60 passed,
`build` finished the release profile,
`inspect:no-software-renderer` reports the software renderer is not linked.
`inspect:source-guards` was not run:
no guard was edited (the guards patch product sources;
none names a file this
work changed except `bin/inspect-refresh-latency.mjs`,
a measurement and not a guard).

## Flaky test ledger

Zero tests are under retries.
Counts are failures over runs;
"heavy" means the banned host-load version of the tool or parallel full runs
under a shared host,
"tool" means the permitted 2-CPU tool.
History sources:
`doc/planning/ide-gpu-snapshot-harness.md` ("Flakes after the merge"),
`doc/handover/slint-ide-0x.md`,
and the entries above.

- `query_input_tests::find_box_context_menu_runs_each_entry`.
  Cause:
  release over a menu entry more than 300 ms after the menu opened.
  Fix:
  `39065b2c14` (click position).
  Before:
  2 of 240 and 5 of 480 heavy,
  once in full runs (README Minimap section,
  "needlex" message).
  After:
  0 of 600 tool,
  with the deterministic control 600 of 600 before and 0 of 600 after.
- `native::annotation_stability_tests` (`a_reload_holds_row_space_until_annotations_return`,
  `moved` users).
  Cause:
  Vello re-renders are not bit-identical (one level,
  rounding ties in the coverage sum).
  Fix:
  `same_pixels` with `RENDER_NOISE` of one level where a frame follows a re-render
  (2026-10-20,
  agent `a717e60922dca0448`);
  guard proof in the planning document.
  Before:
  9 of 70 full runs on `main` at the merge across four causes;
  after:
  15 of 15 and 5 of 5 full runs.
- `minimap_reflection_tests::the_minimap_of_find_matches_is_the_average_of_the_windows_pixels`.
  Cause:
  Enter pressed after the first match was already selected.
  Fix:
  the test presses no Enter.
  Before:
  5 to 7 of 72 at 16 busy loops;
  after:
  0 of 72.
- `refused_tile_tests::a_failed_worker_tile_frees_its_job_and_the_next_placement_gets_its_tile`.
  Cause:
  the test polled for an error message that the retry render cleared within 0.2 ms.
  Fix:
  failures kept in debug builds,
  the test waits for the first one.
  Before:
  4 of about 410 runs hung;
  after:
  no recurrence in the later runs.
- `wrap_tests::a_resize_breaks_kept_rows_again_without_shaping`.
  Cause:
  the test counted passes while the syntax reply could still arrive.
  Fix:
  the test waits for the reply.
  Before:
  3 of about 410;
  after:
  no recurrence in the later runs.
- `wrap_clip_tests::revealing_a_late_row_of_an_unmeasured_last_line_puts_it_in_view`.
  Cause (product):
  `TileWorker::take_pass` closed a pass on the finishing output of a replaced pass.
  Fix:
  `8627edeeba`.
  Before:
  about 1 of 100 at one CPU;
  after:
  0 of 1500 (the fix author),
  and 400 of 400 stress runs (the coordinator).
- Two find tests (the paint test among them).
  Cause:
  waits that did not require the active match to spell the whole query.
  Fix:
  `4ab83c59e`.
  Before:
  failed in gate runs;
  after:
  the rerun passed.
- The scroll quiet-time test (`rows_above_the_view_wait_until_scrolling_has_stopped`).
  Cause:
  the scrolling rule read the real clock,
  so a stalled window thread showed rows before 200 ms of stillness counted.
  Fix:
  `3bcdc17603` (a clock a test can hold);
  the guard `a_reader_that_stalls_past_the_quiet_time_still_has_rows_wait` (third session) stalls 300 ms with the
  clock held.
  Before:
  the guard fails 20 of 20 with the held clock ignored,
  and 10 of 10 as a real-clock copy (deterministic stand-in;
  the natural rate is not recorded and not reproducible with the tool).
  After:
  the guard 10 of 10,
  320 of 320,
  32 of 32;
  the test 600 of 600,
  360 of 360,
  32 of 32,
  320 of 320 and 60 of 60.
- The find reveal test.
  Cause:
  a line the test measured too late.
  Fix:
  recorded in `doc/handover/slint-ide-0x.md` ("Both flaky tests of the guard runs were fixed").
  Before:
  3 of 16 at baseline.
  After:
  no recurrence recorded;
  not re-measured in the third session.
- `the_view_follows_the_caret_through_every_visual_row`.
  Cause (inferred;
  package `README.md`,
  third session):
  the test counted the map's rows while a measurement pass was still running;
  `measured()` was true early because a replaced pass closed the newest one (`0e390da7ca`).
  Before:
  3 of 23 on the minimap branch and 1 of 10 on `main`,
  before the GPU harness (not reproducible now).
  Fix:
  `0e390da7ca` (guard `tile_build::pass_tests::the_finish_of_a_replaced_pass_leaves_the_newer_pass_open`,
  20 of 20 failing without it).
  After:
  0 of 600,
  0 of 600 with the fix and the reader's highlighting wait reverted,
  0 of 60 (tool,
  load average 20 to 40).
  The link between this test and that fix is inference;
  the test now names counts,
  widths and facts if it fails.
- `minimap_tests::fifty_thousand_lines_give_a_bounded_picture_without_blocking_the_window` (third session).
  Cause:
  the test read the statistics of the third picture applied in one poll as those of the cold build.
  Fix:
  the test asserts shaped or reused for the last picture;
  the cold-build claim lives in `tests/minimap_scene.rs`.
  Before:
  317 of 400 (tool,
  8 copies,
  1 thread each).
  After:
  0 of 400 (1 of 400 with the first version of the fix).
- `tall_window_minimap_tests`.
  Cause:
  the fixture read the minimap picture of the window before it was resized.
  Fix:
  `87d02a02d8` (the fixture waits for a picture taller than 1000 rows).
  Before:
  once in a loaded full run,
  6 of 6 alone;
  after:
  no recurrence recorded.
- The refused-tile test of the first flake session.
  Cause:
  its assertion ran after a legitimate later request of the wanted tile (the late reply).
  Fix:
  the direct test in `tile.rs`,
  an integration test that waits for the end state,
  and the reader wait.
  Before:
  10 of 360;
  after:
  0 of 360.
- The wrap clip test with an extra render,
  and every `reader()` fixture.
  Cause:
  the highlighting reply arriving after the first frame.
  Fix:
  `wait_for_highlighting` (reader),
  `a_reader_is_opened_with_its_highlighting_applied` guards it.
  Before:
  28 of 160 probes had the reply missing,
  wrap clip 3 of 240;
  after:
  0 of 160 and 0 of 240.
- `minimap_tests::an_external_edit_of_fifty_thousand_lines_renders_only_the_band_it_touches` (CPU minimap).
  Cause:
  probably the late reply (unproven).
  Fix:
  the test and its subsystem were deleted.
  Before:
  13 of 120.
  The successor `an_external_edit_of_fifty_thousand_lines_shapes_only_the_line_it_touches` passed 120 of 120
  with the wait and 90 of 90 without it (no baseline failure to compare with).
- Open:
  the annotation stability tests above (one failure each of `rows_arriving_in_view...` and
  `scrolling_across_annotated_lines_is_rigid` in 480 heavy runs,
  in a burst with a full host disk;
  0 failures in about 2,100 tool runs of the two,
  829 of them in the third session at load average up to 56).
- Open and not reproduced:
  `write_wait_tests`,
  the two `watch_tests`,
  and the nine-level pixel difference.

## What was not run

Second and third sessions:

- A repeat of any full `test:native`:
  one per snapshot,
  as the rules require.
  The third session's gate was the filtered form `'all()'`,
  which runs one test at a time;
  an unfiltered run at the bound's thread count was not made on any snapshot of this session.
- A comparison of a capped and an uncapped full run for video memory;
  the 21- and 32-process subset runs and the sampler stand in for it.
- The guard runners (`inspect:*-guards`),
  because no guard changed (the third session changed tests,
  the stress task's dependencies and this document).
- The banned heavy-load conditions of the one annotation failure;
  the nearest permitted ones are the 32-process runs of the third session.
  The rigid test ran at 18 processes at most (6 copies x 3 threads),
  not 32,
  because one run of it takes minutes.
- A before count for the caret test (the backend that showed its failures no longer runs).

## Next steps

1.  When an annotation stability test fails again,
    read `target/flake-frames/` of that run and the repeatability fact of its message before changing anything.
2.  If a `vello.scene` error recurs,
    measure its video memory with a sampler like the third session's before proposing a thread cap.
3.  Re-run the stress tool for the menu tests after any change of the find bar layout:
    the click position depends on the sidebar width and the find box padding.
4.  A new failure in a subset run is worth its own hunt:
    the picture test failed 1 in 127 in a 21-thread subset run and 317 of 400 in its own hunt
    (8 copies,
    1 thread each),
    though no earlier hunt had it in its filter.
    Run each suspect alone at 8 copies with 1 to 4 threads before calling it stable.
