# How to write this package's code and tests

Every agent round that writes code or tests in `package/git-policy/cli` reads this file whole before its first edit.
It exists because rounds were briefed on what to build and how to prove it,
but not on how to write the code,
and the debt that left was measured on 2026-11-25:
two tests shared one fixture folder,
a test's stand-in child outlived its test and held the run's output open for 45 minutes,
and a test's signal reached its own test binary.
The owner,
that day:
"So pay the debt."

This guide adds to the repository's skill for writing code in a language other than TypeScript,
`.agents/skills/dum-dum-non-ts/SKILL.md`,
which sets the code's shape (the simplest constructs)
and its comments (the What,
Why and Gotcha template the crate uses);
load that skill before your first Rust edit,
then write by both.
What follows is what that skill does not cover:
how this crate's tests and children behave.

## Tests hold as threads of one process

`cargo test --lib` runs a module's tests as threads of one process,
and that is how rounds run them on this host.
The container gate runs each test in a process of its own,
so a test that only holds alone passes the gate and fails every threaded run.
Before you report,
run each module you touched threaded
(`cargo test --locked --offline --lib --features test-only-switches <module> -- --skip guest --skip _arm_tests`,
default threads,
guest arms skipped)
and name the run's `test result` line.
The `test-only-switches` feature is what lets a control arm a `CLI_GIT_TEST_ONLY_*` variable,
which a build without it never reads (`foundation/test_only_switches.rs`);
`native:test:host` and the gate pass it,
and the one control that proves a release ignores the switches,
`testing::no_switch_is_read_without_its_feature`,
runs in a build without it.
A test that passes alone and fails threaded is a defect of that test until a measurement shows otherwise
(`AGENTS.md`'s SUF);
report its cause,
never host load.

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

Threads share the process,
so a test never changes the current directory,
the environment,
the umask,
a signal's disposition or a resource limit of its own process.
Give them to the child it starts instead (`Command::current_dir`,
`env`,
`env_clear`,
a `pre_exec` that only calls
async-signal-safe functions).
Issue 6454 removed the current-directory changes;
keep them out.

A signal's disposition is one of those facts,
and the standard library hands the test process its first one:
it sets `SIGPIPE` to its ignore at the start of every Rust program
(`library/std/src/sys/pal/unix/mod.rs:181`),
so a write to a pipe nobody reads only answers `EPIPE`.
A start that cannot run its program puts that default back in the process that tried it:
`Command::exec` runs the standard library's `do_exec` in the caller,
not in a child
(`library/std/src/sys/process/unix/unix.rs:374`),
so `forwarding::replace_process_with_real_git` and `push::start_retry::exec_retrying`
leave `SIGPIPE` at its default when the program is missing or the start is refused,
and every later write to a broken pipe ends every thread of the run
(issue 6604:
a threaded run of ten library modules ended with `signal: 13` after 2,985 of its 7,764 tests,
and the two controls that ran such a start were
`invocation/forwarding_tests.rs`'s `unstartable_program_is_an_error`
and `push/start_retry_tests.rs`'s `a_failed_exec_returns_its_error_at_once`,
which is `a_failed_exec_returns_its_error_and_puts_the_signal_action_back` since 2026-11-27;
the measurement is in
`doc/troubleshooting/rust-exec-leaves-sigpipe-at-its-default-in-the-calling-process.md`).
So a control that runs one of those starts,
or a body that ends the process by a signal
(`forwarding::end_as_git_ended`,
`landing/service_commit_input_notes_and_embedded_commit.rs`'s `print_summary`,
`process/stop_of_the_service_noticed_by_its_children.rs`'s `end_by_the_signal`),
runs its body in a child of its own (`testing/process_wide_state_child_test_support.rs`),
reads the action in place before and after with `sigaction` and a null new action
(`testing/signal_disposition_of_the_process.rs`),
and `source_guard/no_control_leaves_sigpipe_at_its_default_tests.rs` refuses a control that does not.
A control that only reads an action changes nothing,
so it may run in the run's own process.

The same holds for what a test reads:
`crate::run_progress::look` and `look_now` add up every task of this process,
so a control that judges the run's own figures runs its reading in a helper process of the test's own
(`testing/helper_process_test_support.rs`),
as issue 6478's descriptor-room control and the `run_progress` controls of issue 6486 do.
The pack decode count is one of them too:
`DECODED_TOTAL` (`repo_read/repo_read_pack.rs`) adds up every entry every pack of this process decoded,
so a control that counts the decodes of its own answer runs its body in a child of its own,
as `repo_read::repo_read_tree_cached::tests::the_directories_the_cache_tree_proves_equal_are_not_read`
and `repo_read::repo_read_query::tests::the_changes_of_a_commit_are_found_without_reading_the_unchanged_directories` do
(issue 6539:
in a threaded run of `repo_read::` the cache tree control counted another test's decodes).
So does a program a control starts that reads standard input:
the run's standard input is the terminal the run was started from,
and a control that starts such a program runs its body in a child of its own,
whose standard input is the null device
(issue 6510's `git add --interactive`,
which printed its `What now>` menu and never ended).

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
A control that reviews through the service's real handlers reaches the same runner without naming any of those calls:
`GitHandlers::review` and `GitHandlers::review_awaiting` run their job in it,
so a control that points those handlers' review at a mock gateway hands them a runner of their own,
`with_review_runner(&OWN)`,
and the same guard refuses a control that does not
(issue 6500:
two such controls of `landing::service_work`,
in one process,
answered `1 passed; 1 failed` in each of three runs,
and `2 passed; 0 failed` after the change).

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
The environment is one of those facts,
and a control that changes what it names runs its body through
`run_the_body_with_an_environment_of_its_own`
in the same file,
which adds the variables the control states and removes the ones it names.
The route a hand-over finds the commit service by reads two facts of the session,
and a test build reads them from one place only,
`foundation/session_runtime_directory_and_bus_address.rs`:
an inherited `XDG_RUNTIME_DIR` or `DBUS_SESSION_BUS_ADDRESS` is not read at all,
and the control hands its own in through
`CLI_GIT_TEST_ONLY_SESSION_RUNTIME_DIRECTORY` and `CLI_GIT_TEST_ONLY_SESSION_BUS_ADDRESS`,
which that file reads in place of the inherited pair.
So a body whose control hands in nothing has no route,
and `source_guard/no_release_read_of_the_session_places_tests.rs` refuses any other read of either name.
A build made for tests also offers the manager no program of its own:
the executable a process remembers is not compiled there,
because in a test process it is the test binary
(issue 6586:
before that,
a control that handed a trash over wrote its fixture's units into `/run/user/<n>/systemd/user/`,
pointed the account-wide program link at the test binary,
and had the manager run libtest for the fixture five times until the start limit hit,
which is what issue 6582's two controls first met).
So a control whose body routes a hand-over hands in a fixture runtime directory and no bus,
as `worktree_copying`'s two hand-over controls do,
and the hand-over fails with "no commit service can be started from this session",
which is the world those controls are about.
The merge that added the two switches changes a real run in three edge cases:
a runtime directory whose name is not UTF-8 is used where it was ignored,
an empty bus address is passed on and treated as before,
and an empty `XDG_RUNTIME_DIR` on the ZFS add route is refused as not set.
A control whose subject is a route with a program in it passes one to `choose_route` itself,
and `landing/service_route_tests.rs` holds both kinds.
A child also inherits the records its parent holds,
so a control whose body reads the chain record closes the record it inherited before it makes its own
(`invocation/started_as_git_tests.rs`,
issue 6514):
the lookup answers with the first descriptor of that name in `/proc/self/fd`,
and a process holding two read the test process's record rather than the one the control wrote.
The switch that keeps a control's durability syncs in the page cache is one of those facts:
it belongs to the process,
so a control that sets it runs its body in a child,
and without that a control that proves a sync was refused fails by which test ran first
(issue 6498,
`foundation/private_storage_syncs_test_support.rs`).
A stand-in for a runtime that refused every thread,
or for a build that outlasted its deadline,
is one too
(`foundation/lazy_runtime.rs`):
`tasks_can_run` and `built_flavor` answer for the whole process,
so a control that arms one runs its body in a child,
and both setters refuse outside it (issue 6511).
A descriptor a control holds open is one of those facts as well,
because a fork copies the whole descriptor table,
and an advisory kernel lock lasts until every copy of its open file description is closed:
a child another test's thread starts between the control's `open` and its `close` keeps the control's `flock`
after the control dropped its own descriptor,
so the older-release conversion control runs its body in a child of its own
(`worktree/worktree_convert_ownership_tests.rs`,
issue 6536).
Measured 2026-11-26 by planting that fork in the control:
the forked child held a copy of the control's descriptor naming `cli-git-convert.lock`,
`/proc/locks` still named the lock once the control closed its own descriptor,
and the second call skipped with "a command of release 0.5.20 or older holds" it.
The window is only the fork to the child's exec,
where the standard library's close-on-exec flag closes the copy,
and a host under load stretches it.
The journal's lock of the fold of an earlier log is one more of those,
and its copy outlives the fork:
`foundation/earlier_log_lines_added_to_current_log.rs` takes an `flock` on the journal for one fold and drops it as the
fold answers,
and a child forked while it was held keeps it without an `exec` to close anything,
so a control that folds one log twice met `LogFold::Busy` where it asserts `LogFold::Folded`.
Measured 2026-11-27 by planting that fork in the control of a log that keeps growing:
`/proc/locks` still named `FLOCK ADVISORY WRITE` for the journal's inode once the control closed its own descriptor,
the next fold answered `left: Busy` against `right: Folded`,
and the same fold answered `Folded` once the planted process was ended.
So every control of that file that folds one log twice runs its body in a child of its own,
and `source_guard/no_control_folds_a_log_twice_in_the_run_tests.rs` refuses one that does not
(issue 6602).
The preparation's named steps are one ledger of the whole process too
(`commit/prepare_work_spans.rs`):
every preparation notes into it and every timing record empties it,
so a control that counts them runs its body in a child of its own,
and a control that names the ledger without one fails
`source_guard/no_control_reads_the_preparation_steps_in_the_run_tests.rs`
(issue 6507).
The commit's four figure ledgers are the same shape
(`commit/commit_index_builds.rs`,
`commit/commit_prepare_part_durations.rs`,
`commit/commit_build_part_durations.rs` and
`transaction/transaction_step_durations.rs`,
which `step_time_between` reads while a stretch is open),
so a control that counts its own command's figures runs its body in a child of its own,
and one that names any of them without one fails
`source_guard/no_control_reads_the_commit_ledgers_in_the_run_tests.rs`
(issue 6509).
The add timing record is one list of the whole process as well
(`worktree/add_timing_steps.rs`:
`begin` replaces its empty list and `finish` takes it),
and beginning a recording also publishes the step the add's standing notice names,
so a control that begins one runs its body in a child of its own
(`worktree/add_timing_tests.rs`),
and a control that ran its body beside another's recording read a line holding none of its own work
(issue 6532).
No `source_guard` ban covers that record yet:
the scan such a ban is built on reads every test-only source,
and seven controls of four other files still begin a recording in the run's own process
(`worktree/add_standing_notice_tests.rs` is issue 6533's,
`worktree/worktree_clone_tests.rs`,
`worktree/worktree_ladder_finish_tests.rs` and
`worktree/worktree_ladder_target_tree_skip_tests.rs`).
The two windows a notice reads are one window for the process as well,
so a control that steps a notice takes them before it starts
(`take_the_windows_a_notice_control_reads` in `commit/native_commit_waiting_notice_tests.rs`),
and a control that names `step_started(` without a window of its own fails
`source_guard/notice_controls_take_their_window_tests.rs`
(issue 6509).
A control that asserts this process has no descendant left owns a child for the same reason:
every other test's child is a descendant of the same process.
A child makes its own fixtures;
nothing here counts this process's descriptors.
A threaded run moves those counters on other tests' threads,
and the comparison then fails on a figure the control never named
(issues 6511 and 6545).
The four share one comparison,
`the_figures_did_not_move` in `testing/run_progress_tests.rs`,
which refuses a look where a task that is not the control's own moved between two looks
and says which helper runs the body alone.
The thread count of the process is one of those figures,
so `renewals_running_at_once_start_no_thread_each` reads it in a child of its own
(`testing/process_wide_state_child_test_support.rs`).
And the same rule covers a count of the tasks a control started:
the renewal makes the count of its one task's ending and hands the control a share of it
(`service_claim_renewal_running.rs`,
`ClaimRenewal::endings`),
so a control reads the endings of the task it made and never a count the whole test process shares
(issue 6526).
And a control that asserts how many workers a run's budget allows names the host that budget is derived from,
as `A_HOST_THAT_BOUNDS_NO_WORKER` in `worktree/worktree_clone_slices.rs` does,
because `HostResources::read` answers the live machine:
the threads it reports are the smaller of the affinity mask and the control group's CPU quota,
so a run in a scope capped below two processors answers one worker,
and a control that read it asserted on the cap the run happened to carry rather than on its fixture
(issue 6537;
the quota's effect on `available_parallelism` is measured in
`doc/troubleshooting/container-nproc-ignores-cpu-quota.md`).
A round's widths are that same answer,
and its memory figure is the one a control trips over:
`HostResources::read` takes the free bytes of this process's control group,
and the page cache a build wrote inside that group is charged to it,
so a run in a capped scope reads no headroom at all and every batch runs one item at a time
(measured for issue 6567:
a threaded `landing::` run whose build filled a 10 GB cap read `memory_available: Some(0)`,
and both side-by-side controls of `landing/service_work_items_side_by_side_tests.rs` then saw one item inside the
handler;
the same run with a warm build read 23.5 GB free and passed,
`doc/troubleshooting/cgroup-free-memory-counts-the-page-cache-a-build-wrote.md`).
So the batches are sized by a host the caller states:
`drain_pending_awaiting` and `drain_the_slow_work_awaiting` take where the host comes from
(`TheHostOfABatch`,
`foundation/parallel_budget.rs`),
the shipped round and the slow lane name the machine itself,
so each batch reads it where the batch is sized and not before the round's earlier work ran,
and a control that needs two items beside each other states a host of its own
(`TheHostOfABatch::Stated(A_HOST_THAT_BOUNDS_NO_BATCH)`),
as a control whose subject is a batch that keeps its turn states one that puts a single item at a time
(`TheHostOfABatch::Stated(A_HOST_THAT_RUNS_ONE_ITEM_AT_A_TIME)`),
both hosts in `landing/service_work_items_side_by_side.rs`.
A control counts those readings (`machine_reads_for_a_control`)
to prove a batch reads where it is sized,
and a count of the whole process is read in a child of the control's own.
A width a control states for the whole process
(`hold_the_width_to`,
`landing/service_work_items_side_by_side.rs`)
is a measurement's alone,
because every other test of the run reads it too:
that value was held at one by `a_round_starts_no_housekeeping_item_of_a_batch_after_its_slice_is_spent`
while the side-by-side controls of the same process read it (issue 6567).

The service's stop is one such fact
(`process/stop_of_the_service_noticed_by_its_children.rs`):
the signal the handler recorded,
the pipe it wakes the shutdown through,
the level every waiter reads and the count of unfinished work
are statics of the whole test process,
so a control that records a stop,
prepares to notice one or reads them runs its body in a child of its own
(`testing/process_wide_state_child_test_support.rs`),
and so does a control that starts a step a recorded stop would refuse
(`process/signal_relay_tests.rs`).
A threaded `push::` run failed 110 of its 600 tests and a threaded `process::` run failed eleven on 2026-11-26
(issues 6530 and 6531),
because the stop one control recorded was the stop every other control then read.
Measured on 2026-11-27:
a threaded run of `process::stop_of_the_service_noticed_by_its_children::tests` answered
`test result: FAILED. 2 passed; 8 failed`,
and `a_process_that_never_prepared_counts_nothing_and_refuses_nothing` recorded the stop first,
after which every control that needed to record its own found one already there.

A control that holds a fault point of the service
(`service_fault_hooks_test_support.rs`)
names the instance it owns in the point itself:
the directory whose instance is read,
the plan the start reads,
the request whose beat reads its note,
the repository a note is written about,
or the record whose trash is emptied
(`instance_read_point`,
`note_read_point`,
`read_at_a_start_point`,
`next_service_note_point`,
`inside_a_slow_step_point`).
The holds are a list of the whole process,
so one slot for a run held nothing of a second control,
and the module of three heartbeat controls failed all three (issue 6497);
the registry refuses a second live hold of one point and names the composition the control must make,
so a control that names one point twice fails at its own first line instead of in another control's wait.

## One fixture folder per test

A test takes its folder from `testing/test_support.rs`.
`fixture(name)` names the folder `native-unit-<process id>-<the number this call took>-<name>`,
so every call gets a folder of its own:
before the number was there,
two tests that passed one name shared one folder in one process,
and each deleted the other's live folder:
`lock::index_writer_forward`'s two signal tests did,
failing with `DirectoryNotEmpty` and with a `create_dir` panic at `test_support.rs:332`.
The binary test target's own helper names its folders the same way
(`native-binary-<process id>-<the number this call took>-<name>`,
`binary/binary_support_fixture_directories.rs`).
Never write a fixed path outside the fixture root (no `/tmp`,
no shared name under the build folder).

A fixture is removed through `test_support::remove`,
which waits `REMOVAL_TRIES` pauses of `REMOVAL_PAUSE` while the tree still gains entries
and,
when the tries run out,
fails with every name the fixture still holds,
each with what it is.
A landing does not write on its own thread alone:
`land_group` starts the shadow pool's fill,
the scratch sweep and a shadow drawn ahead on threads of their own,
and the blocking pool makes the markers the next batches' locks are made from,
each of them after the landing has answered.
Measured 2026-11-27 (issue 6568):
the draw made `monochromatic-git-policy-cli-shadow-drawn/main/drawn-<uuid>` inside a control's fixture `.git`
5 ms after the landing returned,
and a teardown that met it mid-write failed with `DirectoryNotEmpty` beside no entry at all,
so the round that met it could not name the writer.
A folder that has to be short,
because a socket's path must fit one socket address,
comes from `runtime_directory_folders.rs`'s `runtime_directory_parent`:
it picks the longest place this account offers that leaves room for the folder's name,
makes the folder exclusively under a random token,
and hands it to the thread's fixtures,
which remove it however the control ends.
No control builds that folder under the home directory itself:
the helper the hung-write controls used put `hw-<process id>-<name>` under the real `~/temp/agent`,
and every control that panicked or failed left one there:
34 of them by 2026-11-27 (issue 6561).

A fixture's overlay names the place the wrapper records what it learned about the real Git in:
`test_support::cache_home` makes one folder of the thread's own,
and `isolated_overlay` names it as `XDG_CACHE_HOME` beside the configuration's own variables.
The wrapper learns where that Git's system configuration and attributes files are,
and records the answer under that place,
so a control's work writes a file of its fixture and never `~/.cache/monochromatic-git-policy-cli`,
and it reads none another session made (issue 6571).
A service whose work runs in the test process reads those facts through `for_context`,
which builds them from the process's variables with the context's overlay over them,
and a control that hands the writing code no place of its own writes the person's home cache:
`testing/test_support_tests.rs`'s `the_cache_place_a_fixture_hands_its_work_is_a_folder_of_its_fixture`
starts that code and refuses a place outside the fixture root.

## A fixture commits through the standing Git, in an environment that names no session place

On this machine the `git` a run's own `PATH` finds is the wrapper,
which hands a commit to the commit service of the checkout it runs in,
has this account's user manager start one for that checkout,
and points the shared program link at itself as it does.
Measured 2026-11-27 (issue 6620):
`mise run //package/git-policy/cli:native:ts-checks` committed in four fixture checkouts,
each at `~/temp/agent/git-policy-gate-main-tip-<random>/repo`,
through that wrapper;
the manager started four `monochromatic-git-policy-cli@<instance>` services for them at 14:31:32,
and the link named release `0.6.1-alpha.44` until another wrapper command pointed it back at 14:33:15.
The TypeScript unit tests are not a test build:
they start a released wrapper,
so the test-only switches the Rust controls read
(`foundation/session_runtime_directory_and_bus_address.rs`) are in no TypeScript fixture's reach.

The issue's first suggestion,
a fixture handing its child a runtime directory of its own and no bus address,
does not carry a commit on its own:
a released wrapper given those two refuses the commit outright,
with "This computer cannot start the commit service on demand,
so this commit was not made"
(measured the same day with release `0.6.1-alpha.44`),
because the service is the only writer of a commit.
So a fixture that commits starts the standing Git and never the wrapper,
in an environment that names neither `DBUS_SESSION_BUS_ADDRESS` nor `XDG_RUNTIME_DIR`,
and that gives a runtime directory of the fixture's own where the fixture has one to name.
`bin/fixture-git-environment-for-tests.ts` is the one place that answers it,
as `foundation/session_runtime_directory_and_bus_address.rs` is for the controls.
A wrapper that found its way back onto a fixture's `PATH` then has no route to this account's manager,
and refuses the commit in the open instead of handing it to a service the manager started for the fixture.

`bin/stand-in-git-that-records-its-environment-for-tests.ts` holds it:
it records whether a bus address reached it and which runtime directory it was given,
and then runs the standing Git,
so the fixture still makes the commits it asked for.
`bin/native-gate-main-tip.unit.test.ts` gives its fixture and the command it relaunches that environment,
puts the stand-in first on the fixture's search path,
and reads that no bus address reached the fixture's `git`,
that every call was given the fixture's own runtime directory,
and that a `git --version` of the fixture names no wrapper.
The stand-in programs of the gate,
the scope and the install fixtures hand their children
`environmentAFixtureChildMayInherit()` rather than `process.env`,
so no child of those fixtures inherits either place.

## A plan a control plants comes from the removal's writer

A control that needs a removal's destruction plan plants it through
`testing/the_destruction_plan_a_control_plants.rs`,
which builds the plan a removal's first write leaves and publishes it with
`PlanFolder::publish`:
the writer holds every field the grammar holds,
and `publish` reads the record back through the grammar's parser before it lands,
so no control can plant a plan the design refuses.
That file is included by `#[path]` in both test targets,
and each crate declares `the_destruction_plans_writer` so the shared file can
name the writer's items;
`testing/a_planted_plans_snapshot_line.rs` adds the lines of a later history and
stays in the library,
because the binary controls plant a first write and nothing else.

Write a plan's text by hand only where the text is the control's subject:
a record the reader must refuse.
Such a control plants the grammatical plan first and then takes out the one field
or line it is about.

Before 2026-11-27 the controls wrote the three or four fields the add's reader asks
and nothing else,
and when that reader began to require `head=` on 2026-11-25 (`fbd0d43d89`) the
slot-state guest arm planted a plan the reader refused,
so every assertion after its `pending-add` marker read `cannot be proved`
(issue 6595).
`testing/the_destruction_plan_a_control_plants_tests.rs` is the host control that
would have caught it:
it plants a plan and asks the add's reader and the grammar's reader what they make
of it,
for the first write,
for each of the five head words,
and for a history that ends `destroyed`.

## Own every child, end it on every path

A child a test starts is the test's to end,
on success,
on failure and on a panic:
start it with `test_support::spawn_owned`,
whose child is ended with the test thread's fixtures however the test ends,
so a failing assertion cannot leave it running.
A control that needs a signal sent to the wrapper runs the wrapper in such a helper process
and signals only `helper.id()`,
as `process/signal_relay_tests.rs` does:
`lfs::lfs_upload`'s signal control once sent `libc::kill(libc::getpid(), libc::SIGTERM)`,
and `getpid()` is the test binary,
whose signal handlers are process-wide,
so it ended every thread of the run (issue 6463).
Start a stand-in in a process group of its own,
and signal only the process or group the test started and proved,
never a group or session the test binary belongs to,
never by name.
A child must not keep the test's standard output or error open after the test:
close or redirect the streams it does not need.
`lfs::lfs_upload`'s `git-lfs` stand-in,
which writes its process number and `exec`s `/usr/bin/sleep {HOLD_SECONDS}`,
is the shape that leaked.
A helper whose state a control reads is one the control ends,
never one that ends by itself:
`identity_of_process` and `process_birth_identity` answer nothing for a process that has exited,
so a control that reads one passes or fails by how fast this host woke its thread.
Such a helper waits for its own end:
`sleep` with a hold the control states,
or `cat` on a pipe the control holds,
and the control kills it before it asserts on what a gone number means.
On 2026-11-27 the record sweep control read the identity of `talk`,
which printed its usage and exited at once,
so the read lost that race whenever the host was slow (issue 6574).
Every other helper a control of this crate expects to be alive
is a `sleep` with its own argument,
a `cat` or `tee` on a pipe the control holds,
or a shell running a script that waits.
A child a shipped call starts itself is still the test's to end:
a control of the relay's end rule starts the child with `process_group(0)`,
has that child write down the number of the group it leads,
hands the number to `testing/test_process_owner.rs`'s `own_group_of_number`,
and ends the group before it returns.
The read answers `0` for a child no `SignalGuard` started,
and the owner refuses `0` as well as a number whose group holds no process,
so a control that owned the wrong number fails at that line instead of at the run's end.
Without any of it the control returned with the `sleep` the child had started holding its stream,
and the container gate counted the test as one that leaks (issue 6501,
`source_guard/relay_controls_own_the_child_s_group_tests.rs`).

A child that is this test binary running one named test goes through `testing/child_test_process.rs`
and nowhere else:
`ChildTest` is the request (`started` hands back the running child,
`finished` waits with a deadline and reads both pipes once the child is gone,
`passed` adds the check that exactly one test ran and passed),
and `testing/test_process_owner.rs` is the ownership it stands on.
Six bodies built that command their own way until 2026-11-25,
and most of their children were owned by nothing (debt ledger item 10).
A child a control starts for its own body is handed that body,
and the parent never calls it:
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
A child whose ending is the assertion,
such as one that dies by a signal,
takes `finished`
and judges the ending itself;
a child a control keeps and talks to takes `started`,
and its own waits carry the bounds.
A fixture's own Git starts one more child outside any test's reach:
`git commit` starts `git maintenance run --auto` in that repository in the background,
and that runs `git worktree prune --expire 3.months.ago`,
which removes a `.git/worktrees/<name>` directory Git did not make,
recursively and whatever its age,
so a registration a control makes by hand goes while the control still asserts on it
(measured 2026-11-26:
gone 1.237 ms after the commit returned,
in one run of ten,
issue 6518).
A fixture's registration is the one Git's own `worktree add --no-checkout --lock` makes,
and a control whose subject is that registration runs the housekeeping itself,
proving it keeps the registration Git made while removing one made by hand
(`worktree_clone_add_route/add_index_tests.rs`).

## Every wait has a deadline

A test waits for a condition with a bound and fails at the bound with what it waited for,
never by sleeping for a guess and never without an end.
A bound in an assertion holds under this host's load,
because other sessions build on it all the time (issue 6405 is a bound that did not).
A bound that is the control's own wait,
rather than the code's promise,
is stated as a multiple of the bound the code owes,
so a host that wakes the control's own thread late spends the control's window and never the code's promise
(issue 6519:
the control that proves a reader which stopped reading does not hold the commit waited two seconds,
twice the notice's own maximum,
and failed that way in three threaded `commit::` runs;
it waits thirty times the notice's own bound now,
sized from a measured answer of 500.1 to 501.6 ms).
A fixture's own pipe is waited for at the pipe,
not at the thread of this process that reads it
(issue 6538:
three `repo_read_config_typed` controls failed in five kept threaded runs with
`the query [...] ended, and a descendant of it still holds its output`).
`real_git_within` gave a query's output 250 ms to reach its end,
and the news it waited for was a message from a thread of the test process,
so a host that woke that thread past 250 ms made the answer say a pipe was held when nothing held it.
Measured on 2026-11-27 at this host's load average of 86.0 to 142.5:
over 15398 hand-overs of one query's standard output in one threaded module run,
the median came 0.003 ms after the pipe's end and the worst 110.4 ms,
which the window cleared by 2.3 times.
The window is asked of the pipe itself now (`has_ended_within` in `testing/fixture_git_pipes_read_beside_the_query.rs`:
the kernel reports a pipe's end the instant its last writing end closes and keeps that news until it is read),
and the reader's hand-over of the bytes is bounded by `REPORTING_THE_END`,
sixty seconds,
which only stops a hang.
A control's own bound then rests on a promise the test's own threads do not have to keep.

A control of a step the kernel holds states both of the facts such a hold gives the wait.
One is the kernel's answer about the process's state,
which `the_kernel_holds` states
(`commit/scratch_step_look_below_control_window.rs`).
The other is the step's end not collected until the control lets it go,
which `hold_the_reap` states
(`process/wait_after_the_stop_saying_the_kernel_holds_it_control_window.rs`).
One without the other tests a wait that never lasts:
the step acts on the `SIGKILL` at once,
the wait after the stop returns before its first look,
and the control passes on a wait it never made.
On 2026-11-27 the never-released control of `commit/commit_preparation_tests.rs` stated only the state,
passed alone in 1.41 s,
and then failed a gate run in 31.9 s,
because a loaded host's own disk wait held the real step and the control allowed the command twenty seconds
(issue 6589).
The gate's log is what measured it:
the line naming process `12876` is the step,
which that control never stated,
so `/proc` itself reported the step in state `D` until the kernel let it go,
about thirty seconds after the `SIGKILL` at 1.3 s.

No control bounds the kernel's release of a step.
The release waits for it with no bound after the stop
(`SPEC.md`,
the owner's answer of 2026-11-10,
"Keep waiting and say so"),
so a control asserts what the release does promise:
the step was ended at the bound the control planted,
its words say the step was waited for,
and the command answers once the control let the reap go,
within a multiple of the wait's own `AFTER_THE_STOP` pause.

A control whose subject is a notice said to a waiting client reads what the client has said while it still waits,
and reads it before the client ends
(`Told` in `landing/service_land_hung_write_tests.rs`):
the text a client ends with cannot tell a person told when the write started from one told as it gave up,
and the control sizes that wait as a multiple of the notice's own bound,
because issue 6524's control waited one ceiling away from a notice due at one second,
and a journal read that waited on storage carried the words past the client's ceiling,
so it counted none.
A control whose subject is a notice a client says asserts only what holds in either order of the messages that decide it,
and holds the answer away with a seam of its own when that notice must be said to a wait that is still on
(`on_objects_written` in `landing/service_land_hung_write_tests.rs`):
the line saying the service has not said where a commit stands is said by a look that found nothing already here,
so a control that demands it asserts which message the host let arrive first,
and it failed that way in one threaded module run of five (issue 6564).
A window a control times never holds work the release code grants no ceiling to,
because no multiple of a ceiling can bound that work:
a commit's landing costs what this host's disk costs,
so a control that answers a commit times that landing unless its window starts after the commit landed.
Measured 2026-11-26 in a threaded `landing::` run:
the nine landings that run logged took 140 ms to 22576 ms,
and a control that timed a commit's whole answer from its submission failed at three seconds
beside a note ceiling of 500 ms (issue 6525).
So a window starts where the note's write began,
once the commit has landed,
and holds only that ceiling and the answer after it,
and a property that cannot be timed that way is asserted structurally instead:
the store named no repository once both answers had come,
so neither answer came from it.
A control whose subject is a wait the release code must never enter waits on the work's own answer
and on the run's own progress,
never on a clock over the work,
and says what it saw when it gave up
(`worktree_copy_settling_tests.rs`,
`the_copy_answered`):
the copy of two ignored entries took 0.478 s at its slowest on this host at a load average of 198 to 281,
and the ten-second patience the control borrowed from the rendezvous helper failed at a load average of 152
(issue 6548).
The state a claim like that rests on is measured,
never inferred:
a writing open of the named pipe with `O_NONBLOCK` fails with `ENXIO` unless a reader holds it open
(`pipe(7)`),
so a reader that holds it is a copy waiting inside the open,
and the writer that releases it also lets that copy's thread end.
A state a control means to test is reached by the control and not by a sleep:
that control offers a line into the queue the writer takes its lines from,
where sleeping three cadences expected a starved notice thread to have offered one
(a thread with three looks in 400 ms never did).
Code under test bounds its waits too:
every read of a file a person or another program controls opens it so it cannot
block (not a named pipe,
not a device;
the type checked through the open handle,
never by a path check before the open)
and has a size bound (issues 6415 and 6433).
No blocking work runs on the commit's own thread or on the async runtime (`source_guard`'s runtime guard;
issue 6421).

A control that needs two file times to differ states the difference instead of hoping the clock moved between them
(issue 6520:
the copy compare stamp control wrote the source 0.34 to 0.79 ms after taking its stamp,
this host's ZFS takes a write's change and modification times from the kernel's coarse clock,
and that clock advances once per 1 ms tick,
so the write left the stamp exactly as it was and the final check rightly passed in 6 runs of 6).
`stamped_pair` gives its source a modification time `RECENT_CHANGE_NANOS` before now for that reason,
and the measurement is in `doc/troubleshooting/openzfs-file-times-advance-in-one-millisecond-steps.md`.
The controls of this crate whose assertion needs two file times to differ,
and what each rests on
(2026-11-27,
issue 6593):

- `worktree_copying::worktree_copy_compare::tests::a_source_written_after_the_stamp_fails_the_final_check`
  rests on `settle_the_modification_time`,
  which gives the fixture's source the modification time `RECENT_CHANGE_NANOS` before now.
- Every control that pins the clock with `clock_reads`
  (the `settled_clock` fixture of `index::index_reset_worktree_stat::tests`,
  `index::index_reset_file::tests`'s attributes control,
  the three at `worktree_copying::worktree_copy_stage::tests`,
  and the ones in `worktree::worktree_clean_content_attributes::tests`)
  rests on the instant it hands the code,
  never on the clock moving.
- `worktree::worktree_snapshot_index_step::tests::the_index_a_source_with_changes_installs_keeps_the_fields_that_prove_an_edit`
  and the controls of `worktree::worktree_dirty_restore::tests` that hide an edit
  rest on `hide_an_edit`'s two stated nanosecond parts of the modification time
  (`HIDDEN_EDIT_RECORDED_NANOS`
  and `HIDDEN_EDIT_EDITED_NANOS`),
  which is what issue 6593 changed.
- `lock::index_lock_evidence::tests::the_change_time_rounds_nanoseconds_down_to_milliseconds`
  is the one left that rests on the file system's own clock:
  it changes a file's permissions up to fifty times and asserts that one change time has a nonzero
  millisecond part,
  which the clock's offset decides and nothing of the control does.
  It is unchanged,
  and issue 6593 names it.

A control whose subject is the time the record credits to the programs below the step
waits for the looks that credit it
and derives its bound from the record's own counts,
never from a clock:
the credit is the span between the looks that found the program,
one interval for each look after the first,
and a window of wall clock holds fewer of them on a host whose threads wake late.
`until_looks_find_the_step` (`commit/native_commit_waiting_notice_control_window.rs`)
waits for a count of looks that found the step,
and `a_look_of_this_step_waits` in the same file plants the stretch a loaded host put between two of them.
Measured on 2026-11-27:
the control asserted 750 ms of credit inside a 1,200 ms sleep,
which is three intervals of `LOOK_INTERVAL` and is what only a punctual host proves,
and the gate run whose each proved interval was 319.73 ms held four looks in that window instead of five,
three of them finding the step,
so the record proved 639.47 ms and the control failed (issue 6592;
`doc/troubleshooting/git-policy-cli-commit-index-span-control-counts-another-tests-spans.md`).

A control that shortens a deadline the release code keeps itself holds the work that deadline governs before that work starts.
The fence's size read is joined with a deadline measured from the wait and not from the read's start
(`SizeReadBesideTheCheck::answer_within_the_deadline` hands it to `recv_timeout`),
and the join runs after the source check and after `git check-attr`,
so a read started first answers inside a window that began after it did.
The join's control arms `a_control_holds_the_read` before `start_beside_the_check` for that reason
(issue 6603:
a wide threaded `worktree` run answered `3311 passed; 1 failed`,
with the rung held where the control says a held read cannot answer,
and 100 ms of delay planted between the start and the arm reproduced it in a run whose child finished in 0.15 s).

## Write less code

Reuse what `testing/test_support.rs` and the package's modules already have before writing a helper
(`AGENTS.md`'s UTL),
and move a helper you need twice into the shared module instead of copying it.
Return no more than the caller reads (issue 6412).
Delete code nothing calls,
with the text that describes it (`AGENTS.md`'s XSU;
issue 6430).
A file over the linter's `rust/max-lines` splits into sibling modules by role,
never by stripping docs (MXL).
Name files,
folders and functions by what they do (MKY,
VGW),
derive values from named constants (MGN),
and take no lock where atomic rename,
`O_EXCL` or one owner per item will do (NLK).
A list of names,
sites or words is a slice,
`const NAMES: &[&str] = &[...]`,
never `[&str; 47]`:
a hand-counted length is a magic number every added entry has to edit
(found 2026-11-25 in the raw-read guard's list,
beside 277 such arrays crate-wide;
ledger item 19).
An array keeps a fixed length only where the length is the meaning,
such as a header of a fixed byte width.

## A split keeps each item's reach and its platform gate

A split moves code,
and two facts of every item it moves belong to the split's own diff.

Reach:
an item a sibling calls inside the crate takes `pub(crate)`,
`pub(super)` or `pub(in path)`,
never bare `pub`.
The library target is a published surface,
and release `0.6.1-alpha.48` widened it by 31 names a split wrote bare `pub` for,
seven of them private before the split,
with no commit that said it published anything
(`source_guard/nothing_becomes_nameable_from_outside_tests.rs`).

Gate:
a new `mod` line carries the `#[cfg(...)]` of the file it left,
and a struct or a trait a split made out of one gated function carries the gate itself where the file also holds an
ungated neighbour (`AddPlan` and `RemovePlan` in `worktree/worktree_command_facts.rs` are gated while `CopyFacts` in
the same file is not).
A gate that goes moves items into the build of a target that never had them,
and nothing here reports it:
the gate's Clippy step and the test suite build the host's own target (`bin/test-native-container.ts`),
and `native:clippy:windows` is the one task that type-checks another target,
which no gate runs.

The gates do not make a build for another target work either,
so say only what the source shows:
`worktree/worktree_remove_around.rs` names `finish`,
declared in `worktree_remove_clears_the_registration`,
and that module is `#[cfg(target_os = "linux")]` (`worktree/mod.rs`).

A merge can take a gate away with no textual conflict,
and one did on 2026-11-27:
a merge restored seven platform gates,
seven such attributes in its diff outside tests,
where its commit says 29 items and the handover of that day said 28.
The same merges left a Linux release build differing through two changes only,
argued from the source text and not from a build:
an unused constant left it,
and one tag constant went from 141 copies to 123 with every value the same.

## A published name takes a line in the list beside the surface control

`source_guard/nothing_becomes_nameable_from_outside_tests.rs` holds the library target's published surface to
`source_guard/items_nameable_from_outside_release.txt`:
a name another crate can reach without a line fails,
a line whose name is no longer reachable fails,
and a `pub use` that publishes a name the `pub mod` chain does not already reach fails without its line.

A line is added only where a release publishes a name on purpose:
write it as `<module path> <kind> <name>` under a reason of its own that names the commit which added it.
A name a re-export publishes is written as `<module path> pub use <target>::<published name>` instead.
A name a split needs only inside the crate takes `pub(crate)`,
`pub(super)` or `pub(in path)`,
and no line.

The reader's own top doc states the reading:
a bare `pub` item at the first column of a file the `pub mod` chain reaches,
whatever count of qualifiers stands before its kind,
as `pub async fn`,
`pub const unsafe fn` and
`pub extern "C" fn` do;
a `pub` field,
a method or an associated constant of a type that chain publishes,
read in a body that opens its `{` on the declaration's own line or on a later one,
which is how a long generic list is written;
an item written on the same line as the attribute above it;
a `#[macro_export] macro_rules!`,
with or without parentheses inside the attribute;
and a `pub use` over any number of lines,
one entry per name it publishes,
with an `as` rename read as the published name,
a glob written `pub use <module>::*`,
and a re-export written without a leading path read from the module that writes it.
A gate keeps a name out only where it holds only when testing,
so `#[cfg(test)]` and `#[cfg(all(test, unix))]` keep one out
and `#[cfg(all(not(test), unix))]` does not.
A `mod` line the reader cannot place stops every scan of the tree with the line named,
rather than being stepped over:
the visibilities it reads are
`mod`,
`pub mod`,
`pub(crate) mod` and
`pub(super) mod`,
so
`pub(in crate::a) mod x;`,
`pub(self) mod x;`
and a `mod` line with text after its `;` are errors.

The reading was widened on 2026-11-27 to the shapes above:
154 of 155 published names it could not see had no line in the list,
a function with `async` before `fn`,
a name another listed item already has,
a field,
a method,
and a `pub use` written over several lines.

After each of two structure branches merges,
`native:check` runs on the merged tree before the next merge:
a narrowed item or a changed excuse can break the build through a merge with no
textual conflict.

Two facts of the reading are decisions a reader has to see in the list:
a name behind a feature gate counts as published,
because a gate keeps a name out only where it holds only when testing,
so the list holds the three names such gates carry today
(`foundation::test_only_switches fn value`,
`fn value_in` and
`worktree::worktree_clone_conversion_fence_measurement_routes fn fence_of_the_measurement_route`);
and a member is keyed by its type's bare name,
so a `pub` type in a module nothing publishes has its `pub` members read as published whenever a published
type elsewhere writes the same name.
A member of a type whose only public path is a `pub use` facade is read too,
at the facade's path and under the name the facade gives it,
from the module the statement names;
a facade that names an inline `mod` body or a module no chain and no root glob reaches publishes a type whose
members the reading sees nowhere.

What the reading still cannot see,
and why,
is in that same doc:
a `pub` field written in a tuple-like struct's parentheses,
a member of a type a facade names through an inline `mod` body or through a module the tree cannot resolve,
a `pub trait`'s methods and a `pub enum`'s variants,
an item inside an inline `mod name { ... }`,
the names a glob carries out of a module no `pub mod` chain reaches,
the second path a re-export adds when it keeps the name it re-exports,
and a signature or a value that changes under a name that stays.
So a widening in one of those shapes is one only the diff shows,
and a changed signature or constant value never shows at all.

## Show that a guard works

A guard counts once it is shown failing without its code:
commit the control,
take the guarded line out,
run,
restore (`AGENTS.md`'s GFP).
A green run counts only when its output names the expected tests (TRV).

The two rules above are enforced,
not only written down:
`source_guard/lists_are_slices_tests.rs` fails on a `const` whose element type names a text reference
and whose length is a number,
whatever layout the declaration uses,
`source_guard/test_children_owned_tests.rs` fails on a test file that starts a child
which neither `spawn_owned` nor `own_group` owns,
and on any signal whose target is `getpid()`,
`getpgrp()`,
`std::process::id()` or `0`.
The sites the tree still holds are one line each in
`source_guard/test_children_not_yet_owned.txt`,
with why each is not moved yet,
so moving a site means deleting its line there,
and a new site fails until it is moved or entered with its reason.

## A control that reads another module's source follows that file

Some controls read another module's file with `include_str!` and assert on its text,
and then that file's own module tests are not the ones a change to it breaks:
`foundation/notice_stream_that_does_not_hold_its_writer_tests.rs` reads
`landing/service_commit_waiting_notices.rs`,
and `service/wait_ceilings_tests.rs` reads `landing/service_commit.rs`.
`source_guard/hook_self_invocation_tests.rs` read that file as well until the alpha.48 splits;
it reads every release source of the commit path through the shared scan now,
and the one file it still pins with `include_str!` is `src/native/main.rs`.
Before a merge,
name every file the change touches and run the module of each test that reads it
(`rg 'include_str!' src/native` lists them),
beside the module of the file you changed.
The fix for issue 6421 (`700a6e4c02`) rewrote the line that starts the commit's notice stream
and left `the_commit_command_hands_its_waiting_notices_to_this_stream` asserting the line it replaced,
so the whole container gate on `main` at `132843d9d1` failed in a module that fix never ran.

The search a merge owes is wider than that grep:
a control may also read a source file while it runs
(`std::fs::read_to_string` of a path under `src/native`),
and the guards that read a part of the tree do it through the shared scan of
`source_guard/source_scan_banned_lines_tests.rs`,
or,
where a guard lists the tree itself,
through `source_guard/source_scan_tests.rs`.
`rg 'read_to_string' src/native --glob '*_tests.rs'` lists the ones outside `source_guard`.

A call a control requires belongs to the folder that asks it and not to one file of that folder
because a split moves the call into a sibling file and leaves behind the file it left.
The round for issue 6540 moved `output_of_within_bound` out of `repo_read/git_system_config.rs`
into `repo_read/git_system_config_asking.rs` (`e58254244d`).
A control that still named the file it left failed the whole gate on `main` (issue 6573).
`process/captured_child_run_to_its_end_callers_tests.rs` finds the call across the module now:
`source_root()` and `real_roots()` and `classified_sources()` of `source_guard/source_scan_tests.rs`
list the files the build itself reaches.
The control requires the call in one of the folder's release files
and refuses `Command::output` in any of them.
That fails when the call leaves the folder or a caller goes back to `Command::output`
and passes when a split moves the call between the folder's files.
That move is what the claim is about.

No check over this text can replace running those modules.
A needle a control requires may also be a sentence the program prints
and so a scan cannot tell a pin over source from an assertion about a running program.
The search of 2026-11-27 that guessed from literal needles found 58 places where a required
literal was absent from the file read and every one of those modules passes.
Reading the splits of 2026-11-26 and 2026-11-27 against every control that reads a source file
found one stale control.
The files that `include_str!` one are what `rg 'include_str!' src/native` prints,
the files outside `source_guard` that read one at run time are what the search above prints,
and the rest are in `source_guard`,
whose whole module runs.
Every one of those modules ran once and threaded,
and only the control above was stale.

## A guard over part of the tree states its scope, its excuses and its anchor

`source_guard/source_scan_banned_lines_tests.rs` answers four questions for a ban over part of the release tree:
which release sources the ban covers,
which of their code lines hold what it forbids,
which entries excuse one,
and whether the scope still reaches the code the ban is about.
A guard that states its scope as prefixes reads the tree through it
(`release_sources_under`,
`release_sources_holding`),
so a file a split makes inside a scope is covered the day it lands (issue 6599).
Not every guard does:
some list the tree themselves with `release_sources_in` or `classified_sources` of
`source_guard/source_scan_tests.rs`,
some read a few files with `include_str!`,
and one walks `src/native` with its own `files_below`
(`source_guard/no_start_at_login_tests.rs`),
and each of those says in its own doc what its reading covers.

Scope.
`release_sources_under(prefixes)` covers a place and names it as a folder (`landing/`) or as the start of a
file's name;
`release_sources_holding(words)` covers a lane and names the words a file of that lane says.
Write in the guard what a file would have to look like to be missed:
a folder scope misses the file that moves out of the folder,
a prefix misses a file of the same module named without the prefix,
and a lane misses a file that does the banned thing without naming its words.
Where the folder above the files closes that hole,
name the folder
(`standards_review/`,
`worktree_clone_add_route/`);
where it does not,
because that folder holds other code that does the same thing for its own reasons,
the guard's top doc says what a reader has to check.

Excuses.
`MayHold { file, text, line, why }` names one file,
one banned text or name,
the line that one use is written on,
and the reason it may.
The line is what an entry names rather than a count of uses:
a use removed and another of the same text added to the same file is a finding twice over,
because the line the entry names holds nothing and the new use is excused by nothing,
where a count would have kept the file excused.
Each entry covers one finding,
so a second line written exactly like an excused one needs its own entry.
A line `rustfmt` reflows makes the entry stale,
which the guard reports,
and the entry and the reflow are one commit's work.
`entries_that_match_nothing` fails an entry no finding took,
so the list can only shrink.

Anchor.
`anchors_the_scope_no_longer_holds(sources, anchors)` fails when no file of the scope holds a text that proves
the scope is still the code the ban is about,
and a scope that reached nothing would pass every tree.

A banned text is a substring of one line of code,
and a line whose first characters are `//` is not code.
A ban on a call is `lines_calling_one_of`,
which reads the name where the line spells the call:
the last segment of a path (`fs::symlink`),
the call (`symlink(`,
with the whitespace Rust allows before its arguments),
and the call with its types written out (`symlink::<_, _>(`).
The name is read only where no byte of a longer name touches it,
so `link` is a finding and `hard_link`,
`read_link`,
`unlink` and `is_symlink` are not.
That is what makes a rename's import (`use std::os::unix::fs::symlink as make_link;`) and a function taken as a
value (`let f = std::os::unix::fs::symlink;`) findings:
the call they then make cannot be told from any other call,
and the line that brings the name into scope can.
A `use` list may also write that rename in braces (`use std::os::unix::fs::{symlink as make_link};`),
where the name follows a brace rather than a path,
so `name as` is read as a spelling of the call.
A ban on a name no other use shares is `lines_naming_one_of`,
which reads it wherever the line writes it,
so a type in a signature and in a `use` list is a finding too;
`UnixStream` and `Stderr` are read that way,
and `link` and `stderr` are not,
because this crate has a field named `link` and local variables named `stderr`.
Three spellings no reader of lines sees,
and each guard whose scope could hold one says so in its top doc:
a call a macro written outside the scope expands to,
a name a glob import (`use super::*;`) brings into scope without writing it,
and a call whose name and `(` a line break separates.
The alpha.50 read planted nine spellings against these guards on 2026-11-27,
and the call reader reads each of these now but the macro:
a renamed import,
a function taken as a value,
a call with its types written out,
another crate's name for the same call,
a handle reached through a module,
and a key passed through a constant.
A spelling a guard can be made to read,
it reads and plants a control for:
the commit path's ban reads a direct `syscall` call,
because `libc::syscall(libc::SYS_symlinkat, ...)` writes the system call's name inside a longer constant;
the login guard reads the manager's program however its literal is spelled,
bare or as a path,
on the call's own line or on the next one,
and refuses the import that brings the type of a start in under another name;
and the note guard reads a write by descriptor number (`libc::STDERR_FILENO`),
the two macros that write standard error without asking for a stream,
and the paths that open the same handle.
The read of alpha.52 planted six spellings the guards let through and each is read now:
the link guard reads `std::fs::soft_link`;
the note guard reads `eprintln!`,
`eprint!`,
`/dev/stderr`,
`/dev/fd/2` and `/proc/self/fd/2`,
and `eprintln!` is refused in every release source by the door's own guard
(`source_guard/diagnostic_door_tests.rs`) besides;
and the login guard reads the import that renames the type and a literal on the line after its `(`.

## Walk the sources through the one walk, never a recursive read

Every source folder holds a `uses/` folder of links to the folders its code uses,
so a read that follows a link reaches the same module once per chain of links above it.
Measured 2026-11-27:
a recursive read of `src/native` returned 165,978 entries and 163,498 `.rs` names
where the tree holds 4,234 entries and 3,905 Rust sources,
and joining that much text ended the planted-controls unit test with `RangeError: Invalid string length`
(issue 6580).
A TypeScript script or test of this package that reads the sources walks them through
`bin/source-tree-files.ts` (`sourceTreeFiles`),
which enters each folder once,
never descends a link,
and gives the files in code unit order.
A fixture folder a script makes itself is not the source tree,
so a reader of one may list it directly:
two readers of this package list a folder recursively and each follows a link it meets,
`e2e/observation-fixture.ts` (a fixture repository's common directory) and
`perf/fence-size-read-beside-the-check-2026-11-21-control-identical-fence.ts` (the fixture trees it compares),
and neither fixture holds a link as of 2026-11-27,
because no script of the package makes one there.

A Rust test that reads this crate's sources walks them through the shared scan of `source_guard`:
`source_scan_tests::discover` lists every `.rs` file under a root once,
`source_scan_tests::classified_sources` adds the module tree's answer about which of them are test-only,
and `source_scan_tests::source_text` reads one listed file.
`discover` enters each folder once,
reads nothing through a symbolic link,
skips a `uses` link that names a folder below the root and refuses every other link,
and its control `the_walk_reaches_a_linked_back_source_once_where_a_recursive_read_reaches_it_again`
(`source_guard/source_scan_walk_tests.rs`) holds a fixture of one source file and one link back into the tree:
the walk lists that file once and the `read_dir` recursion the walk replaces reaches it once per link above it.
No test of this crate lists a source folder with a `read_dir` recursion of its own,
and every walk of one asks an entry's own type with `entry.file_type()`,
never `path.is_dir()`,
which follows a link.
A check that looks for a needle reads one file at a time and holds that file's text alone.
Measured 2026-11-27 (issue 6598):
the sources were 3,905 Rust files of 46,090,082 bytes when these readings were taken,
and the control that held them all read through the `uses` links read 165,978 entries,
104,219 files and
888,676,493 bytes,
holding 890.5 MiB of resident memory for 22.1 s of a suite that runs for about sixteen minutes;
the same control reads one file at a time now,
in 0.8 to 7.7 s and 27.2 to 28.0 MiB.
A control that builds a tree from the sources reads them the same way,
because a walk that descends one link builds that tree forty times over:
`binary/binary_worktree_clone_zfs_measurement_tests.rs` read 163,706 files and 1,658,426,907 bytes of `src`
where the tree held 4,110 files and 49,829,839 bytes.

A planted control names one file and one needle in it,
and a needle goes stale without anything renaming it:
a split moves the guard into a sibling file,
or `cargo fmt` reflows the expression onto other lines.
`bin/native-planted-controls.unit.test.ts` is what reports that,
so it names the control and the file whose text no longer occurs exactly once;
seven controls were stale on 2026-11-27 and the source walk's crash hid every one of them;
four of the seven had moved with a split,
and three were reflowed by `cargo fmt`.
A control whose guard moved names the file that holds the guard now,
and one whose text was reflowed carries the text as the file holds it,
with a comment saying which commit moved what.
A needle that compiles is not the same as a needle a campaign notices:
`mise run //package/git-policy/cli:native:mutation:planted` is what proves that.

## A filtered run counts only when it ran the tests you meant

A `cargo test` filter that matches no test prints `ok. 0 passed`,
which proves nothing.
A filtered run counts only when its `test result` line names more than zero passed and the tests you meant,
so filter by the module path libtest prints (`service::service_instance::tests`),
never by a file name.
On 2026-11-26 two merge checks named `worktree_landed_after_a_merge::` and `binary::`,
which match no test in the library target,
and passed nothing until they were run again with filters that matched.

## A doc fence holds TypeScript pseudocode, because no task runs doc tests

A fenced block inside a Rust doc comment holds TypeScript pseudocode under
`In TS you'd write (pseudocode):`,
as every other doc fence in this crate does,
and never a Rust example.
The gate's test step runs `cargo nextest`
(`bin/native-test-containerfile.ts`,
`runArguments`),
which runs no doc test,
and no task in this package's `mise.toml` passes `--doc`,
so a Rust example in a fence is compiled by nothing the gate runs.
An assertion a doc example states belongs in a control beside the code,
where the gate runs it.
A fence written without a language label is read as Rust by the repository's linter,
which then asks for rustdoc on the file and on each `use` in it
(`package/linter/monochromatic-lint/src/processors_fences.rs`,
`fences`).

## A doc fence is closed, and nothing reports one left open

Every doc fence is closed on the line after its last code line,
and no rule reports a fence that is never closed:
CommonMark runs an unterminated fence to the end of its container,
so the lines after it are code to every Markdown rule,
and a fix has nothing to say about the missing closing line.
The repository's configuration and `monochromatic-lint` `0.5.2-alpha.24`
reported nothing for the `//!` fence with the info string `ts` in
`policy/policy_forbidden_strings.rs`,
which stayed open from `8ac11438b8` (2026-11-26T19:44:23-05:00) until this round closed it,
while a break planted in the same source was reported in the same run (2026-11-27).
So count the file's doc fence lines before a commit:
an odd count means one is open.

## Search the open branches before deleting anything as unused

Pieces of later steps are built ahead and live on branches until they merge,
so a function,
type or file that nothing on `main` calls may still be called on a branch.
Before you delete one as unused,
search every branch on origin for its name
(`git grep <name> $(git for-each-ref --format='%(refname)' refs/remotes/origin)`).
`1f5aa01430` (2026-11-25T17:25:36-05:00) deleted `the_account_cache` as uncalled,
and step 17's branch called it.

## Format with the root `rustfmt.toml` before each commit

The root `rustfmt.toml` sets the layout,
and this crate was formatted by it in `00b6a940b4` (2026-11-26).
Run `cargo fmt` in the crate before each commit,
and end with `cargo fmt --check` printing nothing.

## Run the whole `source_guard` module before you report

A round that changes Rust under `src/native/` also runs the whole `source_guard` module before it reports
(`cargo test --locked --offline --lib --features test-only-switches source_guard`,
in a capped scope),
and names its `test result` line.
Those tests read the release code for patterns no module test sees,
such as a thread started off the async runtime.
On 2026-11-24 a merged branch broke `the_release_code_keeps_blocking_work_off_the_runtime`
because only its own module's tests had run.

## No test on the host opens `/dev/zfs`

This machine's home is on a ZFS pool,
and a test that opens `/dev/zfs` on the host acts on that pool.
Before you run a module on the host,
grep it for `OpenedDevice::open(`,
`ZfsDevice::open(`,
`LiveGate`,
`GatePaths::live`,
`before_requests(` and `preflight_the_add(`:
a test that reaches one of them opens the device.
Such arms run in a guest of the round's own,
on a file-backed pool,
and the host run's `--skip guest --skip _arm_tests` keeps the named ones out.

## A rewritten guest driver runs beside `main`'s version

A rewritten guest driver is proved by running it in its guest against `main`'s version,
and comparing the lines that say what an arm did:
the two are the same wherever the rewrite changed nothing.
That reading found the start probe's shared header broken since `dcf5877982`:
an unquoted handler assignment ran the wrapper as a command,
so every arm after `green` stopped at its fourth line,
in `main`'s driver and the rewritten one alike.
It is quoted now,
and those arms run.

## A round runs no whole container gate

A branch merges into `main` when its round has ended and its report is read:
no whole container gate and no review is a condition of the merge
(`doc/decision/wrapper-merges-and-publishes-do-not-wait-for-reviews-or-mutation-runs.md`),
and one whole gate runs on `main` for each batch of merges.
So a round runs the modules it touched,
its guest arms where its brief asks for them,
and the controls its brief asks for,
red and green,
and says in its report which checks ran and which did not.
A round that adds or renames a file naming `podman` or `docker` as a program also runs
`mise run //package/container-limit/core:test:unit`,
whose `src/container-start-sites.unit.test.ts` lists every such tracked file:
it fails until the new path is listed with the role it plays,
and the `advisory-core` workflow that runs it does so only when its admission gate lets a push to `main`
through,
so a merge can land without it (issue 6588).

A round that writes or moves a line behind a feature name runs that feature's
`mise run //package/git-policy/cli:native:clippy:<feature>`
as well as the plain `mise run //package/git-policy/cli:native:clippy`
(`cargo clippy --locked --offline --all-targets -- -D warnings`),
because the gate's Clippy step and every plain run compile without a feature
(`bin/test-native-container.ts`),
so nothing else lints a line only that build has.
The controls' test-only switches take
`mise run //package/git-policy/cli:native:clippy:test-only-switches`
(`cargo clippy --locked --offline --all-targets --features test-only-switches -- -D warnings`),
the start probe's raised deadline takes
`mise run //package/git-policy/cli:native:clippy:start-probe-raised-deadline`
(`cargo clippy --locked --offline --all-targets --features start-probe-raised-deadline -- -D warnings`),
and the fence's measurement routes take
`mise run //package/git-policy/cli:native:clippy:measurement`
(`cargo clippy --locked --offline --all-targets --features measurement-routes -- -D warnings`).
On 2026-11-27 the switch task failed alone on an unused re-export in `foundation/lazy_runtime.rs`,
whose only namer,
`testing/no_switch_is_read_without_its_feature_tests`,
is compiled out where the feature is on (issue 6570);
the raised-deadline task,
added the same day,
failed alone on a test-only import
and a test-only constant that a build with the feature names nowhere
(`start_probe/probe_deadline_tests.rs`,
issue 6575),
and the measurement task was green with nothing to fix.
