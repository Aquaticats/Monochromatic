# Two cli-git control windows are taken in opposite orders, so a threaded run wedges and the watchdog aborts it

`package/git-policy/cli`'s controls plant process-wide statements about a step and its looks,
and each group of statements has a `take_the_window()` that a control holds for its whole test.
On a host without `cargo-nextest` the tests run as threads of one process, and two of those windows were taken in
opposite orders there.

## Symptom

A run of

```text
cargo test --locked --offline --lib -- commit::commit_preparation::tests commit::native_commit_waiting_notice::tests
```

wedged. Five controls printed `has been running for over 60 seconds`, the run then reported

```text
the unit-test run showed no sign of work for 120.2 s (no task able to run or waiting for the disk, no processor time,
no bytes moved, and no process or task started): a test is stuck
```

and the unit-test watchdog ended the process with `signal: 6, SIGABRT`.
Two runs of that pair did this.
The one whole-library run this host made showed the same five stuck and, queued behind them,
two controls of `process::child_awaited_until_it_exits_the_stop_or_the_ceiling`
(`a_step_the_kernel_holds_past_its_ceiling_lands_when_it_ends`,
`a_step_the_kernel_holds_that_ends_after_the_bound_is_reported_by_its_own_status`),
which pass alone and as a module: `10 passed` in 3.03 s, three runs of three.

## Root cause

The five stuck controls are exactly the ones that take two windows, and the two modules took them in opposite orders:

- `commit/commit_preparation_tests.rs`, five controls from line 881:
  `scratch_step_look_below::the_controls_window` and then `native_commit_waiting_notice::the_controls_window`;
- `commit/native_commit_waiting_notice_tests.rs`, two controls at lines 709 and 757:
  the notice window and then the look window.

Each is a `std::sync::Mutex` returned by `take_the_window()`,
waited for with no bound and held to the end of the test.
One control holding the look window and another holding the notice window therefore hold each other for ever,
every later taker of either queues behind them,
and the process then has nothing able to run, which is what the watchdog judges a stuck test.

Why it needs the threads:
the two acquisitions are adjacent lines in both files,
so the cycle needs one control to be descheduled between them,
which one test to a process (the container gate, `cargo-nextest`) never produces.

## The reproduction

- Each of the five controls alone: passes, in 2 to 3 s
  (`a_scratch_commit_the_kernel_never_releases_is_ended_with_a_message` alone: `1 passed`, 2.98 s,
  its own line saying the step was `stopped after 1308 ms at the bound of 1200 ms`).
- The module of the two queued `process::` controls, threaded: `10 passed` in 3.03 s.
- The pair of commit modules, threaded: wedged twice, both times with the same five controls stuck.

`gdb -p` could not be attached on this host (`/proc/sys/kernel/yama/ptrace_scope` is 1,
`ptrace: Operation not permitted`), so the cycle is read from the two orders and the stuck set,
not from a backtrace naming the two threads.

## The change

The two controls in `native_commit_waiting_notice_tests.rs` now take the look window first, as every other control
that needs both does, and both windows' `take_the_window` docs state that order.

The same pair of modules after the change: `38 passed; 4 failed` in 17.5 s,
with no control stuck and no watchdog abort.

## What is still wrong in that run, and out of this change

Four controls of those two modules still fail threaded, and their own words name the reason:
`a process was left below the command`, `Some(250.172275ms)`, and a record whose
`ceiling: Some(60s)`, `armed: true` belongs to another control.
The statements those windows guard are read by the code under test, so a commit started by a control that takes
no window reads whatever another control planted:
the windows order the controls that plant, not the commits that look.

`take_the_window()` also still waits with no bound, so a wedge of one holder is a hang until the watchdog ends the
run, rather than a failure naming the window.
