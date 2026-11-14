//! What:
//!  The directory a Git command runs in,
//!  which is where a rule reads a relative path of that command from
//!  and where a measurement made on the command's behalf must start.
//! Why:
//!  `-C` moves Git before it runs the subcommand,
//!  and Git moves again before it starts the shell of a `!` alias
//!  (`git.c:388` to `git.c:407`),
//!  so the directory a rule must use is not always the one the wrapper stands in.
//!  One answer serves every rule that reads such a path or starts such a measurement,
//!  so no rule can measure from a directory Git never goes to.
//!  A directory that cannot be named is an error,
//!  never the caller's own directory by default:
//!  a rule that guessed there would judge a command that never runs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // commandRunDirectory(globalPrefix, shellDirectory)
//! //   === shellDirectory ?? chainDirectory(process.cwd(), globalPrefix)
//! ```

use crate::repository_link_paths::chain_directory;
/// `OsString` is owned operating-system text of raw bytes (sibling `String` must be valid UTF-8).
use std::ffi::OsString;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths of raw bytes.
use std::path::{Path, PathBuf};

/// What:
///  The directory Git runs a command in.
///  `&[OsString]` borrows the global options before the subcommand,
///  and `Option<&Path>` the directory Git runs the shell of a `!` alias in,
///  when the command is the one that shell runs.
/// Why:
///  A built-in or an ordinary alias runs where the caller's `-C` chain ends,
///  which is this process's own directory moved by every `-C` in order,
///  and a relative value read from the directory the move before it left.
///  The command inside a `!` alias's shell runs where Git put that shell,
///  which the guard already asked Git for:
///  a relative path or a clean's prefix is read from there,
///  and reused here rather than asked twice,
///  so the two answers cannot drift apart.
///  A directory this cannot name stops the rule that asked:
///  a path read from the wrong directory names another file,
///  and a measurement started there covers a subtree the command does not touch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function commandRunDirectory(globalPrefix: string[], shellDirectory?: string): string; // throws
/// ```
pub fn command_run_directory(global_prefix: &[OsString], shell_directory: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(found) = shell_directory {
        return Ok(found.to_path_buf());
    }
    // The caller's own directory, which no option changes: Git moves it only for `-C`.
    let Ok(cwd) = std::env::current_dir() else {
        return Err(String::from(
            "monochromatic-git-policy-cli could not name the directory this command runs in, so the command was \
             not judged.",
        ));
    };
    return Ok(chain_directory(cwd.as_path(), global_prefix));
}

/// What:
///  The path from the worktree root to the directory a command runs in,
///  with a trailing slash,
///  which is the prefix Git writes before every name it prints there.
///  `&Path` borrows the worktree root and that directory,
///  and `&[u8]` the prefix the location query answered.
/// Why:
///  Git prints each name relative to the directory the command runs in,
///  and a command a `!` alias runs starts at the top of the working tree,
///  so the prefix of such a reading is the one that goes before those names,
///  which is not the caller's prefix when the caller stood below the top;
///  a reading rooted with the caller's prefix names a file in another directory,
///  and a guard that then finds the entry in the way there refuses a command
///  that writes nothing of the sort.
///  A reading whose directory is not below the worktree is one no name can be
///  joined to,
///  so the prefix the invocation itself has is kept,
///  which is what every reading in the caller's own directory carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function runDirectoryPrefix(root: string, runIn: string, locationPrefix: Buffer): Buffer;
/// ```
pub fn run_directory_prefix(root: &Path, run_in: &Path, location_prefix: &[u8]) -> Vec<u8> {
    let Ok(relative) = run_in.strip_prefix(root) else {
        return location_prefix.to_vec();
    };
    let mut prefix: Vec<u8> = Vec::<u8>::new();
    for component in relative.components() {
        prefix.extend_from_slice(component.as_os_str().as_encoded_bytes());
        prefix.push(b'/');
    }
    return prefix;
}

/// A command's directory is one answer,
/// and its spelling of the `-C` chain is read by one other file.
#[cfg(test)]
#[path = "command_run_directory_tests.rs"]
mod tests;
