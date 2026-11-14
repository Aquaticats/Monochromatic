//! What:
//!  The question real Git is asked about a form of `git commit` this wrapper refuses,
//!  and what the wrapper does with the answer.
//! Why:
//!  The owner's answer of 2026-11-12 is "Refuse as Git does",
//!  with Git's own message
//!  (`doc/handover/cli-git-rust-implementation.md`).
//!  A message copied into this program would be one release behind the Git
//!  installed beside it:
//!  a reworded release would leave the wrapper printing words Git no longer
//!  prints,
//!  and a language the person asked for would never reach them.
//!  So the words are never written here.
//!  Git is asked the same command line the person wrote,
//!  in the same repository,
//!  with `--dry-run` as the question's **first** option.
//!  A token is an option only in option position,
//!  and the first token after `commit` always is,
//!  so no option of the person's can take `--dry-run` as its value:
//!  `git commit -m -- -i` is asked as `git commit --dry-run -m -- -i`,
//!  where the `--` after `-m` is the person's message and not the question's
//!  separator
//!  (`commit_args::separator_index` reads the same rule for the commit's own
//!  arguments).
//!  `--dry-run` makes Git run its own validation,
//!  roll its index lock back and return before it looks for a commit,
//!  so the question leaves the repository as it was.
//!  Two things stand behind that,
//! and neither is the option itself:
//!  the question is given an index of its own,
//!  a copy of the person's in the repository scratch directory,
//!  so a lost `--dry-run` could not write the person's index or hold the lock
//!  beside it;
//!  and the controls over every generated command line record `HEAD`,
//!  every reference,
//!  the index bytes and the worktree before and after the question.
//!  What no copy can protect is the branch reference and the object store,
//!  which are the repository's own:
//!  the option's position is the guarantee there,
//!  and the control that watches the references is what proves it.
//!  Only a form already known to be refused is asked about
//!  (`rule_commit_region_git_refuses.rs`),
//!  so a command that is fine pays no process for this.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const answer = await askGitForTheRefusal({ context, cwd, region });
//! switch (answer.decide()) { case 'end-with-git': ... case 'accept': ... case 'unavailable': ... }
//! ```

/// Running one real-Git start and reading its answer.
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// Copying the person's index into a scratch directory of this repository.
use crate::commit_index_bytes::copy_index_with_timestamps;
/// The directory a refused commit's scratch files are made in.
use crate::scratch_location::prepare_repository_scratch;
/// A unique name for the question's own index.
use crate::random_id::random_uuid;
/// `OsString` is owned operating-system text of raw bytes,
///  so an argument Git reads keeps the bytes the person wrote.
use std::ffi::OsString;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// The option that makes the question a dry run,
/// spelled as Git spells it.
const DRY_RUN: &str = "--dry-run";

/// The subcommand whose validation the question asks for.
const COMMIT: &str = "commit";

/// The query that names the index the person's own command would use.
const INDEX_QUERY: [&str; 4] = ["rev-parse", "--path-format=absolute", "--git-path", "index"];

/// The prefix of the question's own index file in the repository scratch directory.
const QUESTION_INDEX_PREFIX: &str = "cli-git-question-";

/// What:
///  Why Git gave no answer to the question.
/// Why:
///   The wrapper refuses the form in its own words then,
///  and the sentence has to say which of these happened,
///  because "Git could not be started" is false when Git started and was held.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Unanswered = 'not-started' | 'held-past-its-bound' | 'ended-without-a-status';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Unanswered {
    /// Git never started:
    /// the program could not be run in that directory.
    NotStarted,
    /// Git started and showed no sign of work for its step's bound,
    /// so the step was ended.
    HeldPastItsBound,
    /// Git ran and ended without an exit status,
    /// which a signal does.
    EndedWithoutAStatus,
}

