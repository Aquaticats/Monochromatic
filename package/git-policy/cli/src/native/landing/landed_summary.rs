//! What:
//!  The one summary line a landed commit prints,
//! the way `git commit` prints it:
//! `[<branch> <name>] <subject>`,
//! with
//!  `(root-commit)` for a commit without a parent and `detached HEAD` for a detached head,
//! the name abbreviated as Git
//!  abbreviates it.
//! Why:
//!  The line is for a commit the service built itself,
//! where no `git commit` ran to print one.
//! It is printed once,
//! after the
//!  service moved the branch,
//! so the name is the commit that landed;
//! a commit that real Git builds prints Git's own summary,
//!  because the wrapper adds no `--quiet` to it.
//! The abbreviation honors `core.abbrev` and otherwise follows the size of the
//!  repository (`object_abbreviation.rs`),
//! and Git is asked only when the configuration or the object store is not
//!  something that reader models.
//! The name has two faces:
//! the synchronous one a caller on a thread asks (`short_name`),
//! and the mode awaited one a task of the runtime asks
//!  (`short_name_awaiting`),
//! which reads the object store as a job of the blocking pool,
//! awaited no longer than `LANDING_STATE_CEILING`,
//!  and awaits its Git child on the runtime.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.log(summaryLine({ reference, root, name: await shortNameAwaiting(commit), subject: subjectOf(message) }));
//! ```

use crate::blocking_jobs_within_the_budget::{TheAnswerOfAJobAwaited, run_a_job_awaited_no_longer_than};
use crate::config_snapshot::ConfigSnapshot;
use crate::diagnostic_log::warn;
#[cfg(unix)]
use crate::object_abbreviation::{Setting, abbreviate, setting_of};
use crate::service_protocol::is_zero_object;
use crate::transaction_git::{GitContext, GitFailure, GitOutput, GitRequest, run_git_checked};
use crate::transaction_git_awaiting::run_git_checked_awaiting;
use crate::wait_ceilings::LANDING_STATE_CEILING;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The characters a name keeps when nothing could say how Git would abbreviate it.
const LAST_RESORT_LENGTH: usize = 7;

/// The key whose value sets the length.
const ABBREVIATION_KEY: &str = "core.abbrev";

/// What:
///  Whether a line of a message is blank:
/// empty or only white space.
/// Why:
///  `subject_of` skips the blank lines before the subject.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isBlankLine = (line: string) => line.trim() === '';
/// ```
fn is_blank_line(line: &&str) -> bool {
    return line.trim().is_empty();
}

/// What:
///  The subject of a commit message:
/// its first paragraph on one line.
/// Why:
///   Git's `%s` joins the lines of the first paragraph with spaces,
/// and `git commit` prints that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// message.trim().split(/\n\s*\n/)[0].split('\n').map((line) => line.trim()).join(' ')
/// ```
pub fn subject_of(message: &str) -> String {
    let mut words: Vec<&str> = Vec::new();
    for line in message.lines().skip_while(is_blank_line) {
        if line.trim().is_empty() {
            break;
        }
        words.push(line.trim());
    }
    return words.join(" ");
}

/// What:
///  The line,
/// from the branch reference,
/// whether the commit has no parent,
/// the abbreviated name and the subject.
/// Why:
///   The reference is `HEAD` for a detached head,
/// which Git words as `detached HEAD`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `[${branch}${root ? ' (root-commit)' : ''} ${name}] ${subject}`
/// ```
pub fn summary_line(reference: &crate::ref_name::RefName, first_parent: &str, name: &str, subject: &str) -> String {
    // A branch named by a byte that is not text is shown as the word a shell reads back, so the line stays one readable line
    //  that names the branch the person has.
    let shown: String = reference.short().spoken();
    let branch: &str = if reference.is_head() { "detached HEAD" } else { shown.as_str() };
    let root: &str = if first_parent.is_empty() || is_zero_object(first_parent) { " (root-commit)" } else { "" };
    return format!("[{branch}{root} {name}] {subject}");
}

/// The last value `core.abbrev` has in the snapshot.
fn configured(snapshot: &ConfigSnapshot) -> Option<Option<Vec<u8>>> {
    // The last entry with the key is the one Git reads.
    for entry in snapshot.entries().iter().rev() {
        if entry.key == ABBREVIATION_KEY {
            return Some(entry.value.clone());
        }
    }
    return None;
}

/// What:
///  The question both faces ask Git for the name.
/// Why:
///  One request serves both faces,
/// so neither can ask for another commit's name than the other.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const request = { cwd, arguments: ['rev-parse', '--verify', '--short', commit] };
/// ```
fn asked_request(cwd: &Path, commit: &str) -> GitRequest {
    return GitRequest::new(cwd, &["rev-parse", "--verify", "--short", commit]);
}

