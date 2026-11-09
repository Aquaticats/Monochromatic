# A landing control measured a wall window instead of the ordering, and its failure hung the test until the watchdog

`a_branch_switched_while_the_lock_is_awaited_stops_the_landing` in
`package/git-policy/cli/src/native/landing/service_land_tests.rs` failed once in a whole-suite run of
`mise run //package/git-policy/cli:native:test:host`,
and the run took 120 s more to end than the control does alone.

This record is about the two defects behind that,
which the same control can carry on a host with no other session on it:
a clock window standing in for an order of events,
and a failing assertion that leaves a scope joining a thread which waits out a ceiling.

## Symptom

In the whole-suite run the control asserted `the landing reached the wait` after its own 10 s poll for the batch's
intent file.
Alone it passed in 0.359 s.
After the assertion failed,
the scope's join waited on the landing thread,
which was waiting for the `index.lock` the control still held,
so the run ended at the test watchdog's 120 s,
not at the assertion.

## What the poll actually waited for

The intent file was taken to mean "the landing is waiting for the index lock",
because `write_intent` runs before `take_index_lock`.
Two things are wrong with that witness.

The file appears before the wait,
not at it,
so its existence never proved the ordering the control acted on.
And its appearance sits behind a flush:
`write_intent` writes `journal/1.tmp`,
flushes it through `service_durable::flush_file`,
renames it to `journal/1.intent`,
and flushes the journal directory.
A control's landing runs outside `in_a_landing`,
so `flush_file` makes that flush in place instead of queueing it,
and the rename that makes the file visible comes only after the flush returns.

Measured on this host (`strace --follow-forks -T` on the test binary):

- the traced runs show `fsync(journal/1.tmp)` then `rename(journal/1.tmp, journal/1.intent)`,
  and no other step between the landing's last write and the file's appearance;
- that one `fsync` took 6.3 ms and 6.1 ms in two traced runs,
  and 1.299 s in a third,
  with no other work on the host;
- `LandTiming` read through `land_batch_timed` in three untraced runs of the same control gave an intent phase
  (`timing.journal`) of 15.2 ms,
  56.1 ms and 1.320 s;
- one untraced run had the landing reach the lock 4.6 s after the control wrote the foreign lock,
  which the landing's own line records as `lock_found="foreign no-marker size=7 age_ms=4625"`.

`doc/troubleshooting/native-binary-tests-hit-the-shared-bound-under-host-load.md` records the same flush reaching
seconds in this package's binary tests.
Nothing in the control bounded any of it,
and the 10 s window was a bet on the disk.

## Root cause

Two defects,
either of which alone is enough to fail this control on a loaded host:

- The witness was a clock window on a file written before the point the control needed,
  so the control could not tell "the landing is inside its wait" from "the landing is still making its objects".
- The failure path gave nothing back:
  the foreign lock stayed on disk,
  so the scope's join ran into the ceiling this control's own context names,
  `LOCK_CEILING` (600 s),
  and `arm_watchdog` (`testing/test_support.rs`,
  `RUN_LIMIT`) aborted the process at 120 s.
  Note 2026-11-08T23:43:41-05:00 (`21961de4da`):
  the constant is `QUIET_WINDOW`,
  the same 120 s,
  and the watchdog now ends a run that shows no sign of work for it instead of one that outlives it
  (`doc/troubleshooting/git-policy-cli-unit-test-watchdog-judges-progress.md`).

## Reproduction

Replacing the intent write with `Ok(())`,
so the file never appears while the landing waits as before,
reproduces the whole failure deterministically with the old control:

```text
panicked at src/native/landing/service_land_tests.rs:1550:9:
the landing reached the wait
the unit-test run exceeded 120s: a test is stuck
(test aborted with signal 6: SIGABRT)
SIGABRT [ 120.139s] (1/1) ... a_branch_switched_while_the_lock_is_awaited_stops_the_landing
```

## The control now

- The landing says so itself:
  `land_batch_announcing` is given an announcer that is made inside the landing thread and moves an
  `mpsc::Sender` into it.
  The wait announces the holder's notice from that thread before it blocks,
  so the word is the ordering,
  not a clock.
- The channel goes with the landing,
  so a landing that ends before its wait closes it and is reported at once
  (`the landing ended without waiting for the index lock`),
  and a landing that says nothing inside `WAIT_WORD_CEILING` is reported as never reaching the wait.
  That ceiling is the one the shipped landing gives a foreign holder,
  `LANDING_LOCK_CEILING` (1 min),
  which is above the longest measured phase before the wait and below the watchdog,
  while this control's own landing is given `LOCK_CEILING` (600 s),
  the ceiling the failure above used to reach.