/// What:
///  What the question came back with.
/// Why:
///   A status and the bytes Git printed are Git's answer;
///  no status at all is the absence of one,
///  and the two are not the same thing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Answer = { kind: 'answered'; status: number; stderr: bytes } | { kind: 'unanswered'; why: Unanswered };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Answer {
    /// Git exited with this status and printed these bytes on standard error.
    Answered {
        /// The exit status.
        status: i32,
        /// The bytes on standard error.
        stderr: Vec<u8>,
    },
    /// Git gave no status.
    Unanswered(Unanswered),
}

/// What:
///  What the wrapper does with one answer.
/// Why:
///   Git's own answer decides the command:
///  a fatal status is the refusal,
///  a status of zero means Git accepts the form the rule expected it to refuse,
///  and any other status is an answer the wrapper hands on as it came.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Decision = { kind: 'end-with-git'; status: number; stderr: bytes } | { kind: 'accept' } | { kind: 'refuse-in-our-words'; why: Unanswered };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Decision {
    /// End the command with Git's own status and bytes.
    EndWithGit {
        /// The status Git exited with.
        status: i32,
        /// The bytes Git printed on standard error.
        stderr: Vec<u8>,
    },
    /// Do not refuse:
    /// Git accepts this command line,
    /// so the ordinary commit goes on.
    Accept,
    /// Refuse in the wrapper's own words,
    /// naming the form and what became of the question.
    RefuseInOurWords(Unanswered),
}

impl Answer {
    /// What:
    ///  Decide the command from Git's answer.
    /// Why:
    ///   The rule exists to refuse what Git refuses;
    ///  where Git's own answer is that it would run the command,
    ///  the rule's word is not enough to refuse it,
    ///  because the question is a dry run that committed nothing,
    ///  and the ordinary commit is then the only writer of the one commit.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const decide = (a: Answer): Decision => a.kind === 'unanswered' ? { kind: 'refuse-in-our-words', why: a.why } : a.status === 0 ? { kind: 'accept' } : { kind: 'end-with-git', status: a.status, stderr: a.stderr };
    /// ```
    pub fn decide(&self) -> Decision {
        match self {
            Self::Unanswered(why) => return Decision::RefuseInOurWords(*why),
            // Zero means Git would run this command. The question was a dry run, so nothing has been committed by it, and
            //  the transaction that follows makes the one commit this command asks for.
            Self::Answered { status: 0, .. } => return Decision::Accept,
            Self::Answered { status, stderr } => {
                return Decision::EndWithGit { status: *status, stderr: stderr.clone() };
            }
        }
    }
}

/// What:
///  The words the wrapper prints when the form is one Git refuses and Git gave no answer,
/// naming the form,
/// which of the three happened,
/// and that these are the wrapper's words.
/// Why:
///   Every form the wrapper asks about is one Git refuses,
///  so the refusal happens either way;
///  a person whose Git will not start,
///  or whose Git was held past its bound,
///  must still learn which form is refused and why the words are not Git's.
///  `&str` borrows the form's name;
///  `String` is the owned sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unavailableWords = (form: string, why: Unanswered) => `monochromatic-git-policy-cli: git commit refuses this form (${form}) ...`;
/// ```
pub fn unavailable_words(form: &str, why: Unanswered) -> String {
    let happened: &str = match why {
        Unanswered::NotStarted => {
            "Real Git could not be started to print that message here, so this sentence is the wrapper's; \
             check that the Git this wrapper found is installed and executable, then run the command again."
        }
        Unanswered::HeldPastItsBound => {
            "Real Git was asked and showed no sign of work for the bound of that step, so the step was ended \
             before it printed anything; this sentence is the wrapper's, and the command is refused all the same."
        }
        Unanswered::EndedWithoutAStatus => {
            "Real Git was asked, ran and ended without an exit status, printing nothing; this sentence is the \
             wrapper's, and the command is refused all the same."
        }
    };
    return format!(
        "monochromatic-git-policy-cli: `git commit` refuses this form ({form}), which real Git answers with a fatal message. \
{happened}\n"
    );
}

