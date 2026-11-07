# music-player desktop-e2e: Search checks raced the answer, and the headless compositor start is slow

Recorded 2026-11-07T13:28:00-05:00.

## Search result checks timed out waiting for a highlighted result

`click-through/unreadable-row-stays-selected` passed 2 of 3 runs and failed with
`timed out waiting for a highlighted result for Qqq`.
`openResultFor` in `package/music-player/desktop-e2e/src/wide-select.ts` typed the query, paused a fixed 1500 ms and
pressed Down and Up once, then waited for a state line with a highlighted result.
The app wrote no state line when an answer landed, so nothing could say that the answer was on screen.
Measured on 2026-11-07 against the current release binary: with the pause set to zero the check fails on every run with that
message, and the last states show `search_query` empty, `zone` `button` and `focus` `search-back`, that is, the keys
reached the query field before any result existed and the keyboard left it for the back button.
The pause is a guess at the app's latency, so any load that makes the answer slower than the guess gives the same failure.
Fix: `show_view` in `package/music-player/desktop-app/src/ui_search_apply.rs` ends with a `ui_state` line (reason
`search-answer`), and the check waits for that line with the query and a highlighted result, bounded by the time a new
file may take to be found, and presses nothing before it.

## settings/licenses-row and settings/escape-ladder, not reproduced

Both were reported failing on an unmodified main. In headless runs on 2026-11-07 they passed in the full wide session
(once), in `E2E_ONLY=settings` (several times), against the stale 2026-10-26 release binary and against a fresh one,
and with `END_SETTLE_MS` and `TAB_SETTLE_MS` at zero.
No failure text of the reported runs was kept, so the cause is unmeasured.
Both checks sleep a fixed time and then read the latest state (`session.last()`), which is the same shape as the Search
race, but the zero-settle run did not fail, so that is not shown to be the cause.
The reported runs used the visible nested window and these used `--headless`; that difference is untested.

## The headless compositor start

`E2E_COMPOSITOR` can name an executable that adds `--headless` to the compositor's command line, so that no window
appears on the desktop. Two start problems, measured on 2026-11-07 on a loaded host:
- `--headless --gpu software` (llvmpipe): the app ran but never mapped a toplevel within 20 s
  (`no hosted toplevel available for graceful close`), so the run read no state lines.
- `--headless --gpu auto`: the compositor tried Mesa's default device first and took about 56 s to open its control
  socket, against the runner's 30 s wait (`SOCKET_POLL` in `src/timing.ts`).
  `--gpu /dev/dri/renderD129` (the device `auto` chose) started in time in most runs, but a start still stalled in
  some runs with no state line in the first wait, which is a start problem and not a check failure.
A lowered `E2E_UNIT_CPU_QUOTA` (50% and 100%) made the app take over 20 s to map its window, so it is no way to slow a run.
