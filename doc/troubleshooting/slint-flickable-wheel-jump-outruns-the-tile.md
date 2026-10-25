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
Nothing in the IDE was changed for that measurement;
the fix of the same day is the section "Fix of 2026-10-25".

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

## Fix of 2026-10-25

The owner rejected the five options below ("Surely there is a better way?"),
and the IDE was changed in two parts that no option had named.
`package/desktop-app/ide/README.md`,
"A jump faster than the tile",
describes the design as built;
this section holds what was learned from the Slint source and from the measurement.

- The toolkit reports no end for an ease.
  `content-y` is a physics-animated property (`set_physic_animation_value`),
  and nothing exposes the target.
  The `flicked` callback has no arguments.
  A `TouchArea` `scroll-event` handler sees only the first turn of a burst:
  after one turn the `Flickable` intercepts every wheel event of the next 800 ms
  (`SCROLL_FILTER_DURATION`,
  `handle_mouse_filter` in `items/flickable.rs`) before its children.
  The winit backend's window event hook (`WinitWindowAccessor::on_winit_window_event`,
  one hook for each window) sees every event,
  and the end of the ease is the sum the toolkit makes itself:
  the offset the ease started at,
  plus the notches times 60,
  plus the end of an ease still running (`remaining_distance`),
  limited to the text.
  The source view's `flicked` report says the turn was the source view's,
  so a turn over the sidebar asks for no source tile.
- The public window event `WindowEvent::PointerScrolled` carries `TouchPhase::Cancelled`
  (`i-slint-core` `api.rs`),
  which makes the `Flickable` set the offset at once and ease nothing;
  a headless test cannot reproduce the ease with it,
  so the native tests step the ease along its curve themselves and the real path is verified only by the nested compositor.
- Reading the offset property inside the `flicked` report gives the first value of the ease in the real window
  and its end in a headless test.
  The window uses the offset it last placed or was told of.
- The `flicked` report comes twice for one notched turn (inside the wheel branch and at the end of the event);
  the note of the turn is taken,
  not read.
- `rows::settle` places the view when hints change the map,
  and that placement released the follow in the first build:
  a tile for a step was requested 5 ms into the ease.
  It re-asks for the same end under the new map now (`wheel::refit`).
- The hold is a markup binding on the toolkit's live offset,
  so it needs no step to follow the ease and cannot lag it by a frame.
  The clamp edges are the raster's own,
  not the re-center edges that `holds` uses.

### Result of the fix

Harness:
the one of "Verification" (`~/temp/agent/wheel2`,
removed with the run),
two release builds alternating in a shuffled order in every (repetition,
N) pair:
`main` at `52056f6fbc` (sha256 prefix `d9f48256d152b429`,
byte-identical to the installed binary) and the fix (`2dd970ae81`).
N in 40,
150,
300,
and 600;
10 repetitions in each of three batches of freshly started sessions,
so 30 jumps for each binary and N
(239 of 240 were analysed:
one analysis of `main` at 300 notches in the third batch died with its frames).
A new disposable project held the 1,500-line generated file
and the project's TypeScript 7.1.0-beta;
the device was the RX 7600 (RADV) in every run.
Recorder:
60 fps requested,
a median of 21.2 achieved (10th percentile 19.0,
13.4 to 22.6),
0 frames dropped.
Load:
the 1-minute load average at the start of a jump had a median of 8.9 (3.9 to 26.1,
90th percentile 15.3);
other sessions kept it up and nothing was waited for.
The positive control is the launch recording of every session
(`0.5490 0.5000 0.5166 ...`),
and `main` reproduces the earlier blank counts.

Each entry is jumps with a blank code frame,
then blank frames in batch 1 / 2 / 3,
then partial frames and jumps with an edge gap over all 30 jumps.

- N=40:
  `main` 0 of 30,
  0/0/0,
  partial 0,
  edge gap in 7 jumps.
  Fix 0 of 30,
  0/0/0,
  partial 0,
  edge gap in 0.
- N=150:
  `main` 29 of 30,
  11/10/9,
  partial 20,
  edge gap in 6.
  Fix 0 of 30,
  0/0/0,
  partial 0,
  edge gap in 0.
- N=300:
  `main` 29 of 29,
  19/20/19,
  partial 16,
  edge gap in 14.
  Fix 0 of 30,
  0/0/0,
  partial 0,
  edge gap in 0.
- N=600:
  `main` 30 of 30,
  28/28/32,
  partial 17,
  edge gap in 9.
  Fix 0 of 30,
  0/0/0,
  partial 0,
  edge gap in 0.

The spread of `main` between batches is 3 to 4 frames at 600 notches;
the fix has none to spread.
Tiles applied for one jump on average:
`main` 1.03 / 2.70 / 3.48 / 3.90 at 40 / 150 / 300 / 600 notches,
fix 1.00 at every size,
discarded 0.00 (0.03 for `main` at 300).
Window-thread builds after the jump
(`window thread measured and built a tile`,
about half a second later,
when the hints of the new place arrive):
0.00 / 0.07 / 0.21 / 0.10 for `main` and 0.00 / 0.07 / 0.20 / 0.13 for the fix,
so the fix adds none.
The time from the first changed frame to the frame with the final lines is 181 to 189 ms for `main`
and 183 to 186 ms for the fix.