- The switch is made on that word,
  so it is made while the landing is inside its wait,
  and the lock is given up on every path:
  `ForeignIndexLock` removes it when the control lets go and also when the control ends however it ends,
  and `ForeignIndexLock::take` writes content into the lock so the landing never judges it a lock nothing holds
  (`service_index_unheld_removal.rs` keeps a lock with content).

Shown failing,
each plant reverted after its run:

- a landing that does not stop on a switched branch (the post-lock `still_checked_out` check removed) fails in
  0.203 s with `left: Ok([Landed { commit: "1cb94a77e9bec11bae9f85d6312baad81ec940c2" }])`,
  `right: Ok([BranchMoved])`;
- a landing that never reaches its wait (a branch-moved answer before the objects) fails in 0.04 s with
  `left: Err("the landing ended without waiting for the index lock")`,
  where the old control took 10.03 s and said only `the landing reached the wait`.

## The same shape elsewhere in the landing controls

`settlements_racing_renewals_lose_no_claim` (`service_claim_renewal_tests.rs`) set the flag that ends its renewing
thread after the settlements.
A settlement that failed skipped that line,
and the scope then joined a thread that renewed without end.
The flag is now set by `StopRenewing`,
whose `Drop` runs on every path.
Planted a failing settlement:
with the guard the control failed in 0.039 s,
and with the flag set by the plain statement the test binary was still spinning after 25 s,
with 20.9 s of system time in the renewing thread,
when it was killed.

## The other windows, each decided

Branch `fix/git-policy-cli-landing-controls-wait-on-order-everywhere` read every window the first record left
unclaimed and decided it.
Three kinds came out of it:
a window that decides the verdict (fixed with a seam),
a backstop around a real witness (left,
with the witness named),
and a failure that leaves a join or a held resource behind (fixed).

Fixed,
because the window decided the verdict:

- `rules_precompile_tests.rs` waited three seconds and called an empty cache "no warm",
  so a warm slower than that window passed the control that a landing changing no rules path warms nothing,
  and a warm that never ran cost the other controls a hundred and twenty seconds of polling.
  The warm job now counts,
  for test builds,
  that it ended and whether it warmed
  (`WARM_JOBS_ENDED`,
  `WARMS_MADE`),
  and the controls read that answer.
  Planted every landing as one that changes a rules path,
  with a four-second warm:
  the old control passed in 11.1 s,
  the new one fails in 9.2 s with `left: 2 right: 1`.
- `service_claim_renewal_tests.rs` kept an unheld claim's name for the fifty milliseconds it slept,
  so a renewal thread that never ran passed it;
  the renewal counts its finished passes for test builds (`RENEWALS_MADE`) and the control waits for two.
  Planted a renewal that refreshes every claim in the directory:
  the control fails in 0.033 s with `an unheld claim keeps its name`.
- the same file said a dropped renewal renames nothing after a fifty millisecond sleep;
  the task counts its own end (`TASKS_ENDED`) and the drop's own report,
  read through the service log capture,
  says whether it waited for the task or only for its ceiling.
  Planted a drop that never tells the task to stop:
  it fails in 18.1 s with
  `the renewal of claims did not stop within 18.095958s`;
  planted a drop that returns without waiting:
  0.031 s,
  `the task returned before the drop did`.
- the same file gave a renewal whose task cannot end half the pool's busy time to return,
  which a stalled host fails even though the ceiling fired;
  the count of the pool's finished jobs now shows it,
  and the one time left is the ceiling,
  which is the behaviour under test.
  Planted a drop that waits for the task however long it takes:
  it fails in 7.3 s with `8 of 10 jobs had finished after 7.081427656s`.
- `a_renewal_takes_far_less_than_its_stop_ceiling` compared one rename against a thousandth of a ceiling
  derived from a measured rename,
  so any loaded host fails it;
  it is an ignored measurement now,
  as the prestage measurement in this package already is.
- `renewals_running_at_once_start_no_thread_each` counted threads right after asking the runtime to start
  sixteen renewals,
  so the count was read of whatever had begun and nothing asserted that any renewal had;
  the task says it began (`TASKS_STARTED`) and the control waits for all sixteen.
  Planted a renewal that holds a thread of its own:
  it fails in 0.122 s with
  `16 renewals added 17 threads`.
  Not shown:
  the old reading passing wrongly.
  With that plant and every worker of the runtime busy for three seconds it still caught the plant,
  because the control's own fixture work gives the runtime time to start every task.
- `service_index_lock_tests.rs` decided that a live service's lock was not taken over by nothing arriving
  within six hundred milliseconds;
  the wait's own notice now says it reached the held lock,
  and the holder's marker still in the lock says
  nothing was taken over.
  Planted every marker as stale:
  the control fails in 0.058 s with
  `the waiting acquisition said what it waits for, so it waited for the live holder instead of taking its lock`.
