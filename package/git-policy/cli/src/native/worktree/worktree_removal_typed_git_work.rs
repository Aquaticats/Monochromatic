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
//! ## Which command is this, and where does it run?
//!
//!  The guard judges commands Git runs itself:
//!  a removal typed as the built-in,
//!  a plain alias (a word list Git expands),
//!  and an alias of such an alias.
//!  The words judged are the words Git runs, and the directory is the one the caller's `-C` chain ends in.
//!  A `!` alias is **forwarded as it is** and judged by nothing here:
//!  the owner's decision of 2026-11-15, "Forward them unguarded",
//!  keeps every alias working with no opt-out,
//!  and a removal a shell alias runs goes with the edit a mark hides and the commit only its `HEAD` names
//!  (`SPEC.md` states that hole where a person meets the guard).
//!  A text a shell runs cannot be read to the end, and nothing of this program stands inside Git's shell:
//!  it never starts itself as a shim and never writes shell text
//!  (`doc/planning/cli-git-no-self-invocation.md`).
//!  Every Git this module starts runs in a directory it already reached and carries the caller's options that change
//!  what Git answers (`answers_with`: `-c` and `--config-env`) and never one that moves it.
//!
//! ## What does this removal take?
//!
//!  A removal takes the worktree it names and every worktree this repository registers inside it,
//!  with their checkouts and their own Git directories,
//!  so the list is made once (`worktree_removal_worktrees_it_takes`) and every question of every route goes over it:
//!  the commits a removal would leave named by nothing,
//!  the edits a mark hides from Git,
//!  and here as well as in `worktree_remove.rs` and in the last look before a destroy.
//!  A question asked of the outer directory alone is the defect whichever question it is:
//!  a hidden edit in a worktree registered inside the target went with the outer one while the same edit in the outer
//!  directory was refused
//!  (that review, finding 2).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const typed = typedRemoval({ command, escapedByThePerson, checks }); // as-written | of-a-judged-worktree | refused
//! const words = gitWorkRefusal(target, reasons); // the report both guards print
//! ```

use crate::forwarded_command::{ResolvedCommand, Route};
use crate::policy_checks::ShippedChecks;
use crate::repository_facts::RepositoryFacts;
use crate::transaction_git::GitContext;
use crate::worktree_identity::{WorktreeIdentity, common_directory};
use crate::worktree_removal_hidden_edits::hidden_edits_beneath;
use crate::worktree_removal_keeps_git_work::git_work_a_removal_would_strand;
use crate::worktree_removal_worktrees_it_takes::{Taken, the_worktrees_it_takes};
use crate::worktree_remove_plan::{RemovalShape, removal_shape};
use crate::worktree_remove_registry::{OperandUnresolved, OperandWorktree, Registration, git_worktree_for_operand, registered_worktree};
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
    /// Git is handed that worktree's own path where the operand as typed stands,
    /// when the words judged are the words this program forwards.
    Of {
        /// Where the operand stands among the words forwarded,
        /// and nothing when the words judged are those of a `!` alias's shell, which is handed to Git untouched.
        operand_at: Option<usize>,
        /// The worktree Git's own answer names, by its own resolved path.
        target: PathBuf,
    },
    /// A removal this program refuses,
    /// with the report the person reads.
    Refused(String),
}

/// What:
///  The removal this guard judges:
///  the words Git will run,
///  where the subcommand stands among them,
///  the directory they run in,
///  and the caller's own options that change what Git answers.
/// Why:
///  Two questions come before any read,
///  and both are answered here once:
///  which command this is,
///  which after an alias is the words Git runs and not the word the person typed;
///  and where it runs,
///  which is the directory the caller's `-C` chain ends in,
///  or the directory Git starts the shell of a `!` alias in.
///  A guard that read the typed words forwarded an alias straight past itself,
///  and one that applied the `-C` chain twice refused a removal real Git makes
///  (a review of `8858f843ee`, findings 1 and 4).
///  `answers_with` is the caller's options that change what Git answers, which every Git this
///  guard starts carries, and never the ones that move it, which would move those starts a second time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Judged = { words: string[]; subcommandIndex: number; start: string; answersWith: string[] };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
struct Judged {
    /// The words Git will run,
    /// global options included.
    words: Vec<OsString>,
    /// Where the subcommand stands among them.
    subcommand_index: usize,
    /// The directory the removal runs in.
    start: PathBuf,
    /// The caller's `-c` and `--config-env` options,
    /// which change what Git answers without moving it.
    answers_with: Vec<OsString>,
}

