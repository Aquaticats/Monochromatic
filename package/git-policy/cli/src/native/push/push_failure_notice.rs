//! What:
//!  The record of a backup push that failed after its commit was saved,
//! and the rule for when the wrapper shows it:
//!  the resident service writes one record per failed push under the Git directory of the worktree whose commit it was,
//! a
//!  later wrapper command in that worktree shows each record once on standard error,
//! and a later push of the same branch
//!  that succeeds removes the records of that branch.
//! Why:
//!  The push runs in the service after the commit command returned,
//! so nothing is attached to the command that could
//!  print its failure;
//! without a record the person would never learn that their backup push was refused
//!  (`doc/planning/cli-git-resident-service.md`,
//! "After-commit work").
//! Records are plain files in the worktree's
//!  own Git directory so the check on every command is one directory read that finds nothing when nothing failed.
//! A record holds
//!  the branch,
//! the whole commit name and Git's words with credentials already masked;
//! the wrapper words the notice when it
//!  shows it,
//! so the commit name is abbreviated the way Git abbreviates it in the repository as it is then.
//! A record is removed only once its words have been
//!  written out,
//! and only while the file still holds the text that was worded,
//! so a command killed before it prints loses nothing and a
//!  record the service replaced meanwhile is kept for the person.
//! Records are
//!  shown on standard error only,
//! and not at all to a command whose output a program reads (`--porcelain`,
//! `-z`,
//!  plumbing),
//! so that program neither sees the text in its output nor consumes the record before a person can read it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordPushFailure(gitDir, { branchRef, oid, reason }); stderr.write(await takePushFailures(gitDir));
//! ```

