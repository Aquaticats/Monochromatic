//! What:
//!  Whether a merge that would fast-forward is native Git's own,
//!  or is carried:
//!  the question "does anything of the person's stand at the paths the incoming commits write,
//!  or at their ancestors",
//!  and the decision the answer makes.
//! Why:
//!  A fast-forward is a two-way update of the checkout,
//! so it keeps every path it does not write and every staged entry;
//! at the paths it does write,
//!  though,
//! it overwrites an ignored untracked file of the person's without a word,
//! refuses an unsaved edit or a staged one with
//!  `Your local changes ... would be overwritten by merge`,
//! and replaces a file standing at an ancestor of a path it adds with a directory.
//!  The wrapper carried all of that over a merge that creates a commit and handed every fast-forward to native Git,
//!  so one worktree with one set of unsaved work was treated two ways depending on whether the other side happened to be
//!  strictly ahead, which a person cannot see beforehand (`SPEC.md`,
//!  "Merges and commits never fail";
//!  "Carry edits forward").
//!  The ordinary case stays native Git's:
//! a worktree whose paths the merge writes hold nothing
//!  of the person's is handed to Git at once,
//!  so the question costs one `git status` and a `stat` of each written
//!  path and of each directory on the way to it
//!  (`doc/troubleshooting/cli-git-merge-fast-forward-carries.md`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = theFastForward({ context, root, shape, config, fastForwardable, written }); // 'not' | 'forward' | 'carry'
//! ```

/// The paths `git status --porcelain=v1 -z` names.
use crate::auto_push_dirty_paths::status_paths;
/// The entries standing where a directory the merge creates would go.
use crate::merge_isolation_a_file_where_the_merge_puts_a_directory::the_entries_a_directory_would_replace;
/// The edits at the paths the merge writes that an index mark hides from every status,
/// which only a comparison of the bytes finds.
use crate::merge_isolation_marks_the_status_hides::paths_the_status_hides;
/// One path spelled as a Git argument,
/// so its bytes are not read as text.
use crate::merge_isolation_carry_reads::path_argument;
/// The worktree path of a merge tree's own path.
use crate::merge_isolation_changes::worktree_path;
/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// The settings and the command-line reading of the merge.
use crate::merge_isolation_run_types::MergeConfig;
use crate::merge_options::{FastForwardChoice, MergeShape};
/// The bound of one query's path text,
/// and how a path is spelled so no byte of it is read as a pattern.
use crate::command::rule_worktree_dirty::{IGNORED_PATHS_QUERY_BYTES, TOP_LITERAL_PATHSPEC};
/// Real Git.
use crate::transaction_git::{GitContext, GitRequest, run_git};
/// Owned operating-system text.
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// The tag of this module in the diagnostic log.
const TAG: &str = "fastForward";

/// What a merge that would fast-forward is here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FastForward = 'not' | 'forward' | 'carry';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FastForward {
    /// This command makes a merge commit,
    /// or native Git refuses it,
    /// so the fast-forward question does not arise.
    Not,
    /// Nothing of the person's stands in the way,
    /// so native Git's own fast-forward is the whole answer.
    Forward,
    /// Something of the person's stands in the way,
    /// or could not be asked about,
    /// so the merge is carried as a merge that makes a commit is.
    Carry,
}

