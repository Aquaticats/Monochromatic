//! What:
//!  What a waiter can tell its user about the process that holds the lock it waits for: how long the lock has been
//!  held, what the holder runs, and which limits end the wait.
//! Why:
//!  A silent or anonymous wait looks like a hang. The age is the time since the lock directory was published, which
//!  is when the holder took the lock; the command comes from the kernel's record of the holder (Linux `/proc`; other
//!  systems name no command); the limits are the push deadline that stops the holder's push and the wait limit that
//!  stops this wait.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! `The holder has held the lock for ${age} seconds and runs \`${command}\`. ...`
//! ```

/// Time spans.
use std::time::Duration;
/// File locations.
use std::path::Path;

/// The longest command text shown, in characters.
const MAX_COMMAND_CHARACTERS: usize = 120;

/// What:
///  The holder's command line from the kernel's record, with the arguments separated by spaces and the text cut to a
///  readable length, or nothing when the record cannot be read.
/// Why:
///   The notice must stay one line; the `NUL` separators of the record become spaces.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// readFileSync(`/proc/${pid}/cmdline`, 'utf8').split('\0').join(' ').trim().slice(0, 120)
/// ```
#[cfg(target_os = "linux")]
fn holder_command(pid: i64) -> Option<String> {
    let bytes: Vec<u8> = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    let text: String = String::from_utf8_lossy(bytes.as_slice()).replace('\0', " ").replace(['\n', '\r'], " ");
    let shown: String = text.trim().chars().take(MAX_COMMAND_CHARACTERS).collect();
    return if shown.is_empty() { None } else { Some(shown) };
}

/// Only Linux records the command of another process in a file; elsewhere none is named.
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
fn holder_command(_pid: i64) -> Option<String> {
    return None;
}

/// What:
///  How long ago the lock directory was published, in whole seconds, or nothing when that cannot be read.
/// Why:
///   The directory is published once, by the holder, at the moment it takes the lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// Math.floor((Date.now() - statSync(lockDirectory).mtimeMs) / 1000)
/// ```
fn lock_age(lock_directory: &Path) -> Option<Duration> {
    return std::fs::metadata(lock_directory).ok()?.modified().ok()?.elapsed().ok();
}

/// What:
///  The sentences a waiting notice adds about its holder and the limits.
/// Why:
///   They say who holds the lock and for how long, and what ends the holder's push (`push_deadline`); the wait's own
///  limit is already in the notice, so a long wait is told from a stuck one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function holderFacts({ pid, lockDirectory, pushDeadline }): string;
/// ```
pub fn holder_facts(pid: i64, lock_directory: &Path, push_deadline: Duration) -> String {
    let mut facts: String = String::new();
    match (lock_age(lock_directory), holder_command(pid)) {
        (Some(age), Some(command)) => {
            facts.push_str(&format!(" The holder has held the lock for {} seconds and runs `{command}`.", age.as_secs()));
        }
        (Some(age), None) => {
            facts.push_str(&format!(" The holder has held the lock for {} seconds.", age.as_secs()));
        }
        (None, Some(command)) => facts.push_str(&format!(" The holder runs `{command}`.")),
        (None, None) => {}
    }
    facts.push_str(&format!(
        " Its push is stopped after {} seconds; a holder still holding the lock when this wait ends is reported and left in place.",
        push_deadline.as_secs()
    ));
    return facts;
}

/// The facts are controlled in the test module.
#[cfg(test)]
#[path = "lock_holder_facts_tests.rs"]
mod tests;