/// Publish the step that removes a branch's records once a push of it succeeded.
pub use super::push_failure_notice_cleared_after_a_push::{
    clear_push_failures,
    clear_push_failures_before,
};
/// Import the reading of a record's lines back into its kind,
/// and the first line of a report of a push that succeeded,
/// which the reading compares against.
use super::push_failure_notice_record_parsed::parse_record;
/// Import what only the controls read:
/// the display cap they cut to and the masking whose result they assert on.
#[cfg(test)]
pub(crate) use super::push_failure_notice_record_text::LONGEST_REASON;
/// Import the part of a record's name that belongs to one branch,
/// which the controls read.
#[cfg(test)]
use super::push_failure_notice_record_text::branch_prefix;
/// Publish the field names and the caps another module of this folder writes records with.
pub(crate) use super::push_failure_notice_record_text::{
    BRANCH_FIELD,
    COMMIT_FIELD,
    LONGEST_KEY,
    NAME_SEPARATOR,
    NO_REASON,
};
/// Import the first lines a record can carry,
/// the kinds they name,
/// and the record's name in its directory.
use super::push_failure_notice_record_text::{
    LOST_MAGIC,
    RECORD_MAGIC,
    RecordKind,
    SKIPPED_MAGIC,
    UNFINISHED_MAGIC,
    UNFINISHED_NO_RETRY_MAGIC,
    file_name_of,
};
/// Publish the names of a record's directories and of its files,
/// and the words it carries,
/// under the names a caller already reads them by.
pub use super::push_failure_notice_record_text::{
    NOTICE_DIRECTORY,
    ORPHAN_NOTICE_DIRECTORY,
    clean_reason,
    clean_words,
    fnv_hex,
    notice_directory,
    record_name,
};
/// Publish the branches each worktree has checked out,
/// and the way a commit is named for the person.
pub use super::push_failure_notice_worktrees::{
    CommitNamer,
    Worktrees,
    branch_of_head,
    checked_out_branches,
};
/// Import the rule that says a record is the asking worktree's.
use super::push_failure_notice_worktrees::{
    FullCommitName,
    is_for,
};
#[cfg(test)]
use crate::lfs_output_mask::mask_url_secrets_text;
/// Import the private storage a record is written through,
/// which makes the directory and replaces the file in one rename.
use crate::private_storage::{
    create_private_directory_all,
    replace_private_file_unsynced,
    write_private_file_unsynced,
};
/// Import the name a command holds a record under while it shows it,
/// and who may take one over.
use crate::push_failure_notice_claimed_by_one_command::{
    ClaimToken,
    claim,
    claimed_by_a_live_command,
    give_back,
    record_of_claim,
    take_over,
};
/// Publish whether a command's output is read by a program,
/// so that no notice is shown to it.
pub use crate::push_failure_notice_machine_output::is_machine_read;
use crate::push_failure_notice_reports::REPORT_MAGIC;
/// Publish the record of what a push that succeeded did,
/// and the first line that tells it from a failure.
pub use crate::push_failure_notice_reports::record_push_report;
/// Publish one record as a command holds it before it shows it,
/// with the claim it was taken under and the step that removes that claim.
pub use crate::push_failure_notice_shown_then_removed::Notice;
/// Publish the words a notice carries,
/// the text of the failed backup push,
/// and what the push came to.
pub use crate::push_failure_notice_words::{
    PushOutcome,
    push_failure_text,
};
pub(crate) use crate::push_failure_notice_words::{
    worded,
    worded_lost,
    worded_report,
    worded_skipped,
};
/// Import the borrowed and owned path types a record's directory is named with.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Leaves the record of one push the service did not make,
/// replacing an earlier record of the same commit on the same branch.
/// Why:
///   The service settles such a push as done because nothing is left for it to retry,
/// and the person would otherwise never
///  learn that the backup does not exist.
/// The name is the failure record's,
/// so a later successful push of the branch removes
///  it with the failures.
/// The record is replaced in one rename,
/// so a kill between one try and the next leaves the record that was there
///  rather than none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordPushSkip(gitDir, { branchRef, commit, said })
/// ```
pub fn record_push_skip(
    git_dir: &Path,
    branch_ref: &str,
    commit: &str,
    said: &str,
) -> Result<(), String> {
    let directory: PathBuf = notice_directory(git_dir);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!("creating the notice directory failed: {error}"));
    }
    let path: PathBuf = directory.join(record_name(branch_ref, commit));
    let text: String = format!(
        "{SKIPPED_MAGIC}\n{BRANCH_FIELD}{branch_ref}\n{COMMIT_FIELD}{commit}\n\n{}",
        clean_reason(said)
    );
    return match replace_private_file_unsynced(path.as_path(), text.as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("writing the notice failed: {error}")),
    };
}

/// What:
///  Leaves the record of one failed push,
/// replacing an earlier record of the same commit on the same branch.
/// Why:
///   The name is derived from the branch and the commit,
/// so a push that is tried again and fails again leaves one record.
///  The reason is cleaned before it is written,
/// so no credential reaches the disk.
/// The record is replaced in one rename,
/// so a kill while a second try writes its record leaves the first record whole
///  rather than none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordPushFailure(gitDir, { branchRef, commit, outcome, said })
/// ```
pub fn record_push_failure(
    git_dir: &Path,
    branch_ref: &str,
    commit: &str,
    outcome: PushOutcome,
    said: &str,
) -> Result<(), String> {
    let directory: PathBuf = notice_directory(git_dir);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!("creating the notice directory failed: {error}"));
    }
    let path: PathBuf = directory.join(record_name(branch_ref, commit));
    let magic: &str = match outcome {
        PushOutcome::Refused => RECORD_MAGIC,
        PushOutcome::Unfinished => UNFINISHED_MAGIC,
        PushOutcome::UnfinishedWithNoRetry => UNFINISHED_NO_RETRY_MAGIC,
    };
    let text: String = format!(
        "{magic}\n{BRANCH_FIELD}{branch_ref}\n{COMMIT_FIELD}{commit}\n\n{}",
        clean_reason(said)
    );
    return match replace_private_file_unsynced(path.as_path(), text.as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) => Err(format!("writing the notice failed: {error}")),
    };
}

