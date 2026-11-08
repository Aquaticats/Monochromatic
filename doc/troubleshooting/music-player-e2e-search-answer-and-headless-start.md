# music-player desktop-e2e: Search checks raced the answer, the headless start stalls cold, and Settings lost its focus

Recorded 2026-11-07T13:28:00-05:00.

## Search result checks timed out waiting for a highlighted result

`click-through/unreadable-row-stays-selected` passed 2 of 3 runs and failed with
`timed out waiting for a highlighted result for Qqq`.
`openResultFor` in `package/music-player/desktop-e2e/src/wide-select.ts` typed the query,
paused a fixed 1500 ms and
pressed Down and Up once,
then waited for a state line with a highlighted result.
The app wrote no state line when an answer landed,
so nothing could say that the answer was on screen.
Measured on 2026-11-07 against the current release binary:
with the pause set to zero the check fails on every run with that
message,
and the last states show `search_query` empty,
`zone` `button` and `focus` `search-back`,
that is,
the keys
reached the query field before any result existed and the keyboard left it for the back button.
The pause is a guess at the app's latency,
so any load that makes the answer slower than the guess gives the same failure.
Fix:
`show_view` in `package/music-player/desktop-app/src/ui_search_apply.rs` ends with a `ui_state` line (reason
`search-answer`),
and the check waits for that line with the query and a highlighted result,
bounded by the time a new
file may take to be found,
and presses nothing before it.

## The headless start stall is a cold page cache

Recorded 2026-11-07T15:20:00-05:00,
on the development host (28 hardware threads,
load average near 100,
other sessions building),
with the compositor 0.6.1 and the release binary of the app,
headless on `/dev/dri/renderD129`.

The first start of each batch was slow and the next ones were fast,
so the start time is bimodal,
not noisy:
- Warm (the files of the compositor,
  Mesa,
  LLVM and the app in the page cache),
  11 starts across `renderD129`,
  `renderD128`,
  `auto` and `software`:
  control socket at 0.06 to 0.26 s,
  first `ui_state` line at 0.26 to 1.1 s.
- Cold,
  7 starts (four after `dd iflag=nocache count=0` on the 104 files a session maps,
  three first starts of a batch):
  socket at 9.9 to 34.9 s,
  first state line at 11.6 to 56.6 s.
  `--list-gpus` alone took 31.9 s on a first call.
- Where a cold start stops,
  from the unit's log:
  the compositor's first line comes early,
  the control socket comes late (device and Mesa start-up),
  and the app's first line comes late again (the app's own load).
  The two stops are the compositor and the app,
  not the socket or the state line themselves.
- Not the CPU quota:
  `nr_throttled` was 0 in every unit,
  and the CPU time of a unit was 0.2 to 0.9 s over a start of up to 56 s.
  The unit's `io.pressure` `full` total was 1.6 to 9.8 s in the cold starts,
  and an `strace -T` of a cold `--list-gpus` shows its gaps at the `mmap` of shared libraries,
  so the time is page-in under disk contention.
- Not `--gpu auto`:
  after one first start,
  `auto`,
  `renderD128`,
  `renderD129` and `software` each reached the first state line in 0.7 to 1.0 s.
  The earlier 56 s for `auto` and the stalls "with no state line" in the explicit-node runs fit the same cold start meeting the runner's old 20 s first-state wait and 30 s socket wait.
- `software` (llvmpipe) reached the first state line in 0.76 s warm and 11.6 s cold.
  The earlier report that it never mapped a toplevel in 20 s is not reproduced as a start problem;
  whether llvmpipe maps a toplevel is not measured here,
  because the measurement read state lines,
  not the compositor's window list.

What changed in the runner:
- `COLD_START_BOUND_MS` (`src/timing.ts`) bounds the device listing,
  `--version`,
  the unit start,
  the socket and the first state line,
  with the measurements above beside it.
  The waits are long so that a cold start passes;
  a warm start ends them after about a second.
- The runner asks the compositor for `--list-gpus` before the first session (it needs the answer to choose the render node),
  which opens every render node and loads Mesa.
  One run after eviction,
  n=1:
  the listing took 6.45 s and the session then started warm.
  That is one run,
  so the size of the gain is not established.
