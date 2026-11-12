//! What:
//!  Consumer-level controls that start the built native executable.
//! Why:
//!  Forwarding,
//!  self-exclusion,
//!  policy decisions and refusals are promises about the
//!      program a caller runs as `git`,
//!  so they are checked through that program
//!      against real Git 2.56.0,
//!  never only through library functions.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // const wrapped = spawnSync(join(fixture, 'bin/git'), ['--version'], { env });
//! ```
#![cfg(unix)]

/// Fixtures and bounded process helpers shared by every binary-level control.
#[path = "binary_support.rs"]
mod support;

/// Holding a wrapped commit between its preparation and its landing,
/// and landing another session's commit in that window.
#[path = "binary_hold_at_prepare_support.rs"]
mod hold_at_prepare;

/// The names of the two programs a control starts and the time bound every wrapped command runs under.
#[path = "binary_support_program_names_and_time_bound.rs"]
mod support_program_names_and_time_bound;

/// The names a wrapped command left behind in a repository's two scratch places,
/// waited for within a bound.
#[path = "binary_support_scratch_left_behind.rs"]
mod support_scratch_left_behind;

/// The limits a control puts on a fixture service process,
/// and how it applies them.
#[path = "binary_support_resource_limits.rs"]
mod support_resource_limits;

/// The fixture of a control and the resident service process it may start.
#[path = "binary_support_fixture_service.rs"]
mod support_fixture_service;

/// How a fixture tells that its service is at work and that it is ready.
#[path = "binary_support_fixture_service_readiness.rs"]
mod support_fixture_service_readiness;

/// What a fixture's services write on their standard error:
/// the pipe that keeps it while they run,
/// and the end of it that a failure carries.
#[path = "binary_support_fixture_service_log.rs"]
mod support_fixture_service_log;

/// Making and removing the disposable directory of a fixture and the wrapper link inside it.
#[path = "binary_support_fixture_directories.rs"]
mod support_fixture_directories;

/// Running a wrapped command with a time bound and reading what it did.
#[path = "binary_support_observation.rs"]
mod support_observation;

/// The one place a control says that it needs a btrfs fixture root,
/// and what it does where there is none.
#[path = "binary_support_btrfs_controls.rs"]
mod support_btrfs_controls;

/// The hang guard:
/// a thread that ends a wrapped command whose process tree shows no progress or burns too much processor time.
#[path = "binary_supervisor.rs"]
mod supervisor;

/// The readers of `/proc` the hang guard judges by:
/// stat,
/// io,
/// every task's state,
/// the tree,
/// and one look at it.
#[path = "binary_supervisor_proc.rs"]
mod supervisor_proc;

/// The pinned identity of a process the hang guard may signal,
/// and the ending of a tree through those identities.
#[path = "binary_supervisor_pin.rs"]
mod supervisor_pin;

/// The supervisor's judge,
/// readers and monitor,
/// and real commands that loop,
/// deadlock,
/// wait and crawl.
#[path = "binary_supervisor_tests.rs"]
mod supervisor_guard;

/// The one copy of the built executable per build that fixtures on another filesystem link.
#[path = "binary_wrapper_store.rs"]
mod wrapper_store;

/// The stored copy's name,
/// its lock-free publish,
/// its pruning and the sharing of one wrapper file by fixtures.
#[path = "binary_wrapper_store_tests.rs"]
mod wrapper_store_guard;

/// Where fixtures live and that they go away however their test ends.
#[path = "binary_fixture_tests.rs"]
mod fixture_guard;

/// Exact forwarding of arguments,
///  streams,
///  exit codes,
///  signals and environment.
#[path = "binary_forwarding_tests.rs"]
mod forwarding;

/// The lines added after Git's answer to `--version`,
/// `-v`,
/// `--help` and `-h`.
#[path = "binary_announced_tests.rs"]
mod announced;

/// Wrapper self-exclusion and recursion prevention on PATH.
#[path = "binary_resolution_tests.rs"]
mod resolution;