The app's own record of the frames it drew (`the frame's view real=... drawn=... held=...`)
gives the held frames of the fix per jump:
0.0 / 1.0 / 1.7 / 2.6 at 40 / 150 / 300 / 600 notches,
of which behind the toolkit's offset at the old tile's far edge 0.0 / 0.0 / 0.1 / 0.8
and ahead of it at the near edge of the tile of the end 0.0 / 1.0 / 1.6 / 1.7.
A held frame is a frame whose content is not the content of the toolkit's offset;
the median largest distance was 2,335,
6,549,
and 16,620 px at 150,
300,
and 600 notches.
So the first held frames (if any) show the last rows of the displayed tile,
the next show the first rows of the tile of the end
(the view jumps once,
across the whole distance),
and the ease then arrives in that tile;
a recorded sequence at 600 notches showed lines 274 to 277,
then 876 to 880 twice,
then 912 to 917 (four recorded frames,
cropped to the top of the code area).

Regression checks on the fix in the same compositor
(the 286-line `build.unit.test.ts` of `package/build-tool/css`,
a disposable project with the same TypeScript server;
`main` run in the same way for comparison):
the 88-click recording shows 0 blank code frames and 0 blank minimap frames in 730 frames of the fix (856 of `main`),
and the gap above line 96 is 82 px in all 124 frames where it could be measured (146 of `main`);
the hint requests of the three presses at y 1700,
1000,
and 1700 are 1 request and 1 skip,
then 0 and 1,
then 0 and 1,
with the same first lines in both builds
(214 and 220,
157,
220;
the first line of the skip is 223 in the project of the earlier verification,
whose stubbed workspace imports change the hint rows,
and 220 in this one for both builds).

### What remains imperfect

- A held frame is not the content of the offset the wheel asked for:
  for 1 to 3 frames of a jump of 150 notches or more the text is that of the displayed tile's edge or of the new tile's near edge,
  and the minimap's box and picture (which follow the toolkit's offset) run ahead of it.
  The box was left on the toolkit's offset on purpose:
  the picture is built for the bands around that offset,
  so a box on the drawn text would stand over a picture of other bands.
- The first frame that shows the end shows it without the hint rows of that place,
  which arrive half a second later and move the text (unchanged).
- Only a notched wheel (winit `LineDelta`) is followed.
  A touchpad (`PixelDelta`) sets the offset step by step and asks for the usual tile for each,
  and its fling after the finger lifts is not predicted;
  a wheel over the tree asks for no source tile.
- The native tests step the ease themselves,
  because the toolkit's public `PointerScrolled` event carries the gesture phase `Cancelled`
  and moves the scroll at once;
  the toolkit's own ease is exercised only by the nested-compositor measurement above.
- The toolkit's constant (180 ms) and its rule for adding the remainder of a running ease are copied
  (`wheel.rs`);
  a Slint release that changes them would make the end the window asks for wrong,
  and the window would then ask for the right tile when the ease ends (`wheel::finish` and the step at the end),
  as it did before,
  with the hold covering the gap.

## A hover request after a far wheel jump

A verifier on the installed release binary of 2026-10-25 (`e96cdd88596d32e9`,
the hold build) found that the hover request
sent 0.35 to 0.40 s after a 300-notch wheel jump named a line other than the one drawn under the pointer.

### Cause

`Rest` (`src/native/language/pointer.rs`) kept the character that was under the pointer at its last move,
resolved by `character_at` in `on_pointer_moved` (`src/native/language/actions.rs`),
and the dwell timer handed that same character back 350 ms later (`due`,
sent from `rest`).
The toolkit's `TouchArea` sends no pointer event for text that moves under a pointer that does not,
so after a wheel turn,
a jump,
or rows arriving,
the stored character was one of the view before the move of the text.
This is the same as the old behaviour of the code,
not an effect of the hold:
the section "Fix of 2026-10-25" mapped the pointer through `drawn-y` for the events that arrive,
and none arrives here.

The constant position of the verifier's resting trials is the character under the wheel command's own point at the instant
of the command:
the compositor's `wheel X Y` moves the pointer to the point first (one pointer move,
which the log shows twice with identical numbers),
and a pointer that moved less than 350 ms before the turn,
or onto another point with the command,
has its dwell fire after the turn for that character.
A pointer that had already rested 350 ms and been asked about sends nothing after the turn
(`asked` stays set until the pointer moves to another character).

### Is it new

Measured on 2026-10-25,
11 trials in every cell (the turn is 300 to 310 notches,
one of each remainder of the 11-line cycle
of the generated file),
the four sessions alternating inside every repetition,
whatever load the host had.
`main` is `83af48b117` (`e96cdd88596d32e9`),
`before the hold` is `52056f6fbc` (`d9f48256d152b429`),
and the fix is the build of this change.
A request is wrong when the line it names differs from the line whose gutter number (read from the screenshot taken within
a few ms of the log line) is nearest to the pointer.
The control for the spread is the second copy of the fixed build.
A first batch of 10 trials at 300 notches,
with a second copy of `main` as the control,
gave the same numbers for `main`,
the second copy,
and before the hold.

- The pointer moved onto a word 100 ms before the wheel turn,
  which turned at the same point:
  `main` 11 of 11 requests wrong (line 4 against 636 to 652 drawn),
  before the hold 11 of 11 wrong.
  Old,
  not new.
- The pointer rested 900 ms on a word (its popup was shown),
  then the wheel turned at another word
  (the verifier's variant):
  `main` 11 of 11 wrong (line 9 against 641 to 658 drawn),
  before the hold 11 of 11 wrong.
  Old,
  not new.
- The pointer moved onto text 90 ms after the wheel command,
  inside the hold (the verifier's second variant),
  at a column that varies with the trial:
  `main` sent 3 requests,
  all wrong (lines 608,
  610,
  and 616 against 664,
  665,
  and 671 drawn,
  55 to 56 lines off),
  and none in the other 8 trials (the pointer was over no character);
  before the hold sent none in 11 trials,
  most likely because the view then showed no rows of the displayed tile under the pointer and `character_at` found none
  (inferred from the code,
  not logged:
  the build before the hold has no probe).
  New:
  the hold makes the rows drawn under the pointer reachable by `character_at`
  (`drawn-y` maps the move into the held tile),
  and the stored character then names a row of the held frame,
  not of the settled view.
  The cause is the same stored character.
- After the view settled,
  the pointer moved onto a word (the positive control):
  `main` 11 of 11 right,
  before the hold 11 of 11 right.

### Fix

`Rest` keeps the pointer's place as a distance from the top of the view (`Spot`),
`Rest::follow` starts a rest that was not asked for yet again when the drawn offset moves by more than half a pixel
between two ticks,
and `rest` in `src/native/language/rest_step.rs` finds the character from that distance and the offset drawn at the time
the request is built
(a pointer over no character there sends nothing).
The move itself still tests the character it is over,
so moving within one character keeps the timer.
editord resolves the hover position the same way (`getPositionFromPoint` of the last mouse event when the 350 ms debounce
fires,
`package-paused/desktop-daemon/editord/src/client/app/lsp-hover.ts`),
and,
like editord,
asks nothing after a scroll for a rest that was already asked for.
One difference from editord is deliberate:
its debounce restarts only on a mouse move,
and a rest here also starts again while the drawn offset moves,
so a request after a wheel jump lands 0.54 to 0.83 s after the wheel in the fixed build
(5 requests in each of the first two cells above),
against 0.35 to 0.40 s before,
because it waits for the view to stand still for the rest time.

Native tests (`src/native/language/pointer_scroll_tests.rs`) and the guard cases `hover-*` of
`bin/inspect-source-guards.ts`:
each guard removes its change in a disposable copy and the named test fails,
then the restored copy passes.

Measured with the same trials on the fixed build,
11 per cell,
alternating with `main` and before the hold and with a second copy of the fixed build for the spread:

- Moved onto a word 100 ms before the turn:
  5 requests,
  all for the line drawn under the pointer (gap 0 lines),
  and 6 trials without a request;
  the log of each of those six has `resolved=None`
  (the pointer was over a hint row or a line with no character at its column).
  The second copy gave the same 5 and 6.
- Rested 900 ms,
  then the wheel at another word:
  5 requests,
  all for the line drawn (gap 0),
  6 without a request;
  the second copy 6 and 5 (one trial differed in whether the pointer lay over a character,
  and its request was right).
- Moved 90 ms after the wheel command:
  2 requests,
  both for the line drawn (gap 0),
  9 without
  (in 1 of them the rest fell due and `character_at` found none,
  in the other 8 the pointer was over no character where it moved,
  so no rest was started);
  the second copy the same.
- Positive control,
  the pointer moved onto a word after the view settled:
  11 of 11 right in both copies,
  as in `main` and before the hold.
- No request named a line other than the one drawn under the pointer in any of the 44 trials of the fixed build
  (the stale constants,
  line 4,
  line 9,
  and lines 608 to 616,
  did not come back).

### A pointer that rests over no character

A verifier on the installed build (`dd16d49c0d586fb1`,
code of `495bf2f39e`)
saw 9 hover trials with no request and no log line
after a 300-notch jump,
and 2 settled hovers at one row that sent nothing.
The code explains them (read on 2026-10-25),
and the measurements below reproduce them.

#### Cause of the silence

`Rest::due` in `src/native/language/pointer.rs` admitted only a rest whose character at the pointer's move was `Some`.
A move onto a hint row,
the line numbers,
a blank line,
or the space past a line's end stores `None` (`Rest::moved`),
so `due` never returned a place for it:
`rest` in `src/native/language/rest_step.rs` returned before the `resolved=` log line,
and `Rest::idle` only closed a popup.
`Rest::follow` already started such a rest again when the view moved,
so the moved view changed nothing:
a pointer that rested over no character never asked,
even when an identifier arrived under it by a wheel turn,
a jump,
or hints and messages arriving.
Only a pointer move started the rest again.
It is the same code as before the earlier fix,
so it is old,
not a regression of the hold.

#### A second defect of the first version of the fix

The first version of this fix made `due` return the place of a rest over no character whose view had moved,
and the requests were right in every trial.
Screenshots of the popups showed none:
the log has `language popup shown` followed 20 ms later by
`language surface dismissed reason="the pointer rests over no character"` in all of 12 far-jump trials.
The rest still recorded `None` as its character after the request was made,
so `Rest::idle` (a rest over no character for the rest time) closed the popup of the answer as soon as it was shown.
The native tests had waited only for the popup to appear,
and the counts of the trials only counted requests.
`Rest::asked_about` now records the character the request was made for,
and the tests wait the rest time twice more and compare the popup text.

#### Fix

- `Rest::due` returns the place of a rest over no character only after the view moved under it
  (`shifted`,
  set by `follow` when it starts a rest again,
  cleared by a pointer move to another character and by `found_nothing`),
  so a pointer over no character in a view that stands still asks nothing and logs nothing.
- `Rest::follow` compares the drawn offset and the height of the whole text (`View`),
  because hints or messages arriving between the top of the view and the pointer put other text under it
  without changing the offset.
- `rest` resolves the character against the rows and offset that are drawn when the request is built,
  as before.
  A character:
  `Rest::asked_about` marks the rest asked and over that character.
  None:
  nothing is asked and nothing is marked asked,
  `Rest::found_nothing` makes the rest one over no character in a settled view,
  and the next move of the view starts it again.
- `Rest::idle` is unchanged,
  so a popup that is open over a character still closes when the pointer rests over no character in a view that stands still.
- The 350 ms rest of a settled pointer and the 0.53 to 0.57 s after a jump are unchanged.

#### Native tests and guards

Tests are in `src/native/language/pointer_blank_tests.rs`.
A rest over no character:
asks after a near scroll,
after a far jump (held at the tile's edge when the rest starts),
and after hints arrive under it,
and the popup stays open for twice the rest time;
asks nothing in a view that stands still (the record of the rest is the same after three rest times);
asks after the next scroll when the first scroll found no character;
and a popup that is open over a character still closes when a scroll leaves no character under the pointer.
Unit tests cover `due`,
`idle`,
`found_nothing`,
`asked_about`,
the offset,
and the height.
The guard cases `hover-*` of `bin/inspect-source-guards.ts` remove each changed line in a disposable copy;
on the final tree all 24 `hover-*` guards failed the named test with the line removed
and passed again with it restored
(`mise run //package/desktop-app/ide:inspect:source-guards <cache> '' <names>`,
24 of 255 guards).
Two lines have a unit test only:
the height of the text in `follow` (`a_growing_text_restarts_a_rest_that_was_not_asked_for`),
and the height read from the window in `rest` and `on_pointer_moved` has a native test that shows only the constant side
(`a_view_that_stands_still_keeps_the_moment_the_pointer_arrived`).

