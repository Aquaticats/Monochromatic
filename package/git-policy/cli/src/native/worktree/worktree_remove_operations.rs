//! What:
//!  The operations of one removal that touch the kernel or start Git,
//! behind one trait,
//!  and the kernel and Git as they are.
//!  `&self` borrows the implementation read-only;
//! `dyn RemoveOperations` is how a caller holds "any implementation"
//!  (a TS interface).
//! Why:
//!   The ordering of the checks is the same on any filesystem;
//! a stand-in lets the controls run it on a plain directory tree,
//! and swap objects between two steps to prove the identity checks.
//!  It is its own file because `worktree_remove.rs` reached the crate's length limit with it in,
//!  and because this trait is the seam every control of a removal replaces.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! interface RemoveOperations { isSubvolumeRoot(path): boolean; removalAllowed(path): boolean; checkClean(...): void; destroy(held): void }
//! const linuxOperations: RemoveOperations = { isSubvolumeRoot, removalAllowed, checkClean, destroy };
//! ```

// What:     `use crate::diagnostic_log::debug;` imports the debug log sink
//           this file writes a line to when this kernel has no destroy by id.
// Why:      That line says which of the two kernel calls the destroy took, and the
//           person reads it in the journal rather than on the terminal.
//
// In TS you'd write (pseudocode):
// ```ts
// // No 1:1 equivalent — the package's own logger, called by tag.
// ```
use crate::diagnostic_log::debug;
// What:     `use crate::fs_btrfs_ioctl::{snapshot_destroy_by_id, snapshot_destroy_in, subvolume_id};`
//           imports the three kernel calls a destroy stands on: the id of a
//           subvolume below a held directory, the destroy that names that id, and
//           the destroy that names the subvolume below the held parent.
// Why:      The id is what the destroy acts on, so a subvolume renamed under the
//           held parent is still the one that goes.
//
// In TS you'd write (pseudocode):
// ```ts
// import { snapshotDestroyById, snapshotDestroyIn, subvolumeId } from '../fs_btrfs_ioctl';
// ```
use crate::fs_btrfs_ioctl::{
    snapshot_destroy_by_id,
    snapshot_destroy_in,
    subvolume_id,
};
// What:     `use crate::fs_kind::is_subvolume_root;` imports the question "is this
//           directory the root of a subvolume, rather than the subvolume's content".
// Why:      Only a root can be destroyed by one call, and the answer comes from the
//           kernel rather than from a path's shape.
//
// In TS you'd write (pseudocode):
// ```ts
// import { isSubvolumeRoot } from '../fs_kind';
// ```
use crate::fs_kind::is_subvolume_root;
// What:     `use crate::fs_mount_options::read_mount_facts;` imports the mount
//           options of the filesystem a path stands on.
// Why:      The kernel refuses an unprivileged destroy on a mount that does not
//           allow one, so the option is read before the call is made.
//
// In TS you'd write (pseudocode):
// ```ts
// import { readMountFacts } from '../fs_mount_options';
// ```
use crate::fs_mount_options::read_mount_facts;
// What:     `use crate::transaction_git::GitContext;` imports the Git the removal
//           runs and the overlay it runs Git in.
// Why:      The hand-over of a subvolume to the commit service carries that context.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { GitContext } from '../transaction_git';
// ```
use crate::transaction_git::GitContext;
// What:     `use crate::worktree_clean::{check_clean, check_clean_by_content};` imports
//           the two proofs that a worktree holds no change Git would keep.
// Why:      The destroy answers for the same files Git's unforced removal answers
//           for, so the same two proofs decide it here.
//
// In TS you'd write (pseudocode):
// ```ts
// import { checkClean, checkCleanByContent } from '../worktree_clean';
// ```
use crate::worktree_clean::{
    check_clean,
    check_clean_by_content,
};
// What:     `use crate::worktree_removal_holds_the_worktree::Held;` imports the
//           directories a removal holds open, with the identities it proved.
// Why:      The destroy and the hand-over name the subvolume through those held
//           descriptors, never through a path another process could have replaced.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Held } from './worktree_removal_holds_the_worktree';
// ```
use crate::worktree_removal_holds_the_worktree::Held;
// What:     `use crate::worktree_remove_request_and_last_look::{RemoveRequest, TAG};`
//           imports what the removal was asked and the tag its log lines carry.
// Why:      The proofs are given the plan the person named, and every line of this
//           subject writes the same tag.
//
// In TS you'd write (pseudocode):
// ```ts
// import { TAG, type RemoveRequest } from './worktree_remove_request_and_last_look';
// ```
use crate::worktree_remove_request_and_last_look::{
    RemoveRequest,
    TAG,
};
// What:     `use std::ffi::OsStr;` imports a borrowed operating-system string, the
//           type a registration's name arrives in. Its owned sibling is `OsString`,
//           and the UTF-8-only pair is `&str`/`String`.
// Why:      A name read from the file system may be no UTF-8 at all, so the borrowed
//           OS string is the type that carries it without a copy.
//
// In TS you'd write (pseudocode):
// ```ts
// // No 1:1 equivalent — a path's own segment type, which need not be text.
// ```
use std::ffi::OsStr;
// What:     `use std::path::{Path, PathBuf};` imports the borrowed path type and its
//           owned sibling. Their text siblings are `&str` and `String`, which cannot
//           hold a path that is not UTF-8.
// Why:      The paths this file reads are borrowed, and one is joined to a name, which
//           makes an owned path.
//
// In TS you'd write (pseudocode):
// ```ts
// import type { Path, PathBuf } from '../paths';
// ```
use std::path::{
    Path,
    PathBuf,
};

