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

/// The one copy of the built executable per build that fixtures on another filesystem link.
#[path = "binary_wrapper_store.rs"]
mod wrapper_store;

/// The stored copy's name, its lock-free publish, its pruning and the sharing of one wrapper file by fixtures.
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

/// The lines added after Git's answer to `--version`, `-v`, `--help` and `-h`.
#[path = "binary_announced_tests.rs"]
mod announced;

/// Wrapper self-exclusion and recursion prevention on PATH.
#[path = "binary_resolution_tests.rs"]
mod resolution;

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

/// Taking one side of a conflicted file during a merge, rebase, cherry-pick or revert: saved first, conflicted paths only.
#[path = "binary_conflict_side_tests.rs"]
mod conflict_side;

/// Every command form that can discard uncommitted state: rejected, allowed in a clean linked worktree, or hatched.
#[path = "binary_discard_tests.rs"]
mod discard;

/// The hints of blocks outside the discard guard reach the caller beside an unchanged message.
#[path = "binary_hint_tests.rs"]
mod hints;

/// `markdown/autofix` with the linter built from this repository,
///  a missing one and a failing one.
#[path = "binary_markdown_tests.rs"]
mod markdown;

/// `pathBytes` and `changedPathBytes` for a file whose name is not UTF-8.
#[path = "binary_event_path_tests.rs"]
mod event_path;

/// The `git cli-git` management namespace.
#[path = "binary_management_tests.rs"]
mod management;
/// `git cli-git migrate-to-subvolume` and the doctor's filesystem section through the executable.
#[path = "binary_subvolume_tests.rs"]
mod subvolume;

#[path = "binary_coordination_tests.rs"]
mod coordination;
/// No external hook runs, and configured hooks are reported.
#[path = "binary_external_hook_tests.rs"]
mod external_hooks;
/// The dependent-version policy on direct check and fix.
#[path = "binary_dependent_version_tests.rs"]
mod dependent_version;

/// A true merge in a checkout whose index holds another session's staged work: equal to native Git's merge on a
/// clean copy, nothing of the other work disturbed, refusals that change nothing, and recovery after a kill.
#[path = "binary_merge_isolation_tests.rs"]
mod merge_isolation;

/// Real commits through the transaction: landing, policies, replay and conflict.
#[path = "binary_commit_tests.rs"]
mod commit;

/// The Git version text a commit records in the user's cache directory, and when it trusts it.
#[path = "binary_version_cache_tests.rs"]
mod version_cache;

/// Finding `strace`, which the budget, fault and child-bound controls run a wrapped command under.
#[path = "binary_strace_support.rs"]
mod strace;

/// Reads a `strace` trace of a wrapped commit into process starts and sync calls per program, for the budget controls.
#[path = "binary_budget_trace.rs"]
mod budget_trace;

/// Ratchet budgets: the process starts and fsync-class calls of a wrapped commit can only go down.
#[path = "binary_budget_tests.rs"]
mod budget;

/// Content a parent commit's tree already holds is not scanned again by forbidden-strings.
#[path = "binary_known_content_tests.rs"]
mod known_content;

/// `pathBytes` for a file whose name is not UTF-8 through the commit path.
#[path = "binary_event_path_commit_tests.rs"]
mod event_path_commit;

/// `mono/catalog-specifiers` through real commits and direct checks, and the repository's own configuration.
#[path = "binary_catalog_specifiers_tests.rs"]
mod catalog_specifiers;

/// The linked-worktree ignored-state copy: the rules file reaches new worktrees, and an interrupted copy recovers.
#[path = "binary_worktree_copy_tests.rs"]
mod worktree_copy;

/// The cheap-worktree ladder: new worktrees are subvolumes, a forwarded removal destroys them, other forms stay Git's.
#[path = "binary_worktree_ladder_tests.rs"]
mod worktree_ladder;

/// Rung 1 of the cheap-worktree ladder: a snapshot of a clean subvolume worktree equals Git's own checkout.
#[path = "binary_worktree_snapshot_tests.rs"]
mod worktree_snapshot;

/// Rung 2 of the cheap-worktree ladder: clones of a clean worktree's tracked files equal Git's own checkout.
#[path = "binary_worktree_clone_tests.rs"]
mod worktree_clone;

/// The environment of the Git children the auto-push and the manual-push probe start.
#[path = "binary_push_environment_tests.rs"]
mod push_environment;

/// An explicit push that lost a race to another session: the merge is published by a push planned and run inside the same process.
#[path = "binary_push_stacking_tests.rs"]
mod push_stacking;

/// The Git LFS upload before a push: every push the executable starts carries it, and a failure stops the push.
#[path = "binary_lfs_pre_push_tests.rs"]
mod lfs_pre_push;

/// The parent that owns an LFS push explains a failed upload from Git's own trace, and no wrapper is started for it.
#[path = "binary_lfs_parent_tests.rs"]
mod lfs_parent;

/// The manual-push probe through the whole wrapper: verbosity cannot bypass a policy, and upstream options publish.
#[path = "binary_push_probe_tests.rs"]
mod push_probe;

/// The same upload with the real `git-lfs` against an LFS server written inline.
#[path = "binary_lfs_server_tests.rs"]
mod lfs_server;