#### Measurements

Binaries:
main is the installed `dd16d49c0d586fb1` (built from `495bf2f39e`;
`git diff 495bf2f39e cff94d8247 -- package/desktop-app/ide/src package/desktop-app/ide/ui` is empty,
so it is the code of `origin/main`),
the fix is a release build of the branch (`fix`),
and a second copy of the fix binary (`fix2`) gives the spread.
The three ran as three hosted sessions at the same time,
10 trials in every cell,
one headless nested compositor each on the hardware GPU,
the host's load as it was (1-minute load average 3.8 to 4.3 during the runs).
The window is 1100 by 800.
The text is a generated 1,500-line `long.ts` of exports with literal values
(no hint and no message,
so every line is one 24 px row,
except a hint row above line 2),
in a period of five lines:
three with an identifier at the pointer's column,
a `//` line,
and a blank line.
A notch moves the text 60 px (measured).
The pointer is at x = 480 and rests 1.2 s before the wheel.
A trial is right when the line of the request is the line of the settled probe
(the pointer leaves the source and comes back to the same point after the view settled,
and the request of that settled hover names the line),
and the geometry above predicts the same line in every trial
(`line != model` and `probe != model` are 0 everywhere).
Notch counts are odd so that the pointer is over text at the end
(an even count lands on the same kind of line it started on):
301 for the far jump.
Every cell had an identifier under the pointer at the end in all 10 trials of every arm.