/// The wrapper never starts itself as Git:
/// launchers,
/// links,
/// copies and other releases behind counting stand-ins.
#[path = "binary_never_self_start_tests.rs"]
mod never_self_start;

/// With no real Git after the wrapper on `PATH`,
/// the wrapper falls back to the Git its commit service was set up with
///  when that record passes the `PATH` candidate checks,
/// and otherwise keeps its stop.
#[path = "binary_installed_git_fallback_tests.rs"]
mod installed_git_fallback;

/// Configuration loading and the read-only fast path.
#[path = "binary_policy_tests.rs"]
mod policy;

/// Wrapper controls,
///  escape hatches and commands Git refuses.
#[path = "binary_controls_tests.rs"]
mod controls;

/// The refusal frontier:
///  commands that need work this executable does not do.
#[path = "binary_frontier_tests.rs"]
mod frontier;

/// The pre-forward built-in policies and the fixed transforms.
#[path = "binary_builtin_tests.rs"]
mod builtin;

/// The command a Git alias really runs,
///  compared with the command real Git runs for the same configuration.
#[path = "binary_alias_expansion_tests.rs"]
mod alias_expansion;

/// Taking one side of a conflicted file during a merge,
/// rebase,
/// cherry-pick or revert:
/// saved first,
/// conflicted paths only.
#[path = "binary_conflict_side_tests.rs"]
mod conflict_side;

/// Every command form that can discard uncommitted state:
/// rejected,
/// allowed in a clean linked worktree,
/// or hatched.
#[path = "binary_discard_tests.rs"]
mod discard;

/// The remote settings every worktree of a repository shares:
/// refused from a linked worktree,
/// unchanged in the main checkout,
/// and readable everywhere.
#[path = "binary_shared_remote_tests.rs"]
mod shared_remote;

/// A clone made with `--no-checkout` has no index file,
/// and the guard of uncommitted work lets its first checkout run.
#[path = "binary_no_checkout_clone_tests.rs"]
mod no_checkout_clone;

/// A clone a build script made in its own output directory belongs to the build step,
/// so the guard of uncommitted work exempts it.
#[path = "binary_build_script_clone_tests.rs"]
mod build_script_clone;

/// The hints of blocks outside the discard guard reach the caller beside an unchanged message.
#[path = "binary_hint_tests.rs"]
mod hints;

/// Every command a message prints as a remedy runs as printed,
/// in the state that prints it.
#[path = "binary_printed_remedies_tests.rs"]
mod printed_remedies;

/// `markdown/autofix` with the linter built from this repository,
///  a missing one and a failing one.
#[path = "binary_markdown_tests.rs"]
mod markdown;

/// `pathBytes` and `changedPathBytes` for a file whose name is not UTF-8.
#[path = "binary_event_path_tests.rs"]
mod event_path;

/// The program under its new name and its earlier names:
/// the command word,
/// the settings file and the kept folders.
#[path = "binary_program_rename_tests.rs"]
mod program_rename_guard;

/// The `git monochromatic-git-policy-cli` management namespace.
#[path = "binary_management_tests.rs"]
mod management;
/// `git monochromatic-git-policy-cli migrate-to-subvolume` and the doctor's filesystem section through the executable.
#[path = "binary_subvolume_tests.rs"]
mod subvolume;
/// `git monochromatic-git-policy-cli convert-worktree-to-subvolume` started the way a person starts it,
/// with no forward-target variable.
#[path = "binary_convert_tests.rs"]
mod convert;

#[path = "binary_coordination_tests.rs"]
mod coordination;
/// No external hook runs,
/// and configured hooks are reported.
#[path = "binary_external_hook_tests.rs"]
mod external_hooks;
/// The dependent-version policy on direct check and fix.
#[path = "binary_dependent_version_tests.rs"]
mod dependent_version;

/// A true merge in a checkout whose index holds another session's staged work:
/// equal to native Git's merge on a
/// clean copy,
/// nothing of the other work disturbed,
/// refusals that change nothing,
/// and recovery after a kill.
#[path = "binary_merge_isolation_tests.rs"]
mod merge_isolation;

