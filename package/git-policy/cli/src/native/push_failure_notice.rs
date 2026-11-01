//! What:
//!  The record of a backup push that failed after its commit was saved, and the rule for when the wrapper shows it:
//!  the resident service writes one record per failed push, any later wrapper command in the repository shows each
//!  record once on standard error, and a later push of the same branch that succeeds removes the records of that branch.
//! Why:
//!  The push runs in the service after the commit command returned, so nothing is attached to the command that could
//!  print its failure; without a record the person would never learn that their backup push was refused
//!  (`doc/planning/cli-git-resident-service.md`, "After-commit work"). Records are plain files in the common Git
//!  directory so the check on every command is one directory read that finds nothing when nothing failed. Records are
//!  shown on standard error only, and not at all to a command whose output a program reads (`--porcelain`, `-z`,
//!  plumbing), so that program neither sees the text in its output nor consumes the record before a person can read it.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await recordPushFailure(commonDir, { branchRef, oid, reason }); stderr.write(await takePushFailures(commonDir));
//! ```

use super::auto_push::PUSH_FAILED_NOTE;
use super::auto_push_branch_key::encode_branch_key;
use super::lfs_output_mask::mask_userinfo_text;
use super::private_storage::{create_private_directory_all, write_private_file_unsynced};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// Directory under the common Git directory that holds the records for the person.
pub const NOTICE_DIRECTORY: &str = "cli-git/notices";

/// Separates the parts of a record's name; it is not a character a branch key can contain.
const NAME_SEPARATOR: char = '+';

/// The longest branch key a record name carries whole; a longer key is cut and tagged with a hash of the whole key, so the
///  name stays a valid file name and two long branch names do not share records.
const LONGEST_KEY: usize = 100;

/// The most bytes of Git's own words a record keeps.
const LONGEST_REASON: usize = 4000;

/// How many characters of the commit name a record shows.
const SHORT_COMMIT: usize = 7;

/// Stands in for a line of Git's output that names the Git LFS endpoint setting, whose value can carry a credential.
const LEFT_OUT_LINE: &str = "(a line about the Git LFS server setting is left out because it can hold a credential)";

/// Said when Git gave no words at all.
const NO_REASON: &str = "Git gave no reason.";

/// The directory that holds the records of a repository.
pub fn notice_directory(common_dir: &Path) -> PathBuf {
    return common_dir.join(NOTICE_DIRECTORY);
}

/// A 64-bit FNV-1a hash of the text, in hexadecimal.
fn fnv_hex(text: &str) -> String {
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
///  Git's words for the person: credentials in URLs and the Git LFS endpoint setting removed, control characters
///  dropped, and the length bounded.
/// Why:
///   The text came from a remote and from Git, and is printed on a terminal; a URL with a user name and password, a
///  setting that holds a token, or an escape sequence must not reach the screen or a log of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanReason(said)
/// ```
pub fn clean_reason(said: &str) -> String {
    let masked: String = mask_userinfo_text(said);
    let mut kept: Vec<String> = Vec::new();
    for line in masked.lines() {
        if line.to_ascii_lowercase().contains("lfs.url") {
            kept.push(LEFT_OUT_LINE.to_string());
            continue;
        }
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
///  The text shown for a failed backup push.
/// Why:
///   It names the branch and the commit, gives Git's own reason, says the commit is safe on this computer, and says what to
///  do; the service runs with its own settings, which is the first thing to check when a push that works in a terminal
///  fails here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// pushFailureText({ branchRef, commit, said })
/// ```
pub fn push_failure_text(branch_ref: &str, commit: &str, said: &str) -> String {
    let short: &str = commit.get(..SHORT_COMMIT).unwrap_or(commit);
    let branch: &str = branch_ref.strip_prefix("refs/heads/").unwrap_or(branch_ref);
    let reason: String = clean_reason(said);
    let detail: String = if reason.is_empty() {
        NO_REASON.to_string()
    } else {
        format!("Git said:\n{}", reason.lines().map(|line| { return format!("    {line}"); }).collect::<Vec<String>>().join("\n"))
    };
    return format!(
        "The backup push of branch {branch} failed for your commit {short}, so that commit exists only on this computer for now. \
         It is saved in this repository and nothing was lost.\n{detail}\n\
         Fix what Git names, then run `git push`. If it says the remote has commits you do not have, run `git pull` first. \
         The commit service runs with its own settings, not your terminal's, so a push that works in your terminal can \
         fail here when it needs a variable only your terminal has.\n{PUSH_FAILED_NOTE}"
    );
}

/// What:
///  Leaves the record of one failed push, replacing an earlier record of the same commit on the same branch.
/// Why:
///   The name is derived from the branch and the commit, so a push that is tried again and fails again leaves one record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await recordPushFailure(commonDir, { branchRef, commit, said })
/// ```
pub fn record_push_failure(common_dir: &Path, branch_ref: &str, commit: &str, said: &str) -> Result<(), String> {
    let directory: PathBuf = notice_directory(common_dir);
    create_private_directory_all(directory.as_path()).map_err(|error| { return format!("creating the notice directory failed: {error}"); })?;
    let path: PathBuf = directory.join(record_name(branch_ref, commit));
    // A repeat of the same failure replaces its record.
    let _removed = std::fs::remove_file(path.as_path());
    let text: String = push_failure_text(branch_ref, commit, said);
    return write_private_file_unsynced(path.as_path(), text.as_bytes()).map_err(|error| { return format!("writing the notice failed: {error}"); });
}

/// What:
///  Removes the records of one branch, after a push of that branch succeeded.
/// Why:
///   A successful push of the branch carries every earlier commit of it, so an earlier failure is no longer true.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await clearPushFailures(commonDir, branchRef)
/// ```
pub fn clear_push_failures(common_dir: &Path, branch_ref: &str) {
    let Ok(listing) = std::fs::read_dir(notice_directory(common_dir)) else { return };
    let prefix: String = branch_prefix(branch_ref);
    for entry in listing.flatten() {
        if entry.file_name().to_string_lossy().starts_with(prefix.as_str()) {
            let _removed = std::fs::remove_file(entry.path());
        }
    }
}

/// What:
///  Reads and removes the repository's records, oldest name first.
/// Why:
///   A record is shown once. When nothing failed the notice directory does not exist, and this is one directory read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const texts = await takePushFailures(commonDir)
/// ```
pub fn take_push_failures(common_dir: &Path) -> Vec<String> {
    let Ok(listing) = std::fs::read_dir(notice_directory(common_dir)) else { return Vec::new() };
    let mut paths: Vec<PathBuf> = listing.flatten().map(|entry| { return entry.path(); }).collect();
    paths.sort();
    let mut texts: Vec<String> = Vec::new();
    for path in paths {
        // The record is removed before its text is used, so two commands that start together cannot both show it.
        let Ok(text) = std::fs::read_to_string(path.as_path()) else { continue };
        if std::fs::remove_file(path.as_path()).is_ok() {
            texts.push(text);
        }
    }
    return texts;
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
