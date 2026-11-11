//! What:
//!  Controls that read the crate's own source text.
//! Why:
//!  They guard the layout of every other folder.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // source_guard/index.ts: one `export * from './file'` per module below.
//! ```

/// The ban on Git starting the wrapper for a hook:
/// no entry writer,
/// no start-name dispatch.
#[cfg(test)]
#[path = "hook_self_invocation_tests.rs"]
mod hook_self_invocation_tests;

/// The lock inventory of the service and its support modules:
/// a lock stands only on the inventory,
/// with its reason beside it.
#[cfg(test)]
#[path = "service_lock_inventory_tests.rs"]
mod service_lock_inventory_tests;

/// The crate declares only the executable the release archive holds;
/// a second `[[bin]]` breaks cargo-binstall installs.
#[cfg(test)]
#[path = "release_bins_tests.rs"]
mod release_bins_tests;

/// The mechanical ban:
/// no release code starts the wrapper,
/// passes it management words or writes shell text.
#[cfg(test)]
#[path = "no_self_invocation_tests.rs"]
mod no_self_invocation_tests;

/// The owner's move to tokio needs its workers to await:
/// no thread,
/// no sleeping clock,
/// no unretried child start
/// and no blocking work inside a task.
#[cfg(test)]
#[path = "blocking_code_off_the_runtime_tests.rs"]
mod blocking_code_off_the_runtime_tests;

/// The structural check that the review's network client is reachable only from the service run;
/// never in the release build.
#[cfg(test)]
mod standards_review_boundary_tests;

/// The shared base of the checks that read the crate's own source text:
/// a strict walk and the module tree,
/// so a check
/// is right for nested module directories;
/// never in the release build.
#[cfg(test)]
pub(crate) mod source_scan_tests;

/// The owner's rule that nothing of the wrapper starts at system startup or at login:
/// no install line,
/// no enabling call,
/// only listed members.
#[cfg(test)]
#[path = "no_start_at_login_tests.rs"]
pub(crate) mod no_start_at_login_tests;