/// A merge over the person's own unsaved or staged work in the paths it changes:
/// the merge lands,
/// their bytes are carried forward,
/// a conflict keeps both sides in the file,
/// and a kill at each step of the carry leaves either the checkout as it was or the whole result.
#[path = "binary_merge_carries_edits_tests.rs"]
mod merge_carries_edits;

/// The kill controls of the carry:
/// one phase per step of the carry,
/// and the next command's recovery.
#[path = "binary_merge_carries_edits_kill_tests.rs"]
mod merge_carries_edits_kill;

/// One merge installs in a worktree at a time:
/// a second merge waits for the first and the worktree ends in one
/// whole state,
/// a merge that is partly installed names its state to a read and stops a write,
/// and a checkout that never gets its
/// content ends and puts back what it held.
#[path = "binary_merge_one_install_at_a_time_tests.rs"]
mod merge_one_install_at_a_time;

/// Another command landing a commit on the branch while a landed merge still carries:
/// the later commit stands,
/// and the person's work is merged onto the branch as it is now.
#[path = "binary_merge_carries_edits_moved_tests.rs"]
mod merge_carries_edits_moved;

/// The word-level second merge pass after a merge,
/// rebase,
/// cherry-pick or revert that Git stops with a conflict.
#[path = "binary_word_merge_tests.rs"]
mod word_merge;

/// The stand-in Git that holds the second pass at its own attribute query,
/// so a control of the pass acts inside the pass itself.
#[path = "binary_word_merge_pass_gate.rs"]
mod word_merge_pass_gate;

/// The gate's own control:
/// a command that never asks for the attribute leaves the control woken by the command's end.
#[path = "binary_word_merge_pass_gate_tests.rs"]
mod word_merge_pass_gate_guard;

/// The finish after the second pass:
/// the operation ends like a clean run,
/// and `--no-commit`,
/// `--squash` and `--edit` stop it.
#[path = "binary_word_merge_finish_tests.rs"]
mod word_merge_finish;

/// The harness that acts on a repository when a running command shows a line on standard error.
#[path = "binary_held_lock_support.rs"]
mod held_lock_support;

/// A `git pull` that merges:
/// completed in isolation like `git merge`,
/// and every other pull left to native Git.
#[path = "binary_pull_isolation_tests.rs"]
mod pull_isolation;

/// A merge that waits for a held `index.lock` and lands on a branch that moved underneath it.
#[path = "binary_merge_never_fails_tests.rs"]
mod merge_never_fails;

/// The lines a merge prints about a path it kept whole,
/// and the commands in them,
/// run as the person's shell reads them.
#[path = "binary_merge_notice_tests.rs"]
mod merge_notice;

/// Real commits through the transaction:
/// landing,
/// policies,
/// replay and conflict.
#[path = "binary_commit_tests.rs"]
mod commit;

/// The Git version text a commit records in the user's cache directory,
/// and when it trusts it.
#[path = "binary_version_cache_tests.rs"]
mod version_cache;

/// Finding `strace`,
/// which the budget,
/// fault and child-bound controls run a wrapped command under.
#[path = "binary_strace_support.rs"]
mod strace;

/// Reads a `strace` trace of a wrapped commit into process starts and sync calls per program,
/// for the budget controls.
#[path = "binary_budget_trace.rs"]
mod budget_trace;

/// Ratchet budgets:
/// the process starts and fsync-class calls of a wrapped commit can only go down.
#[path = "binary_budget_tests.rs"]
mod budget;

/// A forwarded command makes no thread or child of its own before Git replaces the wrapper,
/// so a runtime is never built for it.
#[path = "binary_forwarded_command_makes_no_thread_tests.rs"]
mod forwarded_command_makes_no_thread;

/// The runtime build finishes and the command carries on under a process limit and under address-space limits,
/// with Git as the control.
#[path = "binary_runtime_build_under_limits_tests.rs"]
mod runtime_build_under_limits;

