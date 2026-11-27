//! What:
//!  Test-only fixtures shared by many unit tests,
//!  never part of the release executable.
//! Why:
//!  Shared fixtures need one obvious home.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // testing/index.ts: one `export * from './file'` per module below.
//! ```

/// A loopback HTTP server that serves `git http-backend`,
/// for the controls of a push over HTTP.
#[cfg(test)]
#[cfg(unix)]
pub(crate) mod smart_http_support;

/// Whether the run is moving,
/// and the clock that ends a run which has stopped,
/// for the watchdog the first fixture arms;
/// never part of the release executable.
#[cfg(test)]
pub(crate) mod run_progress;

/// Disposable fixtures shared by unit tests;
///  never part of the release executable.
#[cfg(test)]
pub(crate) mod test_support;

/// The one place a child of this test binary is started,
/// waited for with a deadline and checked for its one passing
/// test;
///  never part of the release executable,
///  and included by path in the binary controls' crate and the candidate consumer's crate,
/// which cannot reach this one.
#[cfg(test)]
pub(crate) mod child_test_process;

/// The owner a helper process is started with:
/// a process group of its own that dies with the test thread and with this process;
///  never part of the release executable,
///  and included by path in the binary controls' crate and the candidate consumer's crate,
/// which cannot reach this one.
#[cfg(test)]
pub(crate) mod test_process_owner;

/// The one shape for a control that runs in a process of its own,
/// with a verdict file and every wait bounded;
///  never part of the release executable.
#[cfg(test)]
pub(crate) mod helper_process_test_support;

/// The user namespace a control needs when the files it judges must be root's,
/// and the inner half that runs the same test there;
/// never part of the release executable,
/// and included by path in the binary controls' own crate,
/// which cannot reach this one.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod root_namespace_test_support;

/// Whether the fixture root takes a name that is not UTF-8,
/// and the tmpfs a control mounts at it inside a namespace of its own when it does not;
/// never part of the release executable,
/// and included by path in the binary controls' own crate,
/// which cannot reach this one.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod fixture_root_taking_any_name;

/// The unit tests' own wrapper for the fixture root that takes any name;
/// never part of the release executable,
/// and never included by the binary controls' crate,
/// which has a fixture root of its own.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod fixture_root_taking_any_name_of_unit_tests;

/// A mount table whose text a control writes,
///  for the rules that count mounted datasets;
///  never part of the release executable.
#[cfg(test)]
pub(crate) mod mounted_dataset_table;

/// Which folder a runtime directory stands in,
/// and the runtime directory built to a length inside it;
///  never part of the release executable.
#[cfg(test)]
pub(crate) mod runtime_directory_folders;

/// The size the platform lets one command-line argument or one environment string reach,
/// for controls that prove a value cannot travel that way;
/// never part of the release executable.
#[cfg(test)]
pub(crate) mod argument_limit;

/// The owner of a process that really started and was then killed,
/// a fixture of the controls beside `test_support.rs`.
#[cfg(test)]
pub(crate) mod test_support_dead_copy_owner;

/// The configuration and attribute locations a content decision reads in a fixture;
/// never part of the release executable.
#[cfg(all(test, target_os = "linux"))]
pub(crate) mod fixture_content_decision;

/// The variables that isolate a fixture's Git children from the person's configuration,
/// and the cache place a fixture's work records its facts in;
/// never part of the release executable.
#[cfg(test)]
pub(crate) mod fixture_environment_naming_a_cache_place;

/// A Git stand-in that logs each start and runs the real Git;
///  never part of the release executable.
#[cfg(test)]
#[cfg(unix)]
pub(crate) mod recording_git;

/// A test body run in a child process under a chosen umask;
/// never part of the release executable.
#[cfg(test)]
#[cfg(target_os = "linux")]
pub(crate) mod umask_child_test_support;

/// A test body run in a child process that is refused every thread it starts;
/// never part of the release executable.
#[cfg(test)]
#[cfg(target_os = "linux")]
pub(crate) mod no_threads_child_test_support;

/// A test body run in a child process that may write no file past a size limit;
/// never part of the release executable.
#[cfg(test)]
#[cfg(unix)]
pub(crate) mod file_size_limit_child_test_support;

/// A test body run in a child process of its own that lowers its own descriptor limit,
/// and the only process a test may lower that limit in;
/// never part of the release executable.
#[cfg(test)]
#[cfg(target_os = "linux")]
pub(crate) mod descriptor_limit_child_test_support;

/// A test body run in a child process of its own,
/// for a control whose state belongs to the whole process,
/// and the only process a test may set that state in;
/// never part of the release executable.
#[cfg(test)]
pub(crate) mod process_wide_state_child_test_support;

