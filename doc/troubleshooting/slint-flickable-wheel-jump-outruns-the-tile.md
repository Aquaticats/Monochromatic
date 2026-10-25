# Slint 1.18.1 `Flickable` eases a wheel jump in 180 ms, so a far jump blanks the IDE's code area until the tile worker catches up

This was measured on 2026-10-25 with four release builds of `package/desktop-app/ide`
(Slint 1.18.1,
nested compositor `monochromatic-nested-wayland-session` 0.6.1 at 1080x1880 logical px and scale 2,
RX 7600,
a 1,500-line generated TypeScript file,
TypeScript 7.1.0-beta as the language server).
The blank frames are not a regression of the 2026-10-24 minimap,
hint skip,
or hand-over changes:
the build of 2026-10-22 shows them as often as the current `main`.
Nothing in the IDE was changed.

## Symptom

A single wheel command of 150 notches or more (`wheel 500 900 0 N` in the nested compositor)
leaves the code area empty for a short time:
the whole text area is black,
the line-number gutter included,
for one to four captured frames.
A frame or two after the black ones can show only the top or the bottom part of the code area,
with the rest black.
The first frame that shows text shows the right lines for the new offset,
laid out without the hint rows of that place,
and the text reflows about half a second later when the hints arrive.
A jump of 40 notches is almost never blank (2 jumps of 30 in each of three builds,
0 of 30 in the current one)
but often leaves a few lines at the bottom or the top of the code area empty for a frame
(72 logical px in the sample read).

The earlier verification (`doc/handover/slint-ide-0x.md`,
"Not established") saw 5 blank code frames in 14,419,
each after a wheel of 150 or 300 notches,
and 0 blank frames after the 80 other count-changing resets.
Those 80 resets (Ctrl+End,
Ctrl+Home,
minimap clicks,
file switches,
launch) are placements:
they set the offset and paint the tile of the new place before the next frame.
Only the wheel goes through the toolkit's ease and the worker.

## Root cause

A wheel step is not a placement.
The IDE's `Flickable` (`ui/app.slint`,
"Retain native Flickable physics,
including easing for notched wheels")
lets Slint animate it,
and Slint animates the whole delta in a fixed 180 ms,
however long the delta is (`i-slint-core-1.18.1/items/flickable.rs`):

```rust
// items/flickable.rs:48-51
/// Fixed-duration animation used for wheel scrolling,
/// where we don't have enough phase
/// information to derive a fling velocity.
const WHEEL_SCROLL_DURATION: Duration = Duration::from_millis(180);
```

```rust
// items/flickable.rs:545-548 (the same for content_y at :554-557)
let simulation = ConstantDecelerationParameters::new_with_distance(
    delta.y as f32,
    WHEEL_SCROLL_DURATION.as_secs_f32(),
);
```

`new_with_distance` (`animations/simulations/constant_deceleration.rs:33-58`) derives
`v0 = d / (0.5 * t)`:
the initial speed is twice the mean speed and falls to zero linearly.
The winit backend turns a wheel notch into 60 logical px (`i-slint-backend-winit-1.18.1/winitwindowadapter.rs:1472`:
`LineDelta(lx, ly) => (lx * 60., ly * 60.)`).
So one command of N notches is a sweep of 60 N px in 180 ms:

- 40 notches:
  2,400 px,
  27 px/ms at the start (445 px in a 16.7 ms frame).
- 150 notches:
  9,000 px,
  100 px/ms (1,667 px per frame).
- 300 notches:
  18,000 px,
  200 px/ms (3,333 px per frame).
- 600 notches:
  36,000 px,
  400 px/ms (6,667 px per frame).

The tile the IDE keeps is three view heights (`TILE_VIEWS`,
`src/native/viewport.rs:51`):
one above the view,
the view,
and one below.
The view here is 1,848 logical px high (the log line `the view reported its size viewport_width=646.0 height=1848.0`),
so the tile is 5,544 px and reaches at most one view height past either edge of the view when it is built.
Every ease step reports the new offset (`on_viewport_changed`,
`src/native/viewport.rs:376`).
`place_scrolled` asks for a new tile when `holds` fails (`src/native/viewport.rs:254`,
`holds` at `:97`),
and `render` queues it for the worker while the old tile stays displayed
(`src/native/render.rs:188-205`,
"the displayed tile stays on screen meanwhile").
The old tile is displayed where the row map puts it,
so once the view has left it nothing of it is in view.
The worker's result is applied only if it still holds the offset at the moment it is taken
(`accepts`,
`src/native/tile.rs:172-179`,
`holds(current, built.bounds, current.offset, height)`)
and is left for the next scroll step while steps keep coming (`STEP_WAIT`,
`src/native/tile/wait.rs:91`).
The tile is rendered into its texture in the next frame (`draw_pending`,
`src/native/gpu_tile.rs:291-297`).
From a step to the frame that shows its tile the log shows about 60 to 125 ms
(first step about 15 ms after the wheel command in the samples read,
first tile applied 78 to 142 ms after it),
in which a 150 to 600 notch sweep moves the view by several view heights.
A tile that does land was built for an earlier step's offset,
so the view is already partly or wholly beyond its edge when the frame is drawn:
that is the partial frame,
and the black frame is the frame in which no tile holds the view at all.
The last tile,
built for the end of the ease,
is applied 212 to 266 ms after the command
(180 ms of ease plus 30 to 85 ms of build,
poll,
and frame).