/// Content a parent commit's tree already holds is not scanned again by forbidden-strings.
#[path = "binary_known_content_tests.rs"]
mod known_content;

/// `pathBytes` for a file whose name is not UTF-8 through the commit path.
#[path = "binary_event_path_commit_tests.rs"]
mod event_path_commit;

/// `mono/catalog-specifiers` through real commits and direct checks,
/// and the repository's own configuration.
#[path = "binary_catalog_specifiers_tests.rs"]
mod catalog_specifiers;

/// The linked-worktree ignored-state copy:
/// the rules file reaches new worktrees,
/// and an interrupted copy recovers.
#[path = "binary_worktree_copy_tests.rs"]
mod worktree_copy;

/// Worktree commands beside an add whose real Git still runs:
/// none waits,
/// and each new worktree gets exactly its own copy.
#[path = "binary_worktree_copy_concurrent_tests.rs"]
mod worktree_copy_concurrent;

/// The copy for every form of `git worktree add` that Git accepts,
/// and none for each form Git rejects.
#[path = "binary_worktree_copy_forms_tests.rs"]
mod worktree_copy_forms;

/// The cheap-worktree ladder:
/// new worktrees are subvolumes,
/// a forwarded removal destroys them,
/// other forms stay Git's.
#[path = "binary_worktree_ladder_tests.rs"]
mod worktree_ladder;

/// Rung 1 of the cheap-worktree ladder:
/// a snapshot of a clean subvolume worktree equals Git's own checkout.
#[path = "binary_worktree_snapshot_tests.rs"]
mod worktree_snapshot;

/// Rung 2 of the cheap-worktree ladder:
/// clones of a clean worktree's tracked files equal Git's own checkout.
#[path = "binary_worktree_clone_tests.rs"]
mod worktree_clone;

/// The line that says why an add took the slow way:
/// the cause,
/// the remedy,
/// `--quiet`,
/// and the journal entry.
#[path = "binary_worktree_slow_route_tests.rs"]
mod worktree_slow_route;

/// A main checkout with uncommitted changes is snapshotted and restored to what Git's own checkout makes.
#[path = "binary_worktree_dirty_snapshot_tests.rs"]
mod worktree_dirty_snapshot;

/// The checkout of a new worktree gets Git's parallel checkout only where the person's configuration sets no `checkout.workers`.
#[path = "binary_worktree_checkout_workers_tests.rs"]
mod worktree_checkout_workers;

/// The commit a reference lock holds in a linked worktree:
/// a removal or a prune keeps it where Git cannot reach it,
/// and a removal stops when no reference can hold it.
#[path = "binary_worktree_removal_held_commit_tests.rs"]
mod worktree_removal_held_commit;

/// The environment of the Git children the auto-push and the manual-push probe start.
#[path = "binary_push_environment_tests.rs"]
mod push_environment;

/// An explicit push that lost a race to another session:
/// the merge is published by a push planned and run inside the same process.
#[path = "binary_push_stacking_tests.rs"]
mod push_stacking;

/// The Git LFS upload before a push:
/// every push the executable starts carries it,
/// and a failure stops the push.
#[path = "binary_lfs_pre_push_tests.rs"]
mod lfs_pre_push;

/// The parent that owns an LFS push explains a failed upload from Git's own trace,
/// and no wrapper is started for it.
#[path = "binary_lfs_parent_tests.rs"]
mod lfs_parent;

/// The manual-push probe through the whole wrapper:
/// verbosity cannot bypass a policy,
/// and upstream options publish.
#[path = "binary_push_probe_tests.rs"]
mod push_probe;

/// A push that Git itself refuses (an absent branch to delete,
/// a refspec naming nothing,
/// no such remote) ends as Git ends it.
#[path = "binary_push_refusal_tests.rs"]
mod push_refusal;

/// A push the wrapper cannot scope fails with the hint of its cause,
/// not the one hint of its code.
#[path = "binary_push_scope_tests.rs"]
mod push_scope;

/// The same upload with the real `git-lfs` against an LFS server written inline.
#[path = "binary_lfs_server_tests.rs"]
mod lfs_server;

