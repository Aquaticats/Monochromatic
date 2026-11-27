# Rust's `exec` leaves `SIGPIPE` at its default in the calling process

A Rust program ignores `SIGPIPE` from its start,
so a write to a pipe nobody reads answers `EPIPE` and the program goes on.
`std::os::unix::process::CommandExt::exec` breaks that in the process that calls it,
and a refused `exec` leaves the breakage behind:
the next write to a broken pipe,
on any thread,
ends the program by `SIGPIPE`.

Measured on 2026-11-27 for issue 6604,
on a threaded `cargo test --lib` run of `package/git-policy/cli`.

## What the standard library does

Two places in `std` set `SIGPIPE`,
and they are not symmetric.

- The program's start sets it to the ignore
  (`library/std/src/sys/pal/unix/mod.rs:181`,
  `reset_sigpipe`,
  reached from `library/std/src/rt.rs`'s `init`).
  The comment above it says the runtime "sets the `SIGPIPE` handler to `SIG_IGN`" since 2014,
  and that `-Zon-broken-pipe` selects another action.
- Every `exec` sets it back to the default
  (`library/std/src/sys/process/unix/unix.rs:374`,
  `do_exec`):
  "If `-Zon-broken-pipe` is not used,
  reset `SIGPIPE` to `SIG_DFL` for backward compatibility."
  It does so before the `pre_exec` closures run and before `execve`.

`Command::spawn` calls `do_exec` in the forked child,
so the reset reaches the child and the parent keeps its ignore.
`CommandExt::exec` calls `do_exec` in the **calling process**,
because the calling process is what
`execve` replaces.
An `execve` that fails returns from `do_exec` with an error and replaces nothing:
the caller is left with `SIGPIPE` at the default.

## How it showed

A probe interposed `sigaction` and `signal` with `LD_PRELOAD` for a threaded run of
`invocation::forwarding::tests` and `push::start_retry::tests`
(the shim logged the caller's address,
which `addr2line` mapped into `std`):

```text
disposition: signal signal=13 handler=0x1 pid=475640 tid=475640 offset=0x6f38216  # std's start: SIG_IGN
disposition: signal signal=13 handler=0x0 pid=475640 tid=475772 offset=0x6f1e00c  # do_exec, libtest thread
disposition: signal signal=13 handler=0x0 pid=475640 tid=475778 offset=0x6f1e00c  # do_exec, libtest thread
```

`offset=0x6f1e00c` is
`<std::sys::process::unix::common::Command>::do_exec` at
`library/std/src/sys/process/unix/unix.rs:374`;
`offset=0x6f38216` is `std::sys::pal::unix::init::reset_sigpipe`.
The same pid on every line is the test binary:
both resets happened in the test process,
on the two libtest threads whose tests ran a start that could not run its program
(`invocation/forwarding_tests.rs`'s `unstartable_program_is_an_error` and
`push/start_retry_tests.rs`'s `a_failed_exec_returns_its_error_at_once`,
which is `a_failed_exec_returns_its_error_and_puts_the_signal_action_back` since 2026-11-27).

`grep 'signal=13 '` over that log after the two lines found nothing that set the ignore back,
so the whole run stayed one broken-pipe write away from death.

The run the issue reports is the same shape:
`cargo test --lib --features test-only-switches worktree landing push repair commit transaction command service invocation policy`
ended with `signal: 13, SIGPIPE: write on a pipe with no one to read`
after 2,985 of 7,764 tests,
with two `landing::` tests the only ones that had started and not answered.

## What a round does about it

A body that runs such a start runs in a child of its own
(`testing/process_wide_state_child_test_support.rs`),
which is what both controls do now,
`source_guard/no_control_leaves_sigpipe_at_its_default_tests.rs` refuses a control that does not,
and `doc/how-to-write-this-packages-code-and-tests.md`'s section on process-wide state states the rule.

The same rule covers a body that ends the process by a signal on purpose:
`forwarding::end_as_git_ended` sets the default action and raises the signal,
and `landing/service_commit_input_notes_and_embedded_commit.rs`'s `print_summary` does it for `SIGPIPE`
when the commit's summary cannot be written.

A probe of the shape above is worth keeping in mind for the next death by a signal nobody set:
`strace` without `-f` does not trace the threads a process starts,
so it shows a process's own `sigaction` calls and misses every libtest thread's.
An `LD_PRELOAD` shim that logs the caller's address does not.

## What the release does after a failed `exec`

Issue 6607 asked what the wrapper does after a start of the real Git that could not run its program,
and the record decides it:
the wrapper puts the action back and goes on.

`SPEC.md`'s "Stream and exit contract" says what a command means where real Git does not run:
"`2` means usage, config, patch, transaction, or engine failure.",
and `README.md`'s "Output and exit status" says the same for "When real Git does not run".
A start that failed ran no Git, so the wrapper's own words and status are what the caller must read.
Ending by `SIGPIPE` has a place of its own in that record,
for a Git that ran and was ended by a signal:
"So `git log -p | head -1` ends by `SIGPIPE` on this route as it does on native Git".
That death is relayed by `forwarding::end_as_git_ended`,
and a failed start is not it.

Every path that continues after a failed `exec`,
and what it writes:

- `push/start_retry.rs`'s `ExecStart::start` to `retry_refused_start`:
  each refusal that is `EAGAIN` hands one `[warn]` line to the thread that writes standard error,
  pauses,
  and tries the replacement again.
- `invocation/entry.rs`'s `Action::Forward` arm:
  `cannot_start` writes `monochromatic-git-policy-cli: cannot start <git>: <error>. Hint: ...` to standard error,
  and the arm returns `ENGINE_FAILURE_EXIT_CODE`, which is `2`.
- `invocation/entry_run_forwarded_actions.rs`'s `run_coordinated_action`,
  in its `CoordinatedRun::NotCoordinated` arm:
  one `monochromatic-git-policy-cli: cannot start <git>: <error>` line to standard error and status `2`,
  and, for a command that adds commits, one push-trigger line in the repository's trigger log file,
  (`push/push_trigger_log.rs`'s `record_decision`),
  with a `[debug]` line when verbose.
- `main.rs`'s `end_after_the_log`, which every invocation ends through:
  the lines the diagnostic writer thread still holds go to standard error before the process exits with the status.

`exec_keeping_the_signal_action` (`push/start_retry.rs`) is the one function that replaces this process,
and it puts the action back after every failed `exec`, before anything is written.
So the writes above answer `EPIPE` where nobody reads them, and the status stands.
A start that succeeds never returns,
so the program that replaces the wrapper still begins with the kernel's default for `SIGPIPE`,
exactly as native Git does.

The need is stated beside the call.
`source_guard/every_exec_puts_the_signal_action_back_tests.rs` fails a release file that does not put the action back.
The behavioural controls are named in `SPEC.md`'s "Stream and exit contract".

Measured 2026-11-27 on the test host, for issue 6607:
before the change, the parent of a child that took this path,
with a program that cannot be started and standard error a pipe whose reader was gone,
read `signal: 13 (SIGPIPE)`;
after it,
the same parent read `exit status: 2`.
The control is `invocation::forwarding::tests::a_wrapper_that_could_not_start_git_ends_by_the_status_it_means`.
