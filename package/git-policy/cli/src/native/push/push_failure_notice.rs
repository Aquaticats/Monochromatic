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
//! the whole commit name,
//! what the push came to and Git's words with credentials already masked;
//! the wrapper words the notice when it
//!  shows it,
//! so the commit name is abbreviated the way Git abbreviates it in the repository as it is then.
//! A push Git refused delivered nothing,
//! and a push that was stopped or gave up for a reason that can pass may or may
//!  not have reached the remote;
//! the two are worded apart,
//! and which one it was is a fact of the push's own outcome,
//!  never of the words Git and the remote printed,
//! because a remote's hook controls those.
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

use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::auto_push_branch_key::encode_branch_key;
/// The words the notice is made of,
/// the outcome it is worded from,
/// and Git's cleaned words.
pub use crate::push_failure_notice_wording::{
    LONGEST_REASON, NO_REASON, PushOutcome, clean_reason, worded, worded_lost, worded_skipped,
};
/// The masking a control compares Git's words against.
pub use crate::lfs_output_mask::mask_userinfo_text;
/// The note a failed push ends with,
/// which a control compares the notice against.
pub use crate::auto_push::PUSH_FAILED_NOTE;
pub use crate::push_failure_notice_machine_output::is_machine_read;
use crate::private_storage::{create_private_directory_all, write_private_file_unsynced};
use crate::wrapper_names_old_and_new::DATA_FOLDER;
use std::path::{Path, PathBuf};

/// Directory under the worktree's own Git directory that holds the records for the person.
pub const NOTICE_DIRECTORY: &str = "monochromatic-git-policy-cli/notices";

/// Directory under the repository's common Git directory that holds the records of removed worktrees,
/// which any worktree's
///  command shows because the worktree they were made for is gone.
pub const ORPHAN_NOTICE_DIRECTORY: &str = "monochromatic-git-policy-cli/orphaned-notices";

/// First line of a record of a push that was lost with its worktree;
/// its fields are those of a failure record,
/// without words.
const LOST_MAGIC: &str = "monochromatic-git-policy-cli push lost 1";

/// Separates the parts of a record's name;
/// it is not a character a branch key can contain.
const NAME_SEPARATOR: char = '+';

/// The longest branch key a record name carries whole;
/// a longer key is cut and tagged with a hash of the whole key,
/// so the
///  name stays a valid file name and two long branch names do not share records.
///  Exempt from a citation under `LQM` (class:
/// display cap):
/// it limits how much text a message,
/// log field or record
///  names,
/// and no command result or duration depends on it.
/// Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`,
/// "Exempt limits";
/// the coordinator's pick under the owner's delegation,
/// open to the owner's veto.
const LONGEST_KEY: usize = 100;


/// First line of a record of a push Git refused,
/// whose parts are fields;
/// a file without it is the finished text an earlier release wrote.
const RECORD_MAGIC: &str = "monochromatic-git-policy-cli push notice 1";

/// First line of a record of a push that did not finish,
/// so whether the remote took the commit is not known,
/// and that the service keeps trying.
const UNFINISHED_MAGIC: &str = "monochromatic-git-policy-cli push unfinished 1";

/// First line of a record of a push that did not finish and that nothing will try again by itself.
const UNFINISHED_NO_RETRY_MAGIC: &str = "monochromatic-git-policy-cli push unfinished-no-retry 1";

/// First line of a record of a push the service did not make because the branch has no destination it can name;
/// its fields are
///  those of a failure record,
/// and its reason is the service's own words.
const SKIPPED_MAGIC: &str = "monochromatic-git-policy-cli push skipped 1";

/// The field names of a record's header lines.
const BRANCH_FIELD: &str = "branch: ";
const COMMIT_FIELD: &str = "commit: ";



/// The directory that holds the records of a repository.
pub fn notice_directory(git_dir: &Path) -> PathBuf {
    return path_for_reading(git_dir, NOTICE_DIRECTORY);
}

/// A 64-bit FNV-1a hash of the text,
/// in hexadecimal.
pub fn fnv_hex(text: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in text.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x0100_0000_01b3);
    }
    return format!("{hash:016x}");
}

/// The part of every record name that belongs to one branch.
fn branch_prefix(branch_ref: &str) -> String {
    let key: String = encode_branch_key(branch_ref);
    let shown: String = if key.len() > LONGEST_KEY { format!("{}~{}", &key[..LONGEST_KEY], fnv_hex(key.as_str())) } else { key };
    return format!("push{NAME_SEPARATOR}{shown}{NAME_SEPARATOR}");
}