- the same file read the clock after the wait's line,
  against half the interval that repeats it,
  to tell a
  wake by the removal from one by the reminder's timer.
  The watch knows which of the two ended it and now counts the waits it ended at their timer,
  for test builds
  (`WAKES_BY_THE_TIMER`),
  and the control reads that count.
  Planted a watch on a directory nothing happens in,
  so only the timer can end the wait:
  it fails in 10.157 s with `the wait ended at its timer and not at the removal; placements [true, false]`.
  Those placements are the shape no order of the placements can tell apart,
  which is what the clock reading was for.

Fixed,
because the failure left a join or a held resource behind:

- `a_drain_beside_a_slow_push_leaves_its_claim_alone` (`service_work_tests.rs`) joined a held push that
  waited on the control's release channel;
  an assertion that failed first left the scope joining it for its thirty-second bound.
  A guard lets the push go however the control ends.
  Planted a drain that never registers with its renewal driver:
  without the guard `FAIL [ 30.017s]` and `push_ms=30000` in the push's own line,
  with it `FAIL [ 0.017s]`.
- four index-lock controls joined a holder that waited for the wait's own line,
  so a wait that said nothing left the join to run out that holder's sixty-second bound.
  A guard lets the holder go however the control ends.
  Planted that state (the lock removed at the reclaimer's point,
  a staging flush slower than the hold):
  without the guard `FAIL [ 60.022s]` with the holder's `Timeout`,
  with it `FAIL [ 5.518s]` and `the wait said nothing at all, so the lock it was to wait for went away first`.

Left,
because the window is a backstop around a real witness:

- `wait_until` and `NOTE_RETRY_PATIENCE` (`service_push_retry_tests.rs`):
  the witness is the service's own notice file,
  the remote's ref,
  the claim file's disappearance and the
  service's own busy count,
  and the window is about ten seconds,
  which only a service that never acts reaches.
- `until_staged`,
  `until_made` and `until_a_commit_waits` (`service_index_prestage_tests.rs`):
  the witnesses are the staging file's appearance,
  the flush job's own end,
  and the flag `take` sets when a
  commit waits,
  and the windows (ten and sixty seconds) only a job that never ends reaches.
- `RenewalProbe`'s twenty-second bound and `GATE_CEILING` (`service_work_tests.rs`):
  the witness is the claim file's renamed name,
  and the ceiling only a push nobody lets go reaches.
- the lower bounds of `service_index_lock_tests.rs` and `service_index_wait_tests.rs`
  (`began.elapsed() >= holding`,
  `>= SHORT_STOP_CEILING`,
  `>= ceiling`,
  `notes.placing >= 300ms`):
  they fail only when the code returns too early,
  which a slow host makes less likely,
  never more.
- the start-time tolerance of `a_holder_proven_alive_is_named_with_its_program_start_time_and_the_way_out`:
  both readings name the same instant and the second is a rounding of the first,
  which a four-second flush plant left unchanged (the control passed).
- `began.elapsed() < slow.interval` in two `service_index_wait_tests.rs` controls:
  the interval is thirty seconds against a wait of about twenty milliseconds,
  a separation of three orders of magnitude.

## What is not claimed

- One full-suite failure of `landing::service_index_lock::tests` was reported as a wait that said nothing
  (an empty list where a line was expected) and was not reproduced here:
  five runs of the two controls named,
  thirty copies in six processes,
  three module runs at half a processor
  and one run beside the fold sweep all passed.
  The state it reports is reachable and is now said in words
  (`the wait said nothing at all, so the lock it was to wait for went away first`),
  and the control it was recorded against reads its list only after its holder's thread has been joined,
  which cannot happen unless the wait did say a line:
  the note's name and its list do not fit that file,
  so no verdict is claimed about which control it was.
- `service_index_wait_tests.rs`'s own windows were read and are left except where this record names them.
- A landing that hangs before its wait is reported at `WAIT_WORD_CEILING`,
  which is the ceiling the shipped landing gives a foreign holder;
  the control cannot end a landing that is stuck inside a Git step,
  so such a run still ends at the watchdog.
- The fold sweeps the same branch fixed pay a filesystem commit for every step they walk,
  and the run's own 120 s watchdog is what stops them on a disk having a bad minute;
  the measurement and the switch that ends it are in
  `doc/handover/cli-git-rust-implementation.md`,
  "The fold sweeps pay a filesystem commit for every step".
  The other controls of the two fold modules still sync,
  and each walks one or two folds.
