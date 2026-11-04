//! What:
//!  The record of a backup push that failed after its commit was saved, and the rule for when the wrapper shows it:
//!  the resident service writes one record per failed push under the Git directory of the worktree whose commit it was, a
//!  later wrapper command in that worktree shows each record once on standard error, and a later push of the same branch
//!  that succeeds removes the records of that branch.
//! Why:
//!  The push runs in the service after the commit command returned, so nothing is attached to the command that could
//!  print its failure; without a record the person would never learn that their backup push was refused
//!  (`doc/planning/cli-git-resident-service.md`, "After-commit work"). Records are plain files in the worktree's
//!  own Git directory so the check on every command is one directory read that finds nothing when nothing failed. A record holds
//!  the branch, the whole commit name and Git's words with credentials already masked; the wrapper words the notice when it
//!  shows it, so the commit name is abbreviated the way Git abbreviates it in the repository as it is then. Records are
//!  shown on standard error only, and not at all to a command whose output a program reads (`--porcelain`, `-z`,
//!  plumbing), so that program neither sees the text in its output nor consumes the record before a person can read it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordPushFailure(gitDir, { branchRef, oid, reason }); stderr.write(await takePushFailures(gitDir));
//! ```

use super::auto_push::PUSH_FAILED_NOTE;
use super::auto_push_branch_key::encode_branch_key;
use super::lfs_output_mask::mask_userinfo_text;
use super::private_storage::{create_private_directory_all, write_private_file_unsynced};
use super::service_messages::RESTART_COMMAND;
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Directory under the worktree's own Git directory that holds the records for the person.
pub const NOTICE_DIRECTORY: &str = "cli-git/notices";

/// Directory under the repository's common Git directory that holds the records of removed worktrees, which any worktree's
///  command shows because the worktree they were made for is gone.
pub const ORPHAN_NOTICE_DIRECTORY: &str = "cli-git/orphaned-notices";

/// First line of a record of a push that was lost with its worktree; its fields are those of a failure record, without words.
const LOST_MAGIC: &str = "cli-git push lost 1";

/// Separates the parts of a record's name; it is not a character a branch key can contain.
const NAME_SEPARATOR: char = '+';

/// The longest branch key a record name carries whole; a longer key is cut and tagged with a hash of the whole key, so the
///  name stays a valid file name and two long branch names do not share records.
///  Exempt from a citation under `LQM` (class: display cap): it limits how much text a message, log field or record
///  names, and no command result or duration depends on it. Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`, "Exempt limits"; the coordinator's pick under the owner's delegation, open to the owner's veto.
const LONGEST_KEY: usize = 100;

/// The most bytes of Git's own words a record keeps.
///  Exempt from a citation under `LQM` (class: display cap): it limits how much text a message, log field or record
///  names, and no command result or duration depends on it. Classed once in `doc/planning/cli-git-shipped-limits-
///  audit.md`, "Exempt limits"; the coordinator's pick under the owner's delegation, open to the owner's veto.
const LONGEST_REASON: usize = 4000;

/// First line of a record that holds its parts as fields; a file without it is the finished text an earlier release wrote.
const RECORD_MAGIC: &str = "cli-git push notice 1";

/// First line of a record of a push the service did not make because the branch has no destination it can name; its fields are
///  those of a failure record, and its reason is the service's own words.
const SKIPPED_MAGIC: &str = "cli-git push skipped 1";

/// The field names of a record's header lines.
const BRANCH_FIELD: &str = "branch: ";
const COMMIT_FIELD: &str = "commit: ";

/// Said when Git gave no words at all.
const NO_REASON: &str = "Git gave no reason.";

/// The directory that holds the records of a repository.
pub fn notice_directory(git_dir: &Path) -> PathBuf {
    return git_dir.join(NOTICE_DIRECTORY);
}

