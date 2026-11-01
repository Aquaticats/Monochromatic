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

/// Global options that take their value as the next argument.
const VALUE_OPTIONS: [&str; 5] = ["-c", "-C", "--git-dir", "--work-tree", "--namespace"];

/// What:
///  The program and subcommand of a command line, with every other argument left out.
/// Why:
///   Arguments can hold secrets (`-c http.extraHeader=...`), and the notice repeats at an interval into logs, so only
///  the program's name and the subcommand are shown. Global options are skipped to find the subcommand.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${basename(argv[0])} ${argv.slice(1).find((word, i) => !word.startsWith('-') && !valueOptions.includes(argv[i]))}`
/// ```
pub fn program_and_subcommand(words: &[&str]) -> Option<String> {
    let program: &str = words.first()?.rsplit('/').next()?;
    let mut rest = words.iter().skip(1);
    while let Some(word) = rest.next() {
        if VALUE_OPTIONS.contains(word) {
            rest.next();
            continue;
        }
        if !word.starts_with('-') {
            return Some(format!("{program} {word}"));
        }
    }
    return Some(program.to_owned());
}

/// What:
///  The holder's program and subcommand from the kernel's record, and the repository it runs in.
/// Why:
///   The repository is the holder's working directory, which `/proc` records without any argument.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${programAndSubcommand(cmdline)} in ${readlink(`/proc/${pid}/cwd`)}`
/// ```
#[cfg(target_os = "linux")]
fn holder_command(pid: i64) -> Option<String> {
    let bytes: Vec<u8> = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    let text: String = String::from_utf8_lossy(bytes.as_slice()).into_owned();
    let words: Vec<&str> = text.split('\0').filter(|word| {
        return !word.is_empty();
    }).collect();
    let shown: String = program_and_subcommand(words.as_slice())?;
    return Some(match std::fs::read_link(format!("/proc/{pid}/cwd")) {
        Ok(directory) => format!("{shown} in {}", directory.display()),
        Err(_) => shown,
    });
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
