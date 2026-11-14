//! What:
//!  The Git work a typed `git worktree remove` would destroy,
//!  asked once before the wrapper chooses how to send that command Git's way,
//!  and the words a removal refused for it prints.
//! Why:
//!  The guard of a removal is asked where the removal is made,
//!  and a typed removal reaches that place only by the copy route.
//!  A command that skips the copy (`--no-worktree-copy`),
//!  a form the removal plan does not serve,
//!  or a command that stands outside a worktree is handed to real Git as it stands,
//!  with no code of this program left to ask anything:
//!  Git then deletes a tracked edit a mark hides,
//!  and a commit only the worktree's `HEAD` names,
//!  without a word
//!  (a review of `bab700404d`,
//!  finding 1).
//!  So the questions are asked here,
//!  in the one place every forwarded command passes before a route is chosen,
//!  and a refusal stops the command with the same words the removal's own guard prints
//!  (`git_work_refusal`, below).
//!  Every removal that reaches real Git ends in one of three states,
//!  and there is no fourth:
//!  this program resolved the operand to one worktree Git lists and judged that worktree,
//!  then handed Git that worktree's own path in place of the operand as typed,
//!  so the worktree judged and the worktree removed are one thing by construction;
//!  or the words are a form Git itself answers with its usage, which removes nothing;
//!  or the removal is refused in words naming the operand and the opt-out.
//!  A forced removal and the guard's opt-out are the person's own instruction and ask nothing,
//!  and a removal there still reads the worktree Git names so the ladder acts on that same one.
//!  Standing down on a name this program could not resolve is what let three reviews
//!  (`528279d099`, `9e6c8619a9`, `e615fb599b`)
//!  find a removal Git resolved and this program did not.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const typed = typedRemoval({ command, escapedByThePerson, checks }); // as-written | of-a-judged-worktree | refused
//! const words = gitWorkRefusal(target, reasons); // the report both guards print
//! ```

use crate::forwarded_command::ResolvedCommand;
use crate::policy_checks::ShippedChecks;
use crate::repository_facts::RepositoryFacts;
use crate::transaction_git::GitContext;
use crate::worktree_add_plan::start_directory;
use crate::worktree_identity::{WorktreeIdentity, common_directory};
use crate::worktree_removal_hidden_edits::hidden_edits_beneath;
use crate::worktree_removal_keeps_git_work::git_work_a_removal_would_strand;
use crate::worktree_remove_plan::{RemovalShape, removal_shape};
use crate::worktree_remove_registry::{OperandWorktree, Registration, git_worktree_for_operand, registered_worktree};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The prefix every report of this program carries.
const PREFIX: &str = "monochromatic-git-policy-cli: ";

/// What:
///  How a typed `git worktree remove` goes on:
///  as the person wrote it,
///  naming the one judged worktree this program hands Git in its place,
///  or not at all.
/// Why:
///   The forward of this program runs whatever route it chose with these words,
///  so the worktree this program judged and the worktree real Git removes have to be one path,
///  not two resolutions that ought to agree
///  (a review of `e615fb599b`,
///  finding 1).
///  `AsWritten` carries the worktree Git's own answer names where this program read it,
///  so the ladder that runs before Git destroys that same worktree and no other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TypedRemoval =
///   | { kind: 'as-written'; picked?: string }
///   | { kind: 'of'; operandAt: number; target: string }
///   | { kind: 'refused'; words: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TypedRemoval {
    /// No question is asked of this command:
    /// it is no removal,
    /// Git answers it with its usage,
    /// or the person forced the removal or escaped this guard.
    /// `picked` is the worktree Git's own answer names, when this program could read it.
    AsWritten {
        /// The worktree Git's own answer names for the operand, where it was read.
        picked: Option<PathBuf>,
    },
    /// A removal of one worktree that carries no Git work:
    /// Git is handed that worktree's own path where the operand as typed stands.
    Of {
        /// Where the operand stands among the words Git receives.
        operand_at: usize,
        /// The worktree Git's own answer names, by its own resolved path.
        target: PathBuf,
    },
    /// A removal this program refuses,
    /// with the report the person reads.
    Refused(String),
}