/// A 64-bit FNV-1a hash of the text, in hexadecimal.
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
///  Git's words for the person: the user name and password of every URL masked, control characters dropped, and the
///  length bounded.
/// Why:
///   The text came from a remote and from Git, and is printed on a terminal; a URL with a user name and password, or an
///  escape sequence, must not reach the screen or a log of it. Only the credentials are masked, never the line around them,
///  because the line often says why the push failed (the wrapper's own Git LFS failure line names the endpoint setting and
///  carries the exit status).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanReason(said)
/// ```
pub fn clean_reason(said: &str) -> String {
    let masked: String = mask_userinfo_text(said);
    let mut kept: Vec<String> = Vec::new();
    for line in masked.lines() {
        let plain: String = line.chars().filter(|character| { return *character == '\t' || !character.is_control(); }).collect();
        let trimmed: &str = plain.trim_end();
        if !trimmed.is_empty() {
            kept.push(trimmed.to_string());
        }
    }
    let mut text: String = kept.join("\n");
    if text.len() > LONGEST_REASON {
        let mut cut: usize = LONGEST_REASON;
        while !text.is_char_boundary(cut) {
            cut -= 1;
        }
        text.truncate(cut);
        text.push_str("\n(Git's output is cut here.)");
    }
    return text;
}

/// What:
///  The text shown for a failed backup push, with the commit named as given and Git's words already cleaned.
/// Why:
///   It names the branch and the commit, gives Git's own reason, says the commit is safe on this computer, and says what to
///  do; the service runs with its own settings, which is the first thing to check when a push that works in a terminal
///  fails here. The command that shows it may be anyone's, so it never calls the commit the reader's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worded({ branchRef, shown, reason })
/// ```
fn worded(branch_ref: &str, shown: &str, reason: &str) -> String {
    let branch: &str = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
    let detail: String = if reason.is_empty() {
        NO_REASON.to_string()
    } else {
        format!("Git said:\n{}", reason.lines().map(|line| { return format!("    {line}"); }).collect::<Vec<String>>().join("\n"))
    };
    return format!(
        "The backup push of branch {branch} failed for commit {shown}, so that commit exists only on this computer for now. \
         It is saved in this repository and nothing was lost.\n{detail}\n\
         Fix what Git names, then run `git push`. If it says the remote has commits you do not have, run `git pull` first. \
         The commit service runs with its own settings, not your terminal's, so a push that works in your terminal can \
         fail here when it needs a variable only your terminal has; give the service such a variable with \
         `systemctl --user set-environment NAME=value` and then `{RESTART_COMMAND}`.\n{PUSH_FAILED_NOTE}"
    );
}

/// What:
///  The text shown for a failed backup push, from Git's raw words.
/// Why:
///   The commit is named as the caller gives it, so a caller that has the repository passes Git's own abbreviation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pushFailureText({ branchRef, commit, said })
/// ```
pub fn push_failure_text(branch_ref: &str, commit: &str, said: &str) -> String {
    return worded(branch_ref, commit, clean_reason(said).as_str());
}

/// What:
///  The text shown for a push the service did not make, with the commit named as given and the service's words already cleaned.
/// Why:
///   Nothing failed and nothing was sent, so the text says that and not that the push was refused; it names the branch and the
///  commit, gives the service's reason, says the commit is saved here, and names both ways out: a destination the service can
///  name, or a push by hand. The command that shows it may be anyone's, so it never calls the commit the reader's own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// wordedSkipped({ branchRef, shown, reason })
/// ```
fn worded_skipped(branch_ref: &str, shown: &str, reason: &str) -> String {
    let branch: &str = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
    let detail: String = if reason.is_empty() {
        String::from("The commit service gave no reason.")
    } else {
        format!("The commit service said:\n{}", reason.lines().map(|line| { return format!("    {line}"); }).collect::<Vec<String>>().join("\n"))
    };
    return format!(
        "The backup push of branch {branch} for commit {shown} was not made, so that commit exists only on this computer for now. \
         It is saved in this repository and nothing was lost.\n{detail}\n\
         Either give the branch a destination the commit service can name (the `push.default` setting of `git help config` says \
         which values do), or push it yourself with `git push <remote> {branch}`."
    );
}

