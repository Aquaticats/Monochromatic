//! What:
//!  The arguments of the clean the Cargo repair runs,
//! and the command a person can run instead when one fails.
//! Why:
//!  One list builds both clean faces,
//! so the members and the switches each clean names can never drift apart,
//! and the line a failed repair prints is the same command the filling of a build directory
//! reports:
//! a printed line must work as pasted.
//! `--locked` differs between the two faces and says in its own doc why.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! function cleanArguments(members: string[]): string[];
//! function lockedCleanArguments(members: string[]): string[];
//! function manualCommand(root: string, members: string[]): string;
//! ```

/// Import the shell words a printed command is made of.
use crate::shell_quote::shell_quote;
/// Import the owned,
/// platform-native string a program argument is.
use std::ffi::OsString;
/// Import the borrowed filesystem path a person is told to change into.
use std::path::Path;
/// The program the person's environment resolves.
pub(super) const PROGRAM: &str = "cargo";

/// What:
///  The arguments of the clean that removes exactly the members' own artifacts,
/// for the add's own clean.
/// Why:
///   `--offline` keeps the clean from reaching the network;
/// one `--package` per member keeps registry and git dependencies.
///  `--locked` is not used here:
/// a committed `Cargo.lock` that no longer matches its manifest (several do here) would make the clean
///  fail,
/// and the person's own clean would not;
/// the file is put back as checked out instead (`restore_lock`),
/// which `SPEC.md` records as the reason this stands where the worktree was just made and nobody
/// has it yet.
/// The filling of a build directory after the add has returned passes `--locked` instead
/// (`package_repair_prewarm_job.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function cleanArguments(members: string[]): string[];
/// ```
pub fn clean_arguments(members: &[String]) -> Vec<OsString> {
    return clean_arguments_of(members, false);
}

/// What:
///  The arguments of the clean the commit service runs on a copy of a worktree's build state:
/// the same clean,
/// with `--locked`.
/// Why:
///   That clean runs after the add returned,
///  when the person may be working in the worktree,
/// and `--locked` keeps Cargo from rewriting the worktree's tracked `Cargo.lock` at all,
/// so the person's own file is never in the way of the clean.
/// A lock that no longer matches its manifest makes such a clean refuse,
/// and its caller asks Cargo to resolve the workspace first (`package_repair_prewarm_job.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lockedCleanArguments(members: string[]): string[];
/// ```
pub fn locked_clean_arguments(members: &[String]) -> Vec<OsString> {
    return clean_arguments_of(members, true);
}

/// What:
///  The clean's arguments,
/// with `--locked` when the caller's lock must not be rewritten.
/// Why:
///   One list builds both faces,
/// so the members and the switches each clean names can never drift apart.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cleanArgumentsOf = (members, locked) => ['clean', '--offline', ...(locked ? ['--locked'] : []), ...members.flatMap((m) => ['--package', m])];
/// ```
fn clean_arguments_of(
    members: &[String],
    locked: bool,
) -> Vec<OsString> {
    let mut arguments: Vec<OsString> = ["clean", "--offline"].iter().map(OsString::from).collect();
    if locked {
        arguments.push(OsString::from("--locked"));
    }
    for member in members {
        arguments.push(OsString::from("--package"));
        arguments.push(OsString::from(member));
    }
    return arguments;
}

/// What:
///  The command a person runs in a workspace to do what the repair could not.
/// Why:
///   A printed line must work as pasted,
/// so the directory change is part of it,
/// and the filling of a build directory reports the same command the add's repair does
/// (`package_repair_prewarm_job.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function manualCommand(root: string, members: string[]): string;
/// ```
pub fn manual_command(
    root: &Path,
    members: &[String],
) -> String {
    let mut words: Vec<String> = Vec::new();
    for word in clean_arguments(members) {
        words.push(shell_quote(word.as_encoded_bytes()));
    }
    return format!(
        "cd {} && {PROGRAM} {}",
        shell_quote(root.as_os_str().as_encoded_bytes()),
        words.join(" ")
    );
}