/// The mount option that lets a regular user destroy a subvolume with the ioctl.
/// It is `pub(crate)` because the removal's own steps compare it with the answer
/// the operations give about the destroy.
pub(crate) const REMOVAL_OPTION: &str = "user_subvol_rm_allowed";

/// What:
///  The operations of one removal that touch the kernel or start Git,
/// behind one trait.
///  `&self` borrows the implementation read-only;
/// `dyn RemoveOperations` is how a caller holds "any implementation"
///  (a TS interface).
/// Why:
///   The ordering of the checks is the same on any filesystem;
/// a stand-in lets the controls run it on a plain
///  directory tree,
/// and swap objects between two steps to prove the identity checks.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface RemoveOperations { isSubvolumeRoot(path): boolean; removalAllowed(path): boolean; checkClean(...): void; destroy(held): void }
/// ```
pub trait RemoveOperations {
    /// Whether `path` is the root of a subvolume.
    fn is_subvolume_root(
        &self,
        path: &Path,
    ) -> bool;
    /// Whether an unprivileged destroy is allowed on the mount holding `path`.
    fn removal_allowed(
        &self,
        path: &Path,
    ) -> bool;
    /// The proof that the worktree is clean,
    /// or the reason it is not proven.
    fn check_clean(
        &self,
        request: &RemoveRequest<'_>,
        admin: &Path,
        common_dir: &Path,
    ) -> Result<(), String>;
    /// The proof of the same state that settles by reading a file whatever its stat data cannot vouch for,
    /// or the reason it is not proven;
    /// a stand-in proves nothing by content,
    /// so a state it refuses stays refused.
    fn check_clean_by_content(
        &self,
        _request: &RemoveRequest<'_>,
        _admin: &Path,
        _common_dir: &Path,
    ) -> Result<(), String> {
        return Err(String::from(
            "this stand-in proves no file by content, so the state stays unproven",
        ));
    }
    /// Destroy the subvolume that `held` holds,
    /// and nothing else.
    fn destroy(
        &self,
        held: &Held,
    ) -> Result<(), String>;
    /// What:
    ///  Record the subvolume for the service and rename it out of the person's way,
    /// so that no command waits for its delete.
    /// Why:
    ///  A stand-in that hands nothing over answers the reason,
    /// and the removal then destroys the subvolume in place as it always did,
    /// so every
    ///  control of the deletion keeps working on the destroy below.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// handOver(context, commonDir, held, target, registration): void; // throws the reason it did not
    /// ```
    fn hand_over(
        &self,
        _context: &GitContext,
        _common_dir: &Path,
        _held: &Held,
        _target: &Path,
        _registration: &OsStr,
    ) -> Result<(), String> {
        return Err(String::from(
            "this stand-in hands nothing to the service, so the subvolume is destroyed here",
        ));
    }
}