/// The configuration of a push,
/// given on its command line,
/// reaches the real `git-lfs` upload,
/// so an object reaches the remote the ref is published to.
#[path = "binary_lfs_command_scope_tests.rs"]
mod lfs_command_scope;

/// Traditional hooks never run through the wrapper,
/// and `git monochromatic-git-policy-cli doctor` explains the Git LFS shims.
#[path = "binary_hooks_off_tests.rs"]
mod hooks_off;

/// A configuration cannot make a Git the wrapper started run a hook through an alias,
/// a monitor program or an editor.
#[path = "binary_hooks_off_escape_tests.rs"]
mod hooks_off_escape;

/// The notices of a commit that waits:
/// what it waits for,
/// who holds it,
/// the repeat,
/// and the bound.
#[path = "binary_wait_notice_tests.rs"]
mod wait_notice;

/// The fault injection and the recovery invariants of the operating conditions in the README.
#[path = "binary_fault_support.rs"]
mod fault;

/// The wrapper run with no `timeout` in front of it,
/// in a process group of its own,
/// for controls that signal or limit it.
#[path = "binary_fault_run.rs"]
mod fault_run;

/// The hang guard of those runs:
/// a command that keeps moving is left alone,
/// and one that stops moving is ended with the words that name the silence.
#[path = "binary_fault_run_tests.rs"]
mod fault_run_guard;

/// Signals sent to the wrapper alone and to its process group.
#[path = "binary_fault_signal_tests.rs"]
mod fault_signal;

/// Descriptor,
/// process and address-space limits near exhaustion on the wrapper and on the service,
/// and a single processor.
#[path = "binary_fault_limits_tests.rs"]
mod fault_limits;

/// Slow storage,
/// a full disk,
/// failing writes and kills at any call,
/// injected with `strace` into the service and the wrapper's flows.
#[path = "binary_fault_trace_tests.rs"]
mod fault_trace;

/// Standard streams that are closed,
/// unread or full.
#[path = "binary_fault_stream_tests.rs"]
mod fault_stream;

/// Remote trouble met by the push after a commit and by an explicit push.
#[path = "binary_fault_remote_tests.rs"]
mod fault_remote;

/// The experiments behind the hang verdict for the post-commit auto-push with a Git LFS upload.
#[path = "binary_lfs_hang_tests.rs"]
mod lfs_hang;

/// The wrapped commit command through the resident service,
/// compared with native Git.
#[path = "binary_service_commit_tests.rs"]
mod service_commit;

/// A commit whose command line names the repository:
/// `--git-dir` with `--work-tree`,
/// `GIT_DIR` with `GIT_WORK_TREE`,
/// and a relative `-C`.
#[path = "binary_repository_naming_commit_tests.rs"]
mod repository_naming_commit;

/// A commit whose branch another command moved while it was prepared:
/// a first commit on a branch that did not exist yet,
/// a `--patch` selection,
/// and a line that borrows its message and author.
#[path = "binary_moved_branch_commit_tests.rs"]
mod moved_branch_commit;

/// The service as a mode of the one executable:
/// refused starts,
/// the wrapper as `--git`,
/// and the words behind a global option.
#[path = "binary_service_mode_tests.rs"]
mod service_mode;

/// The upgrade from the release with the service as a second program:
/// the unit is rewritten,
/// the old service retires.
#[path = "binary_service_upgrade_one_executable_tests.rs"]
mod service_upgrade_one_executable;

/// A commit whose prepared content was pruned before it landed:
/// prepared again once,
/// then a plain failure.
#[path = "binary_service_pruned_tests.rs"]
mod service_pruned;

/// Two commits of one file in one worktree,
/// the second landing behind the first:
/// its captured bytes merged,
/// never re-read.
#[path = "binary_service_replay_tests.rs"]
mod service_replay;

/// A commit whose branch write meets a reference lock:
/// the wait for a live holder,
/// and the lock that nothing proves gone,
/// named with the command that clears it.
#[path = "binary_service_reference_lock_tests.rs"]
mod service_reference_lock;

