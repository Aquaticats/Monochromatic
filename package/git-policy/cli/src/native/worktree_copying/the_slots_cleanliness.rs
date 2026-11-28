//! What:
//!  Git's own reading of the files at a worktree a removal would destroy,
//!  as the one seam a host control answers without a person's files.
//! Why:
//!  Git's removal runs its cleanliness proof only while the command carries no
//!  force,
//!  and it refuses a worktree that holds modified or untracked files with
//!  "use --force to delete it"
//!  (`builtin/worktree.c:1464` and `:1402` at Git `v2.56.1`).
//!  This route answers a bound slot before Git runs,
//!  so the same reading has to be made here or a removal would destroy files Git
//!  would have kept.
//!  The coordinator's pick of 2026-11-28,
//!  with the owner's veto open,
//!  is Git's own gradation:
//!  no force refuses and names `--force`, one force passes, two pass everything.
//!  The proof itself is the crate's own
//!  (`worktree_clean::check_clean`), so this file adds a name and no second
//!  cleanliness rule.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const clean = liveCleanliness.theSlotIsClean({ slot, registration, commonDir }); // throws the reason
//! ```

use crate::transaction_git::GitContext;
use crate::worktree_clean::check_clean_by_content;
use std::path::Path;

/// What:
///  The three places one cleanliness reading needs.
/// Why:
///  A removal knows the worktree it would delete,
///  the registration whose index and `HEAD` the tracked proof reads,
///  and the repository both belong to,
///  and no caller may pass one where another belongs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TheSlotToCheck = { slot; registration; commonDir };
/// ```
pub struct TheSlotToCheck<'a> {
    /// The worktree the command names.
    pub slot: &'a Path,
    /// The registration directory Git keeps for it.
    pub registration: &'a Path,
    /// The repository's common Git directory.
    pub common_dir: &'a Path,
}

/// What:
///  Whether the worktree at one slot holds changes Git would refuse to delete.
/// Why:
///  A host control may not read a person's files,
///  and a release answers with the crate's own proof,
///  so the decision above the seam is the same code in both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface TheSlotsCleanliness { theSlotIsClean(places): void; }
/// ```
pub trait TheSlotsCleanliness {
    /// The refusal's reason,
    /// which is the proof's own words for what it found.
    ///
    /// # Errors
    /// The first change the proof found,
    /// in the words of the crate's cleanliness reading.
    fn the_slot_is_clean(
        &self,
        places: &TheSlotToCheck<'_>,
    ) -> Result<(), String>;
}

/// What:
///  The release answer:
///  the crate's own cleanliness proof that settles an entry its stat data cannot
///  vouch for by reading the file,
///  run through this process's Git.
/// Why:
///  The proof reads the index in process and asks Git one question about
///  untracked files,
///  so it needs the real Git's path and the overlay a forwarded command carries,
///  and nothing else of the command.
///  The content-settling form is the one Git's own removal meets: a worktree
///  `git worktree add` has just written holds entries as new as the index,
///  and Git hashes those before it decides,
///  so a proof that refused them would refuse every fresh clone a person removes
///  at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const cleanliness = new TheLiveCleanliness(context);
/// ```
pub struct TheLiveCleanliness<'a> {
    /// The Git the proof starts for its untracked scan.
    pub context: &'a GitContext,
}

impl TheSlotsCleanliness for TheLiveCleanliness<'_> {
    fn the_slot_is_clean(
        &self,
        places: &TheSlotToCheck<'_>,
    ) -> Result<(), String> {
        return match check_clean_by_content(
            self.context,
            places.slot,
            places.registration,
            places.common_dir,
        ) {
            Ok(_proof) => Ok(()),
            Err(refusal) => Err(refusal.reason),
        };
    }
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "the_slots_cleanliness_tests.rs"]
mod tests;