/// The kernel and Git,
/// as they are.
#[derive(Clone, Copy, Debug, Default)]
pub struct LinuxOperations;

/// Whether an errno from the destroy by id means this kernel lacks the call,
/// so that the destroy by name below the held
///  parent is the way left.
fn by_id_unavailable(errno: i32) -> bool {
    return errno == libc::ENOTTY || errno == libc::EOPNOTSUPP || errno == libc::EINVAL;
}

/// Every operation is the primitive of the same name.
impl RemoveOperations for LinuxOperations {
    fn is_subvolume_root(
        &self,
        path: &Path,
    ) -> bool {
        return is_subvolume_root(path).unwrap_or(false);
    }

    fn removal_allowed(
        &self,
        path: &Path,
    ) -> bool {
        return match read_mount_facts(path) {
            Ok(Some(facts)) => facts.has_option(REMOVAL_OPTION),
            _ => false,
        };
    }

    fn check_clean(
        &self,
        request: &RemoveRequest<'_>,
        admin: &Path,
        common_dir: &Path,
    ) -> Result<(), String> {
        return match check_clean(
            request.context,
            request.plan.target.as_path(),
            admin,
            common_dir,
        ) {
            Ok(_proof) => Ok(()),
            Err(refusal) => Err(refusal.reason),
        };
    }

    fn check_clean_by_content(
        &self,
        request: &RemoveRequest<'_>,
        admin: &Path,
        common_dir: &Path,
    ) -> Result<(), String> {
        return match check_clean_by_content(
            request.context,
            request.plan.target.as_path(),
            admin,
            common_dir,
        ) {
            Ok(_proof) => Ok(()),
            Err(refusal) => Err(refusal.reason),
        };
    }

    fn hand_over(
        &self,
        context: &GitContext,
        common_dir: &Path,
        held: &Held,
        target: &Path,
        registration: &OsStr,
    ) -> Result<(), String> {
        // The id names the subvolume that was held, whatever path it has by the time the service acts.
        let id: u64 = match subvolume_id(&held.target, target) {
            Ok(found) => found,
            Err(error) => {
                return Err(format!(
                    "the subvolume of {} cannot be identified ({error})",
                    target.display()
                ));
            }
        };
        return crate::worktree_remove_hand_over::hand_over_removal(
            context,
            common_dir,
            &held.parent,
            held.parent_path.as_path(),
            held.name.as_os_str(),
            id,
            target,
            registration,
        );
    }

    fn destroy(
        &self,
        held: &Held,
    ) -> Result<(), String> {
        let shown: PathBuf = held.parent_path.join(held.name.as_os_str());
        // The id names the subvolume that was held, whatever path it has by now.
        match subvolume_id(&held.target, shown.as_path()) {
            Ok(id) => match snapshot_destroy_by_id(&held.parent, id, shown.as_path()) {
                Ok(()) => return Ok(()),
                Err(error) if by_id_unavailable(error.errno) => {
                    debug(TAG, format!("the destroy of {} by id is not available ({error}); destroying it by name below the held parent", shown.display()).as_str());
                }
                Err(error) => return Err(error.to_string()),
            },
            Err(error) if by_id_unavailable(error.errno) => {
                debug(TAG, format!("the id of the subvolume {} is not available ({error}); destroying it by name below the held parent", shown.display()).as_str());
            }
            Err(error) => return Err(error.to_string()),
        }
        return match snapshot_destroy_in(
            &held.parent,
            held.name.as_os_str(),
            held.parent_path.as_path(),
        ) {
            Ok(()) => Ok(()),
            Err(error) => Err(error.to_string()),
        };
    }
}
