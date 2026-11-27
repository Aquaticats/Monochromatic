<!-- Copied from the audit's own ledger on 2026-11-25; absolute paths and its scratch scripts were removed for the public repository. Where each item is being handled is in the section "Routing, 2026-11-25" at the end. -->

# Git policy CLI debt ledger, crate monochromatic-git-policy-cli, 2026-11-25T16:45:00-05:00

Status: incomplete, unapproved and not final.
One Claude Code session (Sonnet 5.5) wrote it from read-only measurement.
No independent reviewer has read it, and no build, test or linter was run.
What was not checked is listed in the section "Not checked and limits".

## How to read this

The crate is `package/git-policy/cli`.
Every path in an item is relative to `B`, which is `package/git-policy/cli/src/native`,
unless it starts with `package/` or `doc/` (repository root) or is absolute.
Line numbers are from the main checkout's working tree, read between 2026-11-25T16:07 and 2026-11-25T16:50 (-05:00).
`main` moved from `d739037e45` to `a849bb436a` meanwhile, and only the practices guide changed in the crate
(measured: `git diff --stat d739037e45 a849bb436a -- package/git-policy/cli`).

"Measured" means a command or script whose output I read; the command is named.
"Inferred" means from reading code, with no run.
Scripts, raw outputs and lists are in the audit's scratch folder (not kept)
(called (audit scratch, not kept) in this ledger).

Release against test code is decided by declaration, not by file name:
(audit scratch, not kept) marks a file test-only when its declaring `mod` carries a `cfg` with `test`,
or when it sits in `testing/`, `binary/` or `source_guard/`, or is a `*_tests.rs` sidecar.
That leaves 1,524 release files and 1,479 test-only files of 3,003
((audit scratch, not kept), (audit scratch, not kept)).
Counts called "release" in this ledger use only those 1,524 files.

Two scripts in (audit scratch, not kept) are superseded and must not be trusted:
(audit scratch, not kept) filed the binary controls under the wrong prefix (seven of its nine hits were false),
and (audit scratch, not kept) sees only one file at a time.
(audit scratch, not kept) is the corrected one, and its raw output still holds false rows
(command names and paths), so each hit named in item 4 was confirmed by reading the helper chain.

Order: harm to correctness first, then how many sites one fix covers.
Items marked "release" are shipped code; "tests" are test code and test support.

## Fix rounds, in the order that suits the dependencies

Round A, test harness core: items 4, 5 and 11.
Land the branch of item 6 first.
One agent, because all of it is `testing/test_support.rs`, `testing/child_test_process.rs`
and the `binary/binary_support_fixture_*.rs` files.
Round B, process-wide state in tests: items 3 and 10, after round A (it needs the child runner).
One agent per mechanism (3a to 3g), because the files are disjoint.
Round C, release reads: items 1 and 2, independent of the others.
Item 2 starts with a source guard, then migrates in classes.
Round D, waits and bounds in tests: item 7, one agent per folder.
Round E, owner decision: items 8 and 18, then a conversion round.
Round F, mechanical duplication and dead code: items 12, 13 and 14, independent of the others.
Round G, file splits: item 15, one agent per file; item 16 only after the owner chooses a direction.
Round H, SPEC edits: item 9, a few passages.

## Items

### Item 1. A pathspec file is read by path before Git reads it, and one read has no bound

Category 3, release.
Rests on `commit/commit_selection_staging.rs:383-389` (`selection_elements`) and `:403-415` (`selection_items`),
which take the size with `std::fs::metadata(file)` and then call `std::fs::read(file)`,
and on `commit/commit_index.rs:518-521`, which calls `std::fs::read(file)` with no size bound at all.
The path comes straight from the command line: `landing/service_prepare_changes_config_and_pathspec_steps.rs:158-166`
returns any name other than `-` unchanged (joined to the working directory).
The preparation that reads it (`landing/service_prepare.rs`) takes the command's standard input as an input,
so I infer it runs in the wrapper process and not in the service (not confirmed).
The shared opener that takes the type from the open handle exists: `foundation/plain_file.rs:164` `read_plain_file`
and `:188` `read_plain_file_through_links`.

What is wrong: `metadata` follows links and reports length 0 for a named pipe, so the size check passes,
and `read` then opens it blocking.
The practices guide section "Every wait has a deadline" forbids that shape.
Baseline: native Git also waits on a pipe with no writer, so the hang is no worse than Git's;
the owner's rule is stricter (open issues 6415 and 6433 apply it to other files).
The sharper point is inferred and not run: a one-shot input such as bash process substitution
(`--pathspec-from-file=<(...)`) is consumed by this early read, so Git later reads end of file from the same path,
which native Git would not do.

Cost: a different commit result from native Git for one-shot inputs, and an unbounded read at `commit_index.rs:520`
(risk; not seen failing).
Issue 6415's branch `fix/git-policy-cli-named-pipe-in-a-record-folder` holds only the merge commit `c81f8c7743`
so far (measured: `git log main..<branch> --oneline`).

Fix: read the file once, at most `PATHSPEC_FILE_LIMIT` bytes, through the open handle, into the scratch snapshot
and hand Git the snapshot, as the `-` route already does at `pathspec_file_for`;
whether a pipe is refused or its bytes are snapshotted is the owner's choice.
One function for the three sites, and a control with a `mkfifo` fixture
(`binary/binary_abort_during_an_operation_tests.rs:1195` already makes one).
Size: three read sites, one function.
Label: measured by reading (`rg -n "pathspec_file\b"`); the process-substitution effect and the client-side run
are inferred.

### Item 2. Raw file reads bypass the plain-file helper, and the same read is written many times

Categories 3 and 4, release.
Measured: `rg -n "std::fs::read_to_string\(|std::fs::read\(|fs::read_to_string\(|fs::read\("` over the release files
finds 181 sites in 119 files; 138 remain after dropping `/proc`, mount table, boot id and cgroup paths;
the helper `foundation/plain_file.rs` has 74 call sites in 55 files
(`rg -c "read_plain_file\(|read_plain_text\(|read_plain_text_at\(|read_plain_file_through_links\(|open_plain_file\("`).
At least twenty of the raw reads open files under a Git directory or a state folder
(a variable-name match: (audit scratch, not kept)).
Examples that a person or another program can replace with a named pipe:
`repo_read/repo_read_config_scan.rs:66` (config), `repo_read/git_directory_roots.rs:37,65` (`commondir`, `.git` file),
`landing/service_head.rs:123,140` (`commondir`, `HEAD`), `landing/service_work_common_directory.rs:57,67,93`,
`push/push_failure_notice.rs:509` (`HEAD`), `merge/merge_second_pass_finish.rs:412` (rebase todo),
`merge/merge_isolation_git_reads.rs:271` and `merge/pull_isolation.rs:599` (`FETCH_HEAD`),
`worktree_clone_add_route/add_fill.rs:281,289`, `service/service_review.rs:76`,
`worktree/worktree_remove_hand_over.rs:1168` (`.git` pointer).

The `commondir` read alone is written seven ways.
`foundation/common_directory_of_a_git_directory.rs:35` is the shared answer (11 callers, itself a raw
`read_to_string`), and five more raw reads of the same file exist:
`repo_read/git_directory_roots.rs:37`, `landing/service_work_common_directory.rs:93`, `landing/service_head.rs:123`,
`service/service_review.rs:76`, `worktree_clone_add_route/add_fill.rs:289`;
`repo_read/repo_read_layout.rs:310` goes through a local `small_file`
(`repo_read/repo_read_layout_disk_and_directory_facts.rs:65`).
Bounded-read bodies are copied: `invocation/real_git_candidate.rs:342` `read_up_to`,
`worktree/worktree_clean_content.rs:84` and `account_companion/companion_mount_table.rs:166` `read_at_most`,
23 hand-written `take(..).read_to_end` sites in 22 files ((audit scratch, not kept)),
and 14 hand-written `custom_flags(... O_NONBLOCK ...)` opens in 12 files ((audit scratch, not kept)).

Cost: the hang of item 1 in each place, plus unbounded memory; native Git also blocks on a pipe at several of these,
but two open issues (6415, 6433) already treat the same shape as defects here.
Many of the 138 read files the wrapper itself wrote into a private folder, which is lower risk (inferred);
triage is part of the fix.
Fix: first a source guard in `source_guard/` (in the style of `no_test_changes_the_process_directory_tests.rs`) that
refuses `std::fs::read*` on non-`/proc` paths outside `foundation/plain_file.rs`, with an allow-list that shrinks;
then migrate in classes (Git-directory reads, record folders, user paths);
make `common_directory_of` the one `commondir` reader.
Size: 138 sites in about 100 files; 6 `commondir` copies; about 40 copied bounded-read or open bodies.