/// What:
///  The name Git's answer to `asked_request` holds,
///  and nothing when it held none.
/// Why:
///  Both faces read Git's answer through this one function,
/// so a name that is empty and a Git that could not be
///  asked leave both of them with nothing,
/// and the last resort is the same for both.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const askedName = (answered) => { if (answered.error) { warn(...); return undefined } const text = answered.out.trim(); return text === '' ? undefined : text; }
/// ```
fn asked_name(answered: Result<GitOutput, GitFailure>) -> Option<String> {
    return match answered {
        Ok(output) => {
            let text: String = String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string();
            if text.is_empty() { None } else { Some(text) }
        }
        Err(failure) => {
            warn("landedSummary", format!("asking Git for the abbreviated name failed: {failure}").as_str());
            None
        }
    };
}

/// What:
///  Git's own answer for the name,
///  for the shapes the in-process reader declines.
/// Why:
///   Git answers once when the files could not,
/// and a question Git could not answer is said in the service's log and leaves the last resort
///  to the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asked = askedName(await runGitChecked(request));
/// ```
fn asked_of_git(context: &GitContext, cwd: &Path, commit: &str) -> Option<String> {
    return asked_name(run_git_checked(context, &asked_request(cwd, commit)));
}

/// What:
///  `asked_of_git` with its Git child awaited on the runtime.
/// Why:
///   A task of the runtime cannot enter it to wait for a child
/// (`transaction_git_awaiting.rs`),
/// and the summary line is printed from the landing once that landing is a task.
/// The question and
///  the reading of Git's answer are the synchronous face's,
/// so the two cannot differ in what they call a name.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const asked = askedName(await runGitCheckedAwaiting(request));
/// ```
async fn asked_of_git_awaiting(context: &GitContext, cwd: &Path, commit: &str) -> Option<String> {
    return asked_name(run_git_checked_awaiting(context, &asked_request(cwd, commit)).await);
}

/// The setting the last value `core.abbrev` has in the snapshot stands for.
#[cfg(unix)]
fn setting_for(snapshot: &ConfigSnapshot) -> Setting {
    return match configured(snapshot) {
        Some(value) => setting_of(value.as_deref()),
        None => Setting::Auto,
    };
}

/// What:
///  The three facts one in-process abbreviation reads,
/// owned so that a job of the blocking pool can take them.
/// Why:
///  The object store is read by a job of the pool once per question,
/// so no task waits on a directory listing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type AbbreviationQuestion = { objectDirectory: string; setting: Setting; commit: string };
/// ```
#[cfg(unix)]
struct AbbreviationQuestion {
    /// The object directory the name is made unique in.
    object_directory: PathBuf,
    /// What `core.abbrev` asked for.
    setting: Setting,
    /// The commit whose name is abbreviated.
    commit: String,
}

/// The reading one job of the pool does:
/// `abbreviate` over the question's own object store.
#[cfg(unix)]
fn abbreviated_from_the_files(question: AbbreviationQuestion) -> Option<String> {
    return abbreviate(question.object_directory.as_path(), question.setting, question.commit.as_str());
}

/// What:
///  The in-process name as a job of the blocking pool,
/// awaited no longer than `ceiling`.
/// Why:
///   A task of the runtime may not read a directory:
/// the object store's listing,
/// its pack indexes and the
///  canonical form of its path wait in the kernel on a mount that stopped answering,
/// which would hold a worker the
///  runtime needs,
/// and no timer on that worker can end it.
/// A reading the ceiling or a refused thread leaves without an
///  answer answers nothing,
/// which is what the files answer already when they cannot describe the name,
/// so the caller
///  asks Git and the person reads Git's own answer.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function abbreviationWithin(ceilingMs: number, question: AbbreviationQuestion): Promise<string | undefined>;
/// ```
#[cfg(unix)]
async fn abbreviation_within(ceiling: Duration, object_directory: &Path, setting: Setting, commit: &str) -> Option<String> {
    let question: AbbreviationQuestion =
        AbbreviationQuestion { object_directory: object_directory.to_path_buf(), setting, commit: commit.to_string() };
    return match run_a_job_awaited_no_longer_than(ceiling, question, abbreviated_from_the_files).await {
        TheAnswerOfAJobAwaited::Ran(answer) => answer,
        // No thread was given, or the job outlasted its ceiling: the files said nothing this task may wait for.
        TheAnswerOfAJobAwaited::NotRun | TheAnswerOfAJobAwaited::NoAnswer => None,
    };
}

/// `abbreviation_within` with the wait a landed commit's summary step is granted
/// (`wait_ceilings.rs`,
/// `LANDING_STATE_CEILING`).
#[cfg(unix)]
async fn abbreviation_awaiting(object_directory: &Path, setting: Setting, commit: &str) -> Option<String> {
    return abbreviation_within(LANDING_STATE_CEILING, object_directory, setting, commit).await;
}