/// A backup push the service made after a commit returned and the remote refused,
/// shown by the next wrapper command.
#[path = "binary_push_failure_tests.rs"]
mod push_failure;

/// A landed commit whose wrapper was killed between promoting its held push and waking the service:
/// the next command says the push is queued,
/// and a service that starts pushes it with no further commit.
#[path = "binary_push_queued_after_a_killed_wrapper_tests.rs"]
mod push_queued_after_a_killed_wrapper;

/// A held push whose commit a later push of the branch carried to the remote:
/// the record is settled on Git's proof,
/// so no command says the opposite of `git status`.
#[path = "binary_held_push_carried_by_a_later_push_tests.rs"]
mod held_push_carried_by_a_later_push;

/// A landed commit whose command died before it settled the push record:
/// the verdict it left settles the record,
/// and a record with none names the command and what sends the commit.
#[path = "binary_held_push_settled_by_the_verdict_tests.rs"]
mod held_push_settled_by_the_verdict;

/// A commit a post-commit gate refused:
/// it stays local past the next commit of its branch,
/// and every command names the commit and the check that hold the branch back.
#[path = "binary_push_stops_before_a_refused_commit_tests.rs"]
mod push_stops_before_a_refused_commit;

/// The report a reconcile owes when it published a merge but could not move the local branch,
/// or moved it beside uncommitted work,
/// shown once by a later command.
#[path = "binary_reconcile_report_shown_once_tests.rs"]
mod reconcile_report_shown_once;

/// Merge,
/// pull's merge,
/// cherry-pick,
/// revert and am hand the push to the service,
/// as a commit does;
/// a rebase does not.
#[path = "binary_push_trigger_tests.rs"]
mod push_trigger;
/// A fetch after a Git was killed while it held reference locks:
/// removed on a proof,
/// described and kept otherwise.
#[path = "binary_stale_ref_lock_tests.rs"]
mod stale_ref_lock;


/// The message and identity options through the service,
/// compared with native Git field by field.
#[path = "binary_service_options_tests.rs"]
mod service_options;

/// Index commits and `commit -a` through the service,
/// compared with native Git including what is left behind.
#[path = "binary_service_selection_tests.rs"]
mod service_selection;

/// Amends through the service,
/// compared with native Git field by field including what is left behind.
#[path = "binary_service_amend_tests.rs"]
mod service_amend;

/// The twin harness of the landing controls:
/// native Git in one byte copy,
/// the service in the other,
/// every field compared.
#[path = "binary_service_landing_support.rs"]
mod service_landing_support;

/// Selection command lines (`-i`,
/// `-p`,
/// `--interactive`,
/// a pathspec file) and the dry run,
/// compared with native Git.
#[path = "binary_service_landing_selection_tests.rs"]
mod service_landing_selection;

/// States that used to be declined (a conclusion,
/// a detached head,
/// a branch not yet born,
/// an unmerged index,
/// nothing to
///  commit,
/// a rewrite,
/// awkward change lists),
/// compared with native Git.
#[path = "binary_service_landing_state_tests.rs"]
mod service_landing_state;

/// A landing while a foreign `git commit` holds the index lock:
/// the wait,
/// the order,
/// and the limit.
#[path = "binary_service_landing_lock_tests.rs"]
mod service_landing_lock;

/// The abort forms while the operation they undo waits for its conflicts:
/// what runs,
/// what is saved first,
/// and what is left to Git.
#[path = "binary_abort_during_an_operation_tests.rs"]
mod abort_during_an_operation;

/// The rerere cache a concluded merge leaves behind,
/// compared with real Git.
#[path = "binary_merge_rr_cleanup_tests.rs"]
mod merge_rr_cleanup;

/// The timing record of a wrapped commit and the service's log lines,
/// which split a slow commit by phase.
#[path = "binary_commit_timing_tests.rs"]
mod commit_timing;

