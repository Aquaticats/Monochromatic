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
`push/start_retry_tests.rs`'s `a_failed_exec_returns_its_error_at_once`).

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
