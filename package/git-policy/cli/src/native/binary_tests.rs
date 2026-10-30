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

/// Where fixtures live and that they go away however their test ends.
#[path = "binary_fixture_tests.rs"]
mod fixture_guard;

/// Exact forwarding of arguments,
///  streams,
///  exit codes,
///  signals and environment.
#[path = "binary_forwarding_tests.rs"]
mod forwarding;

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

#[path = "binary_coordination_tests.rs"]
mod coordination;
#[path = "binary_hook_tests.rs"]
mod hook;
/// A wrapped commit and a direct one agree in every shape of hook, whether or not an event has a dispatcher entry.
#[path = "binary_hook_shim_tests.rs"]
mod hook_shim;
/// The dependent-version policy on direct check and fix.
#[path = "binary_dependent_version_tests.rs"]
mod dependent_version;
#[path = "binary_landing_support.rs"]
mod landing_support;
#[path = "binary_landing_tests.rs"]
mod landing;
#[path = "binary_landing_crash_tests.rs"]
mod landing_crash;

/// A true merge in a checkout whose index holds another session's staged work: equal to native Git's merge on a
/// clean copy, nothing of the other work disturbed, refusals that change nothing, and recovery after a kill.
#[path = "binary_merge_isolation_tests.rs"]
mod merge_isolation;

/// Real commits through the transaction: landing, policies, replay and conflict.
#[path = "binary_commit_tests.rs"]
mod commit;

/// A commit killed at a phase marker whose transaction files are then torn, dropped or emptied: what recovery does.
#[path = "binary_recovery_tearing_tests.rs"]
mod recovery_tearing;

/// The Git version text a commit records in the user's cache directory, and when it trusts it.
#[path = "binary_version_cache_tests.rs"]
mod version_cache;

/// A process start that fails anywhere in a commit never strands the repository.
#[path = "binary_fork_failure_tests.rs"]
mod fork_failure;

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

/// The linked-worktree ignored-state copy: the rules file reaches new worktrees, and an interrupted copy recovers.
#[path = "binary_worktree_copy_tests.rs"]
mod worktree_copy;

/// The environment of the Git children the auto-push and the manual-push probe start.
#[path = "binary_push_environment_tests.rs"]
mod push_environment;

/// An explicit push that lost a race to another session: the merge is published through the executable's own push.
#[path = "binary_push_stacking_tests.rs"]
mod push_stacking;

/// The Git LFS upload before a push: every push the executable starts carries it, and a failure stops the push.
#[path = "binary_lfs_pre_push_tests.rs"]
mod lfs_pre_push;

/// The same upload with the real `git-lfs` against an LFS server written inline.
#[path = "binary_lfs_server_tests.rs"]
mod lfs_server;

/// Traditional hooks never run through the wrapper, and `git cli-git doctor` explains the Git LFS shims.
#[path = "binary_hooks_off_tests.rs"]
mod hooks_off;

/// The notices of a commit that waits: what it waits for, who holds it, the repeat, and the bound.
#[path = "binary_wait_notice_tests.rs"]
mod wait_notice;

/// The fault injection and the recovery invariants of the operating conditions in the README.
#[path = "binary_fault_support.rs"]
mod fault;

/// A full disk or a failed flush at each call of a commit: the next command recovers and nothing stays behind.
#[path = "binary_fault_enospc_tests.rs"]
mod fault_enospc;

/// The wrapper killed at each call of a commit that writes, renames, removes or syncs.
#[path = "binary_fault_kill_tests.rs"]
mod fault_kill;

/// Descriptor, process and file-size limits close to exhaustion, set on the child or injected at each call.
#[path = "binary_fault_limits_tests.rs"]
mod fault_limits;

/// A closed terminal: hangups, interrupts and quits to the wrapper and to its process group at each phase.
#[path = "binary_fault_signal_tests.rs"]
mod fault_signal;

/// Standard streams that are closed, unread or full.
#[path = "binary_fault_stream_tests.rs"]
mod fault_stream;

/// Remote trouble met by the push after a commit and by an explicit push.
#[path = "binary_fault_remote_tests.rs"]
mod fault_remote;

/// Slow flushes and filesystems that refuse links or renames.
#[path = "binary_fault_filesystem_tests.rs"]
mod fault_filesystem;
