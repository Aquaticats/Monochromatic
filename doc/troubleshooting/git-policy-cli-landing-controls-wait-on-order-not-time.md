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

## What is not claimed

- The other clock windows in this package's landing controls were read and are left as they are,
  because the thread each waits on is production code with no in-process signal to take instead,
  and their expiry fails a prompt assertion with no join behind it:
  `wait_until` and the note-retry window of `service_push_retry_tests.rs`,
  the renewal windows of `service_claim_renewal_tests.rs` and `service_work_tests.rs`,
  `until_staged` and `until_a_commit_waits` of `service_index_prestage_tests.rs`,
  and `rules_precompile_tests.rs`.
  Each would need a seam in the code it observes,
  as `Announce` already is for the index lock.
- `service_index_lock_tests.rs` and `service_index_wait_tests.rs` carry the same two shapes and were not read here:
  they belong to other sessions at the time of this record.
- A landing that hangs before its wait is reported at `WAIT_WORD_CEILING`,
  which is the ceiling the shipped landing gives a foreign holder;
  the control cannot end a landing that is stuck inside a Git step,
  so such a run still ends at the watchdog.