/// Traditional hooks never run through the wrapper, and `git cli-git doctor` explains the Git LFS shims.
#[path = "binary_hooks_off_tests.rs"]
mod hooks_off;

/// A configuration cannot make a Git the wrapper started run a hook through an alias, a monitor program or an editor.
#[path = "binary_hooks_off_escape_tests.rs"]
mod hooks_off_escape;

/// The notices of a commit that waits: what it waits for, who holds it, the repeat, and the bound.
#[path = "binary_wait_notice_tests.rs"]
mod wait_notice;

/// The fault injection and the recovery invariants of the operating conditions in the README.
#[path = "binary_fault_support.rs"]
mod fault;

/// The wrapper run with no `timeout` in front of it, in a process group of its own, for controls that signal or limit it.
#[path = "binary_fault_run.rs"]
mod fault_run;

/// Signals sent to the wrapper alone and to its process group.
#[path = "binary_fault_signal_tests.rs"]
mod fault_signal;

/// Descriptor, process and address-space limits near exhaustion on the wrapper and on the service, and a single processor.
#[path = "binary_fault_limits_tests.rs"]
mod fault_limits;

/// Slow storage, a full disk, failing writes and kills at any call, injected with `strace` into the service and the wrapper's flows.
#[path = "binary_fault_trace_tests.rs"]
mod fault_trace;

/// Standard streams that are closed, unread or full.
#[path = "binary_fault_stream_tests.rs"]
mod fault_stream;

/// Remote trouble met by the push after a commit and by an explicit push.
#[path = "binary_fault_remote_tests.rs"]
mod fault_remote;

/// The experiments behind the hang verdict for the post-commit auto-push with a Git LFS upload.
#[path = "binary_lfs_hang_tests.rs"]
mod lfs_hang;

/// The wrapped commit command through the resident service, compared with native Git.
#[path = "binary_service_commit_tests.rs"]
mod service_commit;

/// A commit whose prepared content was pruned before it landed: prepared again once, then a plain failure.
#[path = "binary_service_pruned_tests.rs"]
mod service_pruned;

/// Two commits of one file in one worktree, the second landing behind the first: its captured bytes merged, never re-read.
#[path = "binary_service_replay_tests.rs"]
mod service_replay;

/// A backup push the service made after a commit returned and the remote refused, shown by the next wrapper command.
#[path = "binary_push_failure_tests.rs"]
mod push_failure;

/// Merge, pull's merge, cherry-pick, revert and am hand the push to the service, as a commit does; a rebase does not.
#[path = "binary_push_trigger_tests.rs"]
mod push_trigger;

/// The message and identity options through the service, compared with native Git field by field.
#[path = "binary_service_options_tests.rs"]
mod service_options;

/// Index commits and `commit -a` through the service, compared with native Git including what is left behind.
#[path = "binary_service_selection_tests.rs"]
mod service_selection;

/// Amends through the service, compared with native Git field by field including what is left behind.
#[path = "binary_service_amend_tests.rs"]
mod service_amend;

/// The twin harness of the landing controls: native Git in one byte copy, the service in the other, every field compared.
#[path = "binary_service_landing_support.rs"]
mod service_landing_support;

/// Selection command lines (`-i`, `-p`, `--interactive`, a pathspec file) and the dry run, compared with native Git.
#[path = "binary_service_landing_selection_tests.rs"]
mod service_landing_selection;

/// States that used to be declined (a conclusion, a detached head, a branch not yet born, an unmerged index, nothing to
///  commit, a rewrite, awkward change lists), compared with native Git.
#[path = "binary_service_landing_state_tests.rs"]
mod service_landing_state;

/// A landing while a foreign `git commit` holds the index lock: the wait, the order, and the limit.
#[path = "binary_service_landing_lock_tests.rs"]
mod service_landing_lock;

/// The timing record of a wrapped commit and the service's log lines,
/// which split a slow commit by phase.
#[path = "binary_commit_timing_tests.rs"]
mod commit_timing;

/// A landing that waits for a foreign index lock in one repository, beside a landing in another.
#[path = "binary_service_lane_tests.rs"]
mod service_lane;

/// The scratch directory of a commit in preparation, removed by the service when the client's connection closes.
#[path = "binary_service_scratch_tests.rs"]
mod service_scratch;

/// Commits real Git builds for the service (an editor, a signature), compared with the transaction's commit.
#[path = "binary_service_native_tests.rs"]
mod service_native;

/// The route of a commit and the wrapper's own provisioning of socket activation, against a stand-in session bus.
#[path = "binary_service_provision_tests.rs"]
mod service_provision;

/// A machine with no way to reach or start a service stops a commit plainly; a directory with no repository gets Git's words.
#[path = "binary_service_no_manager_tests.rs"]
mod service_no_manager;

/// The upgrade from the 0.5.4 release, run with that release's own service program.
#[path = "binary_service_upgrade_tests.rs"]
mod service_upgrade;

/// The resident commit service as a real process: kills at each point of a batch, requests sent again, late and older services.
#[path = "binary_service_tests.rs"]
mod service;

/// A Git step past its class bound is stopped, named with its class, and the repository recovers.
#[path = "binary_child_bound_tests.rs"]
mod child_bound;

/// What the person's terminal shows of a commit the service landed: warnings, one summary line, the waits.
#[path = "binary_client_service_tests.rs"]
mod client_service;
