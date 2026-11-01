# nested-wayland-session 0.4.0: recorded frames of a resize drag show the previous step's content

## Status

The compositor now sends the hosted clients' frame callbacks right after the new configure and scale of a `resize` or
`scale` that is made while recording (the section "Improvement: callbacks right after the configure" has the change and
its measured effect).
That roughly halves the stale captures of the 16 ms drag (from about 90 to about 50 percent); the remaining ones were
not analyzed further, and the application's own work against a 16.7 ms tick is a part of them.
The sections up to "What the application contributes" describe the behavior of 0.4.0 and the cause.

## Symptom

On 2026-10-13, the 60 fps recorder of the nested compositor ran over a continuous resize drag of the read-only Slint IDE
(`resize W 660` every 16 ms, 1100 to 600 to 1100 pixels, two sweeps).
Of about 100 recorded frames, 74 to 77 did not fit:
when shrinking, text wrapped for the previous, wider step ran into the right edge and was cut;
when growing, a strip of another shade stood at the right edge beside text wrapped for the previous, narrower step.
In the headless test backend the first frame after a size event is already wrapped for the new width
(`a_frame_drawn_right_after_a_resize_event_is_wrapped_to_the_new_width`), so the question was where the one-step delay
of the real compositor comes from.

## Root cause

The delay is in the compositor's capture model and the toolkit's redraw gate, not in the application.
Four facts decide it (paths under `package/nested-wayland-session/cli/`; the line numbers are those of the tag
`monochromatic-nested-wayland-session-v0.4.0`, and a note after the list gives the current ones):

- `src/screen.rs:209`: the `resize` command calls `set_mode`, so the output mode, which is the size of every recorded
  frame, changes when the command arrives, before the hosted client has seen the new size.
  The hosted window still holds its previous buffer until the client commits a new one.
- `src/recorder.rs:373` to `389`: every recorder tick captures the frame first (`capture_into`) and only then sends
  the frame callbacks and flushes (`render::send_frame_callbacks`, `flush_clients`).
- `src/render.rs:114` and `src/frame_pacing.rs:135`: while a recording runs, the live redraw returns early and the
  fallback pacing timer is never due, so the recorder tick is the only sender of frame callbacks.
- winit 0.30.13, `src/platform_impl/linux/wayland/event_loop/mod.rs:486`: a `RedrawRequested` event is withheld while
  the window's frame callback state is `Requested`, which Slint's renderer sets before every swap through
  `pre_present_notify` (`i-slint-backend-winit` 1.18.1, `renderer/femtovg/glcontext.rs:33`).

Note of 2026-10-14: the line numbers in this section are those of 0.4.0.
After the change below, the `set_mode` call of `apply` is at `src/screen.rs:215`, `tell_surfaces_scale` is at
`src/screen.rs:220`, and the early callbacks are at `src/screen.rs:241`.
The call of `capture_into` is at `src/recorder.rs:376`, `render::send_frame_callbacks` at `src/recorder.rs:388`
and `flush_clients` at `src/recorder.rs:392`, because the module doc of that file grew by three lines.
`src/render.rs:114` and `src/frame_pacing.rs:135` did not move.

So a size that reaches the application between two ticks is drawn right after the next tick's capture
(a median of 0.4 ms after the callbacks), committed a few milliseconds later, and first captured one tick later.
The capture at that tick already has the output mode of the newest command, one step further.
The frame is the old buffer on a canvas of the new size: the canvas outside the buffer is the compositor's clear color
(`src/render.rs:80`, 0.1 gray, pixel value 25) where the application paints 28, which is the strip.
A buffer wider than the canvas is cut by it, which is the cut text.

## Verification

An instrumented copy of the compositor source (probe lines for the control command, the configure, each commit with
the committed buffer size, each capture with the held buffer size and the output mode, and each batch of frame
callbacks) hosted the IDE built from commit `d7df483fc0` of the branch `feat/ide-resize-frame`, which adds debug
lines at the target `monochromatic_ide::native::frame` (winit size event, redraw request, view report, start and end of
each draw).
Timestamps are the log's own wall clock, so compositor and application lines share one axis.
Every run recorded 143 frames or more with no dropped frame, at a one-minute load of 8 to 18 when it started.
The recording of the instrumented copy had 77 frames that did not fit; the installed 0.4.0 binary had 76 in the same
drag, so the probes do not change the picture.

Three runs of the 16 ms drag, 81 to 82 captures in the drag:

- The held buffer width differed from the output mode width in 76, 77, and 77 captures (93.8 to 95.1 percent).
  In 73 to 74 of them the held buffer was exactly one step behind.
