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

use crate::kept_names_in_the_git_directories::path_for_reading;
use crate::auto_push_branch_key::encode_branch_key;
use crate::lfs_output_mask::mask_url_secrets_text;
pub use crate::push_failure_notice_machine_output::is_machine_read;
use crate::private_storage::{create_private_directory_all, replace_private_file_unsynced, write_private_file_unsynced};
/// One record as a command holds it before it shows it,
/// with the claim it was taken under and the step that removes that claim.
pub use crate::push_failure_notice_shown_then_removed::Notice;
/// The record of what a push that succeeded did,
/// and the first line that tells it from a failure.
pub use crate::push_failure_notice_reports::record_push_report;
use crate::push_failure_notice_reports::REPORT_MAGIC;
/// The words a notice carries,
/// the text of the failed backup push,
/// and what the push came to.
pub use crate::push_failure_notice_words::{PushOutcome, push_failure_text};
pub(crate) use crate::push_failure_notice_words::{worded, worded_lost, worded_report, worded_skipped};
/// The name a command holds a record under while it shows it,
/// and who may take one over.
use crate::push_failure_notice_claimed_by_one_command::{CLAIM_SUFFIX, ClaimToken, claim, claimed_by_a_live_command, give_back, record_of_claim, take_over};
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
pub(crate) const NAME_SEPARATOR: char = '+';

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
pub(crate) const LONGEST_KEY: usize = 100;

/// The most bytes of Git's own words a notice of a push that did not land keeps.
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
/// It bounds no account of a push that landed,
/// which nothing can make say the same thing again
/// (`clean_words`).
pub(crate) const LONGEST_REASON: usize = 4000;

/// First line of a record that holds its parts as fields;
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
pub(crate) const BRANCH_FIELD: &str = "branch: ";
pub(crate) const COMMIT_FIELD: &str = "commit: ";

/// Said when Git gave no words at all.
pub(crate) const NO_REASON: &str = "Git gave no reason.";

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
///  Git's words for the person:
/// the user name and password of every URL masked,
/// control characters dropped,
/// and the
///  length bounded.
/// Why:
///   The text came from a remote and from Git,
/// and is printed on a terminal;
/// a URL with a user name and password,
/// or an
///  escape sequence,
/// must not reach the screen or a log of it.
/// Only the credentials are masked,
/// never the line around them,
///  because the line often says why the push failed (the wrapper's own Git LFS failure line names the endpoint setting and
///  carries the exit status).
/// The bound is the notice's own display cap and belongs to a push that will say the same thing again on its next try;
/// the account of a push that landed keeps every line (`clean_words`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanReason(said)
/// ```
pub fn clean_reason(said: &str) -> String {
    return bounded(clean_words(said));
}

/// What:
///  Git's words for the person with nothing cut:
/// every URL credential masked,
/// control characters dropped,
/// and every line kept.
/// Why:
///   The account of a push that landed is the only copy of what a server and the reconcile said,
/// and nothing can make an
///  accepted push say it again:
/// a push of what the remote already holds prints `Everything up-to-date` and runs no hook
///  (`SPEC.md`,
/// "Reconcile and bounds").
/// That account therefore keeps every line the push chose to show,
/// while a failure record
///  keeps the display cap,
/// because a failure is tried again and says the same thing then.
/// Only the credentials are masked,
/// never the line around them,
/// as `clean_reason`
///  says.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// cleanWords(said)
/// ```
pub fn clean_words(said: &str) -> String {
    let masked: String = mask_url_secrets_text(said);
    let mut kept: Vec<String> = Vec::new();
    for line in masked.lines() {
        let mut plain: String = String::new();
        for character in line.chars() {
            if character == '\t' || !character.is_control() {
                plain.push(character);
            }
        }
        let trimmed: &str = plain.trim_end();
        if !trimmed.is_empty() {
            kept.push(trimmed.to_string());
        }
    }
    return kept.join("\n");
}