/// What:
///  The question's own arguments:
///  `commit`,
/// then `--dry-run`,
/// then the person's options exactly as they wrote them.
/// Why:
///   The first token after `commit` is in option position whatever it spells,
///  so `--dry-run` cannot become the value of an option the person wrote
///  (`-m --` is a message,
/// not a separator),
///  and it cannot fall after a `--` that ends the options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theQuestion = (region) => ['commit', '--dry-run', ...region];
/// ```
pub fn the_question(region: &[OsString]) -> Vec<OsString> {
    let mut asked: Vec<OsString> = Vec::with_capacity(region.len() + 2);
    asked.push(OsString::from(COMMIT));
    asked.push(OsString::from(DRY_RUN));
    asked.extend_from_slice(region);
    return asked;
}

/// What:
///  A private copy of the person's index for the question to write to,
///  or nothing when one cannot be made.
/// Why:
///   Git's dry run stages into the index it is given and rolls its lock back;
///  with the person's own index it would be their file and their `index.lock`
///  that a lost `--dry-run` could write.
///  The copy is of the index the person's own command would use,
///  read from Git itself so a linked worktree,
///  `--git-dir` and `GIT_INDEX_FILE` all answer once,
///  and it is named for this question so two questions cannot share one.
///  Nothing here is required for the refusal to be right:
///  when the copy cannot be made the question is asked with the person's index,
///  which is what the released behaviour was.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const privateIndex = (context, cwd) => copyIntoScratch(gitAnswersTheIndexPath(context, cwd));
/// ```
fn a_private_index(context: &GitContext, cwd: &Path) -> Option<PathBuf> {
    let query: GitRequest = GitRequest::new(cwd, &INDEX_QUERY);
    let answered: GitOutput = run_git(context, &query).ok()?;
    if answered.code != Some(0) {
        return None;
    }
    let printed: String = String::from_utf8(answered.stdout).ok()?;
    let source: PathBuf = PathBuf::from(printed.trim_end_matches('\n'));
    if source.as_os_str().is_empty() {
        return None;
    }
    let unique: String = random_uuid().ok()?;
    let directory: PathBuf = prepare_repository_scratch(context.common_dir.as_deref()?).ok()?;
    let copy: PathBuf = directory.join(format!("{QUESTION_INDEX_PREFIX}{unique}.index"));
    copy_index_with_timestamps(source.as_path(), copy.as_path()).ok()?;
    return Some(copy);
}

/// What:
///  Ask the real Git of this invocation what it says about this commit's options,
///  given as the region after `commit`.
/// Why:
///   The answer is Git's own status and bytes,
///  so no message is maintained here,
///  and the three ways of not answering are told apart,
///  so the sentence that replaces them can say what happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = await askGit({ context, cwd, region });
/// ```
pub fn ask_git(context: &GitContext, cwd: &Path, region: &[OsString]) -> Answer {
    // The question writes to an index of its own when one can be made, and to the person's otherwise.
    let private: Option<PathBuf> = a_private_index(context, cwd);
    let mut request: GitRequest = GitRequest::new(cwd, the_question(region).as_slice());
    request.index = private.clone();
    let ran = run_git(context, &request);
    if let Some(path) = private {
        // The command's result never depends on the copy: a file that cannot be removed is swept with its directory
        //  later (`scratch_location`), and a slow disk must not turn a refusal into a failure.
        let _ = std::fs::remove_file(path);
    }
    return match ran {
        Ok(output) => match output.code {
            Some(status) => Answer::Answered { status, stderr: output.stderr },
            None => Answer::Unanswered(Unanswered::EndedWithoutAStatus),
        },
        Err(error) if error.kind() == std::io::ErrorKind::TimedOut => Answer::Unanswered(Unanswered::HeldPastItsBound),
        Err(_) => Answer::Unanswered(Unanswered::NotStarted),
    };
}

/// The question's arguments,
/// the three ways of not answering,
/// and what each answer decides.
#[cfg(test)]
#[path = "commit_form_refusal_from_git_tests.rs"]
mod tests;

/// The width of the form rule and the question's own writes,
/// over every generated command line.
#[cfg(test)]
#[path = "commit_form_refusal_properties_tests.rs"]
mod properties;