/// What:
///  Leaves the record of a push that was lost with its worktree,
/// unless a record of that commit on that branch is already
///  there.
/// Why:
///   An existing record holds Git's own words about a failed push,
/// which say more than this one does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordLostPush(commonDir, { branchRef, commit })
/// ```
pub fn record_lost_push(
    common_dir: &Path,
    branch_ref: &str,
    commit: &str,
) -> Result<(), String> {
    let directory: PathBuf = common_dir.join(ORPHAN_NOTICE_DIRECTORY);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!("creating the notice directory failed: {error}"));
    }
    let path: PathBuf = directory.join(record_name(branch_ref, commit));
    if path.exists() {
        return Ok(());
    }
    let text: String =
        format!("{LOST_MAGIC}\n{BRANCH_FIELD}{branch_ref}\n{COMMIT_FIELD}{commit}\n\n");
    return match write_private_file_unsynced(path.as_path(), text.as_bytes()) {
        Err(error) if error.kind() != std::io::ErrorKind::AlreadyExists => {
            Err(format!("writing the notice failed: {error}"))
        }
        Ok(()) | Err(_) => Ok(()),
    };
}

/// What:
///  Reads the records that are for the asking worktree (all of them when no worktree is given),
/// oldest name
///  first,
/// words each with the commit named by `abbreviate`,
/// and answers what the person is to be shown.
/// Why:
///   A record is shown once,
/// to the worktree whose commit it is about;
/// another worktree's command leaves it where it is,
/// and nothing is
///  removed here:
/// the removal waits until the words are out,
/// which only the caller knows.
///  When nothing failed the notice directory does not exist,
/// and this is one directory read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await readPushFailuresFor(gitDir, worktrees, abbreviate)
/// ```
pub fn push_failures_for(
    git_dir: &Path,
    worktrees: Option<&Worktrees>,
    abbreviate: &dyn CommitNamer,
) -> Vec<Notice> {
    return read_records_from(notice_directory(git_dir).as_path(), worktrees, abbreviate);
}

/// What:
///  Reads the records of removed worktrees,
/// which belong to no worktree and so are shown to the first command
///  that asks.
/// Why:
///   The worktree a record was made for is gone,
/// so nobody else could ever be shown it;
/// one directory read when none exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await readOrphanedPushFailures(commonDir, abbreviate)
/// ```
pub fn orphaned_push_failures(
    common_dir: &Path,
    abbreviate: &dyn CommitNamer,
) -> Vec<Notice> {
    return read_records_from(
        common_dir.join(ORPHAN_NOTICE_DIRECTORY).as_path(),
        None,
        abbreviate,
    );
}

/// Reads the records in one directory that are for the asking worktree,
/// oldest name first,
/// without removing any.
fn read_records_from(
    directory: &Path,
    worktrees: Option<&Worktrees>,
    abbreviate: &dyn CommitNamer,
) -> Vec<Notice> {
    let Ok(listing) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let token: ClaimToken = ClaimToken::of_this_command();
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        paths.push(entry.path());
    }
    paths.sort();
    let mut notices: Vec<Notice> = Vec::new();
    for path in paths {
        let name: String = file_name_of(path.as_path());
        // A claim belongs to the command that made it: one whose command still runs is left alone, and one whose command is
        //  gone is taken over, which is how a notice a killed command held reaches the person.
        if record_of_claim(name.as_str()).is_some() {
            if claimed_by_a_live_command(name.as_str()) {
                continue;
            }
            let Some(claimed) = take_over(path.as_path(), &token) else {
                continue;
            };
            notices.extend(word_records_at(claimed.as_path(), worktrees, abbreviate));
            continue;
        }
        // A notice that is a named pipe answers as unreadable at once instead of holding the command that shows notices.
        let Ok(text) = crate::plain_file::read_plain_text(path.as_path()) else {
            continue;
        };
        let parsed: Option<(&str, &str, RecordKind<'_>)> =
            parse_record(text.as_str(), REPORT_MAGIC);
        if let (Some((branch, _, _)), Some(asking)) = (&parsed, worktrees)
            && !is_for(asking, branch)
        {
            continue;
        }
        // The record is claimed before its words are made: two commands that start together both read it, and only the one
        //  whose rename wins shows it. A command killed while it words the notice leaves the claim, which a later command takes
        //  over once this one is gone.
        let Some(claimed) = claim(path.as_path(), &token) else {
            continue;
        };
        notices.extend(word_records_at(claimed.as_path(), worktrees, abbreviate));
    }
    return notices;
}

