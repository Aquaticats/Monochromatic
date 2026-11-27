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
two such controls of `landing::service_work`, in one process, answered `1 passed; 1 failed` in each of three runs,
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
`drain_pending_awaiting` and `drain_the_slow_work_awaiting` take it,
the shipped round reads the machine once and hands that one reading to every batch it drains,
and a control that needs two items beside each other states a host of its own
(`A_HOST_THAT_BOUNDS_NO_BATCH`),
as a control whose subject is a batch that keeps its turn states one that puts a single item at a time
(`A_HOST_THAT_RUNS_ONE_ITEM_AT_A_TIME`),
both in `landing/service_work_items_side_by_side.rs`.
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
before the number was there, two tests that passed one name shared one folder in one process,
and each deleted the other's live folder:
`lock::index_writer_forward`'s two signal tests did,
failing with `DirectoryNotEmpty` and with a `create_dir` panic at `test_support.rs:332`.
The binary test target's own helper names its folders the same way
(`native-binary-<process id>-<the number this call took>-<name>`,
`binary/binary_support_fixture_directories.rs`).
Never write a fixed path outside the fixture root (no `/tmp`, no shared name under the build folder).

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
and nowhere else: `ChildTest` is the request (`started` hands back the running child,
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
A child whose ending is the assertion, such as one that dies by a signal, takes `finished`
and judges the ending itself; a child a control keeps and talks to takes `started`,
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
Code under test bounds its waits too: every read of a file a person or another program controls opens it so it cannot
block (not a named pipe, not a device; the type checked through the open handle, never by a path check before the open)
and has a size bound (issues 6415 and 6433).
No blocking work runs on the commit's own thread or on the async runtime (`source_guard`'s runtime guard; issue 6421).

A control that needs two file times to differ states the difference instead of hoping the clock moved between them
(issue 6520:
the copy compare stamp control wrote the source 0.34 to 0.79 ms after taking its stamp,
this host's ZFS takes a write's change and modification times from the kernel's coarse clock,
and that clock advances once per 1 ms tick,
so the write left the stamp exactly as it was and the final check rightly passed in 6 runs of 6).
`stamped_pair` gives its source a modification time `RECENT_CHANGE_NANOS` before now for that reason,
and the measurement is in `doc/troubleshooting/openzfs-file-times-advance-in-one-millisecond-steps.md`.

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

The search a merge owes is wider than that grep:
a control may also read a source file while it runs
(`std::fs::read_to_string` of a path under `src/native`).
Four test files outside `source_guard` do that as of 2026-11-27.

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
found one stale control:
22 test files `include_str!` a source file (65 file-and-needle pairs)
and 4 more read one at run time outside `source_guard`
and the rest are in `source_guard` whose whole module runs.
Each of those 26 modules ran once and threaded
and only the control above was stale.

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

A planted control names one file and one needle in it,
and a needle goes stale without anything renaming it:
a split moves the guard into a sibling file,
or `cargo fmt` reflows the expression onto other lines.
`bin/native-planted-controls.unit.test.ts` is what reports that,
so it names the control and the file whose text no longer occurs exactly once;
seven controls were stale on 2026-11-27 and the source walk's crash hid every one of them.
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
