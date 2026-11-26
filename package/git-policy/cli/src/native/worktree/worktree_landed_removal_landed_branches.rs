//! What:
//!  Which branches the merge that just ran landed:
//!  the branches fully contained in its new head and not in the head before it,
//!  the branches whose own reflog says they have committed,
//!  and the branches whose tip another branch's reflog names.
//! Why:
//!   The event that ends a worktree's use is its branch landing,
//!  and a branch that never moved since it was made holds no commit of its own,
//!  so a merge that brings in another branch's commits must not take its worktree
//!  (a cross-vendor audit of `574063773d`,
//!  finding F4).
//!  A person who runs `git reset --hard main` in a task worktree moves its branch onto main's commit and makes nothing,
//!  so the tip alone says nothing,
//!  and another branch's own reflog naming that commit is what a branch that moved onto it looks like
//!  (a cross-vendor review of the landed sweep,
//!  round 208).
//!  What no reflog can prove keeps the worktree:
//!  Git records no owner of a commit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const landed = branchesMergedInto(place, now); // throws the reason
//! const candidates = landedBranches(place, earlier, landed, thisMerge);
//! ```

use crate::transaction_git::{
    GitOutput,
    GitRequest,
    run_git,
};
use crate::worktree_branch_reflog::{
    MergeMove,
    branch_made_at,
    branch_moved_onto,
};
use crate::worktree_landed_removal_records::LandedPlace;
use std::ffi::OsString;
use std::path::Path;

/// One branch a Git query named,
/// with the commit it points at.
/// Why:
///   Which worktrees a merge landed is decided from the branches its new head contains and the commit each one holds:
///  a branch's own commits are what says it landed,
///  and a name alone cannot answer that.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Landing {
    /// The full reference name.
    pub(crate) reference: String,
    /// The commit it points at now.
    pub(crate) tip: String,
}

/// What:
///  The branches that are fully contained in `commit`,
///  each with the commit it points at.
/// Why:
///   A branch has landed when every commit it holds is in the branch that received the merge,
///  which one Git start answers for every branch,
///  and the commit each one points at is what says whether it holds a commit of its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const merged = branchesMergedInto(context, commit); // throws the reason
/// ```
pub(crate) fn branches_merged_into(
    place: &LandedPlace<'_>,
    commit: &str,
) -> Result<Vec<Landing>, String> {
    let arguments: Vec<OsString> = [
        "for-each-ref",
        "--merged",
        commit,
        "--format=%(refname)%09%(objectname)",
        "refs/heads",
    ]
    .map(OsString::from)
    .to_vec();
    let output: GitOutput = match run_git(
        place.context,
        &GitRequest::new(place.cwd, arguments.as_slice()),
    ) {
        Ok(finished) => finished,
        Err(error) => {
            return Err(format!(
                "Git cannot list the branches merged into {commit} ({error})"
            ));
        }
    };
    if output.code != Some(0) {
        return Err(format!(
            "Git cannot list the branches merged into {commit}: {}",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        ));
    }
    let mut found: Vec<Landing> = Vec::new();
    for line in String::from_utf8_lossy(output.stdout.as_slice()).lines() {
        let (reference, tip) = line.split_once('\t').unwrap_or((line, ""));
        found.push(Landing {
            reference: reference.to_owned(),
            tip: tip.to_owned(),
        });
    }
    return Ok(found);
}

/// The reference a landing names,
/// for the set of branches that were already contained in a head.
pub(crate) fn reference_of(one: Landing) -> String {
    return one.reference;
}

/// Whether the branch holds a commit of its own,
/// which is false only when its reflog says it has not moved since it was made.
/// Why:
///   A branch made at a commit another branch holds and left there did no work of its own,
///  so nothing of it can land,
///  and a merge that brings that other branch in must not take its worktree
///  (a cross-vendor audit of `574063773d`,
///  finding F4).
///  A repository that keeps no reflog for the branch proves nothing about it,
///  so it stays a candidate as it was before this rule.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// holdsItsOwnCommits(commonDir, 'refs/heads/task-z', 'dfc6c4e...') // false when its reflog begins at that commit
/// ```
pub(crate) fn holds_its_own_commits(
    common_dir: &Path,
    reference: &str,
    tip: &str,
) -> bool {
    return branch_made_at(common_dir, reference) != Some(tip.to_owned());
}

/// What:
///  The merge that just ran,
///  as much of it as says which reflog line the merge itself wrote.
/// Why:
///  The witness rule reads the reflog of every landed branch,
///  and a fast-forward writes one line in the branch the merge ran in that names the landed tip.
///  Skipping that one line needs the branch,
///  the commit it held before the merge and the one it holds now,
///  so a control names them together rather than as three loose strings.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const thisMerge = { branch: 'main', before: 'a1b2c3d...', now: 'dfc6c4e...' };
/// ```
#[derive(Clone, Copy)]
pub(crate) struct ThisMerge<'a> {
    /// The branch the merge ran in,
    /// in Git's short spelling,
    /// or the words for a `HEAD` that names no branch.
    pub(crate) branch: &'a str,
    /// The commit that branch held before the merge.
    pub(crate) before: &'a str,
    /// The commit it holds now.
    pub(crate) now: &'a str,
}

/// Whether `reference` is the branch `short` names,
/// which a caller gives in Git's own short spelling.
/// Why:
///   A landing names a reference in full,
///  and the branch a merge ran in is named the way Git prints it,
///  so the two are compared through the one prefix Git puts in front of a branch name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesABranch('refs/heads/main', 'main') // true
/// ```
fn names_a_branch(
    reference: &str,
    short: &str,
) -> bool {
    return reference.strip_prefix("refs/heads/") == Some(short);
}

/// Whether another branch's own reflog names `tip`,
/// which is a commit the branch named by `reference` did not make.
/// Why:
///  A person who runs `git reset --hard main` in a task worktree moves its branch onto main's commit and makes nothing.
///  The tip then differs from the commit that worktree's own reflog says the branch was made at,
///  so the rule above read the move as work of the branch's own and the merge that brought main in removed the worktree
///  (a cross-vendor review of the landed sweep,
///  round 208).
///  The branch whose reflog is asked need not still sit on that commit:
///  one that committed again after the reset names the same commit in an older line,
///  and a rule that demanded the tip equal the candidate's found no witness at all,
///  which took the worktree as surely as the equal-tip state did.
///  The one line this merge wrote in the branch it ran in is no witness:
///  a fast-forward leaves that branch on the landed tip,
///  so its own move there would otherwise read as another branch's work.
///  What stays unseen is the mirror state:
///  a branch moved onto a commit this candidate made keeps the worktree too,
///  because Git records no owner of a commit,
///  and so does a landed worktree whose tip another branch's history passed through.
///  Both are worktrees this rule cannot prove it may remove,
///  and they stay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// anotherBranchTookTheTip(landed, commonDir, 'refs/heads/task-z', 'dfc6c4e...', { branch: 'main', before, now })
/// ```
pub(crate) fn another_branch_took_the_tip(
    landed: &[Landing],
    common_dir: &Path,
    reference: &str,
    tip: &str,
    this_merge: ThisMerge<'_>,
) -> bool {
    for other in landed {
        if other.reference == reference {
            continue;
        }
        let merge_move: Option<MergeMove<'_>> =
            match names_a_branch(other.reference.as_str(), this_merge.branch) {
                true => Some(MergeMove {
                    from: this_merge.before,
                    to: this_merge.now,
                }),
                false => None,
            };
        if branch_moved_onto(common_dir, other.reference.as_str(), tip, merge_move) {
            return true;
        }
    }
    return false;
}
