//! Language module sessions against the scripted server:
//!  one child process per test,
//!  because
//! Helix roots servers at the process working directory.

/// What:
///  `#[path = "..."]` names the file a module is read from.
///  A test crate's root file looks
///       for its modules beside itself,
///  so the modules kept under `tests/language/` are named
///       explicitly.
/// Why:
///   One test crate links the application library once and lets every test share the
///       support code without unused-code warnings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// import * as support from './language/support';
/// ```
#[path = "language/support.rs"]
mod support;

/// Start states,
///  the readiness gate,
///  and the start deadline.
#[path = "language/start.rs"]
mod start;

/// External-reload synchronization for every synchronization kind and column unit.
#[path = "language/sync.rs"]
mod sync;

/// The five feature paths and the request states.
#[path = "language/requests.rs"]
mod requests;

/// Hint answers for lines the reader no longer shows.
#[path = "language/windows.rs"]
mod windows;

/// Hint and pull-diagnostics requests asked again after a timeout.
#[path = "language/again.rs"]
mod again;

/// Replies to server requests;
///  refusal of server-initiated edits.
#[path = "language/policy.rs"]
mod policy;

/// Crash,
///  restart,
///  file switches,
///  launch refusal,
///  and shutdown.
#[path = "language/lifecycle.rs"]
mod lifecycle;

/// What a clean open,
///  request,
///  close,
///  and shutdown leave in the log and the process table.
#[path = "language/quiet.rs"]
mod quiet;

/// Working directory and enclosing-tree root handling.
#[path = "language/roots.rs"]
mod roots;

/// File changes made outside the IDE,
///  forwarded to servers that registered file watchers.
#[path = "language/watched.rs"]
mod watched;

/// The entry that says why hover,
///  hints,
///  or definitions are missing.
#[path = "language/notice.rs"]
mod notice;