- Past the end of a `//` line,
  5,
  11,
  29 and 301 notches:
  main 0 of 10 requests in each,
  fix 10 of 10 right in each (0.534 to 0.560 s after the wheel;
  0.543 to 0.559 s at 301),
  fix2 10 of 10 in each.
- On the hint row above line 2,
  5 and 301 notches:
  main 0 of 10 in each,
  fix 10 of 10 right (0.533 to 0.573 s),
  fix2 10 of 10.
- Hints arriving under the pointer (`build.unit.test.ts`,
  the pointer past the end of line 97,
  45 notches,
  a fresh session for each trial because answered hints stay):
  main 0 of 10,
  fix 10 of 10 right,
  fix2 10 of 10,
  0.783 to 0.818 s after the wheel,
  the line named is the line 96 of the screen (the parameter `dir`) in all 20.
- The pointer moved onto an identifier 100 ms before a 300-notch wheel:
  main 10 of 10 right,
  fix 10 of 10 right,
  fix2 10 of 10 (0.545 to 0.573 s in all three).
- The pointer rested 1.2 s on an identifier (asked,
  popup shown)
  and then the wheel turned at the same point:
  0 requests after the wheel in all three arms
  (a rest that was asked stays asked),
  as in editord.
- A settled hover onto an identifier:
  10 of 10 in all three arms,
  0.350 to 0.370 s after the move.
- No request before the wheel and no `resolved=` line before it in any trial of a rest over no character;
  exactly one `resolved=` line after the wheel in every trial of the fix and none in main,
  and no trial of any arm had a second request.

Before the wheel the rest over no character logs nothing,
so the settled pointer has no log noise.

The hold of a first version of this change was also measured with screenshots at 150 ms after `language popup shown`:
see the lines above about the second defect.
After the fix,
12 far-jump requests (4 each past the end of a line,
on the hint row,
and moved onto an identifier;
301 and 300 notches) and 6 settled hovers were looked at in screenshots of the frame after the popup was shown:
the popup heading (`const value0758: 758`,
`const value0753: 753`,
`const value0752: 752`,
`const value0002: 2`) is the identifier of the row under the pointer in every one,
the popup stands below that row at the pointer's column (3 px left of it),
on screen,
and none is for another identifier or at a stale place.
The popup of the hints trial,
`(parameter) dir: any`,
stands below the wrapped row of line 96 and its right edge overlaps the left 10 px of the minimap.