/// What:
///  What this merge is,
///  from the command line,
///  the settings and the worktree:
///  not a fast-forward,
///  a fast-forward native Git runs,
///  or one the wrapper carries.
/// Why:
///   `--ff-only` and `merge.ff=only` state that the branch may move only by a fast-forward,
///  and a merge that cannot
///  fast-forward is native Git's,
///  whose refusal and advice are its own,
///  so neither is a reason to stop the command here.
///  A merge that must make a commit (`--no-ff`, `merge.ff=false`) is no fast-forward either,
///  whichever
///  way the branch stands,
///  and a merge that cannot fast-forward while `--ff-only` or `merge.ff=only` forbids a merge commit is
///  native Git's own refusal,
///  never a merge commit this path would make.
///  A question that could not be answered carries the merge:
/// the carry reads the same paths again for
///  itself,
/// and the paths its own reads cannot explain are native Git's then.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function theFastForward({ context, root, shape, config, fastForwardable, written }): FastForward;
/// ```
pub fn the_fast_forward(
    context: &GitContext,
    root: &Path,
    shape: &MergeShape,
    config: &MergeConfig,
    fast_forwardable: bool,
    written: &[Vec<u8>],
) -> FastForward {
    let wants_a_commit: bool = match shape.fast_forward {
        FastForwardChoice::Never => true,
        FastForwardChoice::Allowed | FastForwardChoice::Only => false,
        FastForwardChoice::Default => config.fast_forward == Some(false),
    };
    let only_fast_forward: bool =
        shape.fast_forward == FastForwardChoice::Only || (config.fast_forward_only && shape.fast_forward == FastForwardChoice::Default);
    if wants_a_commit {
        return FastForward::Not;
    }
    // A merge that cannot fast-forward while only a fast-forward is allowed is native Git's refusal, not a merge commit this
    //  path would make: `fatal: Not possible to fast-forward, aborting.` is what the person asked for.
    if !fast_forwardable {
        return if only_fast_forward { FastForward::Forward } else { FastForward::Not };
    }
    let Some(in_the_way) = paths_in_the_way(context, root, written) else {
        debug(TAG, "the paths this fast-forward writes could not be asked about, so the merge is carried and asks again");
        return FastForward::Carry;
    };
    if in_the_way.is_empty() {
        debug(TAG, "nothing of the person's stands at the paths this fast-forward writes, so native Git runs it");
        return FastForward::Forward;
    }
    debug(TAG, format!("the fast-forward is carried: {} entries of the person's stand at the paths it writes", in_the_way.len()).as_str());
    return FastForward::Carry;
}

/// What:
///  Every path at or below one of the written paths,
///  or at an ancestor of one,
///  that holds something the merge would write over,
///  or nothing when a question about them could not be answered.
/// Why:
///   These are the only paths a merge can lose work at or be refused at,
///  so the question asks about them alone,
///  and a
///  worktree of any size costs what they cost rather than a walk of everything the person ignores.
///  A written path that holds
///  no file is asked apart from Git:
///  `git status` lists no named pipe,
///  no socket and no device,
///  measured with Git 2.56.0,
///  and one of those at a written path is work no carry can hold.
///  An edit an index mark hides is the fourth question:
/// `assume-unchanged` and `skip-worktree` tell
///  `git status`,
///  `git diff-files` and `git diff-index` not to compare the copy,
/// so an edit there is in no answer
///  and is found by comparing the bytes with the entry
///  (`merge_isolation_marks_the_status_hides.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inTheWay = [...entriesTheMergeWritesOver(root, written), ...entriesADirectoryWouldReplace(root, written), ...statusAnswersUnder(context, root, written)];
/// ```
pub fn paths_in_the_way(context: &GitContext, root: &Path, written: &[Vec<u8>]) -> Option<Vec<Vec<u8>>> {
    // A merge that writes no path cannot write over anything, and asking Git about no paths at all would be a question
    //  about the whole worktree.
    if written.is_empty() {
        return Some(Vec::new());
    }
    let mut found: Vec<Vec<u8>> = the_entries_the_merge_writes_over(root, written);
    let marked: Vec<Vec<u8>> = paths_the_status_hides(context, root, written).ok()?;
    for path in the_entries_a_directory_would_replace(root, written)
        .into_iter()
        .chain(marked)
        .chain(the_answers_under(context, root, written)?)
    {
        if !found.contains(&path) {
            found.push(path);
        }
    }
    found.sort();
    return Some(found);
}

/// What:
///  The written paths whose own worktree entry is a directory,
///  a named pipe,
///  a socket or a device.
/// Why:
///   A directory there is a path the merge replaces with a file,
///  and a pipe,
///  a socket or a device holds no bytes any
///  carry can hold;
///  both stop the merge with a line of their own rather than reach Git's update,
///  and neither is
///  something a merge can write over and keep.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// written.filter((path) => { const found = symlinkMetadataSync(join(root, path)); return !found.isFile() && !found.isSymbolicLink(); });
/// ```
fn the_entries_the_merge_writes_over(root: &Path, written: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut found: Vec<Vec<u8>> = Vec::new();
    for path in written {
        let location: PathBuf = worktree_path(root, path.as_slice());
        let Ok(metadata) = std::fs::symlink_metadata(location.as_path()) else { continue };
        let kind: std::fs::FileType = metadata.file_type();
        if !kind.is_file() && !kind.is_symlink() {
            found.push(path.clone());
        }
    }
    return found;
}