/// What:
///  How a typed removal goes on,
///  judged for the Git work it would destroy.
/// Why:
///   Every route a forwarded command takes passes this question before the route is chosen,
///  so no flag that skips another step can skip this one.
///   The operand is resolved by Git's own answer,
///  and nothing is resolved by a guess:
///  a name this program cannot resolve to a worktree Git lists is refused rather than forwarded,
///  because three reviews found a name Git resolves and this program did not
///  (`528279d099`, `9e6c8619a9`, `e615fb599b`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// typedRemoval({ command, escaped, checks }) // TypedRemoval
/// ```
pub(crate) fn typed_removal<F: RepositoryFacts>(command: &ResolvedCommand, escaped_by_the_person: bool, checks: &mut ShippedChecks<F>) -> TypedRemoval {
    if !names_a_removal(command) {
        return TypedRemoval::AsWritten { picked: None };
    }
    let (operand, operand_at, force): (OsString, usize, bool) = match removal_shape(command.arguments.as_slice(), command.subcommand_index) {
        RemovalShape::Served { operand, operand_at, force } => (operand, operand_at, force),
        // Git answers this form with its usage and removes nothing.
        RemovalShape::NotARemoval => return TypedRemoval::AsWritten { picked: None },
        RemovalShape::Unprovable(reason) => return TypedRemoval::Refused(unprovable_operand_refusal(command, reason.as_str())),
    };
    // A forced removal is Git's own `--force` and deletes the same set, and the opt-out is the person asking for Git's own
    // reading: neither is asked a question here, and neither is refused for what this program cannot read.
    let asked: bool = !force && !escaped_by_the_person;
    let location: crate::repository_location::RepositoryLocation = match checks.facts.location() {
        Ok(found) => found,
        Err(reason) => return unanswered(asked, command, format!("this command's repository cannot be read ({reason})")),
    };
    // Outside a worktree no worktree is removed: Git answers "not a git repository" and nothing is asked of a person.
    if location.identity == WorktreeIdentity::OutsideWorktree {
        return TypedRemoval::AsWritten { picked: None };
    }
    let common_dir: PathBuf = match common_directory(&location.identity) {
        Some(found) => found.to_path_buf(),
        None => return unanswered(asked, command, String::from("this command's common Git directory is not one this program can name")),
    };
    let context: GitContext = checks.facts.transaction_context();
    let start: PathBuf = match start_of_the_command(command) {
        Ok(found) => found,
        Err(reason) => return unanswered(asked, command, reason),
    };
    let fold_case: bool = match case_folds(&context, common_dir.as_path()) {
        Ok(found) => found,
        Err(reason) => return unanswered(asked, command, reason),
    };
    let target: PathBuf = match git_worktree_for_operand(&context, common_dir.as_path(), operand.as_os_str(), start.as_path(), fold_case) {
        Ok(OperandWorktree::Linked(found)) => found,
        Ok(OperandWorktree::Main(found)) => {
            return match asked {
                true => TypedRemoval::Refused(main_worktree_refusal(command, found.as_path())),
                false => TypedRemoval::AsWritten { picked: None },
            };
        }
        Err(reason) => return unanswered(asked, command, reason),
    };
    if !asked {
        return TypedRemoval::AsWritten { picked: Some(target) };
    }
    // The registration this worktree's removal deletes: it names the commit service units of the worktree and the
    // records a destroy proves again. A registration that cannot be read leaves the questions below unanswerable, and
    // a worktree whose own directory is gone has no registration a pointer names, which is a refusal of its own:
    // a commit only that registration's `HEAD` names goes with it.
    let registration: Registration = match registered_worktree(common_dir.as_path(), target.as_path()) {
        Ok(found) => found,
        Err(reason) => {
            return TypedRemoval::Refused(git_work_refusal(
                target.as_path(),
                [format!("nothing here can read its own registration ({reason}), so nothing here proves it names no commit that nothing else names")].as_slice(),
            ));
        }
    };
    let mut reasons: Vec<String> = match git_work_a_removal_would_strand(&context, common_dir.as_path(), target.as_path(), registration.admin.as_path()) {
        Ok(stranded) => stranded,
        Err(reason) => vec![format!("nothing proves no commit of this worktree is named by nothing: {reason}")],
    };
    match hidden_edits_beneath(&context, target.as_path(), registration.admin.as_path(), common_dir.as_path()) {
        Ok(mut edits) => reasons.append(&mut edits),
        Err(reason) => reasons.push(format!("nothing proves no edit there is hidden from Git: {reason}")),
    }
    if reasons.is_empty() {
        return TypedRemoval::Of { operand_at, target };
    }
    return TypedRemoval::Refused(git_work_refusal(target.as_path(), reasons.as_slice()));
}

