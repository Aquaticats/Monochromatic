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
