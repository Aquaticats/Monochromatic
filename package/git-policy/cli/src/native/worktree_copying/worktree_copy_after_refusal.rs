//! What:
//!  What becomes of the destination after real Git refused a creating command:
//! the folder this command made is taken
//!  away again while it is empty,
//! and the one it left because something is in it is named.
//! Why:
//!  A destination an empty folder is left at is a trap:
//! a Git command run in it acts on the enclosing checkout as if that
//!  were the new worktree,
//! so the person starts work in the main checkout believing they are in a new one.
//! The folder is only ever this command's to remove while it is still the object the command made and still
//!  empty,
//! and a removal that cannot delete contents covers the rest.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const line = settleRefusedDestination(made); // undefined: nothing to say
//! if (line) process.stderr.write(line);
//! ```

use crate::worktree_copy_notices::refused_destination_written_into_notice;
use crate::worktree_subvolume::{CreatedDestinationSettlement, CreatedSubvolume, settle_created_destination};

/// What:
///  Settle the destination a refused add left behind,
/// answering the line the person must read,
/// when there is one.
/// Why:
///   The removal needs no words;
/// a folder that stays because it holds something does,
/// and only the settlement knows
///  which happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function settleRefusedDestination(made: CreatedSubvolume): string | undefined;
/// ```
pub(crate) fn settle_refused_destination(made: &CreatedSubvolume) -> Option<String> {
    return match settle_created_destination(made) {
        CreatedDestinationSettlement::HoldsSomething => Some(refused_destination_written_into_notice(made.path.as_path())),
        CreatedDestinationSettlement::Removed | CreatedDestinationSettlement::Gone | CreatedDestinationSettlement::LeftAlone => None,
    };
}

/// After-refusal controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_after_refusal_tests.rs"]
mod tests;