/// What:
///  The words cut to the most bytes a notice of a push that did not land keeps,
/// with a line that says so.
/// Why:
///   `LONGEST_REASON` is a display cap of the person's notice,
/// and a push Git refused can print any
///  amount of text;
/// a person reads the start of the refusal,
/// which names what to repair,
/// and the retry that follows
///  says the same thing again.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bounded(words)
/// ```
fn bounded(mut text: String) -> String {
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
pub fn record_push_skip(git_dir: &Path, branch_ref: &str, commit: &str, said: &str) -> Result<(), String> {
    let directory: PathBuf = notice_directory(git_dir);
    if let Err(error) = create_private_directory_all(directory.as_path()) {
        return Err(format!("creating the notice directory failed: {error}"));
    }
    let path: PathBuf = directory.join(record_name(branch_ref, commit));
    let text: String = format!("{SKIPPED_MAGIC}\n{BRANCH_FIELD}{branch_ref}\n{COMMIT_FIELD}{commit}\n\n{}", clean_reason(said));
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
pub fn record_push_failure(git_dir: &Path, branch_ref: &str, commit: &str, outcome: PushOutcome, said: &str) -> Result<(), String> {
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
    let text: String = format!("{magic}\n{BRANCH_FIELD}{branch_ref}\n{COMMIT_FIELD}{commit}\n\n{}", clean_reason(said));
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
    /// What a push that succeeded did,
    /// in the service's own words.
    Reported(&'a str),
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
    for current in [RECORD_MAGIC, UNFINISHED_MAGIC, UNFINISHED_NO_RETRY_MAGIC, SKIPPED_MAGIC, REPORT_MAGIC, LOST_MAGIC] {
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
        REPORT_MAGIC => RecordKind::Reported(reason),
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
        let name: String = entry.file_name().to_string_lossy().into_owned();
        // A claim belongs to the command that holds it, which shows it and takes it with it.
        if older && name.starts_with(prefix.as_str()) && !name.ends_with(CLAIM_SUFFIX) {
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
pub fn push_failures_for(git_dir: &Path, worktrees: Option<&Worktrees>, abbreviate: &dyn CommitNamer) -> Vec<Notice> {
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
pub fn orphaned_push_failures(common_dir: &Path, abbreviate: &dyn CommitNamer) -> Vec<Notice> {
    return read_records_from(common_dir.join(ORPHAN_NOTICE_DIRECTORY).as_path(), None, abbreviate);
}

/// Reads the records in one directory that are for the asking worktree,
/// oldest name first,
/// without removing any.
fn read_records_from(directory: &Path, worktrees: Option<&Worktrees>, abbreviate: &dyn CommitNamer) -> Vec<Notice> {
    let Ok(listing) = std::fs::read_dir(directory) else { return Vec::new() };
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
            let Some(claimed) = take_over(path.as_path(), &token) else { continue };
            notices.extend(word_records_at(claimed.as_path(), worktrees, abbreviate));
            continue;
        }
        let Ok(text) = std::fs::read_to_string(path.as_path()) else { continue };
        let parsed: Option<(&str, &str, RecordKind<'_>)> = parse_record(text.as_str());
        if let (Some((branch, _, _)), Some(asking)) = (&parsed, worktrees)
            && !is_for(asking, branch)
        {
            continue;
        }
        // The record is claimed before its words are made: two commands that start together both read it, and only the one
        //  whose rename wins shows it. A command killed while it words the notice leaves the claim, which a later command takes
        //  over once this one is gone.
        let Some(claimed) = claim(path.as_path(), &token) else { continue };
        notices.extend(word_records_at(claimed.as_path(), worktrees, abbreviate));
    }
    return notices;
}

/// The last part of a path's name,
/// or an empty text for a path that has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// basename(path) ?? ''
/// ```
fn file_name_of(path: &Path) -> String {
    let Some(name) = path.file_name() else {
        return String::new();
    };
    return name.to_string_lossy().into_owned();
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
fn word_records_at(path: &Path, worktrees: Option<&Worktrees>, abbreviate: &dyn CommitNamer) -> Vec<Notice> {
    let Ok(text) = std::fs::read_to_string(path) else { return Vec::new() };
    let parsed: Option<(&str, &str, RecordKind<'_>)> = parse_record(text.as_str());
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
        Some((branch, commit, RecordKind::Failed(reason))) => worded(branch, abbreviate.name_of(commit).as_str(), PushOutcome::Refused, reason),
        Some((branch, commit, RecordKind::Unfinished(reason))) => worded(branch, abbreviate.name_of(commit).as_str(), PushOutcome::Unfinished, reason),
        Some((branch, commit, RecordKind::UnfinishedWithNoRetry(reason))) => {
            worded(branch, abbreviate.name_of(commit).as_str(), PushOutcome::UnfinishedWithNoRetry, reason)
        }
        Some((branch, commit, RecordKind::Skipped(reason))) => worded_skipped(branch, abbreviate.name_of(commit).as_str(), reason),
        Some((branch, commit, RecordKind::Reported(reason))) => worded_report(branch, abbreviate.name_of(commit).as_str(), reason),
        Some((branch, commit, RecordKind::Lost)) => worded_lost(branch, abbreviate.name_of(commit).as_str()),
        // The finished text an earlier release wrote.
        None => text.clone(),
    };
    let record: String = file_name_of(path);
    let record_name: &str = record_of_claim(record.as_str()).unwrap_or(record.as_str());
    return vec![Notice { record: path.with_file_name(record_name), claim: path.to_path_buf(), text: shown }];
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
