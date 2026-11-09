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
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.log(summaryLine({ reference, root, name: shortName(commit), subject: subjectOf(message) }));
//! ```

use crate::config_snapshot::ConfigSnapshot;
use crate::diagnostic_log::warn;
use crate::service_protocol::is_zero_object;
use crate::transaction_git::{GitContext, GitRequest, run_git_checked};
use std::path::Path;

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
pub fn summary_line(reference: &str, first_parent: &str, name: &str, subject: &str) -> String {
    let branch: &str = if reference == "HEAD" { "detached HEAD" } else { reference.trim_start_matches("refs/heads/") };
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

/// Git's own answer for the name,
/// for the shapes the in-process reader declines.
fn asked_of_git(context: &GitContext, cwd: &Path, commit: &str) -> Option<String> {
    let request: GitRequest = GitRequest::new(cwd, &["rev-parse", "--verify", "--short", commit]);
    return match run_git_checked(context, &request) {
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
    use crate::object_abbreviation::{Setting, abbreviate, setting_of};
    let from_files: Option<String> = match config {
        Some(snapshot) => {
            let setting: Setting = match configured(snapshot) {
                Some(value) => setting_of(value.as_deref()),
                None => Setting::Auto,
            };
            abbreviate(object_directory, setting, commit)
        }
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