/// What:
///  Leaves the record of one push the service did not make, replacing an earlier record of the same commit on the same branch.
/// Why:
///   The service settles such a push as done because nothing is left for it to retry, and the person would otherwise never
///  learn that the backup does not exist. The name is the failure record's, so a later successful push of the branch removes
///  it with the failures.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordPushSkip(gitDir, { branchRef, commit, said })
/// ```
pub fn record_push_skip(git_dir: &Path, branch_ref: &str, commit: &str, said: &str) -> Result<(), String> {
    let directory: PathBuf = notice_directory(git_dir);
    create_private_directory_all(directory.as_path()).map_err(|error| { return format!("creating the notice directory failed: {error}"); })?;
    let path: PathBuf = directory.join(record_name(branch_ref, commit));
    let _removed = std::fs::remove_file(path.as_path());
    let text: String = format!("{SKIPPED_MAGIC}\n{BRANCH_FIELD}{branch_ref}\n{COMMIT_FIELD}{commit}\n\n{}", clean_reason(said));
    return write_private_file_unsynced(path.as_path(), text.as_bytes()).map_err(|error| { return format!("writing the notice failed: {error}"); });
}

/// What:
///  Leaves the record of one failed push, replacing an earlier record of the same commit on the same branch.
/// Why:
///   The name is derived from the branch and the commit, so a push that is tried again and fails again leaves one record.
///  The reason is cleaned before it is written, so no credential reaches the disk.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordPushFailure(gitDir, { branchRef, commit, said })
/// ```
pub fn record_push_failure(git_dir: &Path, branch_ref: &str, commit: &str, said: &str) -> Result<(), String> {
    let directory: PathBuf = notice_directory(git_dir);
    create_private_directory_all(directory.as_path()).map_err(|error| { return format!("creating the notice directory failed: {error}"); })?;
    let path: PathBuf = directory.join(record_name(branch_ref, commit));
    // A repeat of the same failure replaces its record.
    let _removed = std::fs::remove_file(path.as_path());
    let text: String = format!("{RECORD_MAGIC}\n{BRANCH_FIELD}{branch_ref}\n{COMMIT_FIELD}{commit}\n\n{}", clean_reason(said));
    return write_private_file_unsynced(path.as_path(), text.as_bytes()).map_err(|error| { return format!("writing the notice failed: {error}"); });
}

/// What:
///  The text shown for a push that had not finished when the worktree that made the commit was removed.
/// Why:
///   Nothing says whether the push ran, failed or never started, so the text says only what is known: the commit is saved here
///  and the branch may not be on the remote.
fn worded_lost(branch_ref: &str, shown: &str) -> String {
    let branch: &str = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
    return format!(
        "The backup push of branch {branch} for commit {shown} had not finished when the worktree that made the commit was \
         removed, so that commit may exist only on this computer for now. It is saved in this repository and nothing was lost. \
         Run `git push` in a worktree that has branch {branch} checked out, or `git push <remote> {branch}` from any worktree."
    );
}