/// The file name of the record of one failed push.
pub fn record_name(branch_ref: &str, commit: &str) -> String {
    return format!("{}{commit}.txt", branch_prefix(branch_ref));
}




/// What:
///  The text shown for a failed backup push,
/// from Git's raw words.
/// Why:
///   The commit is named as the caller gives it,
/// so a caller that has the repository passes Git's own abbreviation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pushFailureText({ branchRef, commit, outcome, said })
/// ```
pub fn push_failure_text(branch_ref: &str, commit: &str, outcome: PushOutcome, said: &str) -> String {
    return worded(branch_ref, commit, outcome, clean_reason(said).as_str());
}


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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordPushSkip(gitDir, { branchRef, commit, said })
/// ```
pub fn record_push_skip(git_dir: &Path, branch_ref: &str, commit: &str, said: &str) -> Result<(), String> {
    let directory: PathBuf = notice_directory(git_dir);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!("creating the notice directory failed: {error}"));
    }
    let path: PathBuf = directory.join(record_name(branch_ref, commit));
    let _removed = std::fs::remove_file(path.as_path());
    let text: String = format!("{SKIPPED_MAGIC}\n{BRANCH_FIELD}{branch_ref}\n{COMMIT_FIELD}{commit}\n\n{}", clean_reason(said));
    return match write_private_file_unsynced(path.as_path(), text.as_bytes()) {
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordPushFailure(gitDir, { branchRef, commit, outcome, said })
/// ```
pub fn record_push_failure(git_dir: &Path, branch_ref: &str, commit: &str, outcome: PushOutcome, said: &str) -> Result<(), String> {
    let directory: PathBuf = notice_directory(git_dir);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!("creating the notice directory failed: {error}"));
    }
    let path: PathBuf = directory.join(record_name(branch_ref, commit));
    // A repeat of the same failure replaces its record.
    let _removed = std::fs::remove_file(path.as_path());
    let magic: &str = match outcome {
        PushOutcome::Refused => RECORD_MAGIC,
        PushOutcome::Unfinished => UNFINISHED_MAGIC,
        PushOutcome::UnfinishedWithNoRetry => UNFINISHED_NO_RETRY_MAGIC,
    };
    let text: String = format!("{magic}\n{BRANCH_FIELD}{branch_ref}\n{COMMIT_FIELD}{commit}\n\n{}", clean_reason(said));
    return match write_private_file_unsynced(path.as_path(), text.as_bytes()) {
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
pub fn record_lost_push(common_dir: &Path, branch_ref: &str, commit: &str) -> Result<(), String> {
    let directory: PathBuf = common_dir.join(ORPHAN_NOTICE_DIRECTORY);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!("creating the notice directory failed: {error}"));
    }
    let path: PathBuf = directory.join(record_name(branch_ref, commit));
    if path.exists() {
        return Ok(());
    }
    let text: String = format!("{LOST_MAGIC}\n{BRANCH_FIELD}{branch_ref}\n{COMMIT_FIELD}{commit}\n\n");
    return match write_private_file_unsynced(path.as_path(), text.as_bytes()) {
        Err(error) if error.kind() != std::io::ErrorKind::AlreadyExists => Err(format!("writing the notice failed: {error}")),
        Ok(()) | Err(_) => Ok(()),
    };
}