/// What:
///  The removal this command is,
///  or nothing when these words are no `worktree remove` this guard can read,
///  or the reason the words or the directory cannot be read.
/// Why:
///   The one place that answers which command the guard decides about and where it runs.
///  A plain alias expands into `arguments`;
///  a `!` alias is run by Git's shell, and the one plain `git` command this crate's resolver read from it
///  (`ResolvedCommand::shell_command`) is the command that shell runs,
///  so that vector is judged instead and its subcommand stands at the front.
///  A shell alias whose command is not one plain `git` word is left to Git as it is:
///  this crate reads no words for it, and `SPEC.md` says only how such a shell is started,
///  not how a guard should read it (see the report of the run that answered this review).
///  The subcommand of the judged words is looked up by the same layout every other reader uses,
///  so a global option before it is skipped and its value is not read as the word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theRemovalItJudges(command, context) // Judged | undefined; throws the reason
/// ```
fn the_removal_it_judges(command: &ResolvedCommand, _context: &GitContext) -> Result<Option<Judged>, String> {
    if command.route == Route::ShellAlias {
        // The owner's decision of 2026-11-15 ("Owner decisions in force", "Answers on waking, second round"):
        // "Forward them unguarded".
        // Every `!` alias is passed to real Git as it is, and every alias keeps working with no opt-out.
        // A `git <alias>` that ends in a worktree removal is not guarded, and `SPEC.md` states that hole.
        // A reader of a shell's text cannot follow it to the end, and the wrapper never starts itself as a shim and never
        // writes shell text (`doc/planning/cli-git-no-self-invocation.md`), so nothing of this program stands inside
        // that shell to judge the removal it runs.
        return Ok(None);
    }
    let layout: crate::global_arguments::GlobalLayout = crate::global_arguments::global_layout(command.arguments.as_slice());
    if layout.prefix_len != command.subcommand_index {
        return Err(String::from("the global options of this command and the subcommand it names disagree about where the subcommand stands"));
    }
    let current: PathBuf = std::env::current_dir().map_err(|error| return format!("the directory this command runs in cannot be read ({error})"))?;
    // The directory the caller's `-C` chain ends in, read by the crate's one answer to that question:
    // every `-C` applied in turn, and the value of another option never read as one.
    let start: PathBuf = crate::external_git_command::effective_directory(current.as_path(), &command.arguments[..command.subcommand_index]);
    return Ok(Some(Judged {
        answers_with: answers_with(&command.arguments[..command.subcommand_index]),
        words: command.arguments.clone(),
        subcommand_index: command.subcommand_index,
        start,
    }));
}

/// What:
///  The caller's options that change what Git answers,
///  and no option that moves it.
/// Why:
///   A Git this guard starts runs in a directory the guard already reached,
///  so carrying `-C` would move it a second time
///  (`without_directory_changes` says the same for the wrapper's own starts),
///  and `--git-dir` or `--work-tree` would name another repository than the one whose common directory this is.
///  Configuration is what the guard's answers turn on:
///  `-c core.ignorecase=...` decides the comparison the whole resolution rests on,
///  so the `-c` and `--config-env` options stay, in the order they were written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// answersWith(globals) // ['-c', 'core.ignorecase=true', ...]
/// ```
fn answers_with(globals: &[OsString]) -> Vec<OsString> {
    let mut kept: Vec<OsString> = Vec::new();
    let mut index: usize = 0;
    while index < globals.len() {
        let argument: &[u8] = globals[index].as_encoded_bytes();
        if argument == b"-c" || argument == b"--config-env" {
            kept.push(globals[index].clone());
            if let Some(value) = globals.get(index + 1) {
                kept.push(value.clone());
            }
            index += 2;
            continue;
        }
        if argument.starts_with(b"--config-env=") {
            kept.push(globals[index].clone());
        }
        index += 1;
    }
    return kept;
}

