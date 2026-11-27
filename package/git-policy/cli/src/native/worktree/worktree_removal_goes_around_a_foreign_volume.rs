//! What:
//!  The removal that goes around a subvolume or a mount point it did not make:
//!  the words it says on the person's command,
//!  the line a landed sweep prints and the record a later command shows,
//!  and the line a later `git worktree add` adds to Git's own refusal.
//! Why:
//!  A worktree can hold a snapshot someone took there,
//! a container's storage or a mounted image,
//! and none of the removal's other routes may take it:
//! the kernel cannot destroy a subvolume that contains another one (`ENOTEMPTY`),
//! and Git's own removal descends into a mount point and deletes what is mounted there.
//! The project decided on 2026-11-12 ("Remove around it") that such a worktree is removed except for
//!  what was not the removal's:
//! the volume and the directories that lead to it stay where they stand,
//! Git forgets the worktree,
//! and the person is told what is left and what frees the path.
//! Both routes make the same decision:
//! the removal a person typed and the one a merge's sweep or the commit service performs.
//! The words live here because four callers say them,
//! and the line a later add prints must not disagree with the removal's own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const report = aroundRemovalReport(target, left);
//! const line = landedAroundLine(landed, worktree, branch, left);
//! await recordLeftInPlace(commonDir, worktree, branch, left);
//! for (const notice of leftInPlaceNotices(commonDir)) notice.show(stderr);
//! const addLine = addBesideAVolumeLine(destination, left);
//! ```
//! This file is one part of the module:
//! the look for the volumes a removal goes around,
//! and the report of a removal that emptied around them and could not finish.
//! The words a person reads are in `worktree_removal_goes_around_a_foreign_volume_reports.rs`,
//! and the record a later command shows is in
//! `worktree_removal_goes_around_a_foreign_volume_records.rs`;
//! both were split out because this module was over the line limit,
//! and each name they defined is re-exported here so no caller changes.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const foreign = volumesBelow('/repo/wt'); // Boundary[]
//! const report = aroundIncompleteReport({ target, admin, commonDir, left, reason });
//! ```

use crate::fs_beneath::open_root;
use crate::subvolume_remove_around::{
    BoundaryLook,
    look_for_boundaries,
};
use crate::subvolume_remove_walk::Boundary;
use crate::worktree_removal_goes_around_a_foreign_volume_reports::{
    PREFIX,
    items,
};
use crate::worktree_removal_takes_its_pending_deletions::is_a_leaving_name;
use std::path::Path;

/// The record a removal leaves for a later command,
/// and the words that command shows for it.
pub use crate::worktree_removal_goes_around_a_foreign_volume_records::{
    LEFT_IN_PLACE_NOTICE_DIRECTORY,
    left_in_place_directory,
    left_in_place_notices,
    record_left_in_place,
};
/// The words a removal that went around a volume prints,
/// and the line a later add adds to Git's own refusal.
pub use crate::worktree_removal_goes_around_a_foreign_volume_reports::{
    BesideAVolume,
    add_beside_a_volume_line,
    around_removal_report,
    emptied_around_report,
    landed_around_line,
    refused_beside_a_volume_report,
    stopped_around_report,
};

/// What:
///  The subvolumes and mount points a tree holds that its removal did not make,
///  and nothing when it may take the tree as it always did.
/// Why:
///   A tree holding one is never given to Git,
/// whose own removal deletes what is mounted at a mount point and unlinks the
///  files of a nested worktree;
/// this is the one look every route makes,
/// and it changes nothing it judges.
/// The names this
///  program leaves a subvolume under while its commit service deletes it are no reason to go around:
/// they hold
///  nothing of the person's,
/// and the removal moves them out before it destroys anything.
/// A tree that cannot be
///  looked at reports nothing here and is the caller's own decision,
/// because a rescue from a look that failed is not a proof.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const foreign = volumesBelow('/repo/wt'); // Boundary[]
/// ```
pub(crate) fn volumes_below(target: &Path) -> Vec<Boundary> {
    let Ok(root) = open_root(target) else {
        return Vec::new();
    };
    let look: BoundaryLook = look_for_boundaries(&root, target);
    let mut foreign: Vec<Boundary> = Vec::new();
    for found in look.found {
        if !is_a_leaving_name(found.path.as_path()) {
            foreign.push(found);
        }
    }
    return foreign;
}

/// What:
///  The report of a removal that emptied the worktree around the volumes it did not make and could not finish:
///  the worktree still holds what was left,
///  and Git still lists it.
/// Why:
///   The report for a removal whose files a destroy took says the subvolume was destroyed and the directory is
///   already gone,
/// and neither is true here:
/// the path still stands with what was left in it,
/// and the
///  worktree is the one Git lists.
/// A person who read those words would look for a directory that is there and for
///  a registration no command of theirs explains.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const report = aroundIncompleteReport({ target, admin, commonDir, left, reason });
/// ```
pub fn around_incomplete_report(
    target: &Path,
    admin: &Path,
    common_dir: &Path,
    left: &[Boundary],
    reason: &str,
) -> String {
    let prune: String = crate::worktree_remove_reports::prune_command(common_dir);
    return format!(
        "{PREFIX}the worktree {target} was emptied around what someone else put inside it, and every entry of the worktree \
         except these was removed, but its registration {admin} was not: {reason}\n{items}\nGit still lists the worktree, \
         and the path still holds what is listed above. To finish, run `{prune}`; it forgets every registration whose \
         directory is missing, and `{prune} --dry-run --verbose` lists them first. Running this removal again finishes it \
         too, and what is listed above stays exactly as it is.",
        target = target.display(),
        admin = admin.display(),
        reason = reason,
        items = items(left),
        prune = prune
    );
}

/// Words and records of a removal that went around a volume stay under the release's own readers.
#[cfg(test)]
#[path = "worktree_removal_goes_around_a_foreign_volume_tests.rs"]
mod tests;