- Each recorded frame was joined to the capture line of its index.
  Of the 84 frames in the drag window, every one of the 77 (first run) that did not fit had a held buffer different
  from the output mode, and all 7 frames with a held buffer equal to the mode fit; the other runs had 8 and 7 such
  frames, all fitting.
  The 100 ms drag had 459 frames with held buffer equal to the mode over two runs, none of them failing to fit.
  So no frame had a committed buffer of the right size and wrongly wrapped content.
- Medians in milliseconds after the command: the re-wrap finished at 4.1 to 4.3, winit's redraw request came at 9.6
  to 9.9 (after the next callbacks, not after the size event, which the application saw at 0.2), the draw ended at 13.1,
  and the commit reached the compositor at 13.0 to 13.5.
  The tick period was 16.7 ms.
  A latency of about 13 ms against a 16.7 ms period leaves almost every capture one step behind.

Positive controls, to show the probe sees a delay that is introduced on purpose:

- The application was stopped with `SIGSTOP` for 40 and 80 ms after one command of a 100 ms drag.
  The probe logged winit's size event at 41.6 and 81.4 ms after the command (0.2 ms in the run without a stop),
  and 3 and 6 consecutive captures held the old buffer (at most one without a stop).
- Counterfactual on the compositor copy: with `NWS_PROBE_CALLBACKS_AT_RESIZE` set, `screen::resize` sends the frame
  callbacks right after the configure, which opens the gate at the command.
  Over three runs of the 16 ms drag the redraw request moved from 9.6 to 9.9 ms to 0.2 ms, the commit from 13.0 to 13.5
  ms to 7.5 to 7.9 ms, and the stale captures fell from 93.8 to 95.1 percent to 55.6 to 60.5 percent.
  The rest is the application's own work (re-wrap about 4 ms, draw about 3.5 ms) against a 16.7 ms period.

The project's drag harness measured the build with the new debug lines against the build of the recorded numbers
(sha1 `9c60b963`), four sessions each per file in the 16 ms drag, the installed 0.4.0 compositor, no dropped frame,
one-minute load 4.7 to 8.2 at the start:
frames that did not fit were 76 to 77 against 75 to 77 on `code50k.txt` and 76 to 77 against 73 to 77 on
`long2000.txt` (new build against earlier build),
the longest gap 17.0 to 17.7 ms in all sessions but one pair
(33.3 ms in the fourth `long2000.txt` session of both builds).
The logging changes no measured number.

## What the application contributes

The width reaches the Rust code at once.
`ui/app.slint:408` (`changed width`) reports the layout width of the source `Flickable`, which Slint updates when it
handles the size event, so `bind_viewport` and `place_scrolled` have it a median 0.4 ms after the command.
At every one of 715 draws in five sessions (three 16 ms and two 100 ms), the width the view had last reported was the
toolkit's window width minus 333 px, and the buffer committed after the draw had that window width.
The wrap width of a draw was therefore never one step behind its own size event.

The re-wrap (or the tile build that takes its place in some steps) usually ends about 5 ms before the gated draw can
begin, so it adds nothing to the latency then.
It does add some: in 9 to 12 of the 79 to 80 draws of each 16 ms session the draw began more than 1.5 ms after the
callbacks (up to 8 ms), and 13 to 18 of the 84 callbacks arrived while such a step was still running.
Three captures per session were two steps behind rather than one.
With the gate open the step is about half of a latency that still leaves more than half of the captures behind.
Nothing the application can set removes the delay: the gate is inside winit, and the mismatch between canvas and buffer
is created by the compositor's `set_mode` at command time.

## Improvement: callbacks right after the configure

`screen::apply` (`package/nested-wayland-session/cli/src/screen.rs`) now calls `render::send_frame_callbacks` and
flushes the clients when a recording runs and the size or scale changed.
It does so after `set_mode`, `reconfigure_fullscreen` and `tell_surfaces_scale`, which is the placement the
counterfactual probe used.
The reasons, from the code:

- The configure, the preferred scale and the callbacks are queued in that order and written by one flush, so the client
  has the new size before its redraw gate opens.
  Callbacks sent before `reconfigure_fullscreen` could let the client draw at the old size.
- The block sits in `apply`, not in `resize`, so `scale` and a parent-initiated resize (both end in `apply`) get
  it too.
  A scale change changes the output mode in the same call, and the toolkit's redraw for it waits on the same winit
  gate by the source named above, so it can have the same stale captures; that was not measured.
  A command that leaves the geometry as it was sends nothing, and outside a recording the live redraw path already
  sends callbacks after every presentation.
- Smithay 0.7.0 drains a surface's committed callbacks when it sends them (`send_frames_surface_tree`,
  `src/desktop/wayland/utils.rs`), so a request is answered once.
  The tick after a resize finds nothing until the client asks again.
  The client therefore draws once more per resize command than the tick alone would make it, which is the whole cost.