/// The action in place for a signal,
/// which belongs to the whole process and which a control reads before and after the body it judges;
/// never part of the release executable.
#[cfg(test)]
#[cfg(unix)]
pub(crate) mod signal_disposition_of_the_process;

/// A point where threads must all arrive,
/// for controls that prove work overlapped.
#[cfg(test)]
pub(crate) mod parallel_test_support;

/// A counting task beside a wait,
/// for controls that prove the wait held no thread;
/// never part of the release executable.
#[cfg(test)]
pub(crate) mod ticker_beside_a_wait;

/// The real `git-lfs` a control proves the upload against,
/// and what a control does in a run that has none;
/// never part of the release executable.
#[cfg(test)]
pub(crate) mod real_git_lfs_the_run_requires;

/// `strace`,
/// which the controls that trace a wrapped command run it under,
/// and the rule a control follows when this run has none;
/// never part of the release executable,
/// and included by path in the binary controls' own crate,
/// which cannot reach this one.
#[cfg(test)]
pub(crate) mod strace_the_controls_run_under;

/// The controls of the rule above;
/// never part of the release executable.
#[cfg(test)]
#[path = "strace_the_controls_run_under_tests.rs"]
pub(crate) mod strace_the_controls_run_under_tests;

/// The controls of the decision above;
/// never part of the release executable.
#[cfg(test)]
#[path = "real_git_lfs_the_run_requires_tests.rs"]
pub(crate) mod real_git_lfs_the_run_requires_tests;

/// Ending a child together with its whole process group,
/// for controls that must leave no process behind;
/// never part of the
///  release executable.
#[cfg(test)]
#[cfg(unix)]
pub(crate) mod child_ended_with_its_whole_group;

/// The control that a build made without the `test-only-switches` feature reads none of the crate's test-only
/// variables;
/// never part of the release executable.
#[cfg(all(test, target_os = "linux"))]
#[path = "no_switch_is_read_without_its_feature_tests.rs"]
pub(crate) mod no_switch_is_read_without_its_feature_tests;

/// One binding record whose slot,
/// registration and pointer are a fixture's own paths,
/// and the publication that puts it under a fixture's Git common directory;
/// never part of the release executable.
#[cfg(test)]
pub(crate) mod fixture_zfs_binding_record;

/// One tmpfs mounted at a fixture slot,
/// and a standing-mount reader whose table names it as a clone's own mount;
/// never part of the release executable.
#[cfg(test)]
#[cfg(target_os = "linux")]
pub(crate) mod standing_slot_mounts;

/// The destruction plan's writer,
/// named once here for the plan helper both test targets include by `#[path]`:
/// that helper reaches these names through `super::`,
/// and the `native_binary` crate declares the same module over its own paths to them;
/// never part of the release executable,
/// which holds no test-only module to mark this one against.
pub(crate) mod the_destruction_plans_writer {
    pub(crate) use crate::worktree_clone_mount_protocol::dataset_guid::DatasetGuid;
    pub(crate) use crate::worktree_clone_mount_protocol::request_nonce::RequestNonce;
    pub(crate) use crate::worktree_clone_removal_proof_and_plan::plan_publication::PlanFolder;
    pub(crate) use crate::worktree_clone_removal_proof_and_plan::plan_record::{
        DestructionPlan,
        PlanAttempt,
        PlanFacts,
        PlanSnapshot,
    };
    pub(crate) use crate::worktree_clone_removal_proof_and_plan::plan_words::{
        HeadWord,
        SnapshotOutcome,
    };
    pub(crate) use crate::worktree_clone_removal_proof_and_plan::started_moment::now_with_offset;
}

/// The destruction plan a control plants,
/// published by the removal's own writer;
/// never part of the release executable,
/// and included by path in the binary controls' crate,
/// which cannot reach this one.
#[cfg(test)]
pub(crate) mod the_destruction_plan_a_control_plants;

/// One line of a planted plan's own snapshot,
/// for the controls that plant a history of more than a first write;
/// never part of the release executable,
/// and kept out of the binary controls' crate,
/// which plants a removal's first write and no history.
#[cfg(test)]
pub(crate) mod a_planted_plans_snapshot_line;

/// The controls of the plan helper above;
/// never part of the release executable,
/// and kept out of the binary controls' crate,
/// which plants plans through the same helper.
#[cfg(test)]
#[path = "the_destruction_plan_a_control_plants_tests.rs"]
pub(crate) mod the_destruction_plan_a_control_plants_tests;
