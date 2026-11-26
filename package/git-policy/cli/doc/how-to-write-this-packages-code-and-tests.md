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
(`cargo test --locked --offline --lib --features test-only-switches <module> -- --skip guest --skip _arm_tests`, default threads, guest arms skipped)
and name the run's `test result` line.
The `test-only-switches` feature is what lets a control arm a `CLI_GIT_TEST_ONLY_*` variable,
which a build without it never reads (`foundation/test_only_switches.rs`);
`native:test:host` and the gate pass it,
and the one control that proves a release ignores the switches,
`testing::no_switch_is_read_without_its_feature`,
runs in a build without it.
A test that passes alone and fails threaded is a defect of that test until a measurement shows otherwise
(`AGENTS.md`'s SUF);
report its cause, never host load.

No round runs the whole library on this host.
Measured on 2026-11-25:
a whole-library threaded run hangs in `process::child_awaited_until_it_exits_the_stop_or_the_ceiling`
and fails three `process::signal_relay` tests on `main`'s own code,
so a round runs the modules it touched and the whole container gate stands in for a whole-library run.
A combined run's output can also carry a `panicked at` line from a thread named `git-policy-worker`
that is not a failure of the run:
the fault-limit controls start the wrapper or the service as a child process on purpose
(`binary/binary_fault_limits_tests.rs`),
and that child's runtime thread carries the name (`foundation/lazy_runtime.rs`).
Read the test's own result line before counting such a line as a failure.

## Nothing a test changes may be process-wide

Threads share the process, so a test never changes the current directory, the environment, the umask,
a signal's disposition or a resource limit of its own process.
Give them to the child it starts instead (`Command::current_dir`, `env`, `env_clear`, a `pre_exec` that only calls
async-signal-safe functions).
Issue 6454 removed the current-directory changes; keep them out.

The same holds for what a test reads:
`crate::run_progress::look` and `look_now` add up every task of this process,
so a control that judges the run's own figures runs its reading in a helper process of the test's own
(`testing/helper_process_test_support.rs`),
as issue 6478's descriptor-room control and the `run_progress` controls of issue 6486 do.

The service's review runner is one more such fact,
and a control hands the code its own instead of starting a child:
`review_item` and its three siblings run the review in `standards_review_run::RUNNER`,
which admits one job and answers every other caller with `Contained::Busy`,
so a threaded run of `service::service_review::tests` failed seven of its sixteen controls
while the gate starts each test in a process of its own and never showed it (issue 6499).
A control declares `static OWN: Containment`
and calls `review_item_in(&OWN, ..)`,
and `source_guard/no_control_takes_the_production_runner_tests.rs` refuses a test-only source that takes the
production runner instead.

A control that gives this process room for blocking jobs,
holds its bound on them,
or sets any other fact that belongs
to the whole process runs its body in a child of its own
(`testing/process_wide_state_child_test_support.rs`):
the room and the bound are made once for a process,
so two controls of a threaded run cannot both have theirs,
and a run that shares one process left eight of them deadlocked holding each other's places (issue 6487).
The setters refuse in any other process and name the starter,
as the descriptor limit's setter does.
The switch that keeps a control's durability syncs in the page cache is one of those facts:
it belongs to the process,
so a control that sets it runs its body in a child,
and without that a control that proves a sync was refused fails by which test ran first
(issue 6498,
`foundation/private_storage_syncs_test_support.rs`).
The preparation's named steps are one ledger of the whole process too
(`commit/prepare_work_spans.rs`):
every preparation notes into it and every timing record empties it,
so a control that counts them runs its body in a child of its own,
and a control that names the ledger without one fails
`source_guard/no_control_reads_the_preparation_steps_in_the_run_tests.rs`
(issue 6507).
A child makes its own fixtures;
nothing here counts this process's descriptors.
A threaded run moves those counters on other tests' threads,
and the comparison then fails on a figure the control never named.
The four share one comparison,
`the_figures_did_not_move` in `testing/run_progress_tests.rs`,
which refuses a look where a task that is not the control's own moved between two looks
and says which helper runs the body alone.

A control that holds a fault point of the service
(`service_fault_hooks_test_support.rs`)
names the instance it owns in the point itself:
the directory whose instance is read,
the plan the start reads,
the request whose beat reads its note,
or the repository a note is written about
(`instance_read_point`,
`note_read_point`,
`read_at_a_start_point`,
`next_service_note_point`).
The holds are a list of the whole process,
so one slot for a run held nothing of a second control,
and the module of three heartbeat controls failed all three (issue 6497);
the registry refuses a second live hold of one point and names the composition the control must make,
so a control that names one point twice fails at its own first line instead of in another control's wait.

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
A child a control starts for its own body is handed that body, and the parent never calls it:
`run_the_body_in_a_child_of_its_own`,
`run_the_body_where_no_thread_can_start` and `run_the_body_under_umask` take the test's `module_path!()`,
its name and a closure,
so no caller holds an answer it could read inverted.
While those starters answered a yes or a no,
two controls read the answer inverted and ran the bodies written for a process at its limit beside the run's own tests:
the run's thread limit came down for every test (issue 6494),
and a lazy runtime's body built the runtime the whole run shares (issue 6503).
The setters still refuse outside the child and name the starter,
so a body that reached the wrong process fails at its own first line,
which is what the two refusals in `testing/no_threads_child_test_support.rs` and
`subvolume/subvolume_file_write_steps_sync_counts_test_support.rs` are for.
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
`source_guard/lists_are_slices_tests.rs` fails on a `const` whose element type names a text reference
and whose length is a number,
whatever layout the declaration uses,
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