/// What:
///  What this program does with a question it could not answer:
///  a refusal where the person asked for a guarded removal,
///  and the command as written where the person forced the removal or escaped this guard.
/// Why:
///   A removal that is not asked a question here is the person's own instruction,
///  and the ladder still acts on the worktree Git names where that was read.
///  A removal that is asked must never go on unanswered
///  (`typed_removal`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unanswered({ asked, command, reason }) // TypedRemoval
/// ```
fn unanswered(asked: bool, command: &ResolvedCommand, reason: String) -> TypedRemoval {
    return match asked {
        true => TypedRemoval::Refused(unprovable_operand_refusal(command, reason.as_str())),
        false => TypedRemoval::AsWritten { picked: None },
    };
}

/// What:
///  Whether the resolved command is a `git worktree remove`,
///  read from the words Git will receive.
/// Why:
///   The questions below cost Git starts and a registration read,
///  and every other command must pay none of that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesARemoval(command) // boolean
/// ```
fn names_a_removal(command: &ResolvedCommand) -> bool {
    let word = |at: usize| -> Option<&[u8]> { return command.arguments.get(at).map(|one| return one.as_encoded_bytes()) };
    return word(command.subcommand_index) == Some(b"worktree") && word(command.subcommand_index + 1) == Some(b"remove");
}

/// What:
///  The directory the removal's operand is relative to,
///  or the reason it cannot be read.
/// Why:
///   Git starts in the directory the global `-C` options name,
///  and the operand of a removal that is not absolute is read from there
///  (`prefix_filename` in `abspath.c`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// startOfTheCommand(command) // string; throws the reason
/// ```
fn start_of_the_command(command: &ResolvedCommand) -> Result<PathBuf, String> {
    let current: PathBuf = std::env::current_dir().map_err(|error| return format!("the directory this command runs in cannot be read ({error})"))?;
    return match start_directory(&command.arguments[..command.subcommand_index], current.as_path()) {
        Some(found) => Ok(found),
        None => Err(String::from("the `-C` options this command carries name a directory this program cannot read")),
    };
}

/// What:
///  Whether Git folds ASCII case when it compares paths in this repository and this invocation,
///  or the reason that cannot be read.
/// Why:
///  Git's `fspathcmp` is `strcasecmp` exactly when `core.ignorecase` is set
///  (`dir.c`, `environment.c` in Git 2.56.0),
///  and that setting may come from any configuration scope the invocation reads,
///  or from a `-c` the person typed,
///  so the answer is Git's own rather than a file this program parses:
///  a wrong answer here names no worktree where Git names one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// caseFolds(context, commonDir) // boolean; throws the reason
/// ```
fn case_folds(context: &GitContext, common_dir: &Path) -> Result<bool, String> {
    let arguments: Vec<OsString> = ["config", "--bool", "core.ignorecase"].map(OsString::from).to_vec();
    let request: crate::transaction_git::GitRequest = crate::transaction_git::GitRequest::new(common_dir, arguments.as_slice());
    let output = match crate::transaction_git::run_git(context, &request) {
        Ok(finished) => finished,
        Err(error) => return Err(format!("Git cannot say whether this repository folds case in paths ({error})")),
    };
    return match output.code {
        // The key is set: Git's own reading of it is true or false.
        Some(0) => match String::from_utf8_lossy(output.stdout.as_slice()).trim() {
            "true" => Ok(true),
            "false" => Ok(false),
            other => Err(format!("Git answered `{other}` for whether this repository folds case in paths")),
        },
        // The key is unset, which is Git's own default of no folding.
        Some(1) => Ok(false),
        _ => Err(format!(
            "Git cannot say whether this repository folds case in paths ({})",
            String::from_utf8_lossy(output.stderr.as_slice()).trim()
        )),
    };
}

