# Two stop-lock controls measured a wall window instead of the ordering, so the stop came before Git held its lock

`a_kill_between_the_pid_file_and_the_lock_still_leaves_no_lock` and
`a_push_stop_of_a_holder_slow_to_finish_its_cleanup_leaves_no_lock`
in `package/git-policy/cli/src/native/lock/git_stop_lock_tests.rs` failed together in one whole host run of
`mise run //package/git-policy/cli:native:test:host` on a branch that changes nothing in that folder,
and each passed when run alone.

This record is about the two defects behind that:
a run limit used as a bet that the holder reaches its lock quickly,
and a witness of "the holder held the lock" written by a shell the stop kills.

## Symptom

Reproduced on this host (2026-11-08,
eleven controls sharing a tenth of a processor,
the package's own test binary run four times):

```text
thread 'lock::git_stop_lock_tests::a_kill_between_the_pid_file_and_the_lock_still_leaves_no_lock' (2439914) panicked at src/native/lock/git_stop_lock_tests.rs:295:5:
bounded run cut between the unlinks SlowToFinishCleanup: the holder took the lock before it was stopped

thread 'lock::git_stop_lock_tests::a_push_stop_of_a_holder_slow_to_finish_its_cleanup_leaves_no_lock' (2439916) panicked at src/native/lock/git_stop_lock_tests.rs:295:5:
bounded run SlowToFinishCleanup: the holder took the lock before it was stopped

test result: FAILED. 9 passed; 2 failed; 0 ignored; 0 measured; 4810 filtered out; finished in 29.55s
```

Five tests failed across those four runs,
both of the controls above among them in every failing run.

There was no slow suite run and no watchdog:
the runs ended in about 30 s and
the failure was an assertion,
not a hang.

## What the controls assumed

Both controls start a holder that takes the lock of `refs/remotes/origin/main`
(`git update-ref --stdin` told to `prepare`,
under `strace -f` with an injected `unlink` delay of 5 s),
run it through `run_bounded_push` with `Patience::Fixed(HOLD_DEADLINE)`,
and only then judge what the stop left on disk.

Two things in that arrangement were decided by elapsed time,
and neither was measured before it was chosen:

- `HOLD_DEADLINE` (2500 ms) decided when the stop is asked.
The controls took that to be after the holder holds the lock,
because Git normally takes it in tens of milliseconds.
- The witness of that ordering was the file the holder's subshell writes,
`: > "$READY"`,
after its `while [ ! -e "$LOCK" ] ... sleep 0.05` loop.
The stop signals the whole tree,
the subshell included,
so the witness is written by a process the stop kills.

Measured with a temporary watcher in the test (uncommitted,
500 µs looks at the lock and its pid file),
against the same 2.5 s limit:

```text
TIMELINE bounded Responsive: deadline=2.5s grace=12.063972s; total=6.400103865s
TIMELINE   lock born	at 1.099612639s
TIMELINE bounded Unresponsive: deadline=2.5s grace=2s; total=9.799988606s
TIMELINE   lock born	at 699.826623ms
TIMELINE bounded SlowToFinishCleanup: deadline=2.5s grace=2s; total=15.39994787s
TIMELINE   lock born	at 2.900171116s
```

The plain-Git holders reached their lock at 0.7 s and 1.1 s,
and the `strace -f`-wrapped one at 2.9 s:
the limit sits inside the spread of the same arrangement,
which is why only the two `strace`-wrapped controls failed.

## Root cause

- The run's limit answered "when is the stop asked" from the clock,
and the controls needed "after Git's lock is on disk".
When the two disagree the stop is asked first,
and the holder's announcement is then killed with the subshell that writes it,
so the assertion reports the holder never took the lock.
- The witness could not outlive the stop:
its writer is inside the tree the stop signals,
and its existence never proved the lock came first anyway.

## Reproduction

The four runs above use the package's test binary directly,
so the whole `lock` module shares one process:

```sh
systemd-run --user --scope -p CPUQuota=10% -p MemoryMax=4G -p TasksMax=512 -p IOWeight=10 -- \
  <test binary> lock::git_stop_lock_tests --test-threads 11
```

At a normal allowance the same run passed every time;
capping the CPU across the whole process tree is what made Git's start longer than the limit.

## The controls now

- The run's limit is held until a look of the control sees the lock and the pid file beside it:
`push_progress::LimitHeldForAnOrdering`,
installed per thread by `hold_limit_for_an_ordering`,
and consulted in `Clock::check` by `limit_has_passed`.
Only test builds carry it,
and no control installed means a limit is reached by time as before.
The hold has a ceiling,
`ANNOUNCEMENT_HOLD` (the owner's "3x worst case",
`STOP_CEILING`),
so a holder that never reaches its lock fails the control's own assertion,
which names the two files and their existence,
instead of waiting for the watchdog.
- The controls read Git's own two files,
never the shell's marker:
`the_holder_holds_its_lock` is the same ordering the stop's own claim reads before it signals
(`LockLedger::claiming`).
The watched and no-progress controls keep the holder's marker,
because a step that is still working is never stopped,
so their stop cannot arrive before the lock.
- The two cut controls decide their stop's wait by the cut on disk rather than by the grace alone
(`StopRule`,
`ControlRules.blocked`),
and each records that it saw the cut:
- `a_kill_between_the_pid_file_and_the_lock_still_leaves_no_lock` waits while the pid file is still there,
so the kill that ends the wait lands between Git's two unlinks however long the host takes to run the handler;
- `a_push_stop_of_a_holder_slow_to_finish_its_cleanup_leaves_no_lock` waits while the lock is still there,
so Git finishes its own cleanup and no kill lands inside it.

Repeats of the same capped run after the change:
10 runs,
11 controls each,
0 failures.

Shown failing,
each plant reverted after its run
(the plants are the shipped defects these controls exist to catch,
not changes to the controls):

- a `LockLedger::claiming` that reads no lock before the stop signals,
failed in 24.9 s with
`bounded run SlowToFinishCleanup: a stopped Git child left lock files: .../refs/remotes/origin/main.lock size=41 born=SystemTime { tv_sec: 1794194824, tv_nsec: 288383998 } pid_file_beside=false`
(the 20 s is this control's `SWEEP_WAIT` allowance for a sweep on a thread,
  not the watchdog);
- an `escalation_waits` that gives a blocked member no wait past the grace,
failed in 5.3 s with
`SlowToFinishCleanup: the stop ended without a look seeing Git between its two unlinks,
so the ordering this control is about never happened`.
The second plant also shows why the cut witnesses are needed:
the killed cleanup left the lock,
the claim then removed it,
and the "no lock left" assertion alone would have passed.

## What is not claimed

- `SWEEP_WAIT` (20 s) is still a positive allowance with no measurement beside it.
A plant of the claim therefore reports after that allowance.
It is not a decision about the stop,
and it is bounded well inside the 120 s watchdog.
- The `t` state `strace` holds the tracee in is no longer read by any control:
the two slow-cleanup controls decide by the cut on disk.
`in_uninterruptible_sleep` and the ceiling keep their own controls in
`process/stop_waits_while_a_member_cannot_be_interrupted_tests.rs`.
- The stop reads the process tree once,
  before it signals.
Measured here:
Git was forked by `strace` after that read,
so `kill_members` never signalled it,
and it went on to finish its cleanup 5.9 s after the push had been reported stopped.
No lock was left,
and this record does not claim it is a defect of the stop;
a stop that re-read its tree would be a production change of its own.

## Upstream filing decision

Nothing to file.
The defects are in this repository's controls.
Git's behaviour is as this repository already records it
(`doc/handover/cli-git-rust-implementation.md`,
"The leftover ref lock is not a T5 regression"):
its `SIGTERM` handler removes the lock's pid file and then the lock,
and the two are 130 µs apart on a quiet disk.
