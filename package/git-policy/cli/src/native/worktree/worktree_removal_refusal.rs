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
//!  a nested repository and a nested worktree's untracked and ignored files all go with the worktree,
//!  and each one is written to the removal's log before it goes
//!  (`worktree_removal_deletions.rs`).
//!  The same decision's first sentence is the exception:
//!  "Git work is never destroyed by a removal:
//!  a commit that no reference holds,
//!  and an edit to a tracked file,
//!  however it is hidden".
//!  A worktree registered inside this one usually sits under a name the outer worktree ignores,
//!  so real Git deletes it without asking its own rules,
//!  and an edit there has no other copy:
//!  a worktree inside the target that holds a changed tracked file,
//!  a staged change,
//!  or a commit no reference names is therefore refused too,
//!  with the same escape
//!  (`worktree_nested_git_work.rs`).
//!  The automatic routes go further and leave any worktree that holds a registered worktree.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const why = whyARemovalIsRefused({ context, commonDir, procRoot, target, force }); // string | undefined
//! ```

use crate::discard_audit_facts::AuditFact;
use crate::process_use::PROCESS_SCAN_CEILING;
use crate::transaction_git::GitContext;
use crate::worktree_nested_git_work::{NestedHoldingGitWork, nested_worktrees_holding_git_work};
use crate::worktree_process_inside::{LeftOut, ProcessWorking, process_working_inside_until};
use crate::worktree_removal_second_look::age_of;
use crate::worktree_remove_registry::{GitRemoval, registered_worktree, worktree_git_removes};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::time::Instant;

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
/// type RemovalInQuestion = { context: GitContext; commonDir: string; procRoot: string; operand: string; commandDirectory: string; force: boolean };
/// ```
pub struct RemovalInQuestion<'a> {
    /// How to start real Git.
    pub context: &'a GitContext,
    /// The canonical common Git directory of the repository the command runs in.
    pub common_dir: &'a Path,
    /// The process table to read,
    /// which a control replaces with a fixture listing.
    pub proc_root: &'a Path,
    /// The name the person typed,
    /// which Git resolves its own way.
    pub operand: &'a OsStr,
    /// The directory the command runs in,
    /// which a relative name is read from.
    pub command_directory: &'a Path,
    /// Whether the person forced the removal.
    pub force: bool,
}

/// What:
///  Why this removal may not run,
///  or nothing when nothing here keeps the worktree.
/// Why:
///   A forced removal asks nothing:
///  the person said the worktree goes whatever it holds.
///  Two conditions are asked otherwise,
///  and both come before anything is removed:
///  a process working in the worktree,
///  which is one walk of a table this program already reads elsewhere,
///  and a worktree registered inside it that holds Git work no other copy has,
///  which is two Git reads per nested worktree and none for the ordinary removal.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// whyARemovalIsRefused({ context, commonDir, procRoot, target, force }); // string | undefined
/// ```
pub fn why_a_removal_is_refused(removal: &RemovalInQuestion<'_>) -> Option<String> {
    if removal.force {
        return None;
    }
    let deadline: Instant = Instant::now() + PROCESS_SCAN_CEILING;
    // The owner's decision of 2026-11-14: this command and its parents are not counted, so a removal started from inside
    //  a namespace where other processes cannot be read still runs (`LeftOut`).
    let left_out: LeftOut = LeftOut::of_this_command(removal.proc_root, deadline);
    return match worktree_git_removes(removal.context, removal.common_dir, removal.command_directory, removal.operand) {
        // Git removes nothing for that name, so there is nothing here to ask about and Git's own words are the answer:
        //  a name that is not a worktree this repository registers is not refused by this guard.
        GitRemoval::Nothing => None,
        GitRemoval::One(worktree) => the_first_reason_to_keep(removal, worktree.as_path(), &left_out, deadline),
        // Case alone cannot decide it: Git folds case only when `core.ignorecase` says so, so every worktree the name
        //  could pick is asked about, and one of them holding a process is enough to keep the removal from running.
        GitRemoval::Among(worktrees) => {
            for worktree in worktrees {
                if let Some(why) = the_first_reason_to_keep(removal, worktree.as_path(), &left_out, deadline) {
                    return Some(why);
                }
            }
            None
        }
    };
}

