# A shadow test that writes a program and starts it is refused with ETXTBSY

`package/mpm-shadow/cli` and `package/search-shadow/cli` run most of their tests as threads of one process
under `cargo test`,
while their own tasks (`mise run //package/mpm-shadow/cli:test`,
`mise run //package/search-shadow/cli:test`)
run `cargo nextest run --release`,
which starts one process per test.
A test that writes a program file and then starts it can meet
`Text file busy (os error 26, ExecutableFileBusy)` in the threaded shape and never in nextest's.
This is issue 6527,
measured on 2026-11-27;
issue 6517 is the same defect in `package/git-policy/cli`
(`doc/troubleshooting/git-policy-cli-a-fixture-git-is-refused-with-etxtbsy.md`).

## Symptom

The issue reports three sightings on 2026-11-26,
each of one test,
each passing when run again:

- `package/mpm-shadow/cli`'s `tests/launcher.rs`:
  `a_signal_that_kills_the_real_program_is_the_launchers_own_status`.
- one run of `package/mpm-shadow/cli`'s `tests/printed_remedy.rs`.
- one test of the `shadow_identity` module in `package/search-shadow/cli`.

Every one of them writes a program and starts it:
`tests/launcher.rs` writes a shell script that stands in for the real tool and copies the built binary
as the launcher,
and the `shadow_identity` tests copy `/usr/bin/echo` or write a script and start it.

## Root cause

`execve` answers `ETXTBSY` for a file that **any** process of the machine holds open for writing,
and a `fork` copies the whole descriptor table of the process that forks.
So a descriptor this process opened for writing on a program
is inherited by every child another thread forks while it is open,
and that child keeps it until its own start closes the close-on-exec descriptors.
That child,
not the test,
is the holder the refusal names.

Measured on 2026-11-27,
three probes the folder beside this record holds
(`doc/troubleshooting/measurements-of-etxtbsy-in-a-test-that-starts-a-program-it-wrote/`):

- Three arms over one program,
  each writing a program and then starting it (`probe-three-arms.c`):

  ```text
  arm 1 (a forked child that starts nothing holds it): start answered: Text file busy (errno 26)
  start failed with status 26
  arm 2 (this process alone holds it): start answered: Text file busy (errno 26)
  start failed with status 26
  arm 3 (nothing holds it): start succeeded
  ```

  Arm 1 is the shape the tests meet:
  the parent only forked,
  and the child held the descriptor.
- A threaded program (`probe-two-threads.c`) whose main thread opens a program for writing
  while a second thread starts `/bin/true` through the same `posix_spawn` a Rust `Command` uses,
  and whose next child starts the program just after the close:

  ```text
  attempts=20 started=16 refused-with-ETXTBSY=4
  attempts=20 started=19 refused-with-ETXTBSY=1
  ```

  Two runs of it,
  so one attempt in five and one in twenty were refused:
  a threaded run meets this rarely,
  which is why the issue saw a different test fail once in a while.

## Why the window is where it is

The failure needs a `fork` in another thread between the write and the start,
and the child's hold lasts from that `fork` to its own `execve` (about 1 to 3 ms),
so the chance is the fork rate multiplied by that hold,
not by the length of the write window.

Two shapes were tried against the real test binary to make it common,
and both are recorded because neither does:

- One run holding the write descriptor open 100 ms before the close,
  and holding every child at its own start with
  `strace -f -ttt -e trace=clone3,execve --inject=execve:delay_enter=200000`:
  19 of 19 tests passed.
  The trace showed 44 writes of program files,
  and 12 other threads' forks inside those windows,
  but only one fork in the last 3 ms of its window,
  which is the only part that can reach the start.
- Delaying every child's `execve` entry by 200 ms with
  `strace --inject=execve:delay_enter=200000`:
  The threaded probe above went from 4 refusals in 20 attempts to 0 in the same run shape,
  because the same injection also delays the start that should be refused,
  by which time the holder has finished its own start.

