//! What:
//!  Why a plain `git worktree remove` may not run now,
//!  in words a person reads and acts on.
//! Why:
//!  Real Git removes a worktree without looking at what is working in it:
//!  a process whose working folder is there keeps running with a folder that is gone,
//!  and the program it was running fails the moment it opens a file.
//!  The owner's decision of 2026-11-12
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  "Worktree removal and ignored files") is "Delete ignored, spare busy":
//!  the files Git ignores are deleted as real Git deletes them and recorded,
//!  and a worktree something is working in is refused,
//!  naming the process,
//!  with the policy's escape.
//!  The refusal is the escape's other half:
//!  a person who means it passes `--no-enforce-worktree` and the removal runs exactly as real Git runs it,
//!  so the wrapper never stands between a person and their own decision.
//!  A forced removal is that decision already:
//!  `--force` says the worktree goes whatever it holds,
//!  and Git deletes it that way,
//!  so nothing here refuses it.
//!  Nothing here reads a command line or an environment:
//!  a process is named by its number and the kernel's name for the program it runs.
//!  A question whose answer could not be read refuses:
//!  a scan of the process table that stopped at its bound,
//!  and a registration list that could not be read,
//!  both leave the worktree where it is,
//!  because neither proves no process is working inside it.
//!  What else a removal deletes is the owner's decision of 2026-11-12T13:04:14
//!  (`doc/handover/cli-git-rust-implementation.md`,
//!  "How far the protection of a worktree being removed reaches"): the files Git ignores,
//!  a nested repository and a nested worktree's own files all go with the worktree,
//!  and each one is written to the removal's log before it goes
//!  (`worktree_removal_deletions.rs`),
//!  never refused.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const why = whyARemovalIsRefused({ context, commonDir, procRoot, target, force }); // string | undefined
//! ```

use crate::discard_audit_facts::AuditFact;
use crate::transaction_git::GitContext;
use crate::worktree_landed_registrations::{Registration, registrations_of};
use crate::worktree_process_inside::{ProcessWorking, process_working_inside};
use crate::worktree_removal_second_look::age_of;
use std::path::{Path, PathBuf};

/// The finding code of a removal this program refuses.
/// Why:
///   A caller identifies the finding by this stable code,
///  and it is the removal's own rather than the linked-worktree policy's,
///  which refuses a different thing in its own words.
pub const WORKTREE_REMOVAL_REFUSED_CODE: &str = "worktree-removal-refused";

/// The policy escape a person passes to run the removal as real Git runs it.
/// Why:
///   `git worktree remove` is the linked-worktree policy's recorded form,
///  so its escape is the one that lets the removal through,
///  and the words that name it say where on the command line it goes.
pub const REMOVAL_ESCAPE: &str = "--no-enforce-worktree";

/// What:
///  One removal about to run and where its questions are answered.
/// Why:
///   Every field is a fact the removal's own path already holds,
///  so the guard changes nothing about how the removal was named.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type RemovalInQuestion = { context: GitContext; commonDir: string; procRoot: string; target: string; force: boolean };
/// ```
pub struct RemovalInQuestion<'a> {
    /// How to start real Git.
    pub context: &'a GitContext,
    /// The canonical common Git directory of the repository the command runs in.
    pub common_dir: &'a Path,
    /// The process table to read,
    /// which a control replaces with a fixture listing.
    pub proc_root: &'a Path,
    /// The worktree the command names,
    /// as the repository resolves that name.
    pub target: &'a Path,
    /// Whether the person forced the removal.
    pub force: bool,
}

/// What:
///  Why this removal may not run,
///  or nothing when nothing here keeps the worktree.
/// Why:
///   A forced removal asks nothing:
///  the person said the worktree goes whatever it holds.
///  The one condition is the process scan,
///  which is one walk of a table this program already reads elsewhere,
///  and which a worktree a person works in answers at once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whyARemovalIsRefused({ context, commonDir, procRoot, target, force }); // string | undefined
/// ```
pub fn why_a_removal_is_refused(removal: &RemovalInQuestion<'_>) -> Option<String> {
    if removal.force {
        return None;
    }
    let registered: Vec<Registration> = match registrations_of(removal.common_dir) {
        Ok(found) => found,
        Err(reason) => return Some(cannot_tell(format!("the worktrees this repository registers could not be read ({reason})").as_str())),
    };
    // A linked worktree's own Git directory lies in the common directory, and a service's marker there is not a
    //  person working in the worktree, so the scan is told which directory not to count (`worktree_process_inside.rs`).
    let admin: PathBuf = registered
        .iter()
        .find(|one| return one.worktree == removal.target)
        .map_or_else(|| return removal.target.join(".git"), |one| return one.admin.clone());
    return match process_working_inside(removal.target, admin.as_path(), removal.proc_root, age_of(removal.target)) {
        ProcessWorking::Nobody => None,
        ProcessWorking::Someone(working) => Some(working_process_keeps_it(removal.target, working.why().as_str())),
        ProcessWorking::Unfinished(why) => Some(cannot_tell(why.as_str())),
    };
}

/// What:
///  The refusal of a removal a process is working in.
/// Why:
///   A person has to know which program to stop,
///  so the words name its number and its program,
///  say what the removal would do to it,
///  and name the one thing that clears the way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// workingProcessKeepsIt('/x', 'process 123 (`sleep`) is working in it')
/// ```
fn working_process_keeps_it(target: &Path, why: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: refusing to remove the worktree {}: {why}, so the removal would delete the folder that program is working in and the program would fail. Nothing was removed. Stop that program and run the command again, and if the folder must go with the program still running, {ESCAPE_SENTENCE}",
        target.display()
    );
}

/// What:
///  The refusal of a removal nothing could be proved safe about.
/// Why:
///  A bound that ended the process scan and a registration list that could not be read leave the same state:
///  nothing proves no process is working in the worktree,
///  and the words say which question went unanswered.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cannotTell('the process table ... stopped at its bound ...')
/// ```
fn cannot_tell(question: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: refusing to remove the worktree, because {question}, so nothing proves it is not working in it. Nothing was removed. Run the command again when that answer can be read, and if you have decided the worktree goes anyway, {ESCAPE_SENTENCE}"
    );
}

/// The sentence that names the escape and what it does,
/// which every refusal of a removal ends with.
/// Why:
///   One wording keeps both refusals saying the same thing about the same flag,
///  and the example puts it where this wrapper reads it,
///  directly after the subcommand.
const ESCAPE_SENTENCE: &str = "pass --no-enforce-worktree directly after the subcommand, for example `git worktree remove --no-enforce-worktree <path>`, which runs the removal exactly as real Git runs it.";

/// What:
///  The facts one refused removal's record carries.
/// Why:
///  A refusal is a decision about the person's work,
///  so the record names the worktree it was about,
///  whether the person forced it,
///  and the words of the refusal,
///  which say whether a process or an unread answer kept it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removalRefusalFacts({ target, force, why }) // [{name:'worktree',...}, {name:'force',...}, {name:'refused',...}]
/// ```
pub fn removal_refusal_facts(target: &Path, force: bool, why: &str) -> Vec<AuditFact> {
    return vec![
        AuditFact::Text("worktree", target.to_string_lossy().into_owned()),
        AuditFact::Flag("force", force),
        AuditFact::Text("refused", why.to_owned()),
    ];
}

/// Controls of the refusal stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_refusal_tests.rs"]
mod tests;