#### Not covered

- Hints that arrive after the rest was resolved to no character start it again through the height of the text
  (`a_growing_text_restarts_a_rest_that_was_not_asked_for`);
  in the trials of the real server the hints arrived before the rest was due (0.44 s after the wheel),
  so the height alone was not shown necessary there.
- The two items that this section left open at the time
  (a fresh session with no character under the pointer after a first far scroll,
  and rows that are drawn late under a rest over no character)
  are answered in the two subsections that follow.

#### The first far scroll of a fresh session with no character under the pointer

The open item:
one trial of a fresh session under the load of a release build that had just finished
(a 301-notch wheel as the first scroll) had `resolved=None`,
its settled probe found no character either,
and the log showed `leaving a finished tile for the next scroll step`.
It was feared that the view stayed at a stale place or that the tile of the end of the ease was never applied.
Answer of 2026-10-25 (branch `worktree-agent-ac50f93d8163f1cf4`):
no stuck view was found in 1,000 trials of the build of `origin/main`
(the installed `0730fa64bdf211e7`,
identical bytes to a release build of the commit `654f87b6b2`),
and the symptom was reproduced without any stuck view.

What the log line means:
`poll_unless_step_due` in `src/native/tile/wait.rs` writes `leaving a finished tile for the next scroll step`
whenever the repeated poll of the worker runs within 20 ms of a scroll step,
before it asks the worker whether anything has finished.
It was in 27 to 30 of 30 trials of every normal cell,
so it says nothing about a stranded tile.

The harness (scripts in the session scratch,
removed afterwards) started a new IDE process for every trial
in a headless nested compositor 0.6.1 on the hardware GPU
(`--gpu /dev/dri/renderD128`,
the window drawn on `AMD Radeon RX 7600 (RADV NAVI33)` as the log of every session says),
1080 by 1880 logical px at scale 2.
The project was a disposable copy with the generated 1,500-line `long.ts` in the five-line period
(three `export const alphaK/betaK/gammaK` lines,
`// note K`,
a blank line)
and the project's own TypeScript 7.1.0-beta server (`tsc --lsp --stdio`).
Three sessions ran at the same time,
and the host's load was as it was
(1-minute load average 3 to 36,
recorded for every trial;
other sessions' jobs and the checkout of this session's worktree raised it).
The first wheel was the first input of the session:
`wheel 462 Y 0 N`,
with a Y that puts a `beta` line under the pointer at the end of the ease.
A trial records:

- The gutter numbers of a screenshot 3 s after the wheel (tesseract on the cropped gutter),
  and the first line the model of the notch count predicts (`ceil(60 N / 24) + 1`,
  clamped to the end of the text).
  A view with fewer than 20 numbers read counts as blank.
- The log lines of the wheel:
  `turned the wheel`,
  `began an ease`,
  `held=true` frames,
  `applied a finished tile`,
  `discarded stale source tile`,
  `a followed wheel ease ended without a step`,
  and `resolved=`.
- A settled probe:
  the pointer leaves for the sidebar and comes back to the same point,
  and the request of that hover names a character or none.

The wheel was sent 0.5,
1,
2,
or 4 s after the first drawn frame
(the first frame is drawn 0.5 to 0.7 s after the start,
and up to 3.6 s on the loaded host),
with the server ready (`ready`)
or not ready for the first 3 s (`late`:
a wrapper of `bin/tsc` that waits 3 s before it starts the server).
The cells and their counts:

- 301 notches,
  8 cells (4 delays by 2 server states) of 30 trials:
  240 trials,
  0 wrong views,
  0 blank views,
  one tile applied for every jump (1.00 on average),
  0 tiles discarded.
- 5,
  40,
  150,
  300,
  and 600 notches,
  40 cells of 10 trials:
  400 trials,
  0 wrong views,
  0 blank views.
  One tile of the end is applied per jump at 40 notches and more,
  and none is asked for at 5 notches.
- 301,
  150,
  600,
  and 40 notches at 0.5 and 2 s,
  both server states,
  10 trials per cell,
  the build of `origin/main` (`head`),
  the fix of item B (`fix`),
  and a second copy of the fix for the spread (`fix2`) at the same time:
  160 trials per binary,
  0 wrong views in all 480.
  The request of the first rest after the wheel named a character in every one
  (`resolved=Some`,
  530 to 620 ms after the wheel,
  median 556 ms in every binary),
  and the settled probe found a character in 157,
  155,
  and 158 trials.
  The rest (3,
  5,
  and 2 trials) had no `resolved=` line within 1.2 s of the probe's move,
  in pairs at the same moment in several binaries,
  with the 1-minute load average 17 to 30 at that moment:
  a stall of the host,
  and the spread of one binary is as large as the difference between binaries.
- The same cells as the last item with the hosted app limited to one core
  (`--isolate --app-cpu-quota 100` of the compositor,
  which puts the app in a transient systemd scope;
  limiting the app is not a synthetic load of the host),
  160 trials of the build of `origin/main`:
  0 stuck views,
  0 blank views,
  tiles applied 1.00 per jump,
  and 5 trials whose view was 3 lines off the model.
  Those 5 are the symptom of the open item:
  the first rest resolved `None`,
  the settled probe found no character,
  and the gutter was exact for the row map the window held.

