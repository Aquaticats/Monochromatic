//! What:
//!  The two sentences a commit reads when the commit service cannot answer this command at all,
//! because the address it would listen at cannot exist or because the manager made it listen somewhere
//!  else.
//! Why:
//!  Both are known before anything is waited for,
//! and both name a setting of the person's own (`XDG_RUNTIME_DIR`) and every way to change it,
//! which is what the message rules ask for and what the report of 2026-11-11 was missing when a commit
//!  waited two minutes and said only that the service did not start.
//! They live beside the other texts rather than inside it,
//! because the file that renders every failure is at its line bound.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! socketPathTooLong({ path, length, longest, longestRuntime }) // the sentence a person reads
//! ```

use crate::service_messages::NOTHING_CHANGED;

/// What:
///  The sentence for an address longer than one socket address can hold.
/// Why:
///  The runtime directory is the part of it a person chooses,
/// so the message names the room it may take and the place it is set.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// socketPathTooLong(tooLong) // 'The commit service would listen at /…, which takes 113 bytes …'
/// ```
pub fn socket_path_too_long(path: &str, length: usize, longest: usize, longest_runtime: usize) -> String {
    return format!(
        "The commit service would listen at {path}, which takes {length} bytes, and this system's addresses of that kind \
         hold at most {longest} bytes, so this commit was not made. That address is built from this command's runtime \
         directory, which comes from XDG_RUNTIME_DIR: set XDG_RUNTIME_DIR to a directory of at most {longest_runtime} \
         bytes and run the commit again. {NOTHING_CHANGED}"
    );
}

/// What:
///  The sentence for a service the manager made listen where this command does not wait.
/// Why:
///  Both addresses and both runtime directories are what tells a person which of the two is the one their session
///  uses,
/// and the setting that decides it is named with the value that works.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// serviceElsewhere(elsewhere) // 'The commit service was started, and it listens at /run/user/1000/…'
/// ```
pub fn service_elsewhere(made: &str, looked_for: &str, manager_runtime: &str, client_runtime: &str) -> String {
    return format!(
        "The commit service was started, and it listens at {made}, while this command waits at {looked_for}, so this \
         commit was not made. Your account's service manager runs with the runtime directory {manager_runtime}, and this \
         command was given {client_runtime}, which is XDG_RUNTIME_DIR: a command reaches the service only when its \
         runtime directory is the manager's. Run the commit from your login session, where XDG_RUNTIME_DIR is \
         {manager_runtime}, or set XDG_RUNTIME_DIR to {manager_runtime} for it. {NOTHING_CHANGED}"
    );
}

/// Text controls stay out of the release executable.
#[cfg(test)]
#[path = "service_messages_of_a_start_that_cannot_come_tests.rs"]
mod tests;