The README sentence "so a scroll never shows a blank frame" (`README.md`,
"Overscan tile")
holds for a scroll whose speed per build latency stays inside the tile's reach:
roughly H per 100 ms,
which is 18 px/ms,
which the first part of any ease above about 30 notches exceeds.
The README already says the same about drags:
"a tile built by the worker for a place the drag has already left would arrive out of date and leave the text blank".

The earlier reading,
that the far-wheel blank might come from the 2026-10-24 slice hand-over changes,
is disproved by the build of 2026-10-22 below.
In the earlier evidence the blank frames sat 0.25 to 0.29 s after the wheel by file modification time,
while the tile was applied 0.1 to 0.15 s after it:
the modification time of a recorded frame is its encode time,
not its capture time,
and the encoder backlog at 22 to 26 fps is of that size,
so the two agree.

## Verification

Versions under test,
all built with `mise run //package/desktop-app/ide:build` in a worktree of the commit
(release,
`target/release/monochromatic-ide`,
sha256 prefix in brackets):

- `oct22`:
  `cd49e20a92`,
  the 2026-10-22 install (`1ab8ee28`,
  byte-identical to the install of that day).
- `reaim`:
  `76f7818e8f`,
  the minimap press re-aim,
  before the hint skip rule (`01cf0477`).
- `skip`:
  `5e9584aa51`,
  the hint skip rule,
  before the slice hand-over fixes (`36c9caad`).
- `main`:
  `d91f415253`,
  current `origin/main`,
  whose IDE sources equal `f32566de81` (`d9f48256`,
  byte-identical to the installed binary).

Harness (scratch `~/temp/agent/wheel`,
TypeScript,
removed with the run):
four hosted IDEs alive at once in headless nested compositors on `/dev/dri/renderD128`
(`--size 1080x1880 --scale 2`),
one jump at a time on one of them,
alternating binaries in a shuffled order inside every (repetition,
N) pair,
the same random start place for the four binaries of a pair.
A jump is:
Ctrl+Home,
a wheel of S notches (S random in 10 to 299) to the start place,
the code mean unchanged for 1.5 s,
`record DIR 60 png`,
0.6 s,
one `wheel 500 900 0 N`,
2.4 s,
`record stop`,
the code mean unchanged for 1.5 s again.
N in 40,
150,
300,
600;
10 repetitions in each of three batches of freshly started sessions,
so 30 jumps for each binary and N,
and the three batches give the spread of the same binary.
480 jumps ran and 478 were analysed (two analysis processes died with their frames:
`b2-r6-n600-reaim` and `b3-r3-n600-skip`).
Each frame was decoded,
reduced to the mean luma of 16-row bands in six column slices and the gutter,
and deleted;
the measure is the one the earlier verification used:
`0.5 + mean luma / 2` of the code crop (x 640 to 1940 physical,
y 60 to 3660),
0.5000 blank,
drawn above 0.508.
A frame is blank below 0.5005,
partial when fewer than 60 percent of the settled frame's ink bands carry ink,
and a gap frame when its first or last inked band is 12 bands (96 physical px) away from the settled one.

Load:
the load average (1 minute) at the start of a jump had median 26.0 (4.9 to 86.2);
other sessions kept it up,
nothing was waited for.
Recorder:
60 fps requested,
492 recordings achieved a median of 19.1 fps (10th percentile 15.4,
6.2 to 22.7),
0 frames dropped,
so one frame is 45 to 65 ms.
The app itself drew 6 to 8 frames in the first 0.5 s after the wheel,
because the compositor answers frame callbacks at the capture rate:
the blank time in ms is wall-clock and holds,
but a 60 Hz display samples the ease about three times as densely.

Positive control:
in all 12 launch recordings the detector reads 0.5000 on the frames after the compositor
background (0.5490) and before the first tile (0.5166),
for example `0.5490 0.5000 0.5166 0.5166 0.5153`.
No jump recorded no change at all (0 of 478).

