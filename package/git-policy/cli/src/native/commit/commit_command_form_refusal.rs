//! What:
//!  The refusal of a `git commit` whose options are a form real Git refuses,
//!  with Git's own status and bytes.
//! Why:
//!   "Refuse as Git does" (owner,
//!  2026-11-12):
//!  the form is decided here and Git is asked for its own answer,
//!  so a release that rewords a message changes nothing in this program.
//!  Nothing is created first:
//!  the question is a `--dry-run` of the same command line in the directory the commit runs in,
//!  which rolls its index lock back and never looks for a commit,
//!  so a refused command leaves no staged path,
//!  no reference and no record.
//!  A command that is fine costs one token comparison here and no process.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const form = formGitRefuses(region); if (!form) return;
//! switch (askGit(context, cwd, asked).decide()) { ... }
//! ```

/// The directory the commit itself runs in,
///  which the question is asked in.
use crate::commit_command_directory::where_the_commit_runs;
/// What the command is handed,
///  and how it ends when Git refuses the form.
use crate::commit_command_input::{
    CommitCommand,
    CommitEnding,
};
/// The two hand-overs of standard error.
use crate::commit_command_words::{
    write_stderr,
    write_stderr_bytes,
};
/// The question Git is asked about a form,
///  and the three ways it can go unanswered.
use crate::commit_form_refusal_from_git::{
    Decision,
    Unanswered,
    ask_git,
    unavailable_words,
};
use crate::diagnostic_log::debug;
/// Which forms of `git commit` real Git refuses.
use crate::rule_commit_region_git_refuses::{
    GitRefusedForm,
    form_git_refuses,
};
/// The transaction Git wrapper,
///  whose context the question runs in.
use crate::transaction_git::GitContext;
/// `OsString` is owned operating-system text of raw bytes.
use std::ffi::OsString;
/// `PathBuf` is an owned filesystem path.
use std::path::PathBuf;

/// What:
///  Refuse a `git commit` whose options are a form real Git refuses,
///  with Git's own status and bytes,
///  or nothing when the command goes on.
/// Why:
///   "Refuse as Git does" (owner,
///  2026-11-12):
///  the form is decided here
///  (`rule_commit_region_git_refuses.rs`) and Git is asked for its own answer
///  (`commit_form_refusal_from_git.rs`),
///  so a release that rewords a message changes nothing in this program.
///  Nothing is created first:
///  the question is a `--dry-run` of the same command line in the directory the
///  commit runs in,
///  which rolls its index lock back and never looks for a commit,
///  so a refused command leaves no staged path,
/// no reference and no record.
///  Where Git's own answer is that it would run the command,
///  the command goes on and the transaction makes the one commit:
///  the question committed nothing,
///  so nothing is made twice,
/// and nothing the person staged is lost.
///  A command that is fine costs one token comparison here and no process.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const form = formGitRefuses(region); if (!form) return;
/// const { context, cwd } = whereTheCommitRuns(command);
/// switch (askGit(context, cwd, asked).decide()) { ... }
/// ```
pub(crate) fn refuse_a_form_git_refuses(command: &CommitCommand<'_>) -> Option<CommitEnding> {
    let form: GitRefusedForm = form_git_refuses(command.region)?;
    let (context, cwd): (GitContext, PathBuf) = where_the_commit_runs(command);
    let asked: &[OsString] = &command.arguments[command.subcommand_index + 1..];
    match ask_git(&context, cwd.as_path(), asked).decide() {
        Decision::EndWithGit {
            status,
            stderr,
        } => {
            debug(
                "commit-form-refusal",
                format!("real Git answers {} with status {status}", form.name()).as_str(),
            );
            write_stderr_bytes(stderr.as_slice());
            return Some(CommitEnding {
                code: status,
                signal: None,
            });
        }
        Decision::Accept => {
            // Git runs this command line, so the rule's word is not enough to refuse it. The question was a dry run, so
            //  no commit was made by it, and the transaction that goes on is the only writer of this commit.
            debug(
                "commit-form-refusal",
                format!(
                    "real Git accepts {} here, so the commit goes on",
                    form.name()
                )
                .as_str(),
            );
            return None;
        }
        Decision::RefuseInOurWords(why) => {
            debug(
                "commit-form-refusal",
                format!("real Git gave no answer about {} ({}), so the wrapper refuses it in its own words", form.name(), reason_name(why)).as_str(),
            );
            write_stderr(unavailable_words(form.name(), why).as_str());
            return Some(CommitEnding {
                code: 128,
                signal: None,
            });
        }
    }
}

/// What:
///  The name of one way the question went unanswered,
///  for the debug line.
/// Why:
///   A person reading the debug log says which of the three happened without reading the sentence's prose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reasonName = (why: Unanswered) => ({ 'not-started': 'not-started', ... })[why];
/// ```
fn reason_name(why: Unanswered) -> &'static str {
    return match why {
        Unanswered::NotStarted => "not started",
        Unanswered::HeldPastItsBound => "held past its bound",
        Unanswered::EndedWithoutAStatus => "ended without a status",
    };
}