Round of 2026-11-25, branch `fix/git-policy-cli-raw-file-reads-through-the-plain-file-reader`:
the guard landed as `source_guard/raw_file_reads_tests.rs`,
and its first measurement was 151 calls in 101 release files
(159 calls in 107 files by text, less the six test-only files the declaration scan leaves out).
Seventeen sites in eleven release files moved onto `plain_file`:
`commondir`,
`HEAD`,
the `.git` pointer file,
`FETCH_HEAD`,
a rebase or sequencer todo list,
and a configuration file,
with `foundation/common_directory_of_a_git_directory.rs` as the one `commondir` reader.
What is left is on `KEPT_SITES` of the guard: 134 calls in 91 files, in five named classes.
The classes for later rounds: the system's own facts (`system`),
the wrapper's own records (`own-record`),
the remaining Git-directory reads (`git-directory`: the index, pack files, `MERGE_MSG`, `sequencer/opts`,
a registration's `gitdir`, `.git/info/exclude`),
paths the person's project holds (`person-path`),
and the copied bounded-read or non-blocking-open bodies
(`invocation/real_git_candidate.rs`'s `read_up_to`,
`worktree/worktree_clean_content.rs`'s and `account_companion/companion_mount_table.rs`'s `read_at_most`,
and `foundation/private_storage.rs`'s `read_regular_file`, which `repo_read` and four other folders still name).

### Item 3. Tests change process-wide state, which only a process per test makes safe

Category 1, tests.
The tests were written for nextest's one process per test; the container gate does that,
and a threaded `cargo test --lib` (how rounds run) does not.
Nineteen comments lean on it ((audit scratch, not kept)).
The seven mechanisms, each with its sites, are one item because one child runner (item 10) fixes them.

Item 3a, the durable-sync switch.
`foundation/private_storage_syncs_test_support.rs:21,34-36`: `SYNCING_IS_FAITHFUL` is an `AtomicBool`;
`skip_the_durability_syncs_here` stores false and nothing stores true.
Six calls: `worktree_copying/worktree_copy_lifecycle_tests.rs:1035`, `foundation/private_storage_tests.rs:379`,
`foundation/earlier_names_folded_into_current_tests.rs:121,157`,
`foundation/earlier_log_lines_added_to_current_log_tests.rs:113,383`.
`foundation/private_storage_tests.rs:372-380` asserts that a faithful flush of a pipe is refused
before it calls the skip, so it fails when any of the other five ran first in the same process (inferred from the code).
After the flip every sync in other tests is a no-op (`foundation/private_storage.rs:357,682`,
`foundation/earlier_log_lines_added_to_current_log.rs:299`), so crash-order controls test less than they say.

Item 3b, the descriptor limit.
`testing/test_support.rs:609-658` (`set_soft_limit`, `restore_limit`, `leave_free`) lowers `RLIMIT_NOFILE` of the
whole process, and `leave_free(0)` fills every remaining descriptor,
so a concurrent thread's `open`, `pipe`, `socket` or spawn gets `EMFILE` meanwhile.
Eight sites in four files: `process/child_awaited_through_a_pidfd_descriptor_limit_tests.rs:85,119,152`,
`commit/native_commit_waiting_notice_tests.rs:330,410`, `worktree/worktree_clean_tests.rs:97`,
`subvolume/fs_beneath_directory_chain_tests.rs:75,108`.
The restore is skipped when an assertion fires between the two calls (`fs_beneath_directory_chain_tests.rs:109`).

Item 3c, descriptor 2.
`landing/work_item_tests.rs:1001-1030` points fd 2 at `/dev/full`;
`repair/package_repair_command_tests.rs:949,995` points it at a pipe filled to capacity and never drained;
`worktree/add_standing_notice_tests.rs:209,291` points it at a pipe.
Under libtest's default capture a test thread's `eprintln!` and panic message go to the capture buffer and not to fd 2,
so the redirect hits what writes fd 2 directly: the watchdog's report (`testing/run_progress.rs:830-834`),
the service log, children started meanwhile (they inherit it), and every `--nocapture` run.
A write to the drained-never pipe blocks, and the watchdog's own write would block before its `abort()` (inferred).

Item 3d, the environment.
`worktree/worktree_clone_conversion_size_read_tests.rs:700-713` sets five `GIT_CONFIG_*` variables and removes them
only if the read returns, and every Git child of other tests inherits them;
`:1217` sets `CLI_GIT_TEST_ONLY_RUNTIME_BUILD_MILLIS=1500` and never removes it
(read at `foundation/lazy_runtime.rs:148`, longer than the one-second build deadline).
The safe form exists: `worktree/worktree_ladder_target_tests.rs:397` and
`worktree/worktree_clean_content_safety_tests.rs:470` set the variable only in a child.
Both sites of the size read's module run in a child process of their own since issue 6493,
so the variables reach no other test of the run.

Item 3e, the umask.
`landing/service_index_claim_tests.rs:398-402` sets `0o077` in the test process and restores it only on success.
The safe forms: `testing/umask_child_test_support.rs`, `binary/binary_merge_mode_tests.rs:55-95`.

Item 3f, signal dispositions and in-process signals.
`service/service_client_interrupted_tests.rs` (sends at `:53,104,156,185,269,274`, nine `sigaction` hits),
`process/signal_report_tests.rs:48-59` (replaces the `SIGTERM` handler), `process/signal_relay_tests.rs`
(its `RELAY_STATICS` mutex at `:302` serialises only that file) and `lfs/lfs_upload_tests.rs:427` (item 6)
all claim `SIGTERM` or `SIGINT` in one process with no shared lock.
`service/service_client_interrupted_tests.rs:104` is the process-directed shape of issue 6463:
`kill(getpid(), SIGINT)` reaches any thread, and the binary dies if another test has put the default action back
at that moment (inferred, not run).

Item 3g, library statics that tests arm (a lower bound).
`service/service_client_reach_service_tests.rs:95` `MANAGER` is shared by 14 tests and set under a lock that is
released before `reach` runs (the file's own header says so);
`service/service_client_wake_tests.rs:28` `STARTS_ASKED` is reset and asserted by two tests;
`testing/fixture_git_commands.rs:118` `ENDING_SEAM` is armed by one test and fired by the first bounded query that
reaches its bound in any thread (low risk);
(the seam sits in `testing/fixture_git_query_ending.rs` since the split of 2026-11-27);
the service stop state is process-wide (comments at `push/bounded_push_awaiting_tests.rs:95`,
`process/stop_of_the_service_noticed_by_its_children_tests.rs:6`, `process/signal_relay_tests.rs:692`).
The uncapped scan finds 70 `Mutex`, `Atomic` or `OnceLock` statics in 39 test-only files
((audit scratch, not kept)); I read the ones named here and not the rest,
and several carry an `ONE_AT_A_TIME` mutex.

Round of 2026-11-25:
issue 6478 is fixed and merged,
a descriptor-room control of 3b whose count another thread can change,
(`3ce0c713e9`,
its test named for the one thread it measures),
and five threaded-only failures are filed and open:
6483 (the content-safety control misses a same-tick write),
6484 (two heartbeat controls share one manager switch),
6485 (two supervisor controls plant kills in one process-wide registry),
6486 (four `run_progress` controls read the shared test process) and
6487 (the process-wide blocking room in `foundation`).

Round of 2026-11-26,
branch `fix/git-policy-cli-issue-6493-size-read-tests-share-one-process`:
issue 6493 is fixed,
the size read's module whose twenty-two controls armed or read one of the six pieces of state above
(the add timing record, the failed-step note, the repository's holder record, the size read's holds and free
counters, the watched run's clear hold, and the process's own environment, runtime and fork);
each of them now runs its body in a child process of the test binary
(`testing/process_wide_state_child_test_support.rs`,
the one helper the 6487 round below landed),
and the size read's holds and the watched run's hold refuse to be set outside such a child;
the measured reds and greens are in
`doc/troubleshooting/git-policy-cli-controls-that-arm-state-the-whole-process-shares.md`.

Round of 2026-11-26,
branch `fix/git-policy-cli-issue-6487-one-process-wide-room-for-blocking-jobs`:
issue 6487 is fixed.
The room a control gives a process,
and the bound on blocking jobs it holds,
are read by the one function a process
plans with (`runtime_plan::workers_for_this_process`),
so `budgeted_workers` and `plan_runtime` are functions of the host and limits a caller states again.
Every control that sets either owns a child process of its own (`testing/process_wide_state_child_test_support.rs`),
and the setters refuse anywhere else,
naming the starter.
`cargo test --locked --offline --lib --features test-only-switches foundation:: -- --skip guest --skip _arm_tests`
answered `test result: ok. 394 passed; 0 failed; 8 ignored` at `3774b4a0cc`,
from 335 passed,
16 failed and a deadlock at `408abb6c0e`.
Mechanisms 3a and 3g,
and the queue and counters this item lists,
lost their `foundation` sites to the same child
and stay open where they reach other modules:
`subvolume::subvolume_index_files::parallel_tests` fails 4 of 12 threaded,
where `main`'s `e58ba6f078` fails 6 of 12,
`service::service_server_connection_handling` fails 3 of 13 on both trees,
and `worktree_copying/worktree_copy_lifecycle_tests.rs` keeps the sixth `skip_the_durability_syncs_here` site in the
shared process.
Status:
incomplete,
unapproved and not final;
the owner declares completion.

Round of 2026-11-26:
6484 (two heartbeat controls share one manager switch) is fixed and merged,
and that file's four statics are gone:
each control writes the word its fake manager answers into the folder its own watch names as a session bus,
which is the first form this item asks for ("give each test its own instance"),
and `source_guard/heartbeat_controls_own_their_manager_switch_tests.rs` refuses a `static` in that file.
The item's other sites stay as measured.

issue 6486 is fixed and merged,
the `run_progress` controls of the round above
(`673512f65d`,
four the issue names,
and `c75f87c23a` for `the_look_sees_this_process_and_the_processes_it_started` beside them,
which the first threaded run after the merge failed with 21 members before its child and 21 after it),
each one reading its figures in a helper process of its own through `testing/helper_process_test_support.rs`,
as 6478's descriptor-room control does,
with one shared comparison that refuses a look where a task which is not the control's own moved,
and three threaded-only failures are filed and open:
6483,
6485 and
6487.

Round of 2026-11-26,
issue 6496:
the child guard of two `ran_in_child` controls was inverted
(`subvolume::subvolume_index_files::parallel_tests`,
`a_report_that_says_an_index_could_not_be_started_leaves_no_job_that_can_rewrite_it`,
and `subvolume::subvolume_index_refresh_of_every_worktree`,
`an_index_rewrite_reported_as_never_started_does_not_run_afterwards`),
so the process that started the child ran the body as well and lowered the whole run's `RLIMIT_NPROC` to one:
three other controls of the module failed at their fixture's first Git start with `EAGAIN`,
and the inverted control itself failed because that process's blocking pool already had a thread.
Both now return in the process that started the child,
and the setters refuse everywhere else and name the starter:
`refuse_threads_again`,
`let_threads_start_again`,
and the syncs' count (`take_sync_counts`,
mechanism 3a),
whose readers now own a process each (the two of this module and the two of `lock::single_owner_claim`),
as this item's fix asks;
the control that holds the process's one place in this module owns a process too.
`cargo test --locked --offline --lib --features test-only-switches
subvolume::subvolume_index_files::parallel_tests -- --skip guest --skip _arm_tests`
answered `test result: ok. 12 passed; 0 failed`,
from 6 passed and 6 failed on `main` at `f9096f7531`,
and `lock::single_owner_claim::tests` answered `ok. 16 passed; 0 failed`,
from 15 passed and 1 failed.
The `hold_the_only_place` sites of other modules stay as measured:
`worktree::worktree_clean::tests` fails
`the_tracked_files_are_proven_as_a_job_of_the_processs_blocking_pool`
in three of four threaded runs on `main` itself.

Round of 2026-11-26,
branch `fix/git-policy-cli-issue-6498-worktree-copy-control-turns-syncs-off-for-the-process`:
issue 6498 is fixed,
and mechanism 3a has no site left.
The switch's last five callers in the tests
(`worktree_copying/worktree_copy_lifecycle_tests.rs`) ran the control bodies that set it in the shared process,
and `skip_the_durability_syncs_here` now refuses any process that is not a child of the control's own
(`testing/process_wide_state_child_test_support.rs`),
as the room and the bound do;
the one control that is the refusal is `foundation::private_storage::tests::the_durability_switch_refuses_a_process_that_is_not_its_child`.
Measured on this host,
two controls of this item in one process with `--test-threads=1` and libtest's `-Z unstable-options --shuffle-seed`:
`worktree_copying::worktree_copy::tests::a_refused_add_leaves_no_record_for_the_worktree_another_command_completed`
beside `foundation::private_storage::tests::directory_sync_needs_the_directory`
answered `1 passed; 1 failed` at seed 1 and at seed 3,
where the second control's refused `sync_directory` of a missing folder answered `Ok`,
and `2 passed; 0 failed` at seeds 0 and 2;
after the change seeds 1 and 3 answer `2 passed; 0 failed`.
`cargo test --locked --offline --lib --features test-only-switches foundation:: -- --skip guest --skip _arm_tests`
answered `test result: ok. 395 passed; 0 failed; 8 ignored`,
and `worktree_copying::worktree_copy::` answered `test result: ok. 47 passed; 0 failed; 0 ignored`.

Round of 2026-11-26,
issue 6499:
nine controls of `service::service_review::tests` called `review_item`,
which runs the review in the one runner the whole service process shares
(`standards_review_run::RUNNER`),
and a runner admits one job and answers every other caller with `Contained::Busy`,
so a threaded run of the module failed seven of its sixteen controls in each of two runs,
with a refusal that an earlier review is still running,
and a varying set of them (the issue measured eight on `main` at `f9096f7531`),
while all sixteen passed with `--test-threads=1`.
Each control now declares `static OWN: Containment`
and calls `review_item_in(&OWN, ..)`,
which `service/service_review.rs` documents for controls
and which the module's two other controls already did;
nothing of the release changed.
`cargo test --locked --offline --lib --features test-only-switches service::service_review::tests -- --skip guest --skip _arm_tests`
answered `test result: ok. 16 passed; 0 failed` in three runs after the change,
from `9 passed; 7 failed` in both runs before it,
and each of the nine controls passed alone with `--exact` before and after.
`source_guard/no_control_takes_the_production_runner_tests.rs` refuses a test-only source that calls
`run_contained(`,
`review_item(`,
`review_item_within(`,
`review_item_awaiting(`
or `review_item_within_awaiting(`,
and with the fix taken out it named the twelve such lines of `service/service_review_tests.rs`;
the narrower `the_controls_share_no_production_runner` of `standards_review_run_tests.rs`,
which covered one face in one file and claimed these controls hold the shared runner for one review only,
went with it.

Round of 2026-11-26:
issue 6497 is fixed,
the three heartbeat controls of `service/service_server_connection_handling_tests.rs` that failed together in one
process (`a_commit_sent_over_a_connection_is_beaten_while_its_directory_check_waits`,
`a_commit_is_beaten_while_the_note_read_waits`,
`a_note_read_that_outlives_its_beat_is_not_started_again`).
The holds a control arms at a fault point stood in one slot of the whole process
(`service/service_fault_hooks_test_support.rs`),
so a second arming dropped the first control's hold and a hold one control armed was taken by another control's read.
The holds are a list now,
and a point names the instance its own control owns in the point text,
as `collection_point` already did for a repository's scratch collection:
`instance_read_point`,
`note_read_point`,
`read_at_a_start_point` and
`next_service_note_point`.
The registry refuses a second live hold of one point and names the composition the control must make,
and `service/service_fault_hooks_test_support_tests.rs` holds both controls:
two holds of one process each taken by its own point,
and one point armed twice refused.
Two further sites of the same mechanism were measured and fixed with it:
the two controls of `landing/service_claim_watch_note_within_a_ceiling_tests.rs` and the three of
`landing/service_land_watch_note_tests.rs` run their bodies in a child of their own,
because the place of the one note write of a process
(`A_NOTE_WRITE_IS_OUTSTANDING`, a shipped static)
and a control's refusal of a thread belong to the whole process:
before the change,
two runs of `landing::service_claim_watch_note_within_a_ceiling::tests` answered `1 passed; 1 failed`
(`left: AnEarlierWriteHasNotEnded`),
and `landing::service_land::watch_note_tests` answered `1 passed; 2 failed`
with the write's reach never awaited.
Measured after:
`service::service_server_connection_handling::tests` answers `test result: ok. 13 passed; 0 failed`,
from `FAILED. 10 passed; 3 failed` at `3a88fa9ce5`;
`service::service_fault::hooks::tests` `ok. 2 passed`;
`landing::service_claim_watch_note_within_a_ceiling::tests` `ok. 2 passed`;
`landing::service_land::watch_note_tests` `ok. 3 passed`;
`service::service_journal_restored_at_start::tests` `ok. 3 passed`;
`service::service_server::greets_before_the_journal_is_restored_tests` `ok. 1 passed`.
Two controls of one process that name one point,
and a control that arms its point in a process of the whole run,
are the two shapes this item still meets elsewhere:
the registry's refusal names both at the arming that would hide them.

Round of 2026-11-26,
issue 6489:
a control of `landing/service_round_tests.rs` asserted a stall the release no longer has,
and the whole container gate on `main` at `6543be0d1a` failed it after 10.016 s.
The control left a named pipe at a worktree's `commondir`
and spent ten seconds opening it for a writer,
to end a read it believed was waiting there,
then asserted that the round's lane read had outlasted its ceiling.
`98f943d61a` had already moved that file's reader onto `foundation/plain_file.rs`,
which opens without blocking and takes the type from the open handle,
and the gates at `2005f92347` and earlier ran the same control without reporting it,
so the read answered at once and the ten seconds were the control's own wait.
Reproduced here with the control as the issue found it:
`0 passed; 1 failed` in 10.01 s alone with `--exact`,
and its sibling of the same file `1 passed; 0 failed` in 10.11 s,
which is the same ten seconds of waiting with an assertion loose enough to pass.
The place the drain's step needs is now held by a job of the control's own,
awaited no longer than a ceiling and let go on a channel the control owns,
and the control owns a process of its own for the bound it holds,
as the setters of this item now require.
Its sibling asserts what the release does,
that a pipe at `commondir` answers at once with the directory's own lane;
with a blocking read put back in `common_directory_of_a_git_directory.rs`
the sibling answered `0 passed; 1 failed` in 30.01 s,
so that control is the guard a blocking `commondir` read has to pass.
Measured after:
`cargo test --locked --offline --lib --features test-only-switches landing::service_round:: -- --skip guest --skip _arm_tests`
answered `test result: ok. 12 passed; 0 failed; 0 ignored`,
and each changed control alone answered `1 passed; 0 failed`.

Round of 2026-11-26:
issue 6495 is fixed by running one control's body in a child of its own
(`process_wide_state_child_test_support.rs`),
`worktree::worktree_clean`'s one-place control (`3a4f2ee6d8` and `8ba6a8c5be`).
Measured there:
a threaded module run has already made the budget with seven places,
so `hold_blocking_jobs_to(1)` arrived too late and the control held one place of seven;
the same control passed in other runs because another test's job happened to be waiting.
The child reads its bound before the work runs and fails when it is not the one place,
so a body back in the parent fails at once,
naming the cause,
instead of at the holder's ten-second patience.
That fixes the one site and not the shared helper:
`hold_the_only_place` holds the process's only place only where it is the process's first
blocking-job user,
which is issue 6477,
and the other call sites of it are unchanged.

Round of 2026-11-26,
branch `fix/git-policy-cli-issue-6505-queued-rewrite-controls-drain-every-shard`:
issue 6505 is fixed,
and the blocking pool's queue was not why those two controls measured nothing.
They are
`subvolume::subvolume_index_files::parallel_tests`'s
`a_report_that_says_an_index_could_not_be_started_leaves_no_job_that_can_rewrite_it`
and
`subvolume::subvolume_index_refresh_of_every_worktree::tests`'s
`an_index_rewrite_reported_as_never_started_does_not_run_afterwards`.
Measured:
with `RewriteEachIndex::run`'s claim check removed,
both answered `1 passed; 0 failed`;
with the claim the process's bound takes removed as well,
the second answered `0 passed; 1 failed` at `the rewrite the report gave up on ran afterwards`
(`left: 1`),
and a print inside the queued closure appeared twice,
so one awaited blocking job does drain the queue.
tokio 1.54.1's blocking queue is one `VecDeque` unless `TOKIO_UNSTABLE_SHARDED_BLOCKING_QUEUE` names a
value other than zero (`runtime/builder.rs`,
`sharded_blocking_queue_default`,
and
`runtime/blocking/pool.rs`,
which picks `LockedImpl`),
pushed at the back before a thread is asked for and popped at the front (`spawn_task`,
`run_worker`).
Two claims guarded the one queued closure,
so either could be removed unseen:
the process's bound takes one before that closure's first effect and takes it again when it gives up,
and the item took a second behind it.
The item's is gone and its answer reads the bound's,
so removing the bound's check fails the item's control (`0 passed; 1 failed`),
and not taking that claim when it gives up fails the whole-report control (`0 passed; 1 failed` at the
report's own words),
as does a job that runs where the bound puts it on the pool (`0 passed; 1 failed` each).
The whole-report control cannot reach a queued rewrite at all,
because the run's first job is the preparation and a refused thread ends the run there;
it now proves what that end can prove:
no index named,
the refused step reported as one that will not run,
and no rewrite after the queue is drained,
and it stays `1 passed; 0 failed` when the bound's check goes,
which is what its own face allows.
Status:
incomplete,
unapproved and not final;
the owner declares completion.

Round of 2026-11-26,
issue 6507:
the preparation's named steps are one ledger of the whole process
(`commit/prepare_work_spans.rs`),
noted by every preparation and emptied by every timing record,
and five controls read it:
the issue's `commit::commit_index::tests::every_selection_mode_states_its_one_read_and_its_seed_walk`,
the three of `commit::prepare_work_spans::tests`,
and `commit::commit_timing::tests::the_record_states_the_preparations_own_work_by_name`.
Each runs its body in a child of its own now
(`testing/process_wide_state_child_test_support.rs`),
as the room,
the bound,
the durability switch and the sync counts do,
and `source_guard/no_control_reads_the_preparation_steps_in_the_run_tests.rs`
refuses a control that names the ledger without one.
The seven further controls of `commit_timing_tests.rs` that empty the ledger through `Timing::finish`
assert no step and stay where they are;
they can no longer take a step a control of a child owns.
Measured on this host:
two threaded runs of `commit::commit_index::tests` answered
`test result: FAILED. 31 passed; 1 failed` with `left: Some(2)` for `index read`,
and `ok. 32 passed; 0 failed` twice after the change;
one process holding `commit::commit_index::tests::explicit_path_stages_only_the_named_paths`
beside the three `prepare_work_spans` controls answered `1 passed; 1 failed`
(the empty start read five steps of the neighbour)
and beside the timing record's control `1 passed; 1 failed`
(four steps of the neighbour in its record),
and `6 passed` and `2 passed` after the change;
the issue's filter
(`index::`,
guest and arm tests skipped)
answered `ok. 230 passed; 0 failed` twice,
from `ok. 230` once and one run the run's watchdog aborted after 229 passed,
where no `test result` line was printed.
That abort is a second defect of the module and not this cause:
`commit::commit_index::tests::interactive_selection_succeeds_without_changes` runs `git add --interactive`,
which read the standard input the run inherits and printed its `What now>` prompt without ending;
the `git add --interactive` it started outlived the aborted run,
held the run's output pipe open and kept the capped scope alive for 21 minutes,
until that scope was stopped by hand;
run again after the change,
the verbatim command aborted the same way after 229 passed,
which is what says the change neither caused nor cured it;
it is recorded in `doc/troubleshooting/git-policy-cli-commit-index-span-control-counts-another-tests-spans.md`
and not fixed here.
Each of the five controls answered `ok. 1 passed` alone with `--exact` before and after the change.
The guard was shown failing with one control's child starter taken out:
it named `commit/prepare_work_spans_tests.rs:80 (a_step_of_no_measurable_time_is_stated_as_one_call)`,
and the same one-process pair read the neighbour's five steps again;
both were restored.
After merging `origin/main` (`68af165851`):
`cargo fmt --check`,
`cargo check --locked --offline --all-targets` and
`cargo clippy --locked --offline --all-targets -- -D warnings` answered nothing to fix;
the four module runs answered
`commit::commit_index::tests::` `test result: ok. 32 passed; 0 failed`,
`commit::prepare_work_spans::tests::` `ok. 5 passed; 0 failed`,
`commit::commit_timing::tests::` `ok. 21 passed; 0 failed` and
`source_guard::` `ok. 141 passed; 0 failed`;
and the locked linter (`monochromatic-lint` 0.5.2-alpha.15) found no `rust/folder-uses-links` and no
`rust/file-order-in-folder` finding over this package.
A second merge of `origin/main` (`6e3768d93a` held handover documents only) answered the same numbers.
Beside this cause,
the rest of the `commit/` folder is still red in a threaded run for the same shape in other ledgers the timing
record drains:
two runs of the whole `commit::` filter answered
`test result: FAILED. 464 passed; 10 failed` and `FAILED. 463 passed; 11 failed`,
and the failures are
`commit::commit_index_builds::tests` (3 followed by 4),
`commit::commit_prepare_part_durations::tests` (2 and 2),
`commit::commit_build_part_durations::tests` (1 and 2),
`commit::commit_preparation::tests` (2 and 2) and
`commit::native_commit_waiting_notice::tests` (2 and 2,
which may be the descriptor limit of mechanism 3b rather than a ledger).
The index-builds ledger alone failed 5 of its 11 controls in a threaded run of its own module,
and 3 of them in one process that had run one preparation first,
which is the shape this round fixed for the spans ledger.
This round fixed the ledger the issue named;
those sites stay measured and unfixed.
Status:
incomplete,
unapproved and not final;
the owner declares completion.

Round of 2026-11-26,
branch `fix/git-policy-cli-issue-6509-commit-ledger-controls-own-their-process`:
issue 6509 is fixed,
and the four ledgers the run above left open read the figures they were meant to.
Each control that counts the index builds,
the preparation's stretches,
the build's stretches or the step ledger runs its body in a child of its own
(`testing/process_wide_state_child_test_support.rs`),
as the room,
the bound,
the durability switch and the preparation's steps do,
and `source_guard/no_control_reads_the_commit_ledgers_in_the_run_tests.rs`
refuses a control that names one of them without a child.
The two causes of the same run that are no ledger are fixed beside them:
the two `commit::commit_preparation::tests` controls that assert this process has no descendant left own a
process too,
and every control that steps a notice takes the two control windows before it starts
(`source_guard/notice_controls_take_their_window_tests.rs`).
All three guards share one scan,
`source_guard/process_wide_fact_readers_tests.rs`,
which issue 6507's guard now uses as well.
Measured on this host:
two threaded runs of `commit::` answered
`test result: FAILED. 464 passed; 10 failed` both times before the change,
and the same baseline answered `FAILED. 467 passed; 7 failed` and `FAILED. 464 passed; 10 failed`
in two later runs,
so the count holds while which controls fail moves;
after the change `ok. 474 passed; 0 failed` in each of two runs;
the modules answered
`commit::commit_index_builds::tests` `ok. 11 passed` (from `4 passed; 7 failed`),
`commit::commit_prepare_part_durations::tests` `ok. 5 passed` (from `3 passed; 2 failed`),
`commit::commit_build_part_durations::tests` `ok. 5 passed`;
it answered `ok. 5 passed` alone before the change and failed only beside other modules,
so its count did not move,
`commit::commit_preparation::tests` `ok. 23 passed` (from `21 passed; 2 failed`),
`commit::native_commit_waiting_notice::tests` `ok. 19 passed` (the `19 passed` of `main` plus the window),
`transaction::transaction_step_durations::tests` `ok. 6 passed` (from `5 passed; 1 failed` in a threaded
run of its own module),
`repo_read::git_metadata::tests` `ok. 11 passed`,
`shadow::shadow_pool::tests` `ok. 29 passed` and
`source_guard::` `ok. 147 passed`.
The step ledger's own module was measured red in a threaded run of itself:
`a_slow_step_names_its_arguments_and_a_short_one_does_not` read no arguments for a step another
control's drain had taken.
`repo_read::git_metadata::tests` was `ok. 11 passed` alone before the change,
and its control owns a child because it reads the same ledger.
After `origin/main` made the starter take the control's body (issue 6503,
`run_the_body_in_a_child_of_its_own(module_path!(), name, || { .. })`),
each of the 28 guards hands its body to that call and the three guards excuse its name;
on the merged tree `commit::` answered `ok. 474 passed; 0 failed` in both runs,
`source_guard::` `ok. 151 passed`,
and each of the 34 controls `1 passed` alone with `--exact`.
A sixth fact of the same shape was measured after that merge,
in the first threaded run of `commit::`:
`commit::git_step_failure::tests::a_note_never_carries_a_path_or_a_line_break` failed with
`the note just written` (`FAILED. 473 passed; 1 failed`),
because `commit/git_step_failure.rs` keeps the note of the last failed Git step in one
`Mutex<Option<FailedStep>>` for the process,
and a control beside it cleared the note between this control's write and its read.
Its four controls hand their body to the child too,
the ledger guard gains that note as a fifth requirement,
and the module answers `ok. 4 passed; 0 failed` alone and `commit::` `ok. 474 passed; 0 failed` twice after it.
The causes and all three guards' failing proofs,
before and after that merge,
are in
`doc/troubleshooting/git-policy-cli-commit-index-span-control-counts-another-tests-spans.md`.
Status:
incomplete,
unapproved and not final.

branch `fix/git-policy-cli-issue-6514-started-as-git-control-reads-its-own-record`:
issue 6514 is fixed,
and it is a mechanism this item did not list:
a child inherits the chain record its parent holds,
because the record's memory file carries no close-on-exec flag
(`invocation/chain_starts_record.rs`),
and `invocation::started_as_git::tests::a_record_of_this_process_is_read_even_when_an_older_marker_sits_above`
ran its body in a child that also made a record of its own under the same name.
The lookup answers with the first descriptor carrying that name,
so the guard read the test process's record:
measured,
the inherited record at descriptor 3 named the parent and the control's at 4 named the child.
The body closes every record it inherited before it makes its own,
runs in a child of its own through `testing/process_wide_state_child_test_support.rs`,
and asserts that the record the lookup finds is the one it made.
Thirteen threaded runs of `started_as_git::` were made before the change
(seven failed the one control,
four passed,
two were cut off in the log),
and thirteen after it
(nine before the guard was taken out,
four after it was put back),
every one `test result: ok. 15 passed; 0 failed`;
`invocation::` answered `394 passed; 0 failed` in three runs after the change,
where the one run this round made before it failed three `git_version_cache` controls with
`Text file busy` and not this one.
Those failures are a fault of that module's own controls and are issue 6516.
The reds and greens are in
`doc/troubleshooting/git-policy-cli-controls-that-arm-state-the-whole-process-shares.md`.
Status:
incomplete,
unapproved and not final;
the owner declares completion.

branch `fix/git-policy-cli-issue-6503-process-wide-state-child-helper-takes-its-body`:
issue 6503 is fixed.
The helper that runs one control's body in a child of its own answered a yes or a no,
and each of its 106 callers held that answer in an `if`,
so a caller that read it inverted ran the body in the process that started the child,
the process the body was written to keep its state out of.
It takes the body now (`run_the_body_in_a_child_of_its_own(module_path!(), name, || { .. })`),
as the no-threads and umask starters have since issue 6494,
and `this_process_is_the_child` is private to the helper's file.
Measured on this host with one control of `foundation/lazy_runtime_tests.rs` read inverted:
`foundation::lazy_runtime::tests` answered `test result: ok. 15 passed; 0 failed; 1 ignored` in both of two runs,
and the same control alone answered `1 passed; 0 failed`,
so neither the run nor one process per test saw a body that built the runtime the whole run shares;
with every guard of that module inverted the run answered `FAILED. 14 passed; 1 failed; 1 ignored`,
at `this control is the first need of a runtime in its own process`.
`foundation::blocking_jobs_within_the_budget::tests` with one guard inverted answered
`FAILED. 10 passed; 1 failed`,
at the refusal of the room a control gives the process.
`source_guard/only_the_process_wide_starter_reads_its_marker_tests.rs` is the guard:
it refuses a source that names the child marker outside the helper's file,
and a read planted in `foundation/lazy_runtime_tests.rs` made it answer
`FAILED. 2 passed; 1 failed` at `foundation/lazy_runtime_tests.rs:37`.
Measured after the change,
after a merge of `origin/main` at `6dcf87146f`,
each module the round touched answered its own run with no failure
except `worktree::worktree_trash_entries::tests`,
which answers `FAILED. 7 passed; 1 failed` on the tree before this change as well
(`the_removals_take_places_of_the_processs_bound_on_blocking_jobs`,
issue 6477's shape, and that control guards no child before or after this round),
and `source_guard::` answered `test result: ok. 144 passed; 0 failed`.
The measured reds and greens are in
`doc/troubleshooting/git-policy-cli-controls-that-arm-state-the-whole-process-shares.md`.

Round of 2026-11-26,
issue 6500:
the service's review runner was reachable through the real handlers,
which name none of the calls issue 6499's round banned,
and `landing/service_work_tests.rs`'s two controls that review through `GitHandlers` took it.
Measured on this host,
those two controls in one process (`--test-threads=2`)
answered `test result: FAILED. 1 passed; 1 failed` in each of three runs,
with the refused control varying between them,
and the service's own line said
"the standards review of <oid> was not started because an earlier review, stopped at its ceiling, is still running".
The module threaded answered `ok. 117 passed; 0 failed` before the change as well,
which is why the issue had no measured failure:
its two reviews rarely meet in one moment,
while a pair of controls run together meets in every run.
`GitHandlers` now carries the runner a control hands it
(`with_review_runner`, `#[cfg(test)]`, carried by `for_round` as the review endpoint is),
and its two review faces run their job in it,
which is `standards_review_run::RUNNER` for every build that hands none,
so the one review at a time `SPEC.md` states is unchanged.
After the change the same pair answered `ok. 2 passed; 0 failed` in each of three runs,
each control answered `ok. 1 passed` alone with `--exact`,
and the module answered `ok. 117 passed; 0 failed`.
The guard sees the route now:
a test-only source that points the real handlers' review at a gateway must also name `with_review_runner(`.
With the two controls' runner calls taken out it answered `FAILED. 2 passed; 1 failed`
and named the four sites of `landing/service_work_tests.rs`,
and with the route check itself taken out the planted-route control failed on its fixture.
The measurement is in
`doc/troubleshooting/git-policy-cli-service-work-controls-share-the-review-runner.md`.
Status:
incomplete,
unapproved and not final.

Round of 2026-11-26,
issue 6510:
the control the note above left unfixed is fixed,
`commit::commit_index::tests::interactive_selection_succeeds_without_changes`,
and its sibling that starts the selection over a pathspec file that cannot be read:
`prepare_interactive_selection` starts `git add --interactive` or `git add --patch`
with the standard input of its own process on purpose,
because the person's terminal answers the selection,
and a threaded run's one process has the terminal the run was started from as its standard input,
so the Git printed its menu and waited.
Both controls run their body in a child of their own now
(`testing/process_wide_state_child_test_support.rs`),
whose standard input is the null device,
and `source_guard/no_control_starts_an_interactive_git_on_the_run_s_input_tests.rs`
refuses a control that names `prepare_interactive_selection(` without one,
reading a test source through the scan it shares with the ledger bans
(`source_guard/process_wide_fact_readers_tests.rs`).
Nothing of the release changed:
the selection's Git still reads the person's terminal.
Measured on this host,
with a terminal on the run's standard input:
before,
the issue's `index::` filter printed `What now>` from the Git the control started,
the run's watchdog aborted the module after 120.1 s with no `test result` line,
and the Git it left behind held the run's output pipe open until the capped scope ended it;
with the body in the run's own process,
the control alone printed the same menu and was aborted at 120.1 s,
and its sibling that the selection fails for a pathspec file answered `ok. 1 passed` in 0.02 s,
so that one never read the terminal even before.
After the change,
the same `index::` filter answered `test result: ok. 230 passed; 0 failed` in 13.47 s,
the control alone `ok. 1 passed; 0 failed` in 0.04 s,
and the two selection controls together `ok. 2 passed; 0 failed`.
With the control's child starter taken out,
the guard answered `FAILED. 2 passed; 1 failed`
and named `commit/commit_index_tests.rs:977 (interactive_selection_succeeds_without_changes)`;
the starter was restored.
`commit::commit_index::` answered `test result: ok. 52 passed; 0 failed`,
and `source_guard::` `ok. 144 passed; 0 failed`.
After `main` renamed the helper to `run_the_body_in_a_child_of_its_own`
(which takes the body as a closure),
the controls follow the new name and main generalised the guards' shared scan
(`source_guard/process_wide_fact_readers_tests.rs`),
so this ban states a `Requirement` there and its own copy of the scan is gone;
the runs were made again:
the issue's `index::` filter with a terminal on the run's standard input answered
`test result: ok. 230 passed; 0 failed` in 17.92 s,
`commit::commit_index::` `ok. 52 passed; 0 failed`,
`source_guard::` `ok. 157 passed; 0 failed`
and `testing::process_wide_state_child_test_support` `ok. 2 passed; 0 failed`;
the guarded control without its child answered `FAILED. 2 passed; 1 failed`
naming `commit/commit_index_tests.rs:981`,
and the starter was restored.
The other controls that start a Git were audited for the same input:
every step of `run_git` gives its Git the null device unless the request carries input,
so the property control that asks Git about `--interactive` and `--patch` lines read no terminal
(`ok. 1 passed; 0 failed` in 1.63 s with a terminal on the run's standard input,
1.72 s with the null device),
the fixture helpers and the binary harness give their Git an input of their own,
and the wrapper's plain commit keeps the terminal for an editor or a signing program on purpose,
as `SPEC.md` says,
while every control that calls it passes a message.
Status:
incomplete,
unapproved and not final.

Round of 2026-11-26,
branch `fix/git-policy-cli-issue-6511-push-child-watch-controls-own-their-clock`:
issue 6511 is fixed,
and it is a mechanism this item did not list:
the two stand-ins `foundation/lazy_runtime.rs` holds for a system that refused every thread
(`a_control_refuses_every_task`),
and for a build that outlasted its deadline (`a_control_stands_in_for_a_late_runtime`).
`tasks_can_run` and `built_flavor` answer for the whole process,
so a threaded run of `push::child_watch::` failed a different set of controls each time:
measured on this host,
`test result: FAILED. 42 passed; 5 failed; 1 ignored`,
whose two groups name the two facts,
no watcher where a thread was refused,
and a count of `/proc/self/fd` reading every thread's table.
Each control that arms one,
and the two controls that count this process's descriptors,
runs its body in a child of its own,
and both setters call `refuse_outside_the_child`,
which names the starter.
After the change the module answered `test result: ok. 47 passed; 0 failed; 1 ignored`
in two threaded runs of 21.38 s each,
and with `--test-threads=1` in 41.35 s.
Issue 6503 landed while this branch was open,
so the ten guards hand their body to `run_the_body_in_a_child_of_its_own`.
The measured reds and greens are in
`doc/troubleshooting/git-policy-cli-controls-that-arm-state-the-whole-process-shares.md`.
Status:
incomplete,
unapproved and not final;
the owner declares completion.

Cost: this is the failure class the owner named ("tests that pass alone but fail in a threaded run").
Seen failing: none of these specifically (the fixture failures of item 4 were seen); the rest is inferred from code.
Fix: run each such body in a child of the test binary through the one helper of item 10;
for 3g give each test its own instance or hold one lock for the whole test.
Size: about 45 sites in about 24 files across seven mechanisms.
Label: sites measured with (audit scratch, not kept) and `rg -n "dup2\(|set_var\(|umask\("`;
the interleaving effects are inferred.

### Item 4. A fixture folder is named by process and name only, and a repeat deletes a live folder

Category 1, tests.
`testing/test_support.rs:323-337` (`fixture_with_prefix`) names the folder `<root>/<prefix>-<pid>-<name>`,
removes an existing folder of that name (`:327-331`), then calls `create_dir` (`:332`),
which panics if another thread created it between the check and the call.
`binary/binary_support_fixture_directories.rs:213-227` repeats it as `native-binary-<pid>-<name>`.
Measured: 3,955 literal `fixture("...")` calls (`rg -o 'fixture\("([^"]+)"\)'`) and 402 non-literal sites in 309
files; names are unique only by convention.
Confirmed repeats in one test executable, each checked by reading the helper chain:
`registry-observe-main` (`worktree_copying/worktree_copy_registry_tests.rs:31,54`),
`merge-recovery-other-branch` (`merge/merge_isolation_recovery_tests.rs:1890,2950`),
`service-land-recover-alternate` (`landing/service_land_tests.rs:1080,2046`),
`service-land-switched` (`landing/service_land_tests.rs:1237,1724`),
`work-lease` (`landing/service_work_tests.rs:323`, `landing/work_item_tests.rs:321`),
`work-given-up` (`landing/service_work_tests.rs:467`, `landing/work_item_tests.rs:782`),
`claim-names` (`lock/single_owner_claim_tests.rs:150`,
`worktree_clone_mount_handler/per_slot_root_claim_tests.rs:225`,
`worktree_copying/worktree_copy_stage_claim_tests.rs:57`),
`claim-vanished` (`landing/service_index_claim_tests.rs:247`,
`worktree_copying/worktree_copy_stage_claim_tests.rs:787`),
and in the binary target `timing-signal` (`binary/binary_commit_timing_tests.rs:1380,2610`).
Other repeated names are in different executables (lib against `native_binary`) and do not collide.

Landed 2026-11-25 as `c14f1156c9`:
`fixture()` names every call's folder with the number that call took
(`native-unit-<process id>-<the number this call took>-<name>`),
the binary target's helper names its own the same way,
and the folder is made with `create_dir` only,
so no call deletes another's live folder.
The three controls that failed threaded went with it,
and their causes are in
`doc/troubleshooting/git-policy-cli-controls-that-name-one-fixture-folder.md` and
`doc/troubleshooting/git-policy-cli-control-windows-taken-in-two-orders.md`.

Cost: seen today (`DirectoryNotEmpty`, and the `create_dir` panic at `test_support.rs:332`, per the practices guide);
the cross-module pairs (`work-lease`, `claim-names`) appear only in an unfiltered threaded run.
Fix: make each call's folder its own inside the helper (a process-wide atomic counter in the name, `create_dir` only,
no remove-stale branch), once in each helper, or once with item 11.
No call site needs an edit: nothing outside the helpers builds the `native-unit-` or `native-binary-` names
(measured: `rg -n "native-unit-|native-binary-"` finds only the helpers and doc paths).
Size: two function bodies cover 4,357 sites and the nine known repeats.

### Item 5. Stand-ins outlive an aborted or killed run, because `spawn_owned` in the lib has no death signal

Category 2, tests.
`testing/test_support.rs:287-293` (`spawn_owned`) sets a process group and registers it; the only cleanup is
`ThreadFixtures::drop` (`:255-268`), which runs `killpg`.
`testing/run_progress.rs:830-834` ends a stuck run with `std::process::abort()`, which runs no destructor,
and a `SIGKILL` from outside runs none either.
The binary target's copy, `binary/binary_support_fixture_service.rs:119-135`, adds `prctl(PR_SET_PDEATHSIG, SIGKILL)`
for exactly this reason ("a test binary killed outright runs no guard at all"); the lib's copy does not.
So every child the lib's tests started survives such an end.
Measured: 214 lines in tests start long-sleeping stand-ins (`sleep` of 30 s or more, `HOLD_SECONDS` 3,000 at
`lfs/lfs_upload_tests.rs:443`, `sleep 600` at `push/push_patience_tests.rs:77`, `sleep 120` at 24 lines in 14 files);
194 raw `.spawn()` lines in 109 test and support files against 87 `spawn_owned` lines in 52 files;
60 `impl Drop` guards in test and support files, and 37 files with such a guard kill or wait a child,
each file with its own (`Helper`, `Held`, `NuisanceGit`).
A child must not keep the run's output open (the guide); the stand-ins that `exec sleep` inherit standard output.

Cost: seen today (a stand-in held the run's output for 45 minutes, per the guide).
Fix: add the `PR_SET_PDEATHSIG` pre-exec to the lib's `spawn_owned` (it fires when the spawning thread ends,
which is the intent), redirect inherited output in the stand-ins,
then route the raw spawns through `spawn_owned` after triage.
Size: one function fixes all 87 current users; about 100 files with raw spawns need triage.
Label: the abort and the missing death signal are measured by reading both files; the survival of children is
inferred.

### Item 6. The fix for issue 6463 exists but is not merged, and main still has the signal at `getpid()`

Categories 1, 2 and 7, tests.
`lfs/lfs_upload_tests.rs:427` on main still sends `libc::kill(libc::getpid(), libc::SIGTERM)`;
the stand-in at `:419` still `exec`s `/usr/bin/sleep` for `HOLD_SECONDS` (`:443`);
the wait at `:423` for the pid file has no deadline.
The fix is `898bdaa86b` (and `1c9c0666f3`) on `fix/git-policy-cli-lfs-upload-tests-signal-their-own-test-binary`,
which is also on origin and is not an ancestor of main
(measured: `git merge-base --is-ancestor 898bdaa86b HEAD` says not an ancestor;
`git log --all --oneline -S'libc::kill(libc::getpid(), libc::SIGTERM)' -- <file>`).
Drift: `package/git-policy/cli/doc/how-to-write-this-packages-code-and-tests.md` says the control "once sent" the
signal, which is true only on that branch.
Fix: merge the branch (no new round); then check that `:423` and the long sleep are gone there.
Size: one file, one branch.

### Item 7. Test waits without deadlines, sleeps as synchronisation, and wall-clock bounds

Category 1, tests.
Waits with no end (measured by (audit scratch, not kept): 34 sleeping loops with no visible bound,
most of them waiting on a stop flag):
`transaction/transaction_phase_tests.rs:145` and `:172` spin until a file appears, and `:176` joins the thread,
so a test that fails first hangs;
`lock/index_holder_wait_tests.rs:47`; `lfs/lfs_upload_tests.rs:423` (item 6).
Candidates not triaged (many follow a kill): 57 `.recv()` with no timeout, 233 `.join().expect/unwrap`,
335 `.wait()` or `.wait_with_output()` in test files.
Sleeps used to wait for another party: 382 literal-duration `thread::sleep` calls in tests, 15 of one second or more
(for example `binary/binary_commit_when_the_service_stops_tests.rs:557`, six seconds), 57 of 100 to 999 ms
(`push/push_progress_tests.rs:568`, `process/signal_relay_tests.rs:483`), 310 shorter.
The bounded helpers exist (`testing/test_support_wait_bounds.rs`: `wait_until`, `finished_within`)
and have 145 uses in 48 files.
Wall-clock upper bounds: 99 single-line assertions `elapsed < Duration::from_*`
(`rg -n "assert!\([^;]*(elapsed|waited|took|spent|lasted)[^;]*<..."`),
the tight ones at 100 ms (`process/stuck_children_reaped_when_the_kernel_lets_go_tests.rs:46,73`),
150 ms (`foundation/side_by_side_on_the_current_task_tests.rs:206`), 500 ms (`service/journald_native_tests.rs:605`);
`push/bounded_query_tests.rs:211` asserts under ten seconds.
Open issue 6405 is one such bound that host load stretched.

Cost: hangs of the run (the watchdog then aborts after 120 s, which feeds item 5) and failures under load.
Fix: replace each fixed sleep with `wait_until`; replace each tight bound with an order of events or a ratio to a
control run in the same round; give the four open loops a deadline.
Size: 4 loops, 72 sleeps of 100 ms or more, 99 bounds, about 625 waits to triage.
Label: counts measured; which waits are really unbounded is inferred except for the four named.

### Item 8. The release carries test-only fault variables that kill or stall the service

Category 3, release; needs an owner decision.
`service/service_fault.rs:1-17,92-105` says the fault point "is compiled into the release executable, and only the
explicit variable arms it"; `reach` then sends the process `SIGKILL` when `CLI_GIT_TEST_ONLY_SERVICE_FAULT` names
the point.
Likewise `transaction/transaction_phase.rs:34,372-376,517` (`CLI_GIT_TEST_ONLY_PHASE_SIGNAL`: kill or pause),
`foundation/lazy_runtime.rs:136-149` (a sleep), `service/journald_native.rs:54,450` (log datagrams to a named socket),
`worktree_clone_add_route/add_route_plan.rs:37-43,212-214,279` (hold, stop, `raise(SIGKILL)`),
and `service/service_warm_request_control_window.rs`, a release file that parks on `CLI_GIT_TEST_ONLY_WARM_SCAN_PARK`.
Measured: 18 `CLI_GIT_TEST_ONLY_*` constants in release files
(`rg -c 'const [A-Z_]+: &str = "CLI_GIT_TEST_ONLY_'` over (audit scratch, not kept));
one is compiled out by a feature (`start_probe/probe_deadline.rs:145,154`, feature `start-probe-raised-deadline`).
The `Cargo.toml` comment on that feature gives the reason: a unit's `Environment=` line can set such a variable.
Cost: risk (inferred): an exported variable or a unit line can kill or stall the commit service
("Commits should never fail").
Fix: decide with the owner whether to gate all of them like the start probe (binary controls then need a second build)
or to record the exception; then one round.
Size: 18 constants; read sites not counted.

### Item 9. SPEC passages the code contradicts or points at nothing

Category 7.
`SPEC.md:10948-10956` lists `SERVICE_GREETING_SILENCE` (3 seconds) as the wait before a commit asks the manager to
restart a running unit that greets nothing.
No such constant exists (`rg -w SERVICE_GREETING_SILENCE` finds only a control and troubleshooting history);
`service/service_client_reach_service.rs:297` ends the wait for a greeting only at `config.start_ceiling`
(one minute);
`service/wait_ceilings_tests.rs:218-221` asserts the constant is absent;
`SPEC.md:11681` and `:12425-12427` say the same as the code.
The reason is recorded: `doc/troubleshooting/plasma-session-ended-after-service-manager-timeouts.md:583,720`
(a three-second silence ended a desktop session).
Four `SPEC.md` citations name files that do not exist:
`:10483` `process/pinned_member_signalled_through_a_pidfd.rs` (now `process/member_signalled_through_its_pidfd.rs`),
`:13475` `push/push_failure_notice_wording.rs` (probably `push/push_failure_notice_words.rs`),
`:15887` `service/service_work_push_environment.rs` (now `landing/service_work_push_environment.rs`),
`:3637` `worktree/worktree_removal_refuses_a_nested_volume.rs` (no file of that name; the refusal text is in
`worktree/worktree_remove_hand_over.rs` near `:995` and `:1118`).
Two more in `doc/` are historical (`doc/step-5-second-executable-2026-11-20.md:27`,
`doc/planted-control-and-durable-survivors-resume.md:14`).

Note of 2026-11-27T10:23:54-05:00:
all four passages are corrected in `SPEC.md` by `52f55a54db`
(2026-11-25T16:57:53-05:00),
which rewrote the nested-volume bullet to point at
`worktree/worktree_removal_goes_around_a_foreign_volume.rs` and its `volumes_below`;
the words this item placed in `worktree/worktree_remove_hand_over.rs` moved in
`05c202a266` (2026-11-27) to
`worktree/worktree_remove_hand_over_around_a_foreign_volume.rs`
(`remove_around_what_is_not_this_removals` and its sentence about a volume that is not the removal's);
the boundary's own refusal words are in
`subvolume/subvolume_remove_walk_boundary_and_refusal.rs` since `f76de23f0e` (2026-11-26).
Cost: the first passage invites the exact mistake the incident recorded.
Fix: delete or rewrite the bullet; correct the four paths.
Caution for the round: `Cargo.toml` says `service::service_instance_tests` and
`worktree_clone_mount::slot_identity_store_tests` read `SPEC.md` at run time, so check what each pins before editing.
Size: five passages.
Checked and consistent (measured): 289 cited source paths (6 missing, the ones named in this item), 191 bare `.rs`
names (1 missing, the example `a.rs`), 44 numeric statements beside 34 constants (no real disagreement;
two hits were a test-local `ANSWER_CEILING`), 53 `CLI_GIT_*` names (two absent from code are stated as removed at
`SPEC.md:13212` and `:15765`).
Method: (audit scratch, not kept), `rg -o "src/native/[A-Za-z0-9_./-]+\.rs"`; the 25,451 lines of prose were
not read.

### Item 10. The test binary is started as a child in 35 places with six helper shapes and little ownership

Categories 4 and 2, tests.
Measured: `rg -n '"--exact"'` finds 35 sites in 32 files ((audit scratch, not kept)).
`testing/child_test_process.rs` is the shared helper and has users in 9 files; others write their own:
`testing/umask_child_test_support.rs`, `testing/no_threads_child_test_support.rs`,
`testing/file_size_limit_child_test_support.rs`, `testing/root_namespace_test_support.rs`,
and `binary/binary_merge_mode_tests.rs:55-95`, a copy of the umask one (seven identical code lines at `:74-81` and
`umask_child_test_support.rs:27-34`).
Only 9 of the 32 files use `spawn_owned`; `testing/child_test_process.rs:58` waits with `.output()`
(no deadline, not owned); only some callers check `1 passed` (`testing/no_threads_child_test_support.rs:127` does).
Cost: maintenance, plus the hangs and leaks of items 5 and 7.
Fix: one helper in `testing/` that starts the child with `spawn_owned`, a deadline, piped output and the `1 passed`
check; migrate the 35 sites; this is the runner item 3 needs.
Size: 35 sites, 6 helper bodies.

Landed 2026-11-25, in two commits on `refactor/git-policy-cli-test-binary-children-through-one-helper`:
`31f4014890` and `d5e781a32a`,
merged as `1911c927c0`.
`testing/child_test_process.rs` holds the one request and its three faces (`started`, `finished`, `passed`),
with a deadline, the two pipes read once the child is gone, and the check that exactly one test ran and passed;
`testing/test_process_owner.rs` holds the ownership those children stand on, which the binary controls' crate and the
candidate consumer's crate include by path, because the library's test-only modules are invisible to them.
The umask, no-threads, file-size-limit, root-namespace and merge-mode bodies keep their caller-facing functions and
hold no child start of their own, and `rg -n -- '"--exact"' src` answers one line: the runner's own argument list.
Each of the five moved sites that started a child with no owner lost its line in
`source_guard/test_children_not_yet_owned.txt` in the same round (`d5e781a32a`).
Status: incomplete, unapproved and not final.
No independent review has read it, the guest arms of the two ZFS controls of this item were skipped in this host run,
and item 3's mechanisms have not been moved onto it yet.

### Item 11. The lib and binary test targets each keep a copy of the fixture support, and the copies diverged

Category 4, tests.
Measured with (audit scratch, not kept): 19 identical code lines in runs of three or more between
`testing/test_support.rs` and `binary/binary_support_fixture_directories.rs`
((audit scratch, not kept)).
Copies: `forbid_core_dumps` (`test_support.rs:67-74` and `binary_support_fixture_directories.rs:348-355`),
`system_temporary_directories` (`:140-150` and `:58-68`), `fixture_root` (`:110-125` and `:38-45`),
fixture creation (`:323-337` and `:213-227`), `settle_fixture` and `remove_patiently` (`:229-241` and `:268`),
`FIXTURE_ROOT_VARIABLE` (`:82` and `:20`), and `spawn_owned` plus `own_group`
(`:287-307` against `binary/binary_support_fixture_service.rs:102-135`), which differ in the death signal (item 5).
Cost: a fix lands in one target and not the other (items 4 and 5 are two such).
Fix: one file included by `#[path]` in both targets; `binary/binary_tests.rs:33` already includes
`../testing/real_git_lfs_the_run_requires.rs` this way.
Size: 8 helpers.

Landed 2026-11-25, in `07fe505696` on `refactor/git-policy-cli-fixture-support-one-copy-for-both-test-targets`,
and merged into item 10's branch by `5e30b7665a`,
both on `main` as `1911c927c0`.
`testing/fixture_support_shared_by_both_test_targets.rs` holds the fixture folders,
the names they are made under and the removal,
and both targets include it by `#[path]`;
the report of that round is
`fixture-support-one-copy-for-both-test-targets-2026-11-25/keep/report-pi-fixture-support-one-copy-for-both-test-targets.md`.
The merge kept one definition of the helper ownership,
in `testing/test_process_owner.rs`,
because that file is the one the candidate consumer's crate takes,
and that crate keeps no fixture folders to run
(item 10's file, whose three includers are unchanged):
`OwnedGroups`, `group_of`, `own_group` and `spawn_owned` are there,
with this item's read-back in `own` and item 10's `end_owned_groups`,
and the shared fixture file no longer defines or names any of them.
Status: incomplete, unapproved and not final.
The merge's checks ran on the host only, no container gate was run, and the guest arms were skipped.

### Item 12. One-line byte tests are copied into 101 definitions

Category 4, release.
Measured with (audit scratch, not kept); the 101 definitions with their `file:line` are in
(audit scratch, not kept).
Eight names (`is_nul_byte` 29, `is_nul` 13, `is_line_feed_byte` 14, `is_newline` 11, `is_line_feed` 7,
`is_slash` 11, `is_space_byte` 7, `is_space` 9) define 101 functions, 98 of them with one of four bodies:
`*byte == 0` (41), `*byte == b'\n'` (32), `*byte == b'/'` (10), `*byte == b' '` (15).
Probably the `rust/avoidable-closure` rule (`monochromatic-lint.config.jsonc`, warn) pushes each file to name its own.
Cost: maintenance only.
Fix: one `foundation/` file with the four named tests, replace the copies (mechanical).
Size: 98 definitions.

### Item 13. Five sync and awaiting twin pairs hold the same code twice

Category 4, release.
Measured with (audit scratch, not kept) (runs of six or more identical code lines; each run's two ranges are listed in
(audit scratch, not kept)), the spans that hold the runs:
`lfs/lfs_upload_plan.rs:446-555` and `lfs/lfs_upload_plan_awaiting.rs:56-165` (88 lines);
`push/auto_push.rs:435-608` and `push/auto_push_awaiting.rs:85-257` (46);
`transaction/transaction_git.rs:298-474` and `transaction/transaction_git_awaiting.rs:147-253` (45);
`foundation/diagnostic_lines_off_the_calling_thread.rs:222-498` and
`service/service_log_lines_off_the_calling_thread.rs:217-425` (44);
`push/auto_push_reconcile.rs:382-403` and `push/auto_push_reconcile_awaiting.rs:295-316` (21).
Thirteen `_awaiting` files exist in all (`rg --files | rg awaiting`).
These are the tokio move's transition; the guard's allow-list (item 18) records its end.
Cost: a fix lands in one twin only.
Fix: finish the move for each pair (delete the synchronous twin) or extract the shared body.
Size: 5 pairs, about 244 identical lines.

### Item 14. Public functions nothing calls, and public functions only tests call

Category 5, release.
Method: for every `fn name` in a release file, count the word's occurrences in all `.rs` files under `B` (comments
stripped), then in `cli.fuzz`, `cli/e2e`, `cli/perf`, `cli/bin`, `SPEC.md`, `README.md`, `DESIGN.md` and `doc/`
((audit scratch, not kept), output (audit scratch, not kept)).
Result: 8,867 function names; 47 are never referenced anywhere ((audit scratch, not kept), with lines in
(audit scratch, not kept); only doc comments mention three of them);
all 47 are `pub`, which is why Clippy with `-D warnings` stays quiet ((audit scratch, not kept)).
Largest clusters: `worktree_clone_unmount/` (11 functions), `zfs_compatibility_gate/gate_paths.rs` (3).
A further 172 `pub` functions in release files are referenced only from test-only files, for example
`landing/service_land.rs:80` `land_batch` (102 test references) and `landing/service_index_lock.rs:297,323,334`
`acquire_noting_wait`, `acquire_on` and `acquire_within`: release code that exists for tests.
Cost: maintenance; the second kind makes tests exercise a function the release does not call.
Fix: delete the 47 with their text (the practices guide, "Write less code"; issue 6430 is the same kind of removal);
move the 172 under `#[cfg(test)]` or into the sidecars.
Size: 47 deletions, 172 moves.
Label: measured by name; a name reused elsewhere would hide dead code (undercount); I saw no overcount
(a call through a macro or `include_str!` text would be missed).
No source file is unreferenced (3,003 files; the two found are Cargo test targets: (audit scratch, not kept)).

### Item 15. Twenty-four files are over the line budget, thirty more are at the edge

Category 6, release and test support.
The rule is `rust/max-lines` 300 as error (`monochromatic-lint.config.jsonc`); `*_tests.rs` files are exempt.
The count is of lines holding a non-comment token, which mirrors
`package/linter/monochromatic-lint/src/rust_source.rs:98-132`
(approximated with `rg -c -v '^\s*(//.*)?$'`; the linter was not run; (audit scratch, not kept)):
24 files over 300 and 30 at 290 to 300.
The largest: `worktree/worktree_remove_hand_over.rs` 569; `worktree_clone_mount_handler/record_checkout_mode.rs` 437;
`worktree_clone_mount_handler/mount_path_descendant_mode.rs` 436; `repair/package_repair_toolchain.rs` 434;
`binary/binary_support_fixture_service.rs` 432; `worktree_copying/worktree_copy_owed_copy.rs` 407;
`landing/service_claim_watch.rs` 404; `repair/package_repair_prewarm_job.rs` 386;
`landing/service_work_drain_pending_claims.rs` 368; `worktree_clone_add_route/add_route_steps.rs` 358;
`worktree_clone_mount_handler/handler_mode_arguments.rs` 356; `service/service_units_of_removed_worktrees.rs` 346;
`commit/native_commit_waiting_notice.rs` 336; the other eleven are 301 to 328 (full list in the keep file).
No `monochromatic-lint-disable` directive and no recorded baseline excuses them
(`rg -c "monochromatic-lint-disable" src/native` finds none).
The package task `lint:monochromatic-lint` runs with `--max-warnings 0` (`package/git-policy/cli/mise.toml`),
but the gate runs Clippy only, and the linter binary is not on the main checkout's `PATH`
(`doc/handover/cli-git-rust-implementation.md`, the note on `monochromatic-lint`).
Cost: lint errors the moment the linter runs here (inferred); the big sidecars are exempt, but a 4,676-line test file
(`merge/merge_isolation_recovery_tests.rs`) is a reading cost.
Fix: split each by role into sibling modules, re-exporting, never stripping docs (MXL); one agent per file.
Size: 24 files.
Label: approximated count; confirm by running the linter once before the round.
Note of 2026-11-26T12:50-05:00: the linter, run on `987635373a`, counts 281 files over 300 and the ledger's measure gives the same count on the largest file (1012).
The difference is the reformat of 2026-11-26 (`00b6a940b4`), which grew the crate's code lines 1.472 times; `worktree/worktree_remove_hand_over.rs` went from 572 to 1012.
19 files were over 300 before it and are split now; the 262 over only because of it wait for the owner's answer on the budget
(answered on 2026-11-26: "Keep 300, split everything", `doc/decision/rust-files-keep-the-300-line-budget-after-the-vertical-reformat.md`)
(`doc/handover/questions-for-the-owner.md`; the counts are in `~/temp/agent/wrapper-line-budget-after-the-reformat-2026-11-26/keep/`).
(scratch,
deleted on 2026-11-27;
what this record uses from it is stated here).

Note of 2026-11-26T14:09-05:00:
the split round for four of these files ended
on branch `refactor/git-policy-cli-split-worktree-ladder-conversion-and-removal-record-files-over-the-line-budget`,
and the linter reports none of the four over 300 now
(`monochromatic-lint` 0.5.2-alpha.15,
run on `package/git-policy/cli`).
Each is split into sibling files in `worktree/`,
declared in that folder's `mod.rs` and re-exported,
so no caller changed,
and each part is at most 300 code lines by the linter's own count:

- `worktree/worktree_clone_conversion_size_read.rs` 489 to seven files,
  the largest 151 (`worktree_clone_conversion_size_read_answer.rs`),
- `worktree/worktree_ladder_target.rs` 453 to two files,
  the largest 254 (`worktree_ladder_target.rs`),
- `worktree/worktree_removal_held_commits.rs` 450 to four files,
  the largest 167 (`worktree_removal_held_commits_words.rs`),
- `worktree/worktree_landed_removal.rs` 443 to four files,
  the largest 165 (`worktree_landed_removal.rs`).

Each module's own controls pass with the same count before and after:

- the size read 36 passed,
- the ladder target 14 passed,
- the held commits 8 passed,
- the landed removal 43 passed.

The run finds zero `rust/folder-uses-links` and zero `rust/file-order-in-folder` findings,
and neither layout allow list gained a line.
The work is unapproved and non-final;
the owner declares completion.

Note of 2026-11-27T05:25:09-05:00:
the round briefed to split `command/rule_linked_worktree.rs` split nothing,
because the loop break of that folder closed the item first.
The 352 code lines the brief carried is the count before `1466cd124c`
("move the guarded forms beside the rejection texts",
merged as `817cc3bb4c`,
whose own message records "352 code lines before the move,
298 after"):
that commit moved `GuardedCommand` and its `ALL` list into
`command/the_guarded_commands.rs` and re-exported the name,
so no caller changed and the file came to 298 code lines.
Measured on the branch,
level with `origin/main` at `caeeb3d2ad`
(`monochromatic-lint` 0.5.2-alpha.17,
run on `package/git-policy/cli`):
the file holds 298 code lines and the linter reports no `rust/max-lines` finding for it,
against 40 such findings in the crate when the count was taken and none of them under `command/`
(fewer on later trees,
as other split rounds land).
Two controls say that reading is the budget and not a file the scan skips:
a hand count of three files whose line counts the linter names matches it exactly
(`foundation/private_storage.rs` 409,
`health/doctor.rs` 362,
`foundation/lines_written_by_a_thread.rs` 391),
and padding the file with five code lines made the linter report 303 for it.
`command/rule_linked_worktree_messages.rs` holds 299 code lines,
one under the budget,
so the next change to it is the one that owes a split.
No file was split,
so neither layout allow list gained a line and none was deleted,
and the linter finds zero `rust/folder-uses-links` and zero `rust/file-order-in-folder` findings.
The work is unapproved and non-final;
the owner declares completion.


### Item 16. Twenty-nine of fifty-five source folders sit in one dependency loop, and the layout rules are not enforced

Category 6, release.
Measured with (audit scratch, not kept) (references `crate::name` and `use crate::{...}` mapped to folders
through each `mod.rs`; 1,534 modules; the 1,524 release files only, so `testing/` is out): one strongly connected set
holds 29 folders over 378 folder edges, and 97 pairs of folders use each other
((audit scratch, not kept)).
The base folder `foundation` uses `invocation` (4 files), `landing` (3), `lfs` (1), `policy` (3), `service` (4) and
`worktree` (1), and is used by `landing` in 115 files and `worktree` in 142.
The repository rules LYD, LYL and LYF ask for `uses/` folders, no mutual use and no file loops;
only 19 of 55 folders have a `uses/` folder, and the linter rules `rust/folder-uses-links` and
`rust/file-order-in-folder` are configured only for the zfs binding package
(`monochromatic-lint.config.jsonc`, block `layout-zfs-request-to-openzfs-kernel-device`).
Cost: structure only; no failure seen.
Fix: the owner chooses a direction first (enable the rules here, or record the exception);
the loops are not a mechanical edit.
Size: 29 folders, 97 pairs.
Label: measured by script; `crate::name` can be a type or function that shares a module's name, which would
over-count (inferred, small).

The round of 2026-11-25 switched the rules on.
The two rules now run for the whole crate
in the block `layout-git-policy-cli` of `monochromatic-lint.config.jsonc`.
The findings of that day stood in two lists beside the crate,
and the lists have moved since as rounds untangled them,
so read the counts below as that day's:

- `layout-allow-list/folder-rule.txt` held 1014 entries:
  524 `uses <folder> -> <folder>` lines for that day's folder and entry loop edges;
  49 lines for the `pub use folder::*` re-exports,
  which keep the crate's flat module paths;
  20 lines for the findings that `health/` is a group folder
  with module files beside its child folder;
  and 421 lines for the `uses` links the crate does not hold.
- `layout-allow-list/file-rule.txt` held 604 entries:
  544 `file-uses <folder> <file> -> <file>` lines for that day's file loops;
  59 lines for the folders whose top doc has no file list or misses a name;
  and one for the function that `main.rs` holds beside `fn main`.

The 421 links are recorded rather than written,
because a link set with mutual uses makes `cargo package --list` walk without end:
cargo follows every `uses` link and the loops make those paths multiply.
Measured that day:
1.9 s with the 109 links the crate had,
and no answer after 120 s with the 421 added ones;
the same eight folders in a ring did not answer in 60 s where the same eight in a chain took 0.13 s.
Two things read that list,
the control `source_guard::packaged_uses_links_tests` and a release's `cargo publish --dry-run`,
so a round writes a folder's links when that folder's uses no longer loop,
and deletes the lines of those links from the list in the same change.
Seven wrong links went:
five pointed at the test-only `testing` folder,
which is outside both rules,
and two named folders that nothing in their folder uses any more.

Both lists may only shrink:
a line that matches no finding any more fails the lint;
and a use added inside a loop a list tolerates is a finding that no line can record.
The untangling this item asks for is what deletes those lines,
one use per line.
What is left for it is 524 folder uses and 544 file uses.
Neither rule runs in the container gate.
They run in `mise run //package/git-policy/cli:lint:monochromatic-lint`,
which fails today on findings of other rules as well:
2295 `markdown/semantic-line-breaks` and 24 `rust/max-lines` over this package.

Fix: low priority; rename cost is high, so only with item 16.
Size: 4 names.

### Item 18. The runtime guard has fifty-four recorded exceptions, and it reads one file at a time

Category 3, release; already tracked.
`source_guard/blocking_code_off_the_runtime_rules_tests.rs` holds 54 `Allowed` rows (measured 2026-11-25 with
`rg -c '    Allowed \{'`): 30 `blocking-sleep`, 22 `thread-start`, 2 `unretried-start` (macOS and Windows only).
Each row names its removal condition.
The scan's own header says it follows no call, so a synchronous helper that reads a file and is called from an
`async fn` is invisible to it (inferred; not measured).

Round of 2026-11-25, branch `fix/git-policy-cli-merge-file-calls-off-the-runtime`,
merged as `10f4826fdc`:
the `merge/` reads are converted and their rows are gone.
The ledger's "eight" was seven by the time the round started
(`rg -c 'rule: "storage-read-on-a-task"'` printed 7 before the first edit):
`merge_second_pass_worktree.rs`, which this item named too, has held no row and no file call inside an `async fn` since
an earlier round, so the round met seven files and the guard held 61 rows then.
Each file's row went in the commit that moved its calls, and every call moved is a job of
`run_blocking_job_within_the_budget`, the crate's own way onto the blocking pool:

- `713b8c1f53` `merge_isolation_carry_place.rs`: the look, the write of the private name, the removals and the renames.
  The same commit gave `rename_that_never_replaces.rs` `rename_noreplace_on_the_blocking_pool` (with `TheAnswerOfARename`)
  and `ProofNothingCanWrite` a `remove_on_the_blocking_pool`, because three files of `merge/` need them.
- `92844c5aac` `merge_isolation_changes.rs`: the entry look and the link read.
- `67a4fc171d` `merge_isolation_kept_copies_decisions.rs`: the two looks per copy, the identity and permission comparison,
  the removals, the put-back rename and the record-folder reads.
- `cc99b4bcee` `merge_isolation_recovery_records.rs`: the listing and the read of every record, as one job.
- `91658658b3` `merge_isolation_recovery_steps.rs`: every path's identity in one job before any content is read, and the
  look and the walk of each path a directory may stand at in one job.
- `7a09da710e` `merge_isolation_take_asides.rs`: every file call there; `put_back` is `put_back_awaiting`, and its only
  caller `install_entries_awaiting` awaits it.
- `7304e3db72` `merge_second_pass_tree.rs`: the scratch directory and its sweep, and the removal of the scratch index.

The guard was shown failing with one call planted back on a task in `merge_second_pass_tree.rs`
(`merge/merge_second_pass_tree.rs:395 [storage-read-on-a-task]`), and green again once the plant was restored.

What is still read on a task in these files:
the holders scan of `index_lock_holders` (`/proc` reads, which the plan allows inline), and the folder readers
`kept_copies`, `asides_of`, `live_merge_keys`, `stale_carry_names_of` and `list_merge_intents` where another merge file
calls them; the calls inside the seven files were moved, the helpers were not made awaited.

Fix for the rest: the owner's tokio plan finishes the remaining 54 rows; keep the list as the tracker.
Size: 54 rows.

### Item 19. Lists written as fixed-length arrays carry a hand-counted length

Category: magic numbers (`AGENTS.md`'s MGN), added 2026-11-25T19:45-05:00 on the owner's "Isn't 95 a magic number that can
be avoided?" about the raw-read guard's `const KEPT_SITES: [(&str, usize, &str); 95]`.
Measured with `rg 'const [A-Z_]+: \[[^\]]*; [0-9]+\] = \['` on `main`: 277 such constants in 187 files of the crate's `src`,
13 of them in 10 `source_guard` files (for example `hook_self_invocation_tests.rs`'s `MAIN_CODE: [&str; 47]` and
`root_mode_name_tests.rs`'s `NAMES_CODE: [&str; 53]`).
Every entry added or removed edits the count beside it, and the count says nothing a reader needs.
The raw-read guard's three lists became slices on its branch (`&[...]`), as other guards already write them.
Not every match is a list: an array whose length is its meaning (a header of a fixed byte width, a table indexed by an
enum) keeps it.
Fix: one round, after the live rounds touching those files merge, that turns every list of names, sites or words into a
slice and leaves meaningful lengths, plus a `source_guard` control that refuses a new `const` list of string literals
with a numeric length; the package guide states the rule since this item.
Size: up to 277 constants, most of them one-line changes.

Landed 2026-11-26,
on `refactor/git-policy-cli-fixed-length-lists-become-slices`:
`14053c4fd5` (run 1:
276 declarations in 226 files),
`b37ed74e76` (run 2:
23 declarations in 21 files and 2 parameters),
`af3461a08f` (the guard reads a list whose `[` starts on the line below),
and `a68915d6f3` (the borrows Clippy named),
after merges of `main`.
The ledger's search answers 7 lines today:
4 are the guard's own planted fixtures,
and the 3 declarations that stay are `REWRITTEN` and `UNTOUCHED`,
the nine words of one subvolume index record,
and `WAIT_LIMITS`,
the window,
poll and cap one wait destructures.
Run 1's sweep matched only a declaration whose type and initializer shared a line
and whose element type held no bracket of its own,
so 23 declarations and 2 parameters were still arrays
(the guard named 19 of them;
four more are a list of a name,
two of a byte tag and one of a watched signal,
whose element type names no text):
`[&[&str]; N]` lists of subcommand words,
`[(&str, &[u8], bool); N]` case rows,
and a `= [` on the line below.
`source_guard/lists_are_slices_tests.rs` refuses a new one:
a `const` whose element type names a text reference and whose length is a number is a finding in any layout.
It was shown failing on a planted `const PLANTED_NAMES: [&str; 2] = ["first", "second"];`
at `foundation/kept_names_in_the_git_directories.rs:65`,
and green again once the plant was removed.
Seven fixed-length arrays stay,
each with a length that is the meaning and none holding text:
the two index records,
`WAIT_LIMITS`,
`PENDING`'s one slot per relayed signal,
a torn UTF-8 lead byte,
the four-byte index magic word and the shell's escaped quote.
Not done:
the 297 local `let NAME: [&str; N] = [...]` lists
(`rg ':\s*\[[^]]*(str|String|OsStr|OsString)[^]]*; [0-9]+\]' src` answers 302 lines,
297 of them `let`,
almost all in test files),
because this item measured constants and the guard refuses constants,
so converting a local is a decision nobody has made.
No container gate and no independent review ran.
Status:
incomplete,
unapproved and not final.

## Already in flight, do not duplicate

`fix/git-policy-cli-lfs-upload-tests-signal-their-own-test-binary`: item 6, unmerged.
`fix/git-policy-cli-lock-controls-that-fail-only-in-a-threaded-run`: commit `841d20b762` gives two forward controls
their own folders (an item 4 instance, 3 files); not an ancestor of main
(`git log main..<branch> --oneline` lists three commits).
`fix/git-policy-cli-named-pipe-in-a-record-folder`: issue 6415, only a merge commit so far.
Open issues (measured: `gh issue view <n> --repo Aquaticats/Monochromatic --json state`):
6405 (wall-clock bound), 6412 (return wider than its caller), 6415, 6421 (notes on the commit thread),
6430 (copy-saving code nothing calls), 6433 (unbounded read of a launcher record); 6454 is closed.

## Looked for and not found

No test calls `set_current_dir` (a source guard enforces it:
`source_guard/no_test_changes_the_process_directory_tests.rs`).
The three release group-kill functions (`account_companion/companion_process_calls.rs:205`,
`start_probe/probe_processes.rs:244`, `repair/package_repair_start.rs:567`) take a pid that comes from a checked
`fork` result or `Child::id`, so no path to `kill(0)` or `kill(-1)` was found (inferred from the callers).
Literal `/tmp` appears in test data strings; the one fixed path is `binary/binary_zfs_step9_m_arms_tests.rs:635,653`
(`/tmp/m9target`), inside an arm that returns unless `in_guest()`.
No orphan source file (item 14). SPEC numbers and paths are mostly right (item 9).

## Not checked and limits

No build, test or linter was run, so every "fails threaded" statement is inferred from code, not seen.
The 402 non-literal `fixture(...)` sites were not all resolved; only helper chains for the nine confirmed names
were read.
The 138 raw reads, the 99 bounds and the 625 candidate waits were counted, not each classified.
Release-code waits on children and channels were scanned by pattern (`rg`), not read one by one.
The SPEC's 25,451 prose lines and the README's 9,427 were not read; only paths, constants and variable names were
compared.
Module and folder names were judged from file and folder lists, not from the contents of each.
Only three in-flight branches and the open issues named in the practices guide were checked; the other worktrees under
`.claude/worktrees/` may already touch these files.
The release set is decided by declarations; a release file that only tests call can still sit in it
(item 14 counts such functions).
Counts that use regular expressions can miss multi-line forms; the patterns are in the commands named in each item.

## Routing, 2026-11-25

Where each item went when the coordinator read the ledger at about 16:50 (-05:00).
Items that wait name what they wait on.

- Item 1: the issue 6415 round (`fix/git-policy-cli-named-pipe-in-a-record-folder`) was told its three sites.
- Item 2: after issue 6415's fix lands, a source guard on raw reads and migration rounds by class.
- Item 3: after the harness rounds of items 4, 5, 10 and 11 merge (it needs one child runner);
  the `process::signal_relay` tests of 3f went to the test-child round,
  because they failed three and then hung 31 minutes in a threaded run on main's code.
- Item 4: the threaded-test round (`fix/git-policy-cli-lock-controls-that-fail-only-in-a-threaded-run`) owns
  `fixture()` and was told of the binary target's copy and the nine confirmed repeats.
- Item 5: the test-child round (`test/git-policy-cli-test-children-owned-and-signalled-only-by-their-starter`) was
  told to add the death signal to the lib's `spawn_owned`.
- Item 6: merged as `124d251aaf` (2026-11-25T16:45:48-05:00).
- Item 7: after the harness rounds merge, one round per folder.
- Item 8: a question for the owner.
- Item 9: its own round.
- Items 10 and 11: one round after the threaded-test and test-child rounds merge, since all three edit
  `testing/test_support.rs`.
- Item 12: after the live rounds of 2026-11-25 merge, since it edits many of their files.
- Item 13: with the move of each pair to the async runtime.
- Item 14: its own round, which keeps every piece built ahead and unwired by
  `doc/decision/zfs-route-later-pieces-built-ahead-unwired-and-up-to-64-rounds-at-once.md`.
- Item 15: one round per file once the repository's linter runs in this package's worktrees.
- Items 16 and 17: a question for the owner.
- Item 18: the `merge/` reads were converted in `fix/git-policy-cli-merge-file-calls-off-the-runtime`
  (2026-11-25, `713b8c1f53` to `7304e3db72`); the remaining rows wait on the owner's tokio plan.