### Blank frames of one wheel command

Each entry is:
jumps with at least one blank code frame in batch 1 / 2 / 3 (10 jumps each),
then blank frames in batch 1 / 2 / 3,
then the partial frames over all 30 jumps.

- N=40:
  `oct22` 0/1/1 jumps,
  blank frames 0/1/1,
  partial 3.
  `reaim` 0/1/1 jumps,
  blank frames 0/1/1,
  partial 3.
  `skip` 0/1/1 jumps,
  blank frames 0/2/1,
  partial 4.
  `main` 0/0/0 jumps,
  blank frames 0/0/0,
  partial 0.
- N=150:
  `oct22` 10/10/9 jumps,
  blank frames 16/10/10,
  partial 21.
  `reaim` 10/9/10 jumps,
  blank frames 13/10/10,
  partial 22.
  `skip` 10/10/9 jumps,
  blank frames 16/14/10,
  partial 16.
  `main` 10/9/10 jumps,
  blank frames 11/9/10,
  partial 25.
- N=300:
  `oct22` 10/10/10 jumps,
  blank frames 21/20/19,
  partial 14.
  `reaim` 10/10/10 jumps,
  blank frames 18/19/21,
  partial 13.
  `skip` 10/10/10 jumps,
  blank frames 20/19/21,
  partial 11.
  `main` 10/10/10 jumps,
  blank frames 18/18/19,
  partial 18.
- N=600:
  `oct22` 10/10/10 jumps,
  blank frames 23/30/29,
  partial 12.
  `reaim` 10/9/10 jumps of 10/9/10 (the second batch lost one jump),
  blank frames 26/27/30,
  partial 13.
  `skip` 10/10/9 jumps of 10/10/9 (the third batch lost one jump),
  blank frames 25/25/28,
  partial 12.
  `main` 10/10/10 jumps,
  blank frames 27/31/36,
  partial 9.

The spread of the same binary between batches is as large as the difference between binaries:
for `main` at N=600 the blank frames are 27/31/36 (a range of 9),
and the four binaries' totals at N=600 are 78 to 94.
Below,
the four values are `oct22`/`reaim`/`skip`/`main`.

- Time from the first changed frame to the last blank or partial one (median over 30 jumps):
  157/154/158/157 ms at N=150,
  161/161/161/179 ms at N=300,
  189/200/173/201 ms at N=600.
- Blank frames over frame rate (mean blank time per jump):
  66/60/92/54 ms at N=150,
  113/108/108/102 ms at N=300,
  158/154/146/165 ms at N=600.
- At N=40 the gap frames (a few lines at the top or bottom of the code area empty) occurred in
  12/16/14/11 of 30 jumps.

### Events after the command

Median over 30 jumps,
in ms after the wheel command,
from the debug log
(`window thread applied a finished tile`;
the four builds agree within 40 ms):

- First tile applied:
  125 to 142 at N=150,
  94 to 119 at N=300,
  78 to 90 at N=600,
  156 to 176 at N=40.
- Last tile applied before 450 ms:
  212 to 226 at N=150,
  237 to 252 at N=300,
  251 to 266 at N=600.
- First synchronous build on the window thread (`window thread measured and built a tile`,
  after the hints of the
  new place arrive and the row map is rebuilt with the view kept still):
  486 to 563 ms,
  10.8 and 14.4 ms of window-thread time in the two samples read (146 and 154 lines).

### What the frames show

Of the 360 jumps with a blank frame (716 blank frames),
10 blank frames had a gutter number drawn and no text.
The first frame after the last blank one was partial/full but different from the settled frame/full and equal to it:
70/19/27 at N=150,
34/30/56 at N=300,
11/11/96 at N=600,
0/0/6 at N=40.
None of the 360 matched the picture from before the jump,
so no frame showed the old lines in place.
Images of `main` and `oct22` at N=300 (kept for the first repetition only,
then deleted):
the last frames before the black ones show lines of the old place (145 to 173,
or 268 to 297),
the next one or two are entirely black with no gutter,
a partial one in `main` shows 13 lines at the top and black below,
then a full frame shows lines 411 to 440 (`main`) or 624 to 674 (`oct22`) without the hint rows of that place,
and the settled frame of `oct22` shows the same top line (624) with the hint rows inserted.

### Control: the same distance as a spinning wheel