/// What:
///  Why this removal may not take `worktree`,
///  or nothing when nothing here keeps it.
/// Why:
///  The two conditions are asked of one worktree and answered in the order the caller reads them:
///  a process working in it,
///  then a worktree registered inside it that holds Git work no other copy has.
///  The chains read are those of the command, which is why the scan is told what to leave out.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theFirstReasonToKeep(removal, worktree, leftOut, deadline) // string | undefined
/// ```
fn the_first_reason_to_keep(removal: &RemovalInQuestion<'_>, worktree: &Path, left_out: &LeftOut, deadline: Instant) -> Option<String> {
    // A linked worktree's own Git directory lies in the common directory, and a service's marker there is not a
    //  person working in the worktree, so the scan is told which directory not to count (`worktree_process_inside.rs`).
    let admin: PathBuf = registered_worktree(removal.common_dir, worktree)
        .map_or_else(|_unregistered| return worktree.join(".git"), |one| return one.admin);
    if let Some(why) = match process_working_inside_until(worktree, admin.as_path(), removal.proc_root, age_of(worktree), deadline, left_out) {
        ProcessWorking::Nobody => None,
        ProcessWorking::Someone(working) => Some(working_process_keeps_it(worktree, working.why().as_str())),
        ProcessWorking::Unfinished(why) => Some(cannot_tell(why.as_str())),
    } {
        return Some(why);
    }
    return match nested_worktrees_holding_git_work(removal.context, removal.common_dir, worktree) {
        Ok(holding) if holding.is_empty() => None,
        Ok(holding) => Some(a_nested_worktree_holds_git_work(worktree, holding.as_slice())),
        Err(reason) => Some(cannot_tell(format!("nothing shows whether a worktree inside it holds work of yours ({reason})").as_str())),
    };
}

/// What:
///  The refusal of a removal that would take the Git work of a worktree registered inside it.
/// Why:
///  The nested worktree is usually under a name the outer worktree ignores,
///  so Git deletes it without a word,
///  and an edit or a commit there has no other copy.
///  The words name each worktree and what it holds,
///  say the two ways forward that lose nothing,
///  and end with the same escape every refusal of a removal ends with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// aNestedWorktreeHoldsGitWork('/x', [{ worktree: '/x/nested', why: 'changed tracked files (`a.txt`)' }])
/// ```
fn a_nested_worktree_holds_git_work(target: &Path, holding: &[NestedHoldingGitWork]) -> String {
    let mut said: Vec<String> = Vec::with_capacity(holding.len());
    for one in holding {
        said.push(format!("`{}` holds {}", one.worktree.display(), one.why()));
    }
    return format!(
        "monochromatic-git-policy-cli: refusing to remove the worktree {}: another worktree this repository registers is inside it, and it holds work that exists nowhere else: {}. Removing this one would delete those files and that worktree's own record with them. Nothing was removed. Commit or move that work, or remove that worktree first with `git worktree remove {}`, and if it must all go anyway, {ESCAPE_SENTENCE}",
        target.display(),
        said.join("; "),
        holding[0].worktree.display()
    );
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
///  directly after the subcommand word `worktree`.
///  Measured through the built wrapper on 2026-11-14 with a busy worktree,
///  which is the only way to see the difference:
///  after `worktree` it runs, in Git's own global position (`git --no-enforce-worktree worktree remove`)
///  it runs too,
///  and after `remove`, or at the end of the line, it does not:
///  a control there is read as that subcommand's own argument
///  (`wrapper_invocation.rs`, `leading_controls`),
///  and Git would refuse it as an unknown option.
const ESCAPE_SENTENCE: &str = "pass --no-enforce-worktree directly after the word `worktree`, for example `git worktree --no-enforce-worktree remove <path>`, which runs the removal exactly as real Git runs it; further on, after `remove`, the token is read as a path of that subcommand and Git refuses it, and before `worktree`, among Git's own global options, it is read too.";

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
