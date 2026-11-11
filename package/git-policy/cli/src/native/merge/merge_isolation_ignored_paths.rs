//! What:
//!  The paths the merge writes that an ignored untracked file of the person's stands at.
//! Why:
//!  `git status` never lists a file an ignore rule covers,
//! so such a path never reached the capture the person's work is read from,
//!  and the merge wrote the branch's bytes over the only copy of their file
//! (the owner's rule "never discard user data";
//!  `SPEC.md`,
//! "A merge whose paths hold the person's own work carries it forward",
//! which names untracked files the
//!  merge adds without excluding the ignored ones).
//!  The answer comes from `git ls-files --others --ignored --exclude-standard`,
//! which lists the untracked files an
//!  ignore rule covers and leaves out every path the index tracks.
//! `git status --ignored` cannot answer it:
//! it groups a directory
//!  the rules ignore whole under the directory's own name,
//! which says nothing about the files the merge writes below it
//!  (measured against Git 2.56.0).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const ignored = ignoredUntrackedAmong(context, root, touched); // only the paths the merge writes survive
//! ```

use crate::auto_push_dirty_paths::diff_paths;
/// The entry of the process runtime a synchronous caller takes.
use crate::lazy_runtime::run_on_runtime;
/// One path spelled for a Git argument.
use crate::merge_isolation_carry_reads::path_argument;
/// Real Git,
/// and the bytes of one of its starts waited for.
use crate::transaction_git::{GitContext, GitOutput, GitRequest};
use crate::transaction_git_awaiting::run_git_awaiting;
/// `OsString` spells a path Git reads as bytes that need not be text.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// The most path text one `git ls-files` call carries.
///  Measured on this host:
/// `getconf ARG_MAX` answers 2097152 bytes,
/// and one path is at most `PATH_MAX`'s 4096,
/// so this bound
///  is a sixty-fourth of the total and leaves the rest for the environment and the other arguments,
/// while a merge
///  of any size still makes calls whose argument list no operating system refuses.
const PATH_TEXT_PER_CALL: usize = 32 * 1024;

/// What:
///  The ignored untracked files of the person's that stand at the paths `written`.
/// Why:
///   These are the paths whose only copy the merge would write over,
/// and the list is read from the top of the worktree,
/// so
///  every name in it is the name the merge's own path list uses,
/// whatever directory the person ran the command in.
///  A merge that writes no path asks Git nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ignored = await ignoredUntrackedAmongAwaiting(context, root, written);
/// ```
pub(crate) fn ignored_untracked_among(context: &GitContext, root: &Path, written: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, String> {
    return match run_on_runtime(ignored_untracked_among_awaiting(context, root, written)) {
        Ok(found) => found,
        Err(error) => Err(format!("the read of the ignored untracked files could not run: {error}")),
    };
}

/// What:
///  The same list,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   A synchronous start registers the step's pipes by entering the runtime,
/// which is refused from inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function ignoredUntrackedAmongAwaiting(context, root, written): Promise<Uint8Array[]>;
/// ```
pub(crate) async fn ignored_untracked_among_awaiting(context: &GitContext, root: &Path, written: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, String> {
    if written.is_empty() {
        return Ok(Vec::new());
    }
    let mut found: Vec<Vec<u8>> = Vec::new();
    for (start, end) in batches(written) {
        for path in listed_awaiting(context, root, &written[start..end]).await? {
            // A batch names the paths the merge writes, so a name outside the merge's own list is no path of it: only the
            //  names that list holds are work this command carries.
            if written.contains(&path) && !found.contains(&path) {
                found.push(path);
            }
        }
    }
    return Ok(found);
}

/// What:
///  The words for a Git that could not be started at all.
/// Why:
///   A start that never happened is not Git's answer,
///  so it is named as the start's own failure,
/// as the carry names it.
fn git_start_failure(error: std::io::Error) -> String {
    return format!("git could not start: {error}");
}

/// What:
///  The ignored untracked files under the pathspecs of one batch.
/// Why:
///  One read serves every path of the batch,
/// and the batch bounds the argument list a merge of any size builds.
///  The names are read from the top of the worktree,
/// where the invocation's global options do not reach,
/// as the carry
///  reads its own paths,
/// and the pathspecs are the wrapper's own
/// (`GitRequest::own_pathspecs`),
/// so no pathspec mode
///  of the person's changes which paths this read answers for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const listed = await git({ cwd: root, withoutPrefix: true, ownPathspecs: true }, ['ls-files', '--others', '--ignored', '--exclude-standard', '-z', '--full-name', '--', ...batch]);
/// ```
async fn listed_awaiting(context: &GitContext, root: &Path, batch: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, String> {
    let mut words: Vec<OsString> = vec![
        OsString::from("ls-files"),
        OsString::from("--others"),
        OsString::from("--ignored"),
        OsString::from("--exclude-standard"),
        OsString::from("-z"),
        OsString::from("--full-name"),
        OsString::from("--"),
    ];
    for path in batch {
        words.push(path_argument(path.as_slice()));
    }
    let mut request: GitRequest = GitRequest::new::<&str>(root, &[]).own_pathspecs();
    request.without_prefix = true;
    request.arguments = words;
    let started: GitOutput = run_git_awaiting(context, &request).await.map_err(git_start_failure)?;
    if !started.succeeded() {
        return Err(format!("git ls-files failed: {}", started.error_text()));
    }
    return Ok(diff_paths(started.stdout.as_slice()));
}

/// What:
///  The batches of paths one `git ls-files` call carries,
/// as the half-open ranges of the list they name.
/// Why:
///   A merge can write more paths than one argument list holds,
/// and a failure to start Git over them would turn the
///  person's own file back into something the merge overwrites;
/// the ranges are ordered,
/// cover the list once,
/// and
///  keep every call's path text under `PATH_TEXT_PER_CALL`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const batches = (paths) => [[0, 40], [40, 120]]; // byte-bounded, each holding at least one path
/// ```
fn batches(paths: &[Vec<u8>]) -> Vec<(usize, usize)> {
    let mut spans: Vec<(usize, usize)> = Vec::new();
    let mut start: usize = 0;
    let mut bytes: usize = 0;
    for (index, path) in paths.iter().enumerate() {
        if index > start && bytes + path.len() > PATH_TEXT_PER_CALL {
            spans.push((start, index));
            start = index;
            bytes = 0;
        }
        bytes += path.len();
    }
    spans.push((start, paths.len()));
    return spans;
}

/// The batches of one read,
/// and the paths they cover.
#[cfg(test)]
#[path = "merge_isolation_ignored_paths_tests.rs"]
mod tests;