The explanation of the symptom:
in those 5 trials the row map ended 72 px taller than the text
(`height=36096.0` in the last `row map rebuilt` line against `36024.0` for 1,501 lines of one row),
so every line stood 3 rows lower in the map than in the model,
and the pointer that the model put on a `beta` line was over a `// note` line or a blank line.
(4 more trials of that run had the taller map and a view on the model,
because the 3 extra rows lay below the rows the view showed.)
The picture and the gutter are consistent with each other
(a screenshot of such a view shows lines 373 to 449 in order with the gutter numbers right),
the tile was applied once,
and the log has no discarded tile.
The earlier fixture's geometry had to assume rows above the view in the same way (a hint row above line 2),
and the one trial under the load of a finished build was most likely a session whose map was taller than the geometry assumed.
That is not proved for that trial,
whose log is gone;
the same symptom with the same log lines is reproduced here.

What was open and not explained at that time is explained in the section "A row map taller than the text"
(the taller map and,
in its subsection "Missing wheel log lines",
the missing `turned the wheel` lines).
The counts of the first account stand:
the taller row map was 9 of 160 trials under the one-core limit,
4 of 53 sessions started with no input at all under the limit,
and 1 of 1,160 trials without a limit;
three trials of the cell of 40 notches at 2 s with the server ready had no `turned the wheel` line at a 1-minute load average of 28 to 35.

Positive controls (301 notches,
1 s,
10 trials each,
the same harness):

