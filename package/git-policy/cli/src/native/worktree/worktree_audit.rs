//! What:
//!  The facts one audit line carries about a `git worktree remove`:
//!  the directory it names,
//!  whether the
//!       person forced it,
//!  the `HEAD` the removed worktree had,
//!  and how many tracked paths differed from it.
//! Why:
//!  The owner chose "Log only" for this command,
//!  with the note that agents are just as likely to run
//!       `rm -rf`,
//!  and asked for one line naming the worktree,
//!  whether `--force` was given,
//!  and what the
//!       removal took,
//!  without making the removal slow.
//!  So the line records what a removal can read cheaply:
//!  the target's
//!       `HEAD` from two file reads,
//!  and the number of tracked paths that differ,
//!  from one `git status` that reads the
//!       index and never walks the untracked or ignored files
//!       (measured:
//! 20 ms on this repository's own worktree).
//!  What it does not
//!       count,
//! it says it does not count,
//!  so a reader is never told the worktree was clean.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // measureRemoval({ context, target, force }) => [{ name: 'force', value: true }, ...]
//! ```

use crate::discard_audit_facts::AuditFact;
use crate::git_metadata::{MetadataOutput, run_metadata_git};
use crate::transaction_git::GitContext;
/// `OsString`/`OsStr` are owned/borrowed operating-system text of raw bytes,
///  and `Path`/`PathBuf` owned/borrowed paths.
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// The sentence the record carries about what it counted.
const COUNTED_TRACKED_ONLY: &str = "tracked changes only";

/// What:
///  The `HEAD` of a worktree,
/// read from the Git directory the worktree's `.git` names.
/// Why:
///   Two file reads cost nothing beside a removal,
///  and they name the branch or commit the worktree was on,
///  which is
///       what brings the work back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function worktreeHead(target: string): string | undefined;
/// ```
pub fn worktree_head(target: &Path) -> Option<String> {
    // A file holding `gitdir: <the worktree's own Git directory>`; a directory for a main worktree, which a removal
    //  never names.
    let pointer: String = std::fs::read_to_string(target.join(".git")).ok()?;
    let git_dir: &str = pointer.trim().strip_prefix("gitdir:")?.trim();
    let head: String = std::fs::read_to_string(Path::new(git_dir).join("HEAD")).ok()?;
    let trimmed: &str = head.trim();
    let named: &str = trimmed.strip_prefix("ref: ").unwrap_or(trimmed);
    if named.is_empty() {
        return None;
    }
    return Some(named.to_string());
}

/// What:
///  How many tracked paths differ from `HEAD` in the worktree at `target`,
/// or nothing when Git
///  cannot answer.
/// Why:
///   This is the one fact worth a Git start,
///  and it is the cheapest one:
///  `--untracked-files=no`
///       reads the index and the tracked files,
///  and no reading of ignored files is added because walking a large
///       ignored tree is what would make a removal slow.
///  The query names the target with its own `-C`,
///  so the invocation's
///       global options cannot point it at another repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await runGit(['-C', target, '--no-optional-locks', 'status', '--porcelain=v1', '--untracked-files=no']);
/// ```
pub fn changed_paths(context: &GitContext, target: &Path) -> Option<usize> {
    let arguments: Vec<OsString> = vec![
        OsString::from("-C"),
        target.as_os_str().to_os_string(),
        OsString::from("--no-optional-locks"),
        OsString::from("status"),
        OsString::from("--porcelain=v1"),
        OsString::from("--untracked-files=no"),
    ];
    let output: MetadataOutput =
        run_metadata_git(context.real_git.as_path(), arguments.as_slice(), context.overlay.as_slice()).ok()?;
    if !output.success {
        return None;
    }
    return Some(output.stdout.split(is_newline).filter(is_entry).count());
}

/// What:
///  Whether a byte of the status answer ends a line.
/// Why:
///   The count is one per entry,
///  and a named function keeps that pass free of closures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (byte: number) => byte === 0x0a
/// ```
fn is_newline(byte: &u8) -> bool {
    return *byte == b'\n';
}

/// What:
///  Whether one piece of the answer is an entry rather than the text after the last newline.
/// Why:
///   A trailing newline leaves one empty piece,
///  which is not an entry,
///  and `filter` hands the predicate a
///       reference to the piece.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (line: Buffer) => line.length > 0
/// ```
fn is_entry(line: &&[u8]) -> bool {
    return !line.is_empty();
}

/// What:
///  The facts one record of a removed worktree carries,
/// best effort:
///  a fact Git could not answer is left out
///  rather than reported as an answer.
///  `Path` is the resolved target and `&OsStr` its spelling as the person typed it.
/// Why:
///   A record that cannot be completed must not stop the removal the person asked for,
///  and a fact that is absent is
///       honest where a guessed one is not.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function removalFacts({ context, target, typed, force }): AuditFact[];
/// ```
pub fn removal_facts(context: &GitContext, target: &Path, typed: &OsStr, force: bool) -> Vec<AuditFact> {
    let mut facts: Vec<AuditFact> = vec![
        AuditFact::Text("worktree", typed.to_string_lossy().into_owned()),
        AuditFact::Flag("force", force),
    ];
    if let Some(head) = worktree_head(target) {
        facts.push(AuditFact::Text("head", head));
    }
    if let Some(changed) = changed_paths(context, target) {
        facts.push(AuditFact::Text("changed", changed.to_string()));
    }
    facts.push(AuditFact::Text("counted", COUNTED_TRACKED_ONLY.to_string()));
    return facts;
}

/// What:
///  The path a `git worktree remove` names,
///  joined to the directory the command runs in.
/// Why:
///   `worktree remove` accepts a path relative to the current directory,
///  and every read of it has to happen
///       before Git removes it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function targetPath(typed: string, directory: string): string;
/// ```
pub fn target_path(typed: &OsStr, directory: &Path) -> PathBuf {
    return directory.join(typed);
}

/// The removal facts and the real-Git controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_audit_tests.rs"]
mod tests;
