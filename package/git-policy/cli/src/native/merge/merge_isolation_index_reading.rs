//! What:
//!  The index read the way a merge's recovery needs it:
//!  the entries that differ from a commit,
//!  and how each stands against the old and the new side of a change.
//! Why:
//!  A staged version of a path the merge replaces has no other name,
//! so it must never be written over;
//! the
//!  straight completion and the settling of a moved branch read the same list and compare each path with the two
//!  sides it knows,
//! and the reading lives in a file of its own so the file that installs a merge stays readable.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const staged = await indexEntries({ place, root, oid }); const standings = changes.map((change) => standingOfIndex(entryOf(staged, change.path), change, change));
//! ```

use crate::merge_isolation_changes::{Change, parse_raw_changes};
use crate::merge_isolation_recovery_steps::{RecoveryPlace, Standing};
use crate::recovery_error::RecoveryError;
use crate::transaction_git::{GitRequest, run_git_checked};
use std::path::Path;

/// What:
///  How the index stands at each changed path,
///  against the two sides of the change.
/// Why:
///   The index is where a person's staged work lives,
/// and a staged version of a path the merge replaces has no other
///  name,
/// so writing the merge's entry over it would discard it with no copy
///  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`;
/// a cross-vendor review of the second round).
///  An entry equal to the old side is the branch's,
/// one equal to the new side is the merge's,
/// and anything else is
///  someone's staged work.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function indexStandings({ place, root, oldOid, changes }): Promise<Standing[]>;
/// ```
pub fn index_standings(
    place: &RecoveryPlace,
    root: &Path,
    old_oid: &str,
    changes: &[Change],
) -> Result<Vec<Standing>, RecoveryError> {
    let staged: Vec<Change> = index_entries(place, root, old_oid)?;
    let mut found: Vec<Standing> = Vec::with_capacity(changes.len());
    for change in changes {
        found.push(standing_of_index(entry_of(staged.as_slice(), change.path.as_slice()), change, change));
    }
    return Ok(found);
}

/// What:
///  The index entries that differ from a commit,
///  one per path.
/// Why:
///   Both the straight completion and the settling of a moved branch read the same list and compare each path with
///  the two sides it knows,
/// so the reading is one function.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseRawChanges(await git(['diff-index', '--cached', '-z', '--raw', oldOid, '--']));
/// ```
pub fn index_entries(place: &RecoveryPlace, root: &Path, old_oid: &str) -> Result<Vec<Change>, RecoveryError> {
    let request: GitRequest = GitRequest::new(
        root,
        &["diff-index", "--cached", "-z", "--raw", "--no-renames", "--no-abbrev", old_oid, "--"],
    );
    let output = match run_git_checked(&place.git, &request) {
        Ok(done) => done,
        Err(error) => return Err(RecoveryError(error.to_string())),
    };
    return parse_raw_changes(output.stdout.as_slice()).map_err(RecoveryError);
}

/// The index entry of one path,
/// or nothing when the index reports none for it.
pub fn entry_of<'a>(staged: &'a [Change], path: &[u8]) -> Option<&'a Change> {
    for candidate in staged {
        if candidate.path == path {
            return Some(candidate);
        }
    }
    return None;
}

/// What:
///  How one index entry stands against the old and the new side of a change.
/// Why:
///   An entry equal to the old side is the branch's (or the merge's,
/// for the settling of a moved branch),
/// one equal to
///  the new side is the end state,
/// and anything else is someone's staged work,
/// which has no other name and must never be written
///  over.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function standingOfIndex(entry, old, next): Standing; // absent or equal to old ? 'old' : equal to next ? 'new' : 'foreign'
/// ```
pub fn standing_of_index(entry: Option<&Change>, old: &Change, next: &Change) -> Standing {
    let Some(found) = entry else {
        return Standing::Old;
    };
    if found.new_mode == next.new_mode && found.new_oid == next.new_oid {
        return Standing::New;
    }
    if found.new_mode == old.old_mode && found.new_oid == old.old_oid {
        return Standing::Old;
    }
    return Standing::Foreign;
}
