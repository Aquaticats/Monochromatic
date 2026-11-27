//! What:
//!  The shapes one act on a holder namespace's copy speaks in:
//!  the copy with every name the act gives it,
//!  what the structural stage left,
//!  what the unmount stage took out,
//!  why the act stopped,
//!  and the word a failed child's report becomes.
//! Why:
//!  Each stage runs its own half in a child of its own,
//!  so the caller hands plain values to the child and reads plain values back;
//!  these are those values,
//!  and the refusal's word is what the receipt's failed step carries,
//!  so the act commits to it here rather than leaving a caller to guess
//!  (`doc/planning/cli-git-zfs-forced-removal-holders-ended-and-caller-spared-design.md`,
//!  "Way 8,
//!  in place by the handle").
//!  The two stages that speak in them are `the_structural_stage.rs` and
//!  `the_unmount_stage.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type NamespaceCopy = { namespace; identity; checkout; slot; dataset; handle; directory };
//! type Ready = { route; mountId; parentId; directory?: string };
//! type WayOut = { mountId: number; directory?: string };
//! ```

use crate::worktree_clone_forced_removal::forced_removal_words::RefusalWord;
use crate::worktree_clone_forced_removal::holder_preflight::Route;
use crate::worktree_clone_forced_removal::mount_namespace_child::ActReport;
use crate::worktree_clone_forced_removal::recorded_handle::RecordedHandle;
use std::path::Path;

// What:     the names the act's controls reach through this module, whose
//           table reads and stages now live in siblings; a test build only,
//           because no code here reads them.
// Why:      the control files that are named for this module and its guest arms
//           reach them with `super::` and with this module's path, so the
//           module must still answer to those names in the runs that check
//           them.
//
// In TS you'd write (pseudocode):
// ```ts
// export { rowOf, rowsOf, topmostAt, makeTheWayReady, takeTheCopyOut } from './theAct.js';   // for the tests
// ```
#[cfg(test)]
pub use super::{
    make_the_way_ready,
    row_of,
    rows_of,
    take_the_copy_out,
    topmost_at,
};

/// The mode way 7's directory carries,
/// which the act asks for when it makes it.
pub(super) const DESTINATION_DIRECTORY_MODE: u32 = DESTINATION_MODE;

/// The mode way 7's directory carries:
/// its owner may enter it and nobody else may read or write it.
pub const DESTINATION_MODE: u32 = 0o700;

/// What:
///  One holder namespace's copy and everything the act names it by.
/// Why:
///  The act reads the checkout's own mount,
///  the slot's own dentry and the add's recorded handle inside the holder's
///  namespace,
///  so all four travel together and no caller can hand one namespace's path to
///  another's descriptor.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type NamespaceCopy = { namespace; identity; checkout; slot; dataset; handle; directory };
/// ```
pub struct NamespaceCopy<'a> {
    /// The mount namespace's descriptor,
    /// which the child enters.
    pub namespace: &'a std::fs::File,
    /// The namespace's kernel identity,
    /// for the journal and the report.
    pub identity: &'a str,
    /// The checkout's own path as that namespace spells it.
    pub checkout: &'a Path,
    /// The slot's path as that namespace spells it.
    pub slot: &'a Path,
    /// The clone's dataset name,
    /// which a mount of the clone matches exactly.
    pub dataset: &'a str,
    /// The handle the add recorded of the slot's underlay.
    pub handle: &'a RecordedHandle,
    /// The reserved name way 7's directory carries in the checkout.
    pub directory: &'a str,
}

/// What:
///  What the structural stage left,
///  in the plain values the unmount stage needs.
/// Why:
///  Each stage runs in its own child,
///  so nothing the first opened can be handed to the second,
///  and the second proves its own preconditions again rather than trusting
///  the first's descriptors.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Ready = { route; mountId; parentId; directory?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ready {
    /// The way the act takes here.
    pub route: Route,
    /// The copy's mount id.
    pub mount_id: u64,
    /// The mount the copy sits on,
    /// which the act requires to be the checkout's own.
    pub parent_id: u64,
    /// Way 7's directory name,
    /// which the unmount stage removes once the copy is gone.
    pub directory: Option<String>,
}

/// What:
///  What the unmount stage took out.
/// Why:
///  The journal names the mount id and,
///  for way 7,
///  the directory whose life the report records.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WayOut = { mountId: number; directory?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WayOut {
    /// The copy's mount id.
    pub mount_id: u64,
    /// Way 7's directory,
    /// already removed when the act finished.
    pub directory: Option<String>,
}

/// What:
///  Why the act stopped,
///  in the design's own refusal word and the call it named.
/// Why:
///  A busy unmount after an empty search is a different answer from a mount
///  under the copy,
///  and the word is what the receipt's failed step carries,
///  so the act commits to it here rather than leaving a caller to guess.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopyRefusal = { word; calling: string; errno?: number; said: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CopyRefusal {
    /// The word the report and the receipt carry.
    pub word: RefusalWord,
    /// Which call refused, or which read.
    pub calling: &'static str,
    /// The errno it answered, where it answered one.
    pub errno: Option<i32>,
    /// What it said, in plain words.
    pub said: String,
}

impl std::fmt::Display for CopyRefusal {
    /// One sentence naming the word, the call, the errno and what it said.
    fn fmt(
        &self,
        formatter: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        return match self.errno {
            Some(errno) => {
                write!(
                    formatter,
                    "{}: {} answered errno {errno}: {}",
                    self.word.word(),
                    self.calling,
                    self.said
                )
            }
            None => write!(
                formatter,
                "{}: {}: {}",
                self.word.word(),
                self.calling,
                self.said
            ),
        };
    }
}

/// What:
///  The word and the sentence one failed child report becomes.
/// Why:
///  A locked copy answers `EINVAL` to both ways,
///  a busy unmount after the search found no blocker is the unseen case,
///  and a plain refusal is the act's own;
///  the word is what the report prints.
pub(super) fn refused_from(report: &ActReport) -> CopyRefusal {
    let errno: Option<i32> = report.errno;
    let word: RefusalWord = match errno {
        Some(libc::EINVAL) => RefusalWord::HolderLocked,
        Some(libc::EBUSY) => RefusalWord::HolderUnseen,
        Some(libc::EPERM) => RefusalWord::HolderUnendable,
        _ => RefusalWord::HolderStructural,
    };
    return CopyRefusal {
        word,
        calling: "the child's act",
        errno,
        said: report.detail.clone(),
    };
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_copy_taken_out_of_a_namespace_tests.rs"]
mod tests;
