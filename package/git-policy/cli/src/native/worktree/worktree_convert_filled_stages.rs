//! What:
//!  What a conversion does about a stage container another command is filling below the tree it would swap or
//!  delete:
//!  the reason it stops,
//!  and the words a person reads.
//! Why:
//!  A swap moves a whole worktree aside and a removal deletes a whole tree,
//! so either one would take the place another
//!  command writes its copy into while that command's claim names a live owner.
//! The process table alone does not find it:
//! a
//!  command filling a container holds no descriptor there between two entries,
//! and the claim is the record that says
//!  the container is somebody's.
//! Both decisions are the phase's,
//! which calls these two questions:
//! the swap asks before the
//!  exchange,
//! and the removal asks before it compares the old tree with the installed one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const reasons = swapRefusals(commonDir, worktree);
//! removalRefusal(commonDir, worktree, oldTree);
//! ```

use crate::subvolume_error::MigrateError;
use crate::worktree_copy_stages_being_filled::{FilledStage, filled_stages_below};
use std::path::Path;

/// What:
///  One sentence per stage container another command is filling below `worktree`.
/// Why:
///   The exchange has not happened when the swap asks,
/// so refusing here leaves that command's work where it is and its
///  add can finish,
/// which keeping the old tree afterwards cannot do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// swapRefusals('/common', '/w'); // ['process 1234 is filling the stage container ...; nothing is swapped, ...']
/// ```
pub fn swap_refusals(common: &Path, worktree: &Path) -> Vec<String> {
    let mut reasons: Vec<String> = Vec::new();
    match filled_stages_below(common, worktree, worktree) {
        Ok(filled) => {
            for stage in &filled {
                reasons.push(format!("{}; nothing is swapped, so run the command again when that command has finished", words(stage)));
            }
        }
        // A claim this release cannot read rules no container out, so the swap is refused rather than guessed at.
        Err(error) => reasons.push(format!(
            "the stage claims of {} cannot be read ({}), so no stage another command is filling can be ruled out",
            common.display(),
            error.0
        )),
    }
    return reasons;
}

/// What:
///  Refuses to remove the old tree at `old_tree` while another command is filling a stage container below
///  `worktree`,
///  or when its claims cannot be read.
/// Why:
///   The old tree is deleted whole,
/// so a container still being filled is that command's work and not this tree's to remove;
///  the refusal names the container,
/// the process and the rerun that finishes the conversion once that command has finished.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removalRefusal('/common', '/w', '/w.cli-git-converting'); // throws 'the old tree at ... is kept: ...'
/// ```
pub fn removal_refusal(common: &Path, worktree: &Path, old_tree: &Path) -> Result<(), MigrateError> {
    // `match` on the read: a claim this release cannot read rules no container out, so the old tree stays.
    let filled: Vec<FilledStage> = match filled_stages_below(common, worktree, old_tree) {
        Ok(found) => found,
        Err(error) => {
            return Err(MigrateError::Journal(format!(
                "the stage claims of {} cannot be read ({}), so no stage another command is filling can be ruled out; the old tree at {} is kept",
                common.display(),
                error.0,
                old_tree.display()
            )));
        }
    };
    if filled.is_empty() {
        return Ok(());
    }
    let mut filling: Vec<String> = Vec::new();
    for stage in &filled {
        filling.push(words(stage));
    }
    return Err(MigrateError::Journal(format!(
        "the old tree at {} is kept: {}; nothing is deleted, so run this command again when that command has finished",
        old_tree.display(),
        filling.join("; ")
    )));
}

/// What:
///  The words for one stage container another command is filling.
/// Why:
///   The person reading a refusal has to know which container it is about and which process fills it,
/// so a swap refusal and a removal refusal say the same thing about it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// words({ container, owner }); // 'process 1234 is filling the stage container ... below it, ...'
/// ```
fn words(stage: &FilledStage) -> String {
    let filler: String = match stage.owner {
        Some(pid) => format!("process {pid} is"),
        None => String::from("its owner is"),
    };
    return format!("{filler} filling the stage container {} below it, which is another command's copy of a worktree's ignored state", stage.container.display());
}

/// Filled-stage controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_convert_filled_stages_tests.rs"]
mod tests;