/// What:
///  The answers `git status` gives for the written paths,
///  or nothing when the read gave no answer at all.
/// Why:
///  One start answers what the worktree holds at those paths instead of four;
/// the ignored answer is the one
///  native Git's fast-forward overwrites without a word,
///  and `--ignored=traditional` is what makes status print
///  it (`--no-optional-locks` keeps the read from writing the index).
///  The pathspecs are the wrapper's own and read from the top of the worktree,
///  so no pathspec mode of the person's,
///  and no directory they ran the command in,
///  changes which paths are asked about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const batch of batches(written)) keep(statusPaths(await git({ cwd: root, withoutPrefix: true, ownPathspecs: true }, ['--no-optional-locks', 'status', '--porcelain=v1', '-z', '--untracked-files=all', '--ignored=traditional', '--', ...batch.map(topLiteral)])));
/// ```
fn the_answers_under(context: &GitContext, root: &Path, written: &[Vec<u8>]) -> Option<Vec<Vec<u8>>> {
    let mut found: Vec<Vec<u8>> = Vec::new();
    for (start, end) in batches(written) {
        for path in status_paths(one_status(context, root, &written[start..end])?.as_slice()) {
            // An answer deeper than a written path is a directory standing there, which the merge writes over too.
            if at_or_below_a_written_path(path.as_slice(), written) && !found.contains(&path) {
                found.push(path);
            }
        }
    }
    return Some(found);
}

/// What:
///  One `git status` for one batch of written paths,
///  with its NUL-separated answer,
///  or nothing when Git gave none.
/// Why:
///   A merge can write more paths than one argument list holds,
///  and the batch bounds the path text one start
///  carries;
///  an answer that never came is not "nothing stands there",
///  so it is reported as no answer and the
///  caller carries the merge rather than leave the person's file to whatever Git does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const output = await git({ cwd: root, withoutPrefix: true, ownPathspecs: true }, [...quietWords, ...batch.map(topLiteral)]);
/// return output.succeeded() ? output.stdout : undefined;
/// ```
fn one_status(context: &GitContext, root: &Path, batch: &[Vec<u8>]) -> Option<Vec<u8>> {
    let mut words: Vec<OsString> = Vec::with_capacity(batch.len() + 7);
    for token in [
        "--no-optional-locks",
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
        "--ignored=traditional",
        "--",
    ] {
        words.push(OsString::from(token));
    }
    for path in batch {
        let mut spec: Vec<u8> = TOP_LITERAL_PATHSPEC.to_vec();
        spec.extend_from_slice(path.as_slice());
        words.push(path_argument(spec.as_slice()));
    }
    let mut request: GitRequest = GitRequest::new::<&str>(root, &[]).own_pathspecs();
    request.without_prefix = true;
    request.arguments = words;
    return match run_git(context, &request) {
        Ok(output) if output.succeeded() => Some(output.stdout),
        Ok(output) => {
            debug(TAG, format!("git status gave no answer ({}), so the merge is carried", output.error_text()).as_str());
            None
        }
        Err(error) => {
            debug(TAG, format!("git status could not start ({error}), so the merge is carried").as_str());
            None
        }
    };
}

/// What:
///  Whether `path` is one of the written paths or lies below one.
/// Why:
///   An answer below a written path means a directory stands there,
///  and the merge writes over that whole
///  directory;
///  an answer beside them is a path the merge never touches.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const atOrBelow = (path, written) => written.some((one) => path === one || path.startsWith(`${one}/`));
/// ```
fn at_or_below_a_written_path(path: &[u8], written: &[Vec<u8>]) -> bool {
    for one in written {
        if path == one.as_slice() {
            return true;
        }
        let mut prefix: Vec<u8> = one.clone();
        prefix.push(b'/');
        if path.starts_with(prefix.as_slice()) {
            return true;
        }
    }
    return false;
}

/// What:
///  The batches of written paths one status query carries,
///  as the half-open ranges of the list they name.
/// Why:
///   A merge of any size still makes calls whose argument list no operating system refuses,
///  and the ranges cover
///  the list once,
///  in order,
///  keeping every call's path text under the bound the ignored-paths read uses.
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
        let carried: usize = path.len() + TOP_LITERAL_PATHSPEC.len() + 1;
        if index > start && bytes + carried > IGNORED_PATHS_QUERY_BYTES {
            spans.push((start, index));
            start = index;
            bytes = 0;
        }
        bytes += carried;
    }
    spans.push((start, paths.len()));
    return spans;
}

/// The decision and the batches it asks in.
#[cfg(test)]
#[path = "merge_isolation_a_fast_forward_in_the_way_tests.rs"]
mod tests;