/// What:
///  Leaves the record of a push that was lost with its worktree, unless a record of that commit on that branch is already
///  there.
/// Why:
///   An existing record holds Git's own words about a failed push, which say more than this one does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordLostPush(commonDir, { branchRef, commit })
/// ```
pub fn record_lost_push(common_dir: &Path, branch_ref: &str, commit: &str) -> Result<(), String> {
    let directory: PathBuf = common_dir.join(ORPHAN_NOTICE_DIRECTORY);
    create_private_directory_all(directory.as_path()).map_err(|error| { return format!("creating the notice directory failed: {error}"); })?;
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
    /// A failed push, with Git's cleaned words.
    Failed(&'a str),
    /// A push the service did not make, with its cleaned words.
    Skipped(&'a str),
    /// A push lost with its worktree.
    Lost,
}

/// The branch, the commit and the kind of a record written by this release.
fn parse_record(text: &str) -> Option<(&str, &str, RecordKind<'_>)> {
    let mut lines = text.splitn(5, '\n');
    let magic: &str = lines.next()?;
    if ![RECORD_MAGIC, SKIPPED_MAGIC, LOST_MAGIC].contains(&magic) {
        return None;
    }
    let branch: &str = lines.next()?.strip_prefix(BRANCH_FIELD)?;
    let commit: &str = lines.next()?.strip_prefix(COMMIT_FIELD)?;
    if !lines.next()?.is_empty() {
        return None;
    }
    let reason: &str = lines.next().unwrap_or("");
    return Some((branch, commit, match magic {
        LOST_MAGIC => RecordKind::Lost,
        SKIPPED_MAGIC => RecordKind::Skipped(reason),
        _ => RecordKind::Failed(reason),
    }));
}

/// What:
///  Removes the records of one branch, after a push of that branch succeeded.
/// Why:
///   A successful push of the branch carries every earlier commit of it, so an earlier failure is no longer true.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await clearPushFailures(gitDir, branchRef)
/// ```
pub fn clear_push_failures(git_dir: &Path, branch_ref: &str) {
    clear_push_failures_before(git_dir, branch_ref, std::time::SystemTime::now());
}

/// What:
///  Removes the records of one branch that were written no later than `cutoff`, after a push of that branch succeeded.
/// Why:
///   A push carries the commits that existed when it began. A record written after that moment is about a newer commit, which
///  another instance of the service may have pushed and failed to push while this push's result was still being handled; the
///  older push says nothing about it, so it stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await clearPushFailuresBefore(gitDir, branchRef, pushBegan)
/// ```
pub fn clear_push_failures_before(git_dir: &Path, branch_ref: &str, cutoff: std::time::SystemTime) {
    let Ok(listing) = std::fs::read_dir(notice_directory(git_dir)) else { return };
    let prefix: String = branch_prefix(branch_ref);
    for entry in listing.flatten() {
        let older: bool = entry.metadata().and_then(|meta| { return meta.modified(); }).is_ok_and(|written| { return written <= cutoff; });
        if older && entry.file_name().to_string_lossy().starts_with(prefix.as_str()) {
            let _removed = std::fs::remove_file(entry.path());
        }
    }
}

/// Where the worktrees of a repository stand: the branch the asking worktree has checked out and every branch any
///  worktree has checked out.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Worktrees {
    /// The branch reference the command's worktree has checked out; nothing for a detached head or an unknown worktree.
    pub here: Option<String>,
    /// The branch references checked out in every worktree of the repository.
    pub all: Vec<String>,
}

/// What:
///  The branch reference a `HEAD` file names, or nothing for a detached head or an unreadable file.
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
    return target.starts_with("refs/heads/").then(|| { return target.to_string(); });
}

/// What:
///  The branches checked out in the repository's worktrees: the main one and each linked one.
/// Why:
///   A linked worktree keeps its own `HEAD` under `worktrees/<name>`; the main worktree's is the common directory's.
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

/// Whether a record of `branch_ref` is for the asking worktree: it has the branch checked out, or no worktree has.
fn is_for(worktrees: &Worktrees, branch_ref: &str) -> bool {
    return worktrees.here.as_deref() == Some(branch_ref) || !worktrees.all.iter().any(|held| { return held == branch_ref; });
}

/// What:
///  Reads and removes the records that are for the asking worktree (all of them when no worktree is given), oldest name
///  first, worded with the commit named by `abbreviate`.
/// Why:
///   A record is shown once, to the worktree whose commit it is about; another worktree's command leaves it where it is.
///  When nothing failed the notice directory does not exist, and this is one directory read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = await takePushFailuresFor(gitDir, worktrees, abbreviate)
/// ```
pub fn take_push_failures_for(git_dir: &Path, worktrees: Option<&Worktrees>, abbreviate: &dyn Fn(&str) -> String) -> Vec<String> {
    return take_records_from(notice_directory(git_dir).as_path(), worktrees, abbreviate);
}