/// What:
///  The abbreviated name of a landed commit.
/// Why:
///   Without starting Git when the configuration was read and the object store is plain;
/// otherwise Git answers once.
/// The
///  last resort is seven characters,
/// which is what Git itself falls back to in a small repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// shortName({ context, cwd, objectDirectory, config, commit }) // '30b2211a4c'
/// ```
#[cfg(unix)]
pub fn short_name(context: &GitContext, cwd: &Path, object_directory: &Path, config: Option<&ConfigSnapshot>, commit: &str) -> String {
    let from_files: Option<String> = match config {
        Some(snapshot) => abbreviate(object_directory, setting_for(snapshot), commit),
        None => None,
    };
    // The files first, then Git's own answer, then the last resort; each is tried only when the one before gave nothing.
    if let Some(name) = from_files {
        return name;
    }
    if let Some(asked) = asked_of_git(context, cwd, commit) {
        return asked;
    }
    return commit.get(..LAST_RESORT_LENGTH).unwrap_or(commit).to_string();
}

/// Without the in-process reader Git answers,
/// and the last resort is the same.
#[cfg(not(unix))]
pub fn short_name(context: &GitContext, cwd: &Path, _object_directory: &Path, _config: Option<&ConfigSnapshot>, commit: &str) -> String {
    return asked_of_git(context, cwd, commit).unwrap_or_else(|| { return commit.get(..LAST_RESORT_LENGTH).unwrap_or(commit).to_string(); });
}

/// What:
///  `short_name` with the object store read as a job of the blocking pool and its Git child awaited on the
///  runtime.
/// Why:
///   A task of the runtime may wait on neither a directory listing nor a child it starts itself,
/// and the summary line is printed
///  from the landing once that landing is a task.
/// The order of the three answers (the files,
/// then Git's own name,
/// then the last
///  resort) and the last resort itself are the synchronous face's,
/// so the two cannot differ in the name they print.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const name = await shortNameAwaiting({ context, cwd, objectDirectory, config, commit }); // '30b2211a4c'
/// ```
#[cfg(unix)]
pub async fn short_name_awaiting(context: &GitContext, cwd: &Path, object_directory: &Path, config: Option<&ConfigSnapshot>, commit: &str) -> String {
    let from_files: Option<String> = match config {
        Some(snapshot) => abbreviation_awaiting(object_directory, setting_for(snapshot), commit).await,
        None => None,
    };
    // The files first, then Git's own answer, then the last resort; each is tried only when the one before gave nothing.
    if let Some(name) = from_files {
        return name;
    }
    if let Some(asked) = asked_of_git_awaiting(context, cwd, commit).await {
        return asked;
    }
    return commit.get(..LAST_RESORT_LENGTH).unwrap_or(commit).to_string();
}

/// Without the in-process reader Git answers,
/// and the last resort is the same.
#[cfg(not(unix))]
pub async fn short_name_awaiting(context: &GitContext, cwd: &Path, _object_directory: &Path, _config: Option<&ConfigSnapshot>, commit: &str) -> String {
    return asked_of_git_awaiting(context, cwd, commit)
        .await
        .unwrap_or_else(|| { return commit.get(..LAST_RESORT_LENGTH).unwrap_or(commit).to_string(); });
}

/// What:
///  The one line the wrapper adds after a commit that real Git built was built again because the branch moved.
/// Why:
///   Git printed a summary for each build,
/// and the earlier ones name commits that never landed;
/// the line says which names
///  those are and which commit did land,
/// so a person does not copy the wrong one.
/// Git's own output is left as it is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rebuiltNote(['1a2b3c4'], '5d6e7f8') // 'monochromatic-git-policy-cli: the branch moved during the commit, so it was built again; 1a2b3c4 did not land, 5d6e7f8 did.'
/// ```
pub fn rebuilt_note(unlanded: &[String], landed: &str) -> String {
    let names: String = match unlanded.split_last() {
        None => String::new(),
        Some((only, [])) => only.clone(),
        Some((last, front)) => format!("{} and {last}", front.join(", ")),
    };
    return format!("monochromatic-git-policy-cli: the branch moved during the commit, so it was built again; {names} did not land, {landed} did.");
}

/// What:
///  The one line that says an amend whose branch moved became a new commit on top of it.
/// Why:
///   The amend did not replace the commit it named,
/// and the person must not look for their changes there:
/// the line names the branch,
///  the commit that landed,
/// and that the commits already on the branch were left as they were.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// amendLandsAsNewNote('main', '5d6e7f8')
/// ```
pub fn amend_lands_as_new_note(branch: &str, landed: &str) -> String {
    return format!(
        "monochromatic-git-policy-cli: the branch {branch} moved while this amend was being prepared, so its changes were made as the new \
         commit {landed} on top of the branch, and the commits that were already there were left as they are."
    );
}

/// Summary controls stay out of the release executable.
#[cfg(test)]
#[path = "landed_summary_tests.rs"]
mod tests;

/// Controls for the awaited face of the name stay out of the release executable.
#[cfg(test)]
#[path = "landed_summary_awaiting_tests.rs"]
mod awaiting_tests;