/// What:
///  How a typed removal goes on,
///  judged for the Git work it would destroy.
/// Why:
///   Every route a forwarded command takes passes this question before the route is chosen,
///  so no flag that skips another step can skip this one.
///   The operand is resolved by Git's own answer,
///  and nothing is resolved by a guess:
///  a name Git's own resolution names no worktree for is Git's to answer,
///  because Git refuses it and removes nothing,
///  while a name two worktrees resolve to is refused,
///  because Git removes the first of them and this program cannot say which that is
///  (three reviews found a name Git resolves and this program did not,
///  `528279d099`, `9e6c8619a9` and `e615fb599b`).
///  The words judged are the words Git runs
///  (`the_removal_it_judges`),
///  and the questions below are asked of every worktree the removal takes,
///  from the one list of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// typedRemoval({ command, escaped, checks }) // TypedRemoval
/// ```
pub(crate) fn typed_removal<F: RepositoryFacts>(command: &ResolvedCommand, escaped_by_the_person: bool, checks: &mut ShippedChecks<F>) -> TypedRemoval {
    let context: GitContext = checks.facts.transaction_context();
    let judged: Judged = match the_removal_it_judges(command, &context) {
        Ok(Some(found)) => found,
        // No words to read: not this guard's command, and Git runs it as the person wrote it.
        Ok(None) => return TypedRemoval::AsWritten { picked: None },
        // An alias's own command the wrapper cannot read gets its own sentence: its way out is the opt-out before the
        // alias, since the alias carries no `worktree` word for the usual one to stand after.
        // The guard's opt-out is the person asking for Git's own reading, so nothing here is refused over a command this
        // program cannot read: Git runs it, and the removal is the person's own instruction.
        Err(_reason) if escaped_by_the_person => return TypedRemoval::AsWritten { picked: None },
        // The words or the directory of a command this program judges cannot be read: that is a reason, and the shell
        // route never reaches here, because a `!` alias is forwarded as it is.
        Err(reason) => return TypedRemoval::Refused(unprovable_removal_refusal(command, reason.as_str())),
    };
    if !names_a_removal(judged.words.as_slice(), judged.subcommand_index) {
        return TypedRemoval::AsWritten { picked: None };
    }
    let (operand, operand_at, force): (OsString, usize, bool) = match removal_shape(judged.words.as_slice(), judged.subcommand_index) {
        RemovalShape::Served { operand, operand_at, force } => (operand, operand_at, force),
        // Git answers this form with its usage and removes nothing.
        RemovalShape::NotARemoval => return TypedRemoval::AsWritten { picked: None },
        // An option this program does not read, with the person's own opt-out, is Git's answer to give.
        RemovalShape::Unprovable(_) if escaped_by_the_person => return TypedRemoval::AsWritten { picked: None },
        RemovalShape::Unprovable(reason) => return TypedRemoval::Refused(unprovable_operand_refusal(judged.words.as_slice(), judged.subcommand_index, reason.as_str())),
    };
    // A forced removal is Git's own `--force` and deletes the same set, and the opt-out is the person asking for Git's own
    // reading: neither is asked a question here, and neither is refused for what this program cannot read.
    let asked: bool = !force && !escaped_by_the_person;
    let location: crate::repository_location::RepositoryLocation = match checks.facts.location() {
        Ok(found) => found,
        Err(reason) => return unanswered(asked, command, judged.words.as_slice(), judged.subcommand_index, format!("this command's repository cannot be read ({reason})")),
    };
    // Outside a worktree no worktree is removed: Git answers "not a git repository" and nothing is asked of a person.
    if location.identity == WorktreeIdentity::OutsideWorktree {
        return TypedRemoval::AsWritten { picked: None };
    }
    let common_dir: PathBuf = match common_directory(&location.identity) {
        Some(found) => found.to_path_buf(),
        None => return unanswered(asked, command, judged.words.as_slice(), judged.subcommand_index, String::from("this command's common Git directory is not one this program can name")),
    };
    let fold_case: bool = match case_folds(&context, common_dir.as_path(), judged.answers_with.as_slice()) {
        Ok(found) => found,
        Err(reason) => return unanswered(asked, command, judged.words.as_slice(), judged.subcommand_index, reason),
    };
    let target: PathBuf = match git_worktree_for_operand(&context, common_dir.as_path(), operand.as_os_str(), judged.start.as_path(), fold_case, judged.answers_with.as_slice()) {
        Ok(OperandWorktree::Linked(found)) => found,
        Ok(OperandWorktree::Main(found)) => {
            return match asked {
                true => TypedRemoval::Refused(main_worktree_refusal(judged.words.as_slice(), judged.subcommand_index, found.as_path())),
                false => TypedRemoval::AsWritten { picked: None },
            };
        }
        // Git's own resolution names no worktree for this name, so Git refuses the removal and removes nothing: there
        // is no worktree for a guard to judge here, and the name is left to Git as the person wrote it. Measured on
        // the builds of `main` and of this branch: a name that differs from a worktree's only by case where the
        // repository folds no case, a name two registrations match, a worktree of another repository, and an empty name
        // all end in Git's own `'<name>' is not a working tree` and take no file with them.
        Err(OperandUnresolved::NoWorktree(reason)) => {
            // The decision this stands on is measured, so it is logged with the reason Git's own resolution gave.
            crate::diagnostic_log::debug("typedRemoval", format!("the removal is left to Git: {reason}").as_str());
            return TypedRemoval::AsWritten { picked: None };
        }
        // A name two worktrees resolve to is a removal Git does make, of whichever its list puts first, and nothing
        // here can name that worktree: refused rather than handed to Git, because standing down here let a removal Git
        // resolved go with the edit a mark hid (a review of `528279d099`, finding 1).
        Err(OperandUnresolved::Ambiguous(reason) | OperandUnresolved::Unreadable(reason)) => {
            return unanswered(asked, command, judged.words.as_slice(), judged.subcommand_index, reason);
        }
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
    // One list of what this removal takes, and every question goes over it.
    let taken: Vec<Taken> = match the_worktrees_it_takes(common_dir.as_path(), target.as_path(), registration.admin.as_path()) {
        Ok(found) => found,
        Err(reason) => return TypedRemoval::Refused(git_work_refusal(target.as_path(), [format!("nothing proves which worktrees this removal takes, so nothing is removed: {reason}")].as_slice())),
    };
    let mut reasons: Vec<String> = match git_work_a_removal_would_strand(&context, common_dir.as_path(), taken.as_slice()) {
        Ok(stranded) => stranded,
        Err(reason) => vec![format!("nothing proves no commit of this worktree is named by nothing: {reason}")],
    };
    match hidden_edits_beneath(&context, taken.as_slice(), common_dir.as_path()) {
        Ok(mut edits) => reasons.append(&mut edits),
        Err(reason) => reasons.push(format!("nothing proves no edit there is hidden from Git: {reason}")),
    }
    if reasons.is_empty() {
        // The operand is written in the words the wrapper forwards only when those are the words judged: a `!` alias is
        // handed to Git as the person typed it, and the shell runs the words judged.
        let forwarded_at: Option<usize> = match command.route {
            Route::ShellAlias => None,
            Route::Direct | Route::Alias | Route::Unresolved => Some(operand_at),
        };
        return TypedRemoval::Of { operand_at: forwarded_at, target };
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
/// unanswered({ asked, command, words, index, reason }) // TypedRemoval
/// ```
fn unanswered(asked: bool, command: &ResolvedCommand, words: &[OsString], subcommand_index: usize, reason: String) -> TypedRemoval {
    let _ = command;
    return match asked {
        true => TypedRemoval::Refused(unprovable_operand_refusal(words, subcommand_index, reason.as_str())),
        false => TypedRemoval::AsWritten { picked: None },
    };
}

/// What:
///  Whether the words Git will run are a `git worktree remove`.
/// Why:
///   The words are the ones Git runs, not the ones the person typed:
///  a `!` alias carrying a removal reaches Git through its shell, and the guard decides about the command that shell
///  runs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesARemoval(words, index) // boolean
/// ```
fn names_a_removal(words: &[OsString], subcommand_index: usize) -> bool {
    let word = |at: usize| -> Option<&[u8]> { return words.get(at).map(|one| return one.as_encoded_bytes()) };
    return word(subcommand_index) == Some(b"worktree") && word(subcommand_index + 1) == Some(b"remove");
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
///  It starts in the common Git directory, which is already the directory the caller's `-C` chain reaches,
///  and carries the caller's configuration options and no option that moves it:
///  forwarding a relative `-C` to this start applied it a second time, and the start failed
///  (a review of `8858f843ee`, finding 4).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// caseFolds(context, commonDir) // boolean; throws the reason
/// ```
fn case_folds(context: &GitContext, common_dir: &Path, answers: &[OsString]) -> Result<bool, String> {
    let mut arguments: Vec<OsString> = answers.to_vec();
    arguments.extend(["config", "--bool", "core.ignorecase"].map(OsString::from));
    let request: crate::transaction_git::GitRequest = crate::transaction_git::GitRequest { without_prefix: true, ..crate::transaction_git::GitRequest::new(common_dir, arguments.as_slice()) };
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
fn unprovable_operand_refusal(words: &[OsString], subcommand_index: usize, reason: &str) -> String {
    let operand: String = words[subcommand_index + 2..]
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
///  The report of a command refused because the words Git will run, or the directory they run in, cannot be read.
/// Why:
///  A command whose words this program cannot read may be a removal,
///  and one whose directory it cannot name may remove a worktree it never judged:
///  neither is handed to Git on a guess, and the person reads what could not be read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// unprovableRemovalRefusal(command, reason) // the report
/// ```
fn unprovable_removal_refusal(command: &ResolvedCommand, reason: &str) -> String {
    let typed: String = command.arguments.iter().map(|word| return word.to_string_lossy().into_owned()).collect::<Vec<String>>().join(" ");
    let quoted: String = crate::shell_quote::shell_quote(typed.as_bytes());
    return format!(
        "{PREFIX}this command names {quoted}, and nothing here can read what it runs, so nothing was removed: {reason}. Run the removal as you mean it, or run it as real Git runs it with the guard's opt-out directly after `worktree`."
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
fn main_worktree_refusal(words: &[OsString], subcommand_index: usize, main: &Path) -> String {
    let operand: String = words[subcommand_index + 2..]
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
