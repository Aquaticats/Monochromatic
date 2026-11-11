//! What:
//!  The words one landed sweep prints,
//!  one line per worktree it removed,
//!  left,
//!  or could not prove it made,
//!  and one for each commit a removal would have deleted.
//! Why:
//!   A command that removed a person's worktree by itself must say so,
//!  and one that left a landed worktree must say what kept it:
//!  the space a snapshot holds is invisible otherwise,
//!  and a person who expected a removed worktree needs to know it is still there and why.
//!  A commit held behind a reference lock lives in the worktree's own Git directory,
//!  so a removal deletes the only thing that names it;
//!  the line about it comes before the removal's line and names where this repository keeps it.
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
use crate::worktree_landed_removal::{HandedOverWorktree, LandedSweep};
use std::path::Path;

/// The prefix every line of this program carries.
const PREFIX: &str = "monochromatic-git-policy-cli: ";

/// The command that removes a worktree a person read and decided to remove.
fn removal_command(worktree: &Path) -> String {
    let quoted: String = shell_quote(worktree.as_os_str().as_encoded_bytes().as_ref());
    return format!("git worktree remove {quoted}");
}

/// What:
///  Where the wrapper keeps what became of a handed-over removal,
///  as the person reads it.
/// Why:
///  The record moves from `pending` to `done` or `failed` under the work directory named here,
///  and a person whose worktree is still there needs a place to look that is not this program's own words.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const recordPlace = (record) => `${workDir(record.gitDir)}/${record.id}`;
/// ```
fn record_place(one: &HandedOverWorktree) -> String {
    let work: std::path::PathBuf = crate::service_work::work_directory(one.record.git_dir.as_path());
    return work.join(one.record.id()).display().to_string();
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
        // A commit that was kept is said even when another one stopped the sweep, so the person reading where one went is not
        //  left believing the removal kept nothing of any of them.
        for kept in &sweep.kept_commits {
            lines.push(format!("{PREFIX}{kept}"));
        }
        lines.push(format!("{PREFIX}no landed worktree was looked at after this merge, because {reason}."));
        if sweep.refused_commits {
            // A later merge is the one retry a person would try, and it is up to date: saying so keeps the silence that follows
            //  from reading as a removal that happened.
            lines.push(format!(
                "{PREFIX}running this merge again removes nothing: a branch that already landed makes the merge up to date, so no \
                 landed worktree is looked at. The commands above are the ones that finish it."
            ));
        }
        return lines;
    }
    // A held commit is said before the removal it would not survive, so a person reading the same output knows where it went.
    for kept in &sweep.kept_commits {
        lines.push(format!("{PREFIX}{kept}"));
    }
    for problem in &sweep.rescue_problems {
        lines.push(format!("{PREFIX}{problem}"));
    }
    for removed in &sweep.removed {
        lines.push(format!(
            "{PREFIX}removed the worktree {} by itself, because its branch {} is now on {}; the branch stays, and the ignored files there went with it because the worktree this merge ran in holds every one of them, which is what frees the space its snapshot held. A directory Git ignores went with it unread, so anything you kept inside one went with the worktree too; reading every file below such a directory would cost the whole tree on every merge.",
            removed.worktree.display(),
            removed.branch,
            sweep.landed_branch
        ));
    }
    // A removal that is Git's own reading of every file there is recorded for the commit service before this line is printed:
    //  the merge waits for none of it, so the person reads where its outcome is and what stands until then.
    for handed in &sweep.handed_over {
        lines.push(format!(
            "{PREFIX}handed the removal of the worktree {} to the commit service, though its branch {} is now on {}: removing it is Git's own reading of every tracked file there, and no merge waits for that. The branch stays. What became of it is the record {}: `done` there means the worktree is gone, and `failed`, with Git's own sentence in the service's journal, means Git kept it. If nothing there is yours and you want it gone now, `{}` removes it.",
            handed.worktree.display(),
            handed.branch,
            sweep.landed_branch,
            record_place(handed),
            removal_command(handed.worktree.as_path())
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