- A build in which `tile::accepts` takes no tile
  (the view stays held at the first tile's edge):
  10 of 10 wrong views,
  10 of 10 probes without a character,
  2,570 tiles discarded.
- A build in which the end of the ease is predicted 6,000 px past the real end
  and the 400 ms backstop (`arm_release`) does nothing:
  10 of 10 wrong views
  (the tile of the predicted end is applied,
  the view stops elsewhere,
  and nothing asks for the tile of the real end).
- The same prediction with the backstop intact:
  0 of 10 wrong,
  the backstop (`a followed wheel ease ended without a step`) fired in 10 of 10
  and 2.00 tiles were applied per jump.

So the harness sees a stuck view,
and the backstop is what keeps a mispredicted end from sticking.

#### Late rows under a rest over no character

The open item:
rows that are drawn late without a move of the drawn offset or a change of the height of the text
do not start a rest again (`Rest::follow` watched those two numbers).
Decision from the code and a measurement:
one real case exists and is fixed;
the others that were named are covered or cannot happen.

The real case is an external change of the displayed file that keeps the number of lines
(a formatter,
a checkout,
an editor elsewhere).
It changes the characters under a pointer that does not move,
and the offset and the height of the text stay the same,
because every line is one row.
A pointer that rested over a `// note` line past its end,
or over a blank line,
had a rest over no character that asked nothing and logged nothing,
so an identifier that stood under it after the reload was never asked about
until the pointer moved by a pixel.
Measured on release builds,
10 trials per case,
a fresh session each,
the pointer resting 1.2 s over no character,
then `long.ts` rewritten on disk
(line 4 `// note 1` to `export const delta1 = 1;`,
or the blank line 5 to `export const eps5 = 5;`),
no pointer move,
2 s of waiting for requests:

- Past the end of line 4:
  `origin/main` 0 of 10 requests,
  the fix 10 of 10 with exactly one request
  (character 89,
  inside the new line 4,
  popup shown 10 of 10),
  a second copy of the fix 10 of 10.
- Blank line 5:
  `origin/main` 0 of 10,
  the fix 10 of 10 with exactly one request (character 99 inside the new line 5,
  popup shown 10 of 10),
  second copy 10 of 10.
- Positive control (`origin/main`):
  a pointer move of 1 px after the reload sends the request in 10 of 10.
- A rest that was asked about (the pointer on `alpha1`,
  popup shown),
  then the file reloaded:
  0 new requests in all three binaries
  (a rest that was asked stays asked,
  as after a scroll).
- Screenshots of the frame after `language popup shown`
  (`const delta1: 1` below line 4 and `const eps5: 5` below line 5,
  at the pointer's column,
  on screen)
  were looked at for both cases.

The fix:
`View` and `Spot` in `src/native/language/pointer.rs` carry the stamp of the displayed text
(`DocumentStamp`:
file generation and content revision,
`surface::displayed`),
`Rest::follow` starts a rest again that was not asked for when the stamp changed,
and `rest` in `src/native/language/rest_step.rs` and `on_pointer_moved` in `src/native/language/actions.rs` read it the same way.
The stamp is the cheapest signal that changes when the characters change:
one pair of integers compared per tick,
no recomputation,
and it also covers a file switch.

Tests are in `src/native/language/pointer_blank_tests.rs`:
`a_replaced_text_restarts_a_rest_that_was_not_asked_for` (unit),
`a_rest_over_no_character_asks_when_a_reload_puts_a_character_under_it` (native,
the popup stays open for twice the rest time),
and `a_rest_that_was_asked_about_is_not_asked_again_by_a_reload` (native).
With `|| text_changed` removed from `follow` the first two fail and the third passes
(it guards the unchanged behavior).
The guard cases `hover-text-change-ignored`,
`hover-text-change-ignored-unit`,
`hover-spot-text`,
and `hover-tick-text` of `bin/inspect-source-guards.ts` remove each changed line;
the two older guards on the same condition (`hover-view-moving-restarts` and its unit twin) follow the changed line.

The other cases that were named:

- A tile that lands after a hold:
  the view is drawn at the edge of the displayed tile's raster until the tile of the end lands,
  and the character is found in the drawn rows and the drawn offset when the request is built.
  The drawn offset moves when the hold ends,
  which starts the rest again;
  a tile that lands without moving the drawn offset holds the same lines at the same places,
  so the same characters stand under the pointer.
- A hint answer that replaces a hint row by another text of the same height:
  a hint row is a virtual row (`in_block`) with no source character,
  so `character_at` answers none before and after.
  A hint that appears or goes changes the height of the text,
  which `Rest::follow` already watches.
- A diagnostic message that appears:
  it adds virtual rows below its line,
  so the height of the text changes.
- A resize or a change of scale:
  a wrapped line changes the height,
  and a line that does not wrap puts the same characters at the same places.
- A blank placeholder before a first tile:
  no case with a pointer in the window was found in the fresh-session trials above
  (the first frame already shows the first tile of the file).

Measured again on the fix against `origin/main`:
the 160-trial grid of the last section shows the same counts for the far jumps in each binary
(0 wrong views,
one tile per jump),
and the first rest after the wheel is unchanged
(`resolved=Some` in 160 of 160 for each of the three binaries).

## A row map taller than the text

Measured on 2026-10-25 on the build of `origin/main` (installed binary `0730fa64bdf211e7`) and on a build with the fix,
in the nested compositor (`--headless`,
1080x1880 logical px,
scale 2,
the hardware GPU through Vulkan RADV),
three sessions at a time,
the 1-minute load average 20 to 31,
the hosted app limited to one core (`--isolate --app-cpu-quota 100`),
a generated `long.ts` of 1,500 lines
(three `export const` lines,
a `// note K` line and a blank line in each period of five;
the text has 1,501 lines,
so the model height is 36,024 px),
and the project's TypeScript 7.1.0-beta server.
The harness (a driver that starts a fresh IDE process for each trial,
waits 5 s without any input,
and reads the last `row map rebuilt` record of the log) was kept only in the session scratch and removed.

### Reproduction

- Sessions with no input,
  one-core limit,
  the `height=` of the last `row map rebuilt` record against 36,024:
  `origin/main` 12 of 160 too tall by exactly 72 px,
  a second copy of the same binary 7 of 160
  (the spread of one binary),
  the fix 0 of 160
  (Fisher exact test,
  one-sided,
  the fix against the 320 sessions of both copies of `origin/main`:
  p = 0.0004).
  Two earlier groups of the first build:
  5 of 60 and 3 of 30.
- Sessions with a taller map by another amount (36,114 to 36,178) held a block of virtual rows:
  an information message of the language support
  (`bwrap: Can't bind mount ... on /tmp`:
  the source directory `.../language/proj-<hash>/mise/tmp` of the bind mount did not exist).
  They occur in every binary (2,
  1 and 4 of 160) and are not a row defect.
  Three sessions at a time share the harness's cache directory and the project hash,
  which points to a race between sessions on one project
  (a separate incident,
  not investigated);
  two IDE processes on one project could meet it too.

### The lines and the cause

A build with a probe (the `row map rebuilt` record listing the first lines that hold more than one row,
and a record of the lines of the displayed tile that the pass measured differently from the table)
showed in both sessions that were read with it the same three lines,
150,
151 and 152 (numbering from 0),
each with two rows in the map and one row at the real width:

1. The window starts with a narrow layout (a view of 173 px,
   18 lines in the tile) and gets its real size (646 px by 1,848 px) some 0.1 to 1 s later.
   Under a one-core limit the worker's first pass over the whole text,
   at the narrow width,
   can be taken by the 20 ms poll before the real size arrives
   (`applied a measurement count=256 changed=true`),
   so the table of measured rows holds two rows for every line that wraps at 171 px.
   With CPU to spare the real size comes first and the narrow output is dropped as `another text or width`.
2. The real width asks for a tile of 99 lines,
   taken from that table.
   `measure_requested` (`src/native/render/measure.rs`) measured the lines of the tile asked for at the real width in three rounds,
   and after each round the map followed the corrected heights,
   so the tile asked for reached further down the text:
   the stale region of the table retreated from line 100 to 136 to 150 while the tile grew to 155 lines.
   After the third round the lines the last tile asked for (here 150 to 154) were measured only for their cost:
   the answer of `spending.measure(...)` was discarded,
   so lines 150 to 152 kept two rows
   (lines 153 and 154,
   the note and the blank line,
   never wrap).
3. The window thread builds the tile of 155 lines,
   which is painted with one row for each line,
   and the map disagrees with it below line 150.
   `apply_pass` (`src/native/tile/pass.rs`) never applies the lines of the displayed tile,
   so the pass that finishes some milliseconds later does not correct them either
   (the probe:
   `held_differs=[(150, 2, 1), (151, 2, 1), (152, 2, 1)]` on the outputs of the pass,
   then `done=true`).
   The map ends 3 rows taller than the text.

The earlier account,
that measurement passes at the narrow width "end late",
was wrong:
the pass at the narrow width is finished or dropped,
and what stays are heights it left in the table that the window thread's rounds did not reach.

### When it corrects itself

Not in a session with no input.
Observed on a build of `origin/main` made to leave stale rows (the positive control below,
600 lines with two rows,
a map 240 px too tall),
12 sessions:

- A window resize (`resize 1000 1880`,
  then back) made the map exact in 6 of 6,
  because the new width measures the lines near the view again.
- Ctrl+End left it 240 px too tall in 6 of 6,
  because the tile of the end does not cover the stale lines.
  Ctrl+Home then made it exact in 6 of 6,
  because the tile of the first lines covers them.
- A tile built on the worker that covers the lines corrects them too (`tile::apply` takes the heights the tile measured),
  so stepping down by rows repairs the map when the view arrives
  (the earlier agent's step-by-step session was exact for 1,351 lines).

Until then the scroll extent is 72 px taller than the text,
every line after line 152 stands 3 rows lower in the map than in the text,
so a distance in pixels (a wheel jump,
a minimap click,
a restored scroll offset) lands 3 rows away from where the text model puts it,
and a pointer placed from the model is over another row.
The picture and the gutter agree with the map:
after Ctrl+End a session with the taller map shows the last row at the bottom with the same gutter numbers
(1,425 at the top and 1,501 at the bottom) as a session with the exact map,
so nothing is drawn blank and a screenshot of one view does not show it.

### Fix

`measure_requested` repeats its rounds until one changes no height
(at most 64,
`ROUNDS`,
with a warn record if the bound is reached),
and the measurement of the last tile asked for is taken into the table too.
The invariant is that every line of the tile built next has in the table the rows the shaper measured for it.
`apply_pass` writes a debug record
(`the measurement found rows in the displayed tile that differ from the map`)
when a line of the displayed tile differs from what the pass measured,
so this class of fault is visible in the log.

- Native test `a_tile_taken_at_once_for_a_new_width_holds_only_measured_lines` (`src/native/wrap_measure_tests.rs`):
  the pass is held still,
  a pass chunk of the narrow width is simulated by 2 rows for the lines 0 to 599,
  a new width is taken by the window thread,
  and every line of the displayed tile must have the rows a fresh shaper measures for it.
  Before the fix:
  4 lines of the tile kept a stale height (`[(50, 2, 1), (51, 2, 1), (52, 2, 1), (53, 2, 1)]`).
  After:
  it passes.
- Guard case `measure-rounds-capped` of `bin/inspect-source-guards.ts` (3 rounds and no absorb of the last measurement).

### Measurements of the fix

- 160 sessions with no input under the one-core limit for each of three binaries,
  alternating (`origin/main`,
  the fix,
  a second copy of `origin/main`):
  too tall by 72 px in 12,
  0 and 7 of 160.
  The record of a pass finding rows of the displayed tile that differ from the map:
  0 of 160 in the fix.
- Positive control for the harness and the fix:
  a throwaway build (both logics) that applies 2 rows to the lines 0 to 599 when the pass at the narrow width is posted,
  the stand-in for a pass chunk applied before the real size.
  The old logic gave 10 of 30 sessions with a map 240 px too tall (and 2 with the message block),
  the fixed logic 0 of 30.
  A first stalled build (one round,
  no absorb of the last measurement) gave 1 of 60,
  no more than the spread of the old logic,
  so it was no control.
- 90 sessions with Ctrl+End after the wait,
  alternating the old build and the fix:
  3 sessions with a map 72 px too tall in the old build and none in the fix,
  and the same last rows on the screen in all.

### Missing wheel log lines

The wheel path writes two records per turn,
both at debug level:

- `the windowing system turned the wheel` (`src/native/wheel.rs`,
  `note_event`),
  once for each winit wheel event with a line delta,
  before the toolkit sees it and whatever the displayed tile holds.
- `a wheel turn began an ease` (`turn`),
  once for each turn the toolkit's scroll reports within 50 ms of the event,
  with `covered=true` when the displayed tile holds the end.

A turn the displayed tile holds therefore logs both records,
and nothing in the code skips them.
Two branches dropped a turn without a record
(`note_turn` when the window state was in use,
`flicked` when the note was older than 50 ms);
they write a debug record now.
`src/logging.rs` does not write straight to the file:
`src/logging/background.rs` queues records for a writer thread
and drops records,
with a warning that carries the count,
only when 8 MiB wait in the queue;
no such warning stood in any log read here.

Measured:
40 and 301 notches turned 300 ms after the server was ready,
2 s of waiting,
one-core limit,
210 sessions.
The 197 sessions with a `turned the wheel` line all show a scrolled gutter,
and the 13 sessions without one all show the top of the file
(the gutter pixels equal those of a session that never scrolled).
What separates the sessions is the gutter and nothing else that was measured:
the time of the toolkit's report of the real view size (`viewport_width=646`) after the server was ready overlaps
(0.12 to 1.0 s in the 9 silent sessions of the cell of 40 notches,
negative to 0.19 s in the 4 of the cell of 301,
negative to 0.3 s in the sessions that took the wheel),
so it is not the discriminator.
The likely mechanism is a hypothesis the IDE log cannot show:
the first surface is small (a view of 173 px by 288 px),
the pointer of the wheel command stands at 540 px by 900 px,
and the layout report is not the commit of the fullscreen surface,
so the pointer was over no surface of the IDE and the event reached nobody.
The IDE never received an event,
so the log is right to be silent,
and it has no record to write for an event it never receives.

### Not explained

- Where such a wheel is dropped (the compositor or the toolkit's first surface),
  and why the report of the real size does not tell.
- Whether the pass at the narrow width is taken before the real size only under a one-core limit:
  the same symptom was seen once in 1,160 trials without a limit,
  whose logs are gone.

## Workarounds before the fix

The reader could avoid the blank by moving the view with a placement (Ctrl+End,
Ctrl+Home,
a minimap click,
a page key),
which paints the tile of the new place before the next frame,
or by turning the wheel a few notches at a time.

## Options considered

The owner rejected all five on 2026-10-25;
the section above is what was built instead.
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