/// What:
///  Reads and removes the records of removed worktrees, which belong to no worktree and so are shown to the first command
///  that asks.
/// Why:
///   The worktree a record was made for is gone, so nobody else could ever be shown it; one directory read when none exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = await takeOrphanedFailures(commonDir, abbreviate)
/// ```
pub fn take_orphaned_failures(common_dir: &Path, abbreviate: &dyn Fn(&str) -> String) -> Vec<String> {
    return take_records_from(common_dir.join(ORPHAN_NOTICE_DIRECTORY).as_path(), None, abbreviate);
}

/// Reads and removes the records in one directory that are for the asking worktree, oldest name first.
fn take_records_from(directory: &Path, worktrees: Option<&Worktrees>, abbreviate: &dyn Fn(&str) -> String) -> Vec<String> {
    let Ok(listing) = std::fs::read_dir(directory) else { return Vec::new() };
    let mut paths: Vec<PathBuf> = listing.flatten().map(|entry| { return entry.path(); }).collect();
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
            Some((branch, commit, RecordKind::Failed(reason))) => worded(branch, abbreviate(commit).as_str(), reason),
            Some((branch, commit, RecordKind::Skipped(reason))) => worded_skipped(branch, abbreviate(commit).as_str(), reason),
            Some((branch, commit, RecordKind::Lost)) => worded_lost(branch, abbreviate(commit).as_str()),
            // The finished text an earlier release wrote.
            None => text,
        });
    }
    return texts;
}

/// What:
///  Reads and removes every record, with each commit named in full.
/// Why:
///   The service's own controls and a repository with no worktree to ask need all of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = await takePushFailures(gitDir)
/// ```
pub fn take_push_failures(git_dir: &Path) -> Vec<String> {
    return take_push_failures_for(git_dir, None, &|commit| { return commit.to_string(); });
}

/// Commands that a program reads the output of, or that carry a protocol on their streams.
const MACHINE_COMMANDS: &[&[u8]] = &[
    b"cat-file", b"check-attr", b"check-ignore", b"check-ref-format", b"commit-tree", b"config", b"credential", b"diff-files",
    b"diff-index", b"diff-tree", b"fast-export", b"fast-import", b"for-each-ref", b"hash-object", b"index-pack", b"ls-files",
    b"ls-tree", b"merge-base", b"mktree", b"name-rev", b"pack-objects", b"read-tree", b"receive-pack", b"rev-list",
    b"rev-parse", b"show-ref", b"symbolic-ref", b"unpack-objects", b"update-ref", b"upload-archive", b"upload-pack",
    b"var", b"write-tree", b"update-index", b"verify-pack",
];

/// Option spellings that ask a command for output made to be read by a program.
fn asks_for_machine_output(option: &[u8]) -> bool {
    return option == b"-z"
        || option == b"--null"
        || option.starts_with(b"--porcelain")
        || option.starts_with(b"--format")
        || option.starts_with(b"--pretty");
}

/// What:
///  Whether a command's output is read by a program, so that no record may be shown to it or taken from it.
/// Why:
///   A prompt, an editor or a script runs `git status --porcelain` or `git rev-parse` often and reads every byte; the
///  record is meant for the person, and the first such call would otherwise take it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isMachineRead(commandWord, region)
/// ```
pub fn is_machine_read(command: &[u8], region: &[OsString]) -> bool {
    if MACHINE_COMMANDS.contains(&command) {
        return true;
    }
    for argument in region {
        let bytes: &[u8] = argument.as_encoded_bytes();
        // Everything after the separator is a name, not an option.
        if bytes == b"--" {
            break;
        }
        if asks_for_machine_output(bytes) {
            return true;
        }
    }
    return false;
}

/// Record controls stay out of the release executable.
#[cfg(test)]
#[path = "push_failure_notice_tests.rs"]
mod tests;