/// What:
///  The notice the claim at `path` carries,
/// worded,
/// or nothing when the claim holds no record this command reads.
/// Why:
///   The claim is read after the rename,
/// so the words are made from exactly the text this command holds;
/// a claim that
///  cannot be read is left where it is for a later command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const text = await readFile(claim); const parsed = parseRecord(text); return parsed ? worded(parsed) : text
/// ```
fn word_records_at(
    path: &Path,
    worktrees: Option<&Worktrees>,
    abbreviate: &dyn CommitNamer,
) -> Vec<Notice> {
    let Ok(text) = crate::plain_file::read_plain_text(path) else {
        return Vec::new();
    };
    let parsed: Option<(&str, &str, RecordKind<'_>)> = parse_record(text.as_str(), REPORT_MAGIC);
    if let (Some((branch, _, _)), Some(asking)) = (&parsed, worktrees)
        && !is_for(asking, branch)
    {
        // A claim of another worktree's branch goes back to its own name, where that worktree's command reads it.
        let record: String = file_name_of(path);
        let record_name: &str = record_of_claim(record.as_str()).unwrap_or(record.as_str());
        give_back(path, path.with_file_name(record_name).as_path());
        return Vec::new();
    }
    let shown: String = match parsed {
        Some((branch, commit, RecordKind::Failed(reason))) => worded(
            branch,
            abbreviate.name_of(commit).as_str(),
            PushOutcome::Refused,
            reason,
        ),
        Some((branch, commit, RecordKind::Unfinished(reason))) => worded(
            branch,
            abbreviate.name_of(commit).as_str(),
            PushOutcome::Unfinished,
            reason,
        ),
        Some((branch, commit, RecordKind::UnfinishedWithNoRetry(reason))) => worded(
            branch,
            abbreviate.name_of(commit).as_str(),
            PushOutcome::UnfinishedWithNoRetry,
            reason,
        ),
        Some((branch, commit, RecordKind::Skipped(reason))) => {
            worded_skipped(branch, abbreviate.name_of(commit).as_str(), reason)
        }
        Some((branch, commit, RecordKind::Reported(reason))) => {
            worded_report(branch, abbreviate.name_of(commit).as_str(), reason)
        }
        Some((branch, commit, RecordKind::Lost)) => {
            worded_lost(branch, abbreviate.name_of(commit).as_str())
        }
        // The finished text an earlier release wrote.
        None => text.clone(),
    };
    let record: String = file_name_of(path);
    let record_name: &str = record_of_claim(record.as_str()).unwrap_or(record.as_str());
    return vec![Notice {
        record: path.with_file_name(record_name),
        claim: path.to_path_buf(),
        text: shown,
    }];
}

/// What:
///  Reads every record,
/// with each commit named in full.
/// Why:
///   The service's own controls and a repository with no worktree to ask need all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const notices = await readPushFailures(gitDir)
/// ```
pub fn push_failures(git_dir: &Path) -> Vec<Notice> {
    return push_failures_for(git_dir, None, &FullCommitName);
}

/// Record controls stay out of the release executable.
#[cfg(test)]
#[path = "push_failure_notice_tests.rs"]
mod tests;