150 notches sent as 150 single-notch commands about 10 ms apart (5 px/ms against 100 px/ms),
10 jumps each:
`main` 0 of 10 jumps with a blank frame,
0 partial,
0 gap frames;
`oct22` 1 of 10 (one blank frame).
One command of the same distance blanks 29 of 30 jumps in each build.
So the blank follows the speed of the sweep,
not the distance,
and a wheel turned by a person does not reach it.

## Verified workarounds

None applied:
the IDE is unchanged.
The reader can avoid the blank by moving the view with a placement (Ctrl+End,
Ctrl+Home,
a minimap click,
a page key),
which paints the tile of the new place before the next frame,
or by turning the wheel a few notches at a time.

## Options considered

Ranked,
nothing built (the owner treats choppiness and blank flashes as defects,
so the first two are the ones
that remove them):

1.
   Treat a wheel delta that the displayed tile cannot hold as a placement.
   The IDE would take the wheel event before the `Flickable` for deltas above a threshold,
   set the offset itself,
   and build the tile of the new place before the next frame,
   the path drags,
   presses,
   and page steps already use (`render.rs:207`,
   "Any other change builds the requested tile
   here").
   Pro:
   one build of 11 to 14 ms in one frame instead of ten builds that arrive late,
   and no black frame,
   because there is no sweep.
   Con:
   the sweep of a 150 notch jump is no longer eased,
   and the minimap's wheel path
   ("the touch area has no scroll handler,
   so the toolkit's own wheel physics get the event") would need the same rule.
   The wheel event would have to be taken in the markup before the `Flickable`;
   whether that fits was not checked.
2.
   Build a small tile on the window thread in the frame in which the displayed tile misses the view,
   for the view and a margin only,
   and let the worker's three-view tile replace it.
   Pro:
   the toolkit's ease stays;
   the row cache and `measure_requested` exist.
   Con:
   shaping cold lines on the window thread in every step of the ease (the README keeps shaping off it while
   scrolling and `overscan_tile_tests` counts zero),
   about ten builds in 180 ms,
   and a texture render in each.
   The cost of a view-sized tile was not measured (a third of the lines of the 11 to 14 ms sample is an inference).
3.
   Prefetch from the ease curve:
   from two steps solve `v0` (with the constant 180 ms) and queue the tile of the end
   at the first step.
   Pro:
   the last tile would land about 100 to 150 ms earlier,
   so the black time falls from about 200 ms to about 100.
   Con:
   it does not cover the sweep in between,
   it depends on a constant inside Slint
   (a new Slint can change it),
   and a second wheel event adds the remaining distance of the running ease
   (`flickable.rs:496-505`).
4.
   A larger overscan.
   Pro:
   no new mechanism;
   the stack of up to four textures allows a taller tile (`README.md`,
   "`render::tile_cap`").
   Con:
   it moves the speed the tile can follow in proportion to its height,
   so two view heights each side would move
   the threshold from about 30 to about 60 notches (the same arithmetic as above,
   not measured)
   while 150 and above stay blank,
   and every re-center shapes and renders more lines.
5.
   Keep the previous tile drawn at its stale position until the new one lands.
   The IDE does this already (`tile.rs:324`);
   for a far jump the previous tile is out of view,
   so it shows nothing.
   Pinning it to the view would show lines that are not at the offset,
   with their line numbers,
   which is worse.

Ranking:
1 > 2 because the first builds once and removes the sweep,
the second builds ten times and adds window-thread
work in the frames that already miss their budget;
2 > 3 because the third leaves the black sweep and rests on a Slint constant;
3 > 4 because the fourth only moves the threshold;
4 > 5 because the fifth cannot help a jump larger than the tile.

## What does not work

- Reading the earlier blank frames as a regression of the hand-over fix:
  the build of 2026-10-22 blanks 29 of 30 jumps
  at N=150 and 30 of 30 at N=300,
  as `main` does.
- Counting only blank frames:
  at N=40 a few lines at an edge are empty in a third to a half of the jumps and the
  code mean stays above 0.5005,
  so the gap and partial counts above are needed.
- Comparing frame times by file modification time:
  it is the encode time;
  use the debug log's own time stamps.

## Upstream filing decision

`.out-of-scope/` has no entry for Slint (`cargo-workspace.md` names Slint only as a build dependency).

1.
   Is it upstream's fault?
   No:
   the fixed 180 ms ease is documented in the source as the intended wheel behaviour,
   and the IDE's tile worker is ours.
2.
   Can upstream fix it?
   Not needed.
3.
   Do they support the use case?
   A wheel jump of thousands of pixels is a rare input for a scroll view.
4.
   Would the repository welcome a contribution?
   Not checked:
   nothing to contribute.
5.
   Will they fix it?
   Not asked.
6.
   Prototype:
   none.

Nothing is filed and no draft is kept.
