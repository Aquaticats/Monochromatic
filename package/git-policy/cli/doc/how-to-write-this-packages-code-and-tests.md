# How to write this package's code and tests

Every agent round that writes code or tests in `package/git-policy/cli` reads this file whole before its first edit.
It exists because rounds were briefed on what to build and how to prove it,
but not on how to write the code,
and the debt that left was measured on 2026-11-25:
two tests shared one fixture folder,
a test's stand-in child outlived its test and held the run's output open for 45 minutes,
and a test's signal reached its own test binary.
The owner, that day: "So pay the debt."

This guide adds to the repository's skill for writing code in a language other than TypeScript,
`.agents/skills/dum-dum-non-ts/SKILL.md`,
which sets the code's shape (the simplest constructs)
and its comments (the What, Why and Gotcha template the crate uses);
load that skill before your first Rust edit, then write by both.
What follows is what that skill does not cover: how this crate's tests and children behave.

## Tests hold as threads of one process

`cargo test --lib` runs a module's tests as threads of one process, and that is how rounds run them on this host.
The container gate runs each test in a process of its own,
so a test that only holds alone passes the gate and fails every threaded run.
Before you report, run each module you touched threaded
(`cargo test --locked --offline --lib <module> -- --skip guest --skip _arm_tests`, default threads, guest arms skipped)
and name the run's `test result` line.
A test that passes alone and fails threaded is a defect of that test until a measurement shows otherwise
(`AGENTS.md`'s SUF);
report its cause, never host load.

## Nothing a test changes may be process-wide

Threads share the process, so a test never changes the current directory, the environment, the umask,
a signal's disposition or a resource limit of its own process.
Give them to the child it starts instead (`Command::current_dir`, `env`, `env_clear`, a `pre_exec` that only calls
async-signal-safe functions).
Issue 6454 removed the current-directory changes; keep them out.

## One fixture folder per test

A test takes its folder from `testing/test_support.rs`.
`fixture(name)` names the folder `native-unit-<process id>-<name>` and removes a folder of that name first,
so two tests that pass the same name share one folder in one process,
and each deletes the other's live folder:
`lock::index_writer_forward`'s two signal tests did,
failing with `DirectoryNotEmpty` and with a `create_dir` panic at `test_support.rs:332`.
Give every test a name no other test uses until the helper makes each call's folder its own.
Never write a fixed path outside the fixture root (no `/tmp`, no shared name under the build folder).

## Own every child, end it on every path

A child a test starts is the test's to end, on success, on failure and on a panic:
start it with `test_support::spawn_owned`, whose child is ended with the test thread's fixtures however the test ends,
so a failing assertion cannot leave it running.
A control that needs a signal sent to the wrapper runs the wrapper in such a helper process
and signals only `helper.id()`,
as `process/signal_relay_tests.rs` does:
`lfs::lfs_upload`'s signal control once sent `libc::kill(libc::getpid(), libc::SIGTERM)`,
and `getpid()` is the test binary, whose signal handlers are process-wide,
so it ended every thread of the run (issue 6463).
Start a stand-in in a process group of its own, and signal only the process or group the test started and proved,
never a group or session the test binary belongs to, never by name.
A child must not keep the test's standard output or error open after the test:
close or redirect the streams it does not need.
`lfs::lfs_upload`'s `git-lfs` stand-in, which writes its process number and `exec`s `/usr/bin/sleep {HOLD_SECONDS}`,
is the shape that leaked.

## Every wait has a deadline

A test waits for a condition with a bound and fails at the bound with what it waited for,
never by sleeping for a guess and never without an end.
A bound in an assertion holds under this host's load,
because other sessions build on it all the time (issue 6405 is a bound that did not).
Code under test bounds its waits too: every read of a file a person or another program controls opens it so it cannot
block (not a named pipe, not a device; the type checked through the open handle, never by a path check before the open)
and has a size bound (issues 6415 and 6433).
No blocking work runs on the commit's own thread or on the async runtime (`source_guard`'s runtime guard; issue 6421).

## Write less code

Reuse what `testing/test_support.rs` and the package's modules already have before writing a helper
(`AGENTS.md`'s UTL), and move a helper you need twice into the shared module instead of copying it.
Return no more than the caller reads (issue 6412).
Delete code nothing calls, with the text that describes it (`AGENTS.md`'s XSU; issue 6430).
A file over the linter's `rust/max-lines` splits into sibling modules by role, never by stripping docs (MXL).
Name files, folders and functions by what they do (MKY, VGW), derive values from named constants (MGN),
and take no lock where atomic rename, `O_EXCL` or one owner per item will do (NLK).

## Show that a guard works

A guard counts once it is shown failing without its code:
commit the control, take the guarded line out, run, restore (`AGENTS.md`'s GFP).
A green run counts only when its output names the expected tests (TRV).