A scan of every process's descriptors at the end of a 100 ms window found only the test process's own
descriptor,
for the same reason issue 6517 gives:
the holder lives from a child's fork to its start.

## Verified fix

Every test of the two crates that writes a program it starts now writes it through a child process,
so no descriptor of the test process ever refers to that program:

- `package/mpm-shadow/cli/src/toolchain/fixture_programs.rs` writes through a child `tee`
  and copies through a child `cp --reflink=never`,
  and the unit tests reach it through `fakes::write_executable`,
  `launcher_tests`' fixtures and
  `repository_link_cause_tests`' check program.
- `package/mpm-shadow/cli/tests/written_programs/through_a_child_process.rs` is the same two functions
  for the binary tests,
  included by each test binary.
- `package/search-shadow/cli/tests/written_programs/through_a_child_process.rs` is the same for that crate.
- `src/shadow_identity_tests.rs` carries its own copy,
  because the four shadow crates keep that file byte-identical.

The measurement that shows each change is a preloaded library
(`planted-holder-and-window-shim.c`,
beside this record) that plants the holder the real failure needs:
when a threaded process is about to close a descriptor open for writing on a file that starts with `#!` or
the ELF magic,
it forks a child that holds everything it inherits for 300 ms and then leaves.
The test process holds no such descriptor after the change,
so nothing is planted and the same runs pass.
It is built with `gcc -shared -fPIC -O2 -o libetxtbsy_window.so planted-holder-and-window-shim.c -ldl`
and read by the test binary through `LD_PRELOAD`,
`ETXTBSY_SHIM_HOLD_US=300000` and `ETXTBSY_SHIM_PREFIX=<the build directory of the crate>`.
Each binary was run once with default threads before and once after,
on the same build profile and the same machine:

```text
                       before                              after
mpm-shadow lib         689 passed, 10 failed               699 passed, 0 failed
launcher               2 passed, 17 failed (19 tests)      19 passed, 0 failed
never_start_another…   1 passed, 9 failed                  10 passed, 0 failed
old_copies             2 passed, 1 failed                  3 passed, 0 failed
printed_remedy         0 passed, 3 failed                  3 passed, 0 failed
shim                   6 passed, 15 failed                 21 passed, 0 failed
search-shadow lib      82 passed, 3 failed                 85 passed, 0 failed
announce               4 passed, 6 failed                  10 passed, 0 failed
never_start_another…   2 passed, 7 failed                  9 passed, 0 failed
shim                   4 passed, 9 failed                  13 passed, 0 failed
```

The failures were all `Os { code: 26, kind: ExecutableFileBusy, message: "Text file busy" }`,
raised where the test starts the program.

`tests/copied_checkout_record.rs` (1 test) and `tests/toolchain.rs` (24 tests) wrote no program they start and
passed in the shape before the change,
so nothing in them changed.
`tests/shadow_identity_copies.rs` compares files and starts nothing,
so the red shape was not run against it;
it passes in that crate's own suite and in a threaded `cargo test` of its binary.

## What a future test of either crate must do

A test that starts a program it wrote goes through `fixture_programs::write_program`,
`fixture_programs::copy_program`,
or the `tests/written_programs/through_a_child_process.rs` copy of them.
A new test binary includes that file with

```rust
// tests/<binary>.rs
#[path = "written_programs/through_a_child_process.rs"]
mod through_a_child_process;
```

The other shape issue 6517 uses,
running the whole body in a child of its own
(`run_the_body_in_a_child_of_its_own` in `package/git-policy/cli`),
was not needed here:
none of these fixtures depends on sharing a process with the test.

## What this record rests on

- The mechanism:
  the three probes above,
  each run on this host,
  and their printed lines quoted here.
- The exposure of each test:
  the planted-holder run of each test binary,
  before and after.
- The green runs:
  the same shape on the changed tree.

This record is unapproved and non-final.
What was done is recorded here and in the commits;
what is open is the owner's review,
and no release was published.