- The capture cadence, the drop accounting and the order inside a tick are unchanged.

Guards:

- `mise run //package/nested-wayland-session/cli:inspect:resize-callbacks` hosts an animating Slint scene, records at 2
  fps, resizes 150 ms after a tick, and reads the client's `WAYLAND_DEBUG` log.
  On the unchanged compositor it failed with
  `600x340: the frame callback came 337.5 ms after the configure, not at once`
  and the first frame recorded after each resize had green 64 (shrinking) or 25 (growing, the compositor's clear color)
  at the right edge instead of the strip's 192.
  With the change the first callback comes 0.0 ms after the configure in all four resizes, two callbacks (by the queue
  names in the log, one requested by the toolkit and one through Mesa's EGL surface queue) are answered together with
  none after them in the next 100 ms, and the
  first recorded frame has the strip.
- `frame_pacing::tests::early_callbacks_follow_a_geometry_change_while_recording` covers the decision and failed
  with `assertion failed: resize_needs_early_callbacks(true, true)` when the function was made to return false.

Measured effect, 16 ms drag (`resize W 660` every 16 ms, 1100 to 600 to 1100 pixels, two sweeps), the project's drag
harness, instrumented copies of the tree before and after the change (the same probe lines as above, no environment
switch), interleaved before and after in the same window of time, 60 fps recording.
A run counts when it has no dropped frame and the one-minute load at its start is below 20; every run listed counts.
The window of captures is from the first command to 100 ms after the last reply (82 to 84 captures per run), so the
percentages differ a little from the earlier ones.
In every run the number of frames that did not fit equaled the number of captures with a held buffer width that
differs from the output mode width; the two counts were kept, the frames themselves were not, so the equality of the
sets was not rechecked.

- Series with the IDE bundle `trace-rewrap` of the verification (8 runs per side, load at start 4.77 to 14.7):
  stale captures before the change 88.1 to 92.7 percent (mean 90.3), after it 45.2 to 56.0 percent in 7 runs (mean 49.5)
  and 91.5 percent in one run.
  The median delay from a command to the commit of its width fell from 11 to 13 ms to 6 to 7 ms in the same runs.
- Series with the current bundle `package/ide/app/dist/monochromatic-ide` (4 runs per side, load at start
  8.3 to 11.4): stale captures before 90.5 to 91.7 percent, after 42.9 to 46.4 percent,
  commit delay median 11.5 to 13 ms before and 6 to 7 ms after.
- Two runs are slow: before-r7 (92.7 percent stale, commit delay median 47 ms) and the 91.5 percent run
  after the change (commit delay median 34 ms).
  Both recorded 141 frames instead of 143 and neither dropped one.
  Whether the application or the host was slow was not determined; both runs meet the gates, so they stay in the lists.
  The 91.5 percent run is the reason the after list has a range and a count instead of one number.
- The earlier counterfactual probe gave 55.6 to 60.5 percent after.
  The runs now give 45.2 to 56.0 percent; the capture window and the host's load at the time differ, and no cause of
  the difference was measured.

Method notes:

- The series with the current bundle waited a fixed 8 s after the first drawn frame, because that bundle does not print
  the measurement-pass line the other series waits for.
- All 24 runs (16 with `trace-rewrap`, 8 with the current bundle) recorded 141 to 143 frames and dropped none on both
  sides, so the capture cadence and the drop accounting did not change.
- The instrumented copies of both sides are of the commits `e751a32` (before) and `2f30a8630a` (after).

The application's own work (the earlier medians were a re-wrap of about 4 ms and a draw of about 3.5 ms) against the
16.7 ms period is the part that this compositor cannot remove; how the remaining stale captures split between that work
and the phase of the commands against the ticks was not analyzed.

## Reading recorded resize frames

- A frame whose held buffer differs from the output mode is not evidence about the application's wrapping.
  The instrumented build above is the way to tell: join each frame to its capture line.
- A drag slower than the tick period leaves at most one such frame per step, and a 100 ms drag had 15, 19, and 28 of
  243 captures in three runs without a stop.
  That number moves with the phase of the commands against the ticks.
- After the improvement above, a frame whose held buffer differs from the output mode is rarer but still not evidence
  about the application's wrapping.
- A recorder that sends the callbacks before the capture and captures a few milliseconds later, or an output mode that
  follows the committed buffer instead of the command, would remove the remaining mismatch.
  Neither was built; this document does not ask for either.

## What was not measured

The live path of the compositor (no recording, parent redraw events) was not measured, before or after the change; the
change does not run there.
A host with a load above 20 was not measured.
A scale change while recording has the inspect task `inspect:scale` (frames recorded after a switch have the new size
and the client's drawing) but no before and after count of stale captures.
Drags slower than 16 ms were not measured after the change.
