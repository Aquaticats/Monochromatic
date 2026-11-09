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

/// A point where threads must all arrive,
/// for controls that prove work overlapped.
#[cfg(test)]
pub(crate) mod parallel_test_support;

/// A counting task beside a wait,
/// for controls that prove the wait held no thread;
/// never part of the release executable.
#[cfg(test)]
pub(crate) mod ticker_beside_a_wait;

/// Ending a child together with its whole process group,
/// for controls that must leave no process behind;
/// never part of the
///  release executable.
#[cfg(test)]
#[cfg(unix)]
pub(crate) mod child_ended_with_its_whole_group;