/// A landing that waits for a foreign index lock in one repository,
/// beside a landing in another.
#[path = "binary_service_lane_tests.rs"]
mod service_lane;

/// The scratch directory of a commit in preparation,
/// removed by the service when the client's connection closes.
#[path = "binary_service_scratch_tests.rs"]
mod service_scratch;

/// The pool of finished shadow repositories of a worktree Git no longer lists:
/// a wrapped command removes it,
/// and one of a worktree Git lists never.
#[path = "binary_shadow_pool_tests.rs"]
mod shadow_pool;

/// A pooled shadow member the commit refuses:
/// nothing on the terminal,
/// and the refusal named in the commit's own timing record.
#[path = "binary_shadow_pool_refusal_tests.rs"]
mod shadow_pool_refusal;

/// The ends of the waits on the commit path:
/// a commit withdrawn at its bound never lands,
/// not even after a restart of the service.
#[path = "binary_service_withdraw_tests.rs"]
mod service_withdraw;

/// A commit service whose runtime build outlasted its deadline:
/// the commit lands through it,
/// and its log names the stand-in.
#[path = "binary_service_late_runtime_build_tests.rs"]
mod service_late_runtime_build;

/// The client's wait for the commit service:
/// heartbeats,
/// silence,
/// a start the manager reports failed,
/// a late start.
#[path = "binary_service_heartbeat_tests.rs"]
mod service_heartbeat;

/// The connections the service holds open,
/// counted against its threads:
/// a connection is a task.
#[path = "binary_service_connections_without_a_thread_each_tests.rs"]
mod service_connections_tasks;

/// Commits real Git builds for the service (an editor,
/// a signature),
/// compared with the transaction's commit.
#[path = "binary_service_native_tests.rs"]
mod service_native;

/// A commit with named paths during a revert,
/// a cherry-pick or a merge,
/// compared with real Git.
#[path = "binary_service_sequencer_tests.rs"]
mod service_sequencer;

/// A commit of named paths while another session's merge is unfinished lands,
/// and the merge is carried forward.
#[path = "binary_commit_during_merge_tests.rs"]
mod commit_during_merge;

/// A commit that concludes a merge,
/// a cherry-pick or a revert,
/// and an amend,
/// whose branch another command moved while the commit waited.
#[path = "binary_conclusion_on_a_moved_branch_tests.rs"]
mod conclusion_on_a_moved_branch;

/// Guard:
/// no option of the wrapper's follows the person's `--` in any Git start,
/// and no `--quiet` is added.
#[path = "binary_option_order_guard_tests.rs"]
mod option_order_guard;

/// The route of a commit and the wrapper's own provisioning of socket activation,
/// against a stand-in session bus.
#[path = "binary_service_provision_tests.rs"]
mod service_provision;

/// A machine with no way to reach or start a service stops a commit plainly;
/// a directory with no repository gets Git's words.
#[path = "binary_service_no_manager_tests.rs"]
mod service_no_manager;

/// The upgrade from the 0.5.4 release,
/// run with that release's own service program.
#[path = "binary_service_upgrade_tests.rs"]
mod service_upgrade;

/// The resident commit service as a real process:
/// kills at each point of a batch,
/// requests sent again,
/// late and older services.
#[path = "binary_service_tests.rs"]
mod service;

/// A Git step past its class bound is stopped,
/// named with its class,
/// and the repository recovers.
#[path = "binary_child_bound_tests.rs"]
mod child_bound;

/// What the person's terminal shows of a commit the service landed:
/// warnings,
/// one summary line,
/// the waits.
#[path = "binary_client_service_tests.rs"]
mod client_service;

/// A stop of the commit service during a landing:
/// the shutdown ends the Gits it waits on within a bound,
/// gives up the index lock and keeps the commit's work on its side ref.
#[path = "binary_service_stop_ends_its_gits_tests.rs"]
mod service_stop_ends_its_gits;

/// The package-manager repair of a new worktree:
/// pnpm,
/// Cargo and the toolchain launchers work at the new location.
#[path = "binary_package_repair_tests.rs"]
mod package_repair;
