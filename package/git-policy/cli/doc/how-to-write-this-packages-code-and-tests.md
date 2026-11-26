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
`fixture(name)` names the folder `native-unit-<process id>-<the number this call took>-<name>`,
so every call gets a folder of its own:
before the number was there, two tests that passed one name shared one folder in one process,
and each deleted the other's live folder:
`lock::index_writer_forward`'s two signal tests did,
failing with `DirectoryNotEmpty` and with a `create_dir` panic at `test_support.rs:332`.
The binary test target's own helper names its folders the same way
(`native-binary-<process id>-<the number this call took>-<name>`,
`binary/binary_support_fixture_directories.rs`).
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

A child that is this test binary running one named test goes through `testing/child_test_process.rs`
and nowhere else: `ChildTest` is the request (`started` hands back the running child,
`finished` waits with a deadline and reads both pipes once the child is gone,
`passed` adds the check that exactly one test ran and passed),
and `testing/test_process_owner.rs` is the ownership it stands on.
Six bodies built that command their own way until 2026-11-25,
and most of their children were owned by nothing (debt ledger item 10).
A child whose ending is the assertion, such as one that dies by a signal, takes `finished`
and judges the ending itself; a child a control keeps and talks to takes `started`,
and its own waits carry the bounds.

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
A list of names, sites or words is a slice, `const NAMES: &[&str] = &[...]`, never `[&str; 47]`:
a hand-counted length is a magic number every added entry has to edit
(found 2026-11-25 in the raw-read guard's list, beside 277 such arrays crate-wide; ledger item 19).
An array keeps a fixed length only where the length is the meaning, such as a header of a fixed byte width.

## Show that a guard works

A guard counts once it is shown failing without its code:
commit the control, take the guarded line out, run, restore (`AGENTS.md`'s GFP).
A green run counts only when its output names the expected tests (TRV).

The two rules above are enforced, not only written down:
`source_guard/test_children_owned_tests.rs` fails on a test file that starts a child
which neither `spawn_owned` nor `own_group` owns,
and on any signal whose target is `getpid()`, `getpgrp()`, `std::process::id()` or `0`.
The sites the tree still holds are one line each in
`source_guard/test_children_not_yet_owned.txt`, with why each is not moved yet,
so moving a site means deleting its line there,
and a new site fails until it is moved or entered with its reason.

## A control that reads another module's source follows that file

Some controls read another module's file with `include_str!` and assert on its text,
and then that file's own module tests are not the ones a change to it breaks:
`foundation/notice_stream_that_does_not_hold_its_writer_tests.rs` reads
`landing/service_commit_waiting_notices.rs`,
and `service/wait_ceilings_tests.rs` and `source_guard/hook_self_invocation_tests.rs` read
`landing/service_commit.rs`.
Before a merge, name every file the change touches and run the module of each test that reads it
(`grep -rn 'include_str!' src/native` lists them),
beside the module of the file you changed.
The fix for issue 6421 (`700a6e4c02`) rewrote the line that starts the commit's notice stream
and left `the_commit_command_hands_its_waiting_notices_to_this_stream` asserting the line it replaced,
so the whole container gate on `main` at `132843d9d1` failed in a module that fix never ran.
