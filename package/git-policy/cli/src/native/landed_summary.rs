//! What:
//!  The one summary line a landed commit prints, the way `git commit` prints it: `[<branch> <name>] <subject>`, with
//!  `(root-commit)` for a commit without a parent and `detached HEAD` for a detached head, the name abbreviated as Git
//!  abbreviates it.
//! Why:
//!  The line is printed once, after the service moved the branch, so the name is the commit that landed and not a build that
//!  lost a race; a person may copy it. The abbreviation honors `core.abbrev` and otherwise follows the size of the
//!  repository (`object_abbreviation.rs`), and Git is asked only when the configuration or the object store is not
//!  something that reader models.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! console.log(summaryLine({ reference, root, name: shortName(commit), subject: subjectOf(message) }));
//! ```

use super::config_snapshot::ConfigSnapshot;
use super::diagnostic_log::warn;
use super::service_protocol::is_zero_object;
use super::transaction_git::{GitContext, GitRequest, run_git_checked};
use std::path::Path;

/// The characters a name keeps when nothing could say how Git would abbreviate it.
const LAST_RESORT_LENGTH: usize = 7;

/// The key whose value sets the length.
const ABBREVIATION_KEY: &str = "core.abbrev";

/// What:
///  The subject of a commit message: its first paragraph on one line.
/// Why:
///   Git's `%s` joins the lines of the first paragraph with spaces, and `git commit` prints that.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// message.trim().split(/\n\s*\n/)[0].split('\n').map((line) => line.trim()).join(' ')
/// ```
pub fn subject_of(message: &str) -> String {
    let mut words: Vec<&str> = Vec::new();
    for line in message.lines().skip_while(|line| { return line.trim().is_empty(); }) {
        if line.trim().is_empty() {
            break;
        }
        words.push(line.trim());
    }
    return words.join(" ");
}

/// What:
///  The line, from the branch reference, whether the commit has no parent, the abbreviated name and the subject.
/// Why:
///   The reference is `HEAD` for a detached head, which Git words as `detached HEAD`.
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
    return snapshot.entries().iter().rev().find(|entry| { return entry.key == ABBREVIATION_KEY; }).map(|entry| { return entry.value.clone(); });
}

/// Git's own answer for the name, for the shapes the in-process reader declines.
fn asked_of_git(context: &GitContext, cwd: &Path, commit: &str) -> Option<String> {
    let request: GitRequest = GitRequest::new(cwd, &["rev-parse", "--verify", "--short", commit]);
    return match run_git_checked(context, &request) {
        Ok(output) => Some(String::from_utf8_lossy(output.stdout.as_slice()).trim().to_string()).filter(|text| { return !text.is_empty(); }),
        Err(failure) => {
            warn("landedSummary", format!("asking Git for the abbreviated name failed: {failure}").as_str());
            None
        }
    };
}

/// What:
///  The abbreviated name of a landed commit.
/// Why:
///   Without starting Git when the configuration was read and the object store is plain; otherwise Git answers once. The
///  last resort is seven characters, which is what Git itself falls back to in a small repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// shortName({ context, cwd, objectDirectory, config, commit }) // '30b2211a4c'
/// ```
#[cfg(unix)]
pub fn short_name(context: &GitContext, cwd: &Path, object_directory: &Path, config: Option<&ConfigSnapshot>, commit: &str) -> String {
    use super::object_abbreviation::{Setting, abbreviate, setting_of};
    let from_files: Option<String> = config.and_then(|snapshot| {
        let setting: Setting = match configured(snapshot) {
            Some(value) => setting_of(value.as_deref()),
            None => Setting::Auto,
        };
        return abbreviate(object_directory, setting, commit);
    });
    return from_files
        .or_else(|| { return asked_of_git(context, cwd, commit); })
        .unwrap_or_else(|| { return commit.get(..LAST_RESORT_LENGTH).unwrap_or(commit).to_string(); });
}

/// Without the in-process reader Git answers, and the last resort is the same.
#[cfg(not(unix))]
pub fn short_name(context: &GitContext, cwd: &Path, _object_directory: &Path, _config: Option<&ConfigSnapshot>, commit: &str) -> String {
    return asked_of_git(context, cwd, commit).unwrap_or_else(|| { return commit.get(..LAST_RESORT_LENGTH).unwrap_or(commit).to_string(); });
}

/// What:
///  The lines the commit command prints below its summary line: what changed against the first parent, and the mode lines
///  of created, deleted and renamed files.
/// Why:
///   A commit Git builds used to print them, and the build is now quiet because its first print may name a commit that
///  does not land; the same lines are asked of Git once, for the commit that landed. Nothing is returned when Git cannot say.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await changeSummary({ context, cwd, firstParent, commit }) // ' 1 file changed, 1 insertion(+)\n create mode 100644 b.txt'
/// ```
pub fn change_summary(context: &GitContext, cwd: &Path, first_parent: &str, commit: &str) -> String {
    let root: bool = first_parent.is_empty() || is_zero_object(first_parent);
    let mut arguments: Vec<&str> = vec!["diff-tree", "--shortstat", "--summary", "-M"];
    if root {
        arguments.extend(["--root", commit]);
    } else {
        arguments.extend([first_parent, commit]);
    }
    return match run_git_checked(context, &GitRequest::new(cwd, arguments.as_slice())) {
        Ok(output) => String::from_utf8_lossy(output.stdout.as_slice()).trim_end().to_string(),
        Err(failure) => {
            warn("landedSummary", format!("asking Git what the commit changed failed: {failure}").as_str());
            String::new()
        }
    };
}

/// Summary controls stay out of the release executable.
#[cfg(test)]
#[path = "landed_summary_tests.rs"]
mod tests;