/// What a record says besides its branch and commit.
enum RecordKind<'a> {
    /// A push Git refused,
    /// with Git's cleaned words.
    Failed(&'a str),
    /// A push that did not finish,
    /// so whether the remote took the commit is not known,
    /// with Git's cleaned words.
    Unfinished(&'a str),
    /// A push that did not finish and that nothing will try again by itself,
    /// with Git's cleaned words.
    UnfinishedWithNoRetry(&'a str),
    /// A push the service did not make,
    /// with its cleaned words.
    Skipped(&'a str),
    /// A push lost with its worktree.
    Lost,
}

/// The line of a record in the earlier spelling of the data folder,
/// from what follows the current spelling.
fn earlier_spelling(rest: &str) -> String {
    return format!("{}{rest}", DATA_FOLDER.earlier);
}

/// What:
///  The current first line a record's first line stands for:
/// the line itself,
/// or the same line in the spelling an earlier
///  release wrote (`cli-git push notice 1`).
/// Why:
///   The folder of the records is moved to its current name in place,
/// so records written before the rename are still there with
///  their own first lines,
/// and they are records of the same fields.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const currentMagic = (line: string) => MAGICS.find((magic) => line === magic || line === magic.replace(CURRENT, EARLIER))
/// ```
fn current_magic_of(line: &str) -> Option<&'static str> {
    for current in [RECORD_MAGIC, UNFINISHED_MAGIC, UNFINISHED_NO_RETRY_MAGIC, SKIPPED_MAGIC, LOST_MAGIC] {
        let earlier: Option<String> = current.strip_prefix(DATA_FOLDER.current).map(earlier_spelling);
        if line == current || earlier.as_deref() == Some(line) {
            return Some(current);
        }
    }
    return None;
}

/// The branch,
/// the commit and the kind of a record written by this release or by the release before the rename.
fn parse_record(text: &str) -> Option<(&str, &str, RecordKind<'_>)> {
    let mut lines = text.splitn(5, '\n');
    let magic: &str = current_magic_of(lines.next()?)?;
    let branch: &str = lines.next()?.strip_prefix(BRANCH_FIELD)?;
    let commit: &str = lines.next()?.strip_prefix(COMMIT_FIELD)?;
    if !lines.next()?.is_empty() {
        return None;
    }
    let reason: &str = lines.next().unwrap_or("");
    return Some((branch, commit, match magic {
        LOST_MAGIC => RecordKind::Lost,
        SKIPPED_MAGIC => RecordKind::Skipped(reason),
        UNFINISHED_MAGIC => RecordKind::Unfinished(reason),
        UNFINISHED_NO_RETRY_MAGIC => RecordKind::UnfinishedWithNoRetry(reason),
        _ => RecordKind::Failed(reason),
    }));
}

/// What:
///  Removes the records of one branch,
/// after a push of that branch succeeded.
/// Why:
///   A successful push of the branch carries every earlier commit of it,
/// so an earlier failure is no longer true.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await clearPushFailures(gitDir, branchRef)
/// ```
pub fn clear_push_failures(git_dir: &Path, branch_ref: &str) {
    clear_push_failures_before(git_dir, branch_ref, std::time::SystemTime::now());
}

/// What:
///  Removes the records of one branch that were written no later than `cutoff`,
/// after a push of that branch succeeded.
/// Why:
///   A push carries the commits that existed when it began.
/// A record written after that moment is about a newer commit,
/// which
///  another instance of the service may have pushed and failed to push while this push's result was still being handled;
/// the
///  older push says nothing about it,
/// so it stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await clearPushFailuresBefore(gitDir, branchRef, pushBegan)
/// ```
pub fn clear_push_failures_before(git_dir: &Path, branch_ref: &str, cutoff: std::time::SystemTime) {
    let Ok(listing) = std::fs::read_dir(notice_directory(git_dir)) else { return };
    let prefix: String = branch_prefix(branch_ref);
    for entry in listing.flatten() {
        let older: bool = match entry.metadata() {
            Ok(meta) => match meta.modified() {
                Ok(written) => written <= cutoff,
                Err(_) => false,
            },
            Err(_) => false,
        };
        if older && entry.file_name().to_string_lossy().starts_with(prefix.as_str()) {
            let _removed = std::fs::remove_file(entry.path());
        }
    }
}

/// Where the worktrees of a repository stand:
/// the branch the asking worktree has checked out and every branch any
///  worktree has checked out.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Worktrees {
    /// The branch reference the command's worktree has checked out;
    /// nothing for a detached head or an unknown worktree.
    pub here: Option<String>,
    /// The branch references checked out in every worktree of the repository.
    pub all: Vec<String>,
}

/// What:
///  The branch reference a `HEAD` file names,
/// or nothing for a detached head or an unreadable file.
/// Why:
///   Only the plain `ref: refs/heads/<name>` form names a branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// branchOfHead(gitDir)
/// ```
pub fn branch_of_head(git_dir: &Path) -> Option<String> {
    let text: String = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let target: &str = text.lines().next()?.strip_prefix("ref: ")?.trim();
    if target.starts_with("refs/heads/") {
        return Some(target.to_string());
    }
    return None;
}

/// What:
///  The branches checked out in the repository's worktrees:
/// the main one and each linked one.
/// Why:
///   A linked worktree keeps its own `HEAD` under `worktrees/<name>`;
/// the main worktree's is the common directory's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// checkedOutBranches(commonDir)
/// ```
pub fn checked_out_branches(common_dir: &Path) -> Vec<String> {
    let mut branches: Vec<String> = branch_of_head(common_dir).into_iter().collect();
    if let Ok(listing) = std::fs::read_dir(common_dir.join("worktrees")) {
        for entry in listing.flatten() {
            branches.extend(branch_of_head(entry.path().as_path()));
        }
    }
    return branches;
}

/// Whether a record of `branch_ref` is for the asking worktree:
/// it has the branch checked out,
/// or no worktree has.
fn is_for(worktrees: &Worktrees, branch_ref: &str) -> bool {
    if worktrees.here.as_deref() == Some(branch_ref) {
        return true;
    }
    for held in &worktrees.all {
        if held == branch_ref {
            return false;
        }
    }
    return true;
}

/// What:
///  Where a failure notice asks for the name of a commit as the person should read it.
///  A `trait` is a named set of methods that different types can each supply,
/// like a TypeScript `interface`.
/// Why:
///   The name is Git's abbreviation in the repository as it is when the notice is shown,
/// asked of Git once per shown notice;
/// the
///  state the question needs (the Git context,
/// the directories) is a small struct that implements this.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// interface CommitNamer { nameOf(commit: string): string }
/// ```
pub trait CommitNamer {
    /// The name to show for `commit`.
    fn name_of(&self, commit: &str) -> String;
}

/// What:
///  Reads and removes the records that are for the asking worktree (all of them when no worktree is given),
/// oldest name
///  first,
/// worded with the commit named by `abbreviate`.
/// Why:
///   A record is shown once,
/// to the worktree whose commit it is about;
/// another worktree's command leaves it where it is.
///  When nothing failed the notice directory does not exist,
/// and this is one directory read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = await takePushFailuresFor(gitDir, worktrees, abbreviate)
/// ```
pub fn take_push_failures_for(git_dir: &Path, worktrees: Option<&Worktrees>, abbreviate: &dyn CommitNamer) -> Vec<String> {
    return take_records_from(notice_directory(git_dir).as_path(), worktrees, abbreviate);
}

/// What:
///  Reads and removes the records of removed worktrees,
/// which belong to no worktree and so are shown to the first command
///  that asks.
/// Why:
///   The worktree a record was made for is gone,
/// so nobody else could ever be shown it;
/// one directory read when none exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = await takeOrphanedFailures(commonDir, abbreviate)
/// ```
pub fn take_orphaned_failures(common_dir: &Path, abbreviate: &dyn CommitNamer) -> Vec<String> {
    return take_records_from(common_dir.join(ORPHAN_NOTICE_DIRECTORY).as_path(), None, abbreviate);
}

/// Reads and removes the records in one directory that are for the asking worktree,
/// oldest name first.
fn take_records_from(directory: &Path, worktrees: Option<&Worktrees>, abbreviate: &dyn CommitNamer) -> Vec<String> {
    let Ok(listing) = std::fs::read_dir(directory) else { return Vec::new() };
    let mut paths: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        paths.push(entry.path());
    }
    paths.sort();
    let mut texts: Vec<String> = Vec::new();
    for path in paths {
        let Ok(text) = std::fs::read_to_string(path.as_path()) else { continue };
        let parsed: Option<(&str, &str, RecordKind<'_>)> = parse_record(text.as_str());
        if let (Some((branch, _, _)), Some(asking)) = (&parsed, worktrees)
            && !is_for(asking, branch)
        {
            continue;
        }
        // The record is removed before its text is used, so two commands that start together cannot both show it.
        if std::fs::remove_file(path.as_path()).is_err() {
            continue;
        }
        texts.push(match parsed {
            Some((branch, commit, RecordKind::Failed(reason))) => worded(branch, abbreviate.name_of(commit).as_str(), PushOutcome::Refused, reason),
            Some((branch, commit, RecordKind::Unfinished(reason))) => worded(branch, abbreviate.name_of(commit).as_str(), PushOutcome::Unfinished, reason),
            Some((branch, commit, RecordKind::UnfinishedWithNoRetry(reason))) => {
                worded(branch, abbreviate.name_of(commit).as_str(), PushOutcome::UnfinishedWithNoRetry, reason)
            }
            Some((branch, commit, RecordKind::Skipped(reason))) => worded_skipped(branch, abbreviate.name_of(commit).as_str(), reason),
            Some((branch, commit, RecordKind::Lost)) => worded_lost(branch, abbreviate.name_of(commit).as_str()),
            // The finished text an earlier release wrote.
            None => text,
        });
    }
    return texts;
}

/// What:
///  Reads and removes every record,
/// with each commit named in full.
/// Why:
///   The service's own controls and a repository with no worktree to ask need all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = await takePushFailures(gitDir)
/// ```
pub fn take_push_failures(git_dir: &Path) -> Vec<String> {
    return take_push_failures_for(git_dir, None, &FullCommitName);
}

/// A commit name as it is,
/// for the callers that show commits in full.
struct FullCommitName;

impl CommitNamer for FullCommitName {
    fn name_of(&self, commit: &str) -> String {
        return commit.to_string();
    }
}


/// Record controls stay out of the release executable.
#[cfg(test)]
#[path = "push_failure_notice_tests.rs"]
mod tests;