- A start that does not finish says how far it got,
  from the log:
  the compositor wrote nothing,
  the compositor is running but has no socket,
  the socket is open but no app was started,
  the app was started but wrote nothing,
  or the app logs but wrote no state line (`startStageOf` in `src/log-scan.ts`).

## Headless is the default

An agent's run must never show anything on the owner's desktop.
The runner starts the compositor with `--headless --gpu <node>` and the unit gets none of `WAYLAND_DISPLAY`,
`WAYLAND_SOCKET`,
`DISPLAY` and `XAUTHORITY` (left out of the environment and listed in `UnsetEnvironment`,
which a probe of the running unit confirmed:
the compositor's command line carried `--headless --gpu /dev/dri/renderD129` and its environment held none of the four).
Only `--visible` or `E2E_VISIBLE=1` opens a window;
a failed device probe is an error,
never a visible window.
The node comes from the compositor's own `--list-gpus` (the node `auto uses:` names,
else the first one Mesa could open),
and a host with only the software renderer is refused with the listing quoted,
unless `E2E_GPU=software` names it.
The unit tests assert on the argument vector and the environment of both modes,
and the guard tests fail when the `--headless` arguments or the display variable handling are removed (shown by removing them and running the tests).

## Failure evidence

Each failing check writes `<E2E_OUT>/failures/<id with / as -->.failure.txt`:
its text,
the session,
the last ten state changes (`seq`,
`reason`,
`zone`,
`focus`,
`route`,
`unfolded`),
the count of position ticks skipped,
and the final state.
Position ticks are skipped because a playing track writes them without pause:
the first file written from a real failure held ten of them and showed nothing.
A session that stops between two checks is recorded as `session/<name>` with the same evidence;
before,
a settings-only run ended with a stack trace and no summary when a click-through setup step timed out on the page the failed checks had left open.

Beside each failure file sit the files that say what the session itself was doing.
They are read while the session still runs,
because the scratch directory and the unit go away with the run:
- `<id>.unit.log`:
the unit's standard output and error,
which hold the compositor's lines and the app's.
- `<id>.cpu.stat`:
the accounting of the unit's control group,
whose `nr_throttled` and `throttled_usec` say whether the unit was held back by its CPU quota.
- `<id>.io.pressure`:
the same group's `io.pressure`,
which says how long its tasks waited for the disk.

Each of the three is listed in the failure file with the path it was read from.
A unit that had already stopped has no control group to read and no log still being written,
and the failure file says which file was not kept and why,
instead of leaving it out silently.

## settings/licenses-row and settings/escape-ladder fail headless, and why

Recorded 2026-11-07T15:20:00-05:00.
The two checks failed in the earlier visible-window runs and passed in the earlier headless runs of the previous agent.
In this work,
headless,
`E2E_ONLY=settings`,
the release binary of commit `6720c6e7dc` unchanged:
- 15 runs:
  6 with all six checks passing,
  9 with `settings/licenses-row` and `settings/escape-ladder` failing together.
  The other four settings checks passed in all 15 runs.
- `browse`,
  8 runs of 4 checks on the same binary:
  32 of 32 passed.
  `browse/alt-down-up` did not fail again.

