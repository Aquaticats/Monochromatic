//! What:
//!  The record of what a push that succeeded did,
//! for the person,
//! and the name it is kept under.
//! Why:
//!  A push that reconciled a moved remote says what it published and what became of the local branch in its own output,
//! and the command that ran the push returned long before it ended,
//! so the words reach nobody unless they are left where the next command reads them.
//!   The record is a `.txt` file of the notice directory,
//! so the reading,
//! the claiming and the removal of a shown record are the steps every
//!  notice takes (`push_failure_notice.rs`),
//! and its name carries a prefix of its own,
//! because a report is not a failure:
//! a later
//!  successful push of the branch removes the failure records of that branch
//! (`clear_push_failures_before`) and
//!  must not remove a report nobody has read.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordPushReport(gitDir, { branchRef, commit, said }); // report+<branch key>+<commit>.txt
//! ```

use crate::auto_push_branch_key::encode_branch_key;
use crate::private_storage::{create_private_directory_all, replace_private_file_unsynced};
use crate::push_failure_notice::{BRANCH_FIELD, COMMIT_FIELD, LONGEST_KEY, NAME_SEPARATOR, clean_words, fnv_hex, notice_directory};
use std::path::{Path, PathBuf};

/// First line of a record of what a push that succeeded did,
/// which the command that ran it could not print.
pub(crate) const REPORT_MAGIC: &str = "monochromatic-git-policy-cli push report 1";

/// What:
///  The file name of the record of what a push that succeeded did,
/// for the person.
/// Why:
///   A report is not a failure,
/// so it does not carry the prefix a successful push of the branch removes
/// (`clear_push_failures_before`):
/// a report nobody has read yet must survive the next push of its branch,
/// which
///  says nothing about what the earlier push did.
/// It is a `.txt` record of the notice directory,
/// so the reading,
/// the claiming and the
///  removal of a shown record are the same steps every notice takes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const reportName = (branchRef, commit) => `report+${branchKey(branchRef)}+${commit}.txt`;
/// ```
fn report_name(branch_ref: &str, commit: &str) -> String {
    let key: String = encode_branch_key(branch_ref);
    let shown: String = if key.len() > LONGEST_KEY { format!("{}~{}", &key[..LONGEST_KEY], fnv_hex(key.as_str())) } else { key };
    return format!("report{NAME_SEPARATOR}{shown}{NAME_SEPARATOR}{commit}.txt");
}

/// What:
///  Leaves the record of what a push that succeeded did,
/// replacing an earlier record of the same commit on the same branch.
/// Why:
///   A push that reconciled a moved remote publishes a merge and says what became of the local branch in its own output,
/// and the
///  command that ran it returned long before the push ended,
/// so the words would reach nobody.
/// The record carries them to the
///  next command,
/// which shows it once.
///  It keeps every line the push chose to show,
/// because nothing makes an accepted push say it again:
/// a push of what the
///  remote already holds prints `Everything up-to-date` and runs no hook,
/// so a line cut here is a line lost
///  (`SPEC.md`,
/// "Reconcile and bounds").
///  The record is replaced in one rename,
/// so a kill while a second try writes its record leaves the first record whole
///  rather than none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordPushReport(gitDir, { branchRef, commit, said })
/// ```
pub fn record_push_report(git_dir: &Path, branch_ref: &str, commit: &str, said: &str) -> Result<(), String> {
    let directory: PathBuf = notice_directory(git_dir);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!("creating the notice directory failed: {error}"));
    }
    let path: PathBuf = directory.join(report_name(branch_ref, commit));
    let text: String = format!("{REPORT_MAGIC}\n{BRANCH_FIELD}{branch_ref}\n{COMMIT_FIELD}{commit}\n\n{}", clean_words(said));
    return match replace_private_file_unsynced(path.as_path(), text.as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("writing the notice failed: {error}")),
    };
}

/// Record controls stay out of the release executable.
#[cfg(test)]
#[path = "push_failure_notice_reports_tests.rs"]
mod tests;
