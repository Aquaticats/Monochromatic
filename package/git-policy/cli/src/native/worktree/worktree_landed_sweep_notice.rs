//! What:
//!  The words one landed sweep prints,
//!  one line per worktree it removed,
//!  left,
//!  or could not prove it made.
//! Why:
//!   A command that removed a person's worktree by itself must say so,
//!  and one that left a landed worktree must say what kept it:
//!  the space a snapshot holds is invisible otherwise,
//!  and a person who expected a removed worktree needs to know it is still there and why.
//!  Every line names the worktree,
//!  its branch,
//!  the branch that received the landed commits,
//!  and the one command that removes what the wrapper left.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const line of landedSweepLines(sweep)) process.stderr.write(line + '\n');
//! ```

use crate::shell_quote::shell_quote;
use crate::worktree_landed_removal::LandedSweep;
use std::path::Path;

/// The prefix every line of this program carries.
const PREFIX: &str = "monochromatic-git-policy-cli: ";

/// The command that removes a worktree a person read and decided to remove.
fn removal_command(worktree: &Path) -> String {
    let quoted: String = shell_quote(worktree.as_os_str().as_encoded_bytes().as_ref());
    return format!("git worktree remove {quoted}");
}

/// What:
///  The lines one sweep prints on standard error,
///  each without its newline.
/// Why:
///   A caller writes them once and a control reads them as text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// landedSweepLines(sweep) // string[]
/// ```
pub fn landed_sweep_lines(sweep: &LandedSweep) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    if let Some(reason) = sweep.failed.as_ref() {
        lines.push(format!("{PREFIX}no landed worktree was looked at after this merge, because {reason}."));
        return lines;
    }
    for removed in &sweep.removed {
        lines.push(format!(
            "{PREFIX}removed the worktree {} by itself, because its branch {} is now on {}; the branch stays, and the ignored files there went with it because the worktree this merge ran in holds every one of them, which is what frees the space its snapshot held.",
            removed.worktree.display(),
            removed.branch,
            sweep.landed_branch
        ));
    }
    for kept in &sweep.kept {
        lines.push(format!(
            "{PREFIX}left the worktree {} in place, though its branch {} is now on {}: {}. Remove it with `{}` when nothing there is yours.",
            kept.worktree.display(),
            kept.branch,
            sweep.landed_branch,
            kept.why,
            removal_command(kept.worktree.as_path())
        ));
    }
    for unproven in &sweep.unproven {
        lines.push(format!(
            "{PREFIX}left the worktree {} in place, though its branch {} is now on {}, because nothing records that this program made it and the files Git ignores in it may be yours. Read what is there, then remove it with `{}`.",
            unproven.worktree.display(),
            unproven.branch,
            sweep.landed_branch,
            removal_command(unproven.worktree.as_path())
        ));
    }
    return lines;
}

/// Notice controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_landed_sweep_notice_tests.rs"]
mod tests;
