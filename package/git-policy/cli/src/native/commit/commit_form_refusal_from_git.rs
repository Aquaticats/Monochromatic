//! What:
//!  The refusal real Git prints for a form of `git commit` it refuses,
//!  obtained by asking Git itself.
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
//!  with `--dry-run` added:
//!  Git then runs its own validation exactly as it would for the real command,
//!  and stops at the first form it refuses,
//!  printing its own message.
//!  `--dry-run` makes Git roll its index lock back and return before it looks
//!  for a commit to make
//!  (measured:
//! no index file changes and no reference moves for `-a`,
//!  `-i` and `-o`),
//!  so the question leaves the repository as it was.
//!  Only a form already known to be refused is asked about
//!  (`rule_commit_region_git_refuses.rs`),
//!  so a command that is fine pays no process for this.
//!  When Git cannot be asked at all,
//!  the refusal is the wrapper's own sentence saying so,
//!  because the form is still one Git refuses:
//!  `unavailable_words`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const refusal = await askGitForTheRefusal({ context, cwd, arguments, subcommandIndex });
//! if (refusal) { process.stderr.write(refusal.stderr); process.exit(refusal.code); }
//! ```

/// Running one real-Git start and reading its answer.
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
/// `OsString` is owned operating-system text of raw bytes,
///  so an argument Git reads keeps the bytes the person wrote.
use std::ffi::OsString;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// The word the question needs,
/// spelled as Git spells it.
const DRY_RUN: &str = "--dry-run";

/// The subcommand whose validation the question asks for.
const COMMIT: &str = "commit";

/// The separator that ends the options of a commit region.
const PATHSPEC_SEPARATOR: &str = "--";

/// The status Git exits with when it refuses a command line (`die`).
const GIT_FATAL: i32 = 128;

/// What:
///  The words the wrapper prints when the form is one Git refuses and Git cannot be asked for its own:
///  a sentence naming both,
///  so a person on a machine whose Git will not start is still told which form is refused and why these are not Git's words.
/// Why:
///   Every form the wrapper asks about is one Git refuses,
///  so the refusal still happens;
///  only its wording is the wrapper's here,
///  and it says which words are missing and why.
///  `&str` borrows the form's name;
///  `String` is the owned sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unavailableWords = (form: string) => `monochromatic-git-policy-cli: git commit refuses this form (${form}) ...`;
/// ```
pub fn unavailable_words(form: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: `git commit` refuses this form ({form}), which real Git answers with a fatal message. \
Real Git could not be started to print that message here, so this sentence is the wrapper's; \
correct the options and run the command again.\n"
    );
}

/// What:
///  A refusal read from Git:
///  the status it exited with and the bytes it printed.
/// Why:
///   Both are Git's,
///  so the person sees exactly what the real command would have printed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Refusal = { code: number; stderr: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Refusal {
    /// The status Git exited with.
    pub code: i32,
    /// The bytes Git printed on standard error.
    pub stderr: Vec<u8>,
}

/// What:
///  Whether one token is the separator that ends a commit's options.
/// Why:
///   `position` takes the test as a function of one token,
///  and only the token written `--` after every option ends them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isTheSeparator = (token) => token === '--';
/// ```
fn is_the_separator(token: &OsString) -> bool {
    return token == PATHSPEC_SEPARATOR;
}

/// What:
///  The commit arguments with `--dry-run` in option position.
/// Why:
///   A `--dry-run` written after the separators `--` would be a path to Git,
///  so the option goes before the first one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const withDryRun = (args) => { const at = args.indexOf('--'); return at < 0 ? [...args, '--dry-run'] : [...args.slice(0, at), '--dry-run', ...args.slice(at)]; };
/// ```
pub fn with_dry_run(arguments: &[OsString]) -> Vec<OsString> {
    // The first `--` is the one Git reads as the end of options, because everything before it is an option and its value.
    let separator: Option<usize> = arguments.iter().position(is_the_separator);
    let mut asked: Vec<OsString> = Vec::with_capacity(arguments.len() + 1);
    match separator {
        Some(at) => {
            asked.extend_from_slice(&arguments[..at]);
            asked.push(OsString::from(DRY_RUN));
            asked.extend_from_slice(&arguments[at..]);
        }
        None => {
            asked.extend_from_slice(arguments);
            asked.push(OsString::from(DRY_RUN));
        }
    }
    return asked;
}

/// What:
///  Ask the real Git of this invocation what it says about this commit's options,
///  given as the region after `commit`.
/// Why:
///   The answer is Git's own status and wording,
///  so no message is maintained here.
///  An answer that is not a refusal (status 0,
/// or any status but `die`'s)
///  means Git accepts the form,
///  and the command goes on to the transaction.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = await askGit({ context, cwd, arguments });
/// ```
pub fn ask_git(context: &GitContext, cwd: &Path, region: &[OsString]) -> Option<Refusal> {
    // The question is the person's own command line, `commit` and then their options, with `--dry-run` added.
    let mut asked: Vec<OsString> = Vec::with_capacity(region.len() + 2);
    asked.push(OsString::from(COMMIT));
    asked.extend(with_dry_run(region));
    let request: GitRequest = GitRequest::new(cwd, asked.as_slice());
    let output: GitOutput = match run_git(context, &request) {
        Ok(answered) => answered,
        // Git could not be started, so it printed nothing; the caller then says which form is refused and why the words are
        //  the wrapper's (`unavailable_words`).
        Err(_) => return None,
    };
    if output.code != Some(GIT_FATAL) {
        return None;
    }
    return Some(Refusal { code: GIT_FATAL, stderr: output.stderr });
}

/// The reading of Git's answer.
#[cfg(test)]
#[path = "commit_form_refusal_from_git_tests.rs"]
mod tests;