Cause,
from the session log of a failing run (headless,
the app's own `ui_state` lines):
- Ctrl+Alt+S opens Settings and `settings-list` takes focus (reason `programmatic`).
- 13 to 36 ms later,
  with no key pressed and no log line from the app,
  focus moves to `settings-back` with reason `programmatic` (the app was rebuilt once with a log line in `FocusStop` that prints the label and Slint's focus reason,
  to see this).
- `settings/ctrl-alt-s` had already passed (it waits for the route,
  and its detail text says `focus track-list`),
  the next two checks watch things other than focus,
  and `settings/licenses-row` then presses End and Enter on the Back button,
  which closes Settings:
  "timed out waiting for the Licenses page".
  `settings/escape-ladder` then fails because the page is the player and Tab never reaches a text field.
- The mechanism:
  `tracks.slint` and `folders.slint` have a focus timer and a `changed focus-tick` handler that call `focus()` on the list stop whenever the list is asked for.
  That stop refuses focus while a page covers the player (`Focus.player-active` is false),
  and Slint's `focus()` on a stop that refuses walks forward to the next element that accepts,
  which on the Settings page is the Back button (`window.rs`,
  `move_focus` and `publish_focus_item` in i-slint-core 1.18.1).
  A track list made again under Settings,
  which the wide layout does whenever `first-run-state` goes to a pane and back (a rescan),
  starts the timer and takes the keyboard 16 ms later.
- The window test that makes this happen (`a_track_list_made_again_while_settings_is_open_leaves_the_keyboard_on_the_settings_list`) failed with `settings-back` before the fix and passes after it.
  That the live runs make the list again at the same moment was not observed directly;
  the fix removing the failures from the live runs is the evidence (counts below).

Fix:
the timer and the tick handler also require `Focus.player-active`,
so the list waits and takes the keyboard when the player is back.

After the fix,
headless,
the release binary rebuilt from the fixed tree,
alternating `settings` and `browse`,
10 runs of each:
- `settings`:
  60 of 60 checks passed (10 runs with all six checks passing);
  before the fix 9 of 15 runs failed two checks.
- `browse`:
  40 of 40 checks passed;
  with the 8 runs before,
  72 of 72,
  and no `browse/alt-down-up` failure.
- One of the 10 `settings` runs also recorded `session/wide` failing after all six settings checks had passed:
  the click-through group's setup,
  which runs outside any check,
  timed out waiting for the folder of thirty tracks.
  A filtered run skips the earlier groups that leave the app in the state that setup expects.
  The same failure appeared in one of four runs on the instrumented pre-fix binary,
  so it is not caused by the fix;
  its own cause is not measured.
- The cause that was measured and fixed is the keyboard moving to the Back button.
  That the live failures came from a track list made again was not observed directly:
  the window test shows the mechanism,
  and the live counts moved from 9 of 15 failing to 0 of 10.

The visible-window runs that first reported the two failures are not repeated here (a run must not open a window).
The factor that differs between a visible run and a headless one that matters for this failure is when the rescan makes the list again,
which is a matter of timing in the app and not of the display;
the headless runs reproduce the failure without it,
so no headless way of varying output size,
scale or frame timing was needed.

## ui_render_support.rs reads the wall clock, and cannot change a result

Recorded 2026-11-07T15:20:00-05:00.
`src/ui_render_support.rs` installed a Slint platform without a clock,
so animations read the wall clock,
and `Rendered::wait` slept on it.
Measured on the desktop-app suite,
1603 tests:
- Clock frozen for the draw passes and `wait` still sleeping:
  three auto-play window tests failed (`in_the_dark_scheme_the_off_track_is_pure_black_with_an_outline_border`,
  `the_keyboard_ring_hugs_the_switch_row_with_the_group_corners`,
  `the_switch_shows_its_state_by_thumb_position_track_fill_and_mark`).
  They call `wait(300)` to let a 100 ms or 150 ms animation end,
  so they need time to pass on the clock the animations read:
  that shows the dependence,
  not an exposure to build speed.
- Clock frozen for the draw passes,
  `wait` moving the same clock in 16 ms frames (the limit of a build fast enough that no time passes between draws):
  every render test passed (the run stopped at the one failing guard test,
  1507 of 1508 run).
- Clock moved by 150 ms before each draw pass (the longest animation time in `ui/*.slint`),
  `wait` as above:
  1603 of 1603.
No test drew a page mid-animation and passed only because the build was slow,
which is what made the Search ellipsis test fail in release builds.
The wall-clock read could not change a result:
a sleep always lets at least the asked time pass,
and the two limits of draw-pass progress give the same results.
It is removed anyway,
in the shape of the fade-in fix,
so the tests need no sleeps and a guard test (`the_pass_step_covers_the_longest_animation_time_of_the_markup`) fails if a longer animation is written.
The guard test failed with the pass step at zero,
and passes with 150 ms.

## Three wide-session checks failed together at host load 40 to 55, and the cause is not found

Recorded 2026-11-07T19:20:00-05:00.
One full run (123 checks,
load average 40 to 55 on 28 hardware threads) failed `play/enter-plays-row`,
`audio/private-sink` and `play/space-button`.
The same binary passed 123 of 123 on the runs before and after it.

### What the failure evidence shows

- `play/enter-plays-row`:
  the first Enter on the ring row was handled (a `key` state with `shortcut used`),
  and no `now-playing` state followed within the 6 s `STATE_TIMEOUT_MS`,
  so `current_index` stayed at `-1`.
  In a passing run at a 25 percent CPU quota the same select to `now-playing` took 0.6 s.
- `audio/private-sink`:
  it never starts playback.
  It polls the PipeWire graph for 9 s and needs a stream that an earlier check started,
  so the first failure makes it fail with `0 output streams, 0 links`.
- `play/space-button`:
  the app was playing and the focus was on the Play button,
  and Space did not pause within 6 s.
  `press` accepts any state with a newer `seq` that satisfies the condition,
  so after one missed or late effect the Space checks that ran before it are off by one toggle;
  that is the reading the states support,
  and it is not measured.

### What was tried, and what it showed

The e2e session units take `E2E_UNIT_CPU_QUOTA` (default `400%`).
Runs of `E2E_ONLY=playback` (17 checks;
the checks before and after them in the session are skipped):
- 400 percent:
  1 run,
  17 of 17 pass,
  17.3 s.
- 40 percent:
  1 run,
  17 of 17 pass,
  52.1 s.
- 25 percent:
  2 runs,
  17 of 17 pass in both;
  the session step after the checks failed in 1 of the 2 (below).
- 15 percent:
  12 runs,
  17 of 17 pass in all;
  the step after them failed in 5 of the 12.
- 10 percent:
  4 runs,
  17 of 17 pass in all,
  and the step after them passed in all 4.
The three checks of the full run did not fail in any of these.
A CPU quota alone did not reproduce them.
The run time did not follow the quota either:
the 10 percent runs took 20.5 s to 25.4 s and the 40 percent run 52.1 s,
so the host's own load moved the result more than the cap did.

The step that failed under the cap is not a recorded check:
the click on the folder of thirty tracks (`WIDE_CLICK_POINTS.longFolder`) in `src/wide-interaction.ts`,
`timed out waiting for the folder of thirty tracks shown in the track pane`.
A screenshot taken after the 6 s wait showed the folder list laid out normally,
with the click point over the name `Gotta Montage line`,
the track pane still on `Zzz Tones`,
and the app log held no line from the click.
A `move` to the point plus 300 ms before the click did not change this in two runs.
The failing runs took 33.0 s to 75.1 s and the passing capped runs 17.5 s to 52.1 s,
at a host load average of 24 to 30 throughout,
so neither the cap nor the duration decided it.
So the click is lost or never reaches the touch area on some runs under load.
This failure and the full-run failures are different symptoms (a pointer press lost,
a key's effect missing);
they are not shown to share a cause.

### What is open

- Whether the engine answered the first Enter late or never.
  The failure file lists `ui_state` changes only,
  and the run that recorded this failure kept neither the unit's log nor its accounting beside the file,
  which `### Next measurement` describes.
- Why a pointer press is lost on a slow run.
  Neither the compositor (`package/nested-wayland-session/cli/src/input.rs` sends motion,
  press and release in one batch)
  nor the app log records the press.
- Whether the baseline binary behaves the same;
  a failure that no capped run reproduces on demand gives no comparison.

### Next measurement

The run now keeps what this measurement needs.
Every failure file has the unit's log,
its `cpu.stat` and its `io.pressure` beside it in `<E2E_OUT>/failures/`
(`E2E_OUT` defaults to `target/e2e` under the app directory),
and names each one with the path it came from (see `## Failure evidence`).

That the files hold real content was shown on 2026-11-08T15:00:47-05:00,
headless on `/dev/dri/renderD129`,
by one filtered run of `E2E_ONLY=play/enter-plays-row`
whose check was made to fail on purpose
(the build under test was the release binary already in the tree,
not one with a stall):
all three were written beside `play--enter-plays-row.failure.txt`.
- The log held 98 lines of the compositor's and the app's output,
  from the compositor's backend choice to the `ui_state` line with `seq=10`.
- `cpu.stat` held `usage_usec 899391` and `nr_throttled 0`.
- `io.pressure` held `some avg10=1.44 avg60=0.26 avg300=0.05 total=246028`
  and `full avg10=1.44 avg60=0.26 avg300=0.05 total=242058`.

What is left is to wait for a failing run under the host's own load,
then read the log for what the app did with the first Enter
and the two accounting files for what the unit waited on.
The failing check's own wait is not a fix:
raising 6 s would hide the stall,
which `SLW` in `CLAUDE.md` names as the defect to measure and remove.
