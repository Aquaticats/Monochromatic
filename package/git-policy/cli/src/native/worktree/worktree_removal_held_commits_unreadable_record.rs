//! What:
//!  The commit behind a held record this build cannot read:
//!  the batch number its file name carries,
//!  the per-worktree reference that names the commit,
//!  the worktree its registration names,
//!  and the keeping of that commit or the refusal that keeps the worktree.
//! Why:
//!   A record this build cannot read names a batch whose commit the per-worktree reference holds alone,
//!  and the removal deletes that reference with the worktree's own Git directory.
//!  So the commit is read from that reference and kept where a removal cannot reach it,
//!  and when nothing of this repository can name it the worktree stays,
//!  because going on as though an unreadable record were no record left the commit with nothing
//!  that names it.
//!  The questions and the writes are the ones a readable record's keeper uses,
//!  so an unreadable record is never kept differently from a readable one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! keepTheCommitOfAnUnreadableRecord(git, commonDir, gitDir, record); // { kind, line }
//! ```

use crate::plain_file::{
    SMALL_FILE_LIMIT,
    read_plain_file,
};
use crate::service_land_side_reference_write::keep_on_a_side_ref;
use crate::service_scratch_ref_of_a_dropped_batch::{
    RefValue,
    kept_ref,
    rescue_ref,
    scratch_ref,
    value_of,
};
use crate::transaction_git::GitContext;
use crate::worktree_removal_held_commits_keeping::remember_the_commit_of_a_worktree;
use crate::worktree_removal_held_commits_words::{
    words_of_a_commit_kept_from_an_unreadable_record,
    words_of_a_commit_nothing_keeps_behind_an_unreadable_record,
};
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  What keeping the commit behind a record this build cannot read came to.
/// Why:
///   Two outcomes decide what the command does next:
///  the commit now has a reference a removal leaves,
///  or no removal may run because nothing else would name it.
///  The record itself is never removed here,
///  because removing it would leave a commit nothing points at.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type UnreadableKeeping = { kind: 'kept'; line: string } | { kind: 'refused'; line: string };
/// ```
pub(crate) enum UnreadableKeeping {
    /// A reference of this repository now names the commit,
    /// and this is the line the person reads.
    Kept(String),
    /// Nothing a removal spares names the commit,
    /// and this is the paragraph that keeps the worktree.
    Refused(String),
}

/// What:
///  The batch number a record's file name carries,
///  which is all a record this build cannot read still says.
/// Why:
///  Every name of a record is its batch number and the extension,
///  and the batch number is what names the per-worktree reference the commit is read from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// batchOf('/<git dir>/monochromatic-git-policy-cli/held-commits/7.held') // 7
/// ```
pub(crate) fn batch_of(record: &Path) -> Option<u64> {
    let stem: &std::ffi::OsStr = record.file_stem()?;
    return stem.to_str()?.parse::<u64>().ok();
}

/// What:
///  The worktree root a registration names,
///  read from its `gitdir` record without waiting for a writer,
///  or nothing when that record cannot be read.
/// Why:
///   A paragraph that keeps a worktree names the command that finishes the removal,
///  which is the worktree's own path,
///  and the record a registration keeps it in is a file any process may replace with a pipe.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreeOf(gitDir) // '<the worktree root>' | undefined
/// ```
pub(crate) fn worktree_of(git_dir: &Path) -> Option<PathBuf> {
    let recorded: Vec<u8> =
        read_plain_file(git_dir.join("gitdir").as_path(), SMALL_FILE_LIMIT).ok()?;
    let named: PathBuf = PathBuf::from(
        String::from_utf8_lossy(recorded.as_slice())
            .trim()
            .to_owned(),
    );
    let pointer: PathBuf = if named.is_absolute() {
        named
    } else {
        git_dir.join(named)
    };
    let root: PathBuf = pointer.parent()?.to_path_buf();
    return root.is_dir().then_some(root);
}