/// What:
///  The report of a removal refused because the name it carries could not be told from the worktree Git would pick.
/// Why:
///   Git resolves the name itself,
///  and this program must not stand down on a name it cannot resolve:
///  standing down is what let a case-differing name remove a worktree unguarded
///  (a review of `528279d099`,
///  finding 1),
///  and what the reviews of `9e6c8619a9` and of `e615fb599b` found again.
///  The way out is the same opt-out every refusal of this guard names,
///  and the operand as the person typed it is the one Git resolves,
///  so the printed command is the command Git runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unprovableOperandRefusal(command, reason) // the report
/// ```
fn unprovable_operand_refusal(command: &ResolvedCommand, reason: &str) -> String {
    let operand: String = command.arguments[command.subcommand_index + 2..]
        .iter()
        .rfind(|word| return word.as_encoded_bytes() != b"--")
        .map(|word| return word.to_string_lossy().into_owned())
        .unwrap_or_default();
    let quoted: String = crate::shell_quote::shell_quote(operand.as_bytes());
    return format!(
        "{PREFIX}nothing here proves which worktree the name {quoted} a removal names, so nothing was removed: {reason}. If that worktree must go anyway, `git worktree --no-enforce-worktree remove {quoted}` runs as real Git runs it and deletes all of it."
    );
}

/// What:
///  The report of a removal refused because its name is the repository's main working tree.
/// Why:
///   A removal takes only a linked worktree of the repository,
///  and the person who typed the main checkout's own name meant something else.
///  Nothing there is removed,
///  and the sentence says so rather than leaving a Git message about a worktree the person did not mean.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// mainWorktreeRefusal(command, main) // the report
/// ```
fn main_worktree_refusal(command: &ResolvedCommand, main: &Path) -> String {
    let operand: String = command.arguments[command.subcommand_index + 2..]
        .iter()
        .rfind(|word| return word.as_encoded_bytes() != b"--")
        .map(|word| return word.to_string_lossy().into_owned())
        .unwrap_or_default();
    let quoted: String = crate::shell_quote::shell_quote(operand.as_bytes());
    return format!(
        "{PREFIX}the name {quoted} names the main working tree {}, and a removal takes only a linked worktree, so nothing was removed. Name the linked worktree's own directory, as `git worktree list` prints it.",
        main.display()
    );
}

/// What:
///  The report of a removal refused because it would destroy Git work.
/// Why:
///   Two guards print it:
///  the one where the removal is made,
///  and the one a typed command meets before any route,
///  so one function words both.
///  The path is quoted,
///  because the report hands the person a command to copy and run,
///  and a path with a space,
///  a quote,
///  a dollar sign or a line break otherwise runs as something else
///  (a review of `bab700404d`,
///  finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitWorkRefusal(target, reasons) // the report
/// ```
pub(crate) fn git_work_refusal(target: &Path, reasons: &[String]) -> String {
    let quoted: String = crate::shell_quote::shell_quote(target.as_os_str().as_encoded_bytes());
    return format!(
        "{PREFIX}{} keeps Git work that a removal would destroy, or that nothing here proves it would not, so nothing was removed: {}. If this worktree must go anyway, `git worktree --no-enforce-worktree remove {quoted}` runs as real Git runs it and deletes all of it.",
        target.display(),
        reasons.join(". ")
    );
}

/// The controls of this question stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_typed_git_work_tests.rs"]
mod tests;
