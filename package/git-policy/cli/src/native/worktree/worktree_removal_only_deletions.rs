//! What:
//!  What Git reports about a worktree:
//!  nothing at all,
//!  entries missing from it,
//!  or something else.
//! Why:
//!  A removal asks this to tell a tree its own earlier attempt half emptied from one that is
//!  untouched,
//! and the two are not the same words:
//! an empty report says nothing about entries that are nowhere missing,
//!  so reading it as
//! "only deletions" made a fresh, clean worktree read as a half-emptied one.
//! Why:
//!  Two removals finish a tree their own earlier attempt half emptied:
//! the commit service finishing a removal it began with Git,
//! and the removal that went
//!  around a volume and was cut short.
//! Both may finish with `--force`, and `--force` deletes every file Git would keep,
//! so the question is asked of
//!  Git itself:
//! a modified file, a staged addition or an untracked one makes the answer false,
//! and nothing of the
//!  person's then goes with the tree.
//! The two routes ask it the same way, so the answer is one function rather than two
//!  copies that can drift apart.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reported = gitStatusOf({ context, worktree, admin, volumes }); // 'nothing' | 'only-deletions' | 'something'
//! ```

use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use std::ffi::OsString;
use std::path::Path;

/// What:
///  What a `git status` of a worktree says,
///  as a removal needs to read it.
/// Why:
///   An empty report is not a report of deletions:
/// it says the tree holds what Git already holds,
/// and a removal that read the two alike told a person their fresh worktree had been half
///  emptied by a pass of this program.
/// A report of entries that are all missing is the shape a killed pass leaves;
/// anything else is a
///  change of the person's and keeps the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Reported = 'nothing' | 'only-deletions' | 'something';
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Reported {
    /// Git names nothing at all.
    #[default]
    Nothing,
    /// Git names entries, and every one of them is an entry missing from the tree.
    OnlyDeletions,
    /// Git names a change, an addition, a rename or an entry inside no volume the removal leaves.
    Something,
}

/// What:
///  Whether everything Git reports about this worktree is a deletion,
///  which is the state a removal that died in the middle leaves.
/// Why:
///   A staged deletion and a working-tree deletion are both deletions;
/// anything else Git names is not,
/// and a tree Git
///  names nothing in is not one a removal was cut short in either.
///  A Git that cannot answer proves nothing, and the answer is false.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const onlyDeletions = gitStatusIsOnlyDeletions({ context, worktree, admin });
/// ```
pub(crate) fn only_deletions(context: &GitContext, worktree: &Path, admin: &Path) -> bool {
    return reported_outside(context, worktree, admin, &[]) == Reported::OnlyDeletions;
}

/// What:
///  What Git reports about this worktree,
///  leaving out what stands inside one of the volumes the removal keeps where they are.
/// Why:
///  The pass that goes around a volume deletes the worktree's own entries and leaves the volume,
/// and the ignore rule
///  that hid the volume is often one of the entries it deleted:
/// Git then reports the volume itself as
///  untracked,
/// and a strict reading would refuse the very state the pass left.
///  An entry inside a volume is nothing this removal deletes
/// (the volume stays whole),
/// so it is left out of the
///  question;
///  anything else that is not a deletion makes the answer `Something`,
/// because `--force` or the pass would
///  delete it,
///  and an empty report is `Nothing`,
/// which is no shape of a half-emptied tree at all.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reported = gitStatusOutside({ context, worktree, admin, volumes: ['snap'] });
/// ```
pub(crate) fn reported_outside(context: &GitContext, worktree: &Path, admin: &Path, volumes: &[&Path]) -> Reported {
    let mut inside: Vec<String> = Vec::with_capacity(volumes.len());
    for volume in volumes {
        let Ok(relative) = volume.strip_prefix(worktree) else { return reported_outside(context, worktree, admin, &[]) };
        inside.push(relative.display().to_string());
    }
    let request: GitRequest = GitRequest {
        cwd: worktree.to_path_buf(),
        arguments: ["status", "--porcelain", "-z", "--untracked-files=normal"].map(OsString::from).to_vec(),
        without_prefix: true,
        environment: crate::worktree_ladder_target::checkout_environment(admin, worktree),
        ..GitRequest::default()
    };
    // A Git that cannot answer proves nothing, and nothing is what it then says: a caller may not read it as a shape of the
    // tree.
    let Ok(output): std::io::Result<GitOutput> = run_git(context, &request) else { return Reported::Nothing };
    if output.code != Some(0) {
        return Reported::Nothing;
    }
    let mut reported: Reported = Reported::Nothing;
    let mut entries = output.stdout.split(|byte: &u8| return *byte == 0).filter(|entry: &&[u8]| return !entry.is_empty());
    while let Some(entry) = entries.next() {
        if entry.len() < 2 {
            return Reported::Something;
        }
        let deleted: bool = &entry[..2] == b" D" || &entry[..2] == b"D ";
        if deleted {
            reported = Reported::OnlyDeletions;
            continue;
        }
        // What follows the two status letters and a space is the path; a rename or a copy names two of them.
        let renamed: bool = entry[0] == b'R' || entry[0] == b'C' || entry[1] == b'R' || entry[1] == b'C';
        if !names_only_volumes(entry, &inside) {
            return Reported::Something;
        }
        if renamed && !entries.next().is_some_and(|second| return names_only_volumes(second, &inside)) {
            return Reported::Something;
        }
    }
    return reported;
}

/// What:
///  Whether one `git status` entry names a path inside one of `volumes`,
///  or exactly a volume.
/// Why:
///   An entry inside a volume is what the removal leaves untouched,
///  so it says nothing about the worktree's own state;
/// the pathspec-free comparison keeps a path with a quote or a newline in it out of a shell-like parse.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const inside = namesOnlyVolumes('?? snap/', ['snap']);
/// ```
fn names_only_volumes(entry: &[u8], volumes: &[String]) -> bool {
    if entry.len() < 4 {
        return false;
    }
    let named: &[u8] = entry.get(3..).unwrap_or_default();
    let path: String = String::from_utf8_lossy(named).trim_end_matches('/').to_owned();
    for volume in volumes {
        if path == *volume || path.starts_with(format!("{volume}/").as_str()) {
            return true;
        }
    }
    return false;
}

/// The question's controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_only_deletions_tests.rs"]
mod tests;