/// What:
///  The commit behind a record this build cannot read,
///  kept where a removal cannot reach it,
///  or the refusal that keeps its worktree.
/// Why:
///   A record this build cannot parse still names its batch in its file name,
///  and that batch's per-worktree reference is the only other holder of the commit.
///  Reading that reference and keeping the commit on the common Git directory's reference
///  is what makes the removal safe,
///  and a batch whose commit no reference can be read or written for keeps its worktree
///  (`worktree_removal_held_commits.rs`,
///  the module doc).
///  The questions and writes are the ones `keep_one` uses,
///  so an unreadable record can never be kept differently from a readable one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepTheCommitOfAnUnreadableRecord(git, commonDir, gitDir, record) // { kind, line }
/// ```
pub(crate) fn keep_the_commit_of_an_unreadable_record(
    git: &GitContext,
    common_dir: &Path,
    git_dir: &Path,
    record: &Path,
) -> UnreadableKeeping {
    let worktree: PathBuf = match worktree_of(git_dir) {
        Some(found) => found,
        None => git_dir.to_path_buf(),
    };
    let Some(seq) = batch_of(record) else {
        return UnreadableKeeping::Refused(
            words_of_a_commit_nothing_keeps_behind_an_unreadable_record(
                record,
                worktree.as_path(),
                "its name is no batch number, so no reference of that batch can be found",
            ),
        );
    };
    let scratch: String = scratch_ref(seq);
    let commit: String = match value_of(git, git_dir, scratch.as_str()) {
        RefValue::Names(found) => found,
        RefValue::Absent | RefValue::Unasked => {
            return UnreadableKeeping::Refused(words_of_a_commit_nothing_keeps_behind_an_unreadable_record(
                record,
                worktree.as_path(),
                format!("its batch's own reference `{scratch}` names no commit this command can read").as_str(),
            ));
        }
        // A symbolic reference names another reference, not an object, and the command that moves the one it leads to takes this
        //  commit with it: nothing is proven kept here, so no name is kept and the worktree stays.
        RefValue::Symbolic(target) => {
            return UnreadableKeeping::Refused(words_of_a_commit_nothing_keeps_behind_an_unreadable_record(
                record,
                worktree.as_path(),
                format!(
                    "its batch's own reference `{scratch}` is a symbolic reference to `{target}`, which another command can move, so it names no commit this command can prove kept"
                )
                .as_str(),
            ));
        }
    };
    return match keep_this_commit(git, common_dir, seq, commit.as_str(), worktree.as_path()) {
        Some(name) => UnreadableKeeping::Kept(words_of_a_commit_kept_from_an_unreadable_record(record, scratch.as_str(), commit.as_str(), name.as_str())),
        None => UnreadableKeeping::Refused(words_of_a_commit_nothing_keeps_behind_an_unreadable_record(
            record,
            worktree.as_path(),
            format!(
                "neither `{}` nor `{}` could be written for the commit `{scratch}` names, and a name another commit holds is never replaced",
                kept_ref(seq),
                rescue_ref(seq)
            )
            .as_str(),
        )),
    };
}

/// What:
///  The name a reference of the common Git directory now holds `commit` under,
///  for a batch whose own record cannot be read.
/// Why:
///   The batch's reference is read first and the name beside it second,
///  as a readable record's commit is,
///  and a name another commit holds is left where it is.
///  An answer of nothing means no reference of the repository names this commit,
///  which is the one state in which the worktree must stay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// keepThisCommit(git, commonDir, seq, commit) // the name it is kept at, or undefined
/// ```
pub(crate) fn keep_this_commit(
    git: &GitContext,
    common_dir: &Path,
    seq: u64,
    commit: &str,
    worktree: &Path,
) -> Option<String> {
    for name in [kept_ref(seq), rescue_ref(seq)] {
        if matches!(value_of(git, common_dir, name.as_str()), RefValue::Names(named) if named == commit)
        {
            return Some(name);
        }
        if keep_on_a_side_ref(git, common_dir, name.as_str(), commit).is_none() {
            remember_the_commit_of_a_worktree(common_dir, name.as_str(), commit, worktree);
            return Some(name);
        }
        if matches!(value_of(git, common_dir, name.as_str()), RefValue::Names(named) if named == commit)
        {
            return Some(name);
        }
    }
    return None;
}
