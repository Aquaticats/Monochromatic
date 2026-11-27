//! What:
//!  What a record's text is:
//!  the first line that names its kind,
//!  the field lines that name its branch and its commit,
//!  the length its names and its words are cut to,
//!  and the reading of those lines back into a kind.
//! Why:
//!  The step that writes a record and the step that reads it must agree on its text,
//!  and the name of a record derives from its branch alone,
//!  so a push of that branch finds every record of it.
//!  The words are cleaned here too,
//!  because no credential may reach the disk or a screen.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! type RecordKind = { kind: 'failed' | 'unfinished' | 'skipped' | 'reported' | 'lost'; reason?: string };
//! const recordName = (branchRef: string, commit: string): string;
//! ```

/// Import the reversible key a branch name becomes in a file name,
/// and the reader that finds a kept name under a Git directory.
use crate::auto_push_branch_key::{
    encode_branch_key,
    fnv_hex,
};
use crate::kept_names_in_the_git_directories::path_for_reading;
/// Import the rule that masks the credentials of every URL in a remote's words.
use crate::lfs_output_mask::mask_url_secrets_text;
/// Import the data folder's names,
/// because a record written before the rename carries the earlier spelling of its first line.
use crate::wrapper_names_old_and_new::DATA_FOLDER;
/// Import the borrowed and owned path types a record's directory is named with.
use std::path::{
    Path,
    PathBuf,
};

/// Directory under the worktree's own Git directory that holds the records for the person.
pub const NOTICE_DIRECTORY: &str = "monochromatic-git-policy-cli/notices";

/// Directory under the repository's common Git directory that holds the records of removed worktrees,
/// which any worktree's
///  command shows because the worktree they were made for is gone.
pub const ORPHAN_NOTICE_DIRECTORY: &str = "monochromatic-git-policy-cli/orphaned-notices";

/// First line of a record of a push that was lost with its worktree;
/// its fields are those of a failure record,
/// without words.
pub(super) const LOST_MAGIC: &str = "monochromatic-git-policy-cli push lost 1";

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
pub(super) const RECORD_MAGIC: &str = "monochromatic-git-policy-cli push notice 1";

/// First line of a record of a push that did not finish,
/// so whether the remote took the commit is not known,
/// and that the service keeps trying.
pub(super) const UNFINISHED_MAGIC: &str = "monochromatic-git-policy-cli push unfinished 1";

/// First line of a record of a push that did not finish and that nothing will try again by itself.
pub(super) const UNFINISHED_NO_RETRY_MAGIC: &str =
    "monochromatic-git-policy-cli push unfinished-no-retry 1";

/// First line of a record of a push the service did not make because the branch has no destination it can name;
/// its fields are
///  those of a failure record,
/// and its reason is the service's own words.
pub(super) const SKIPPED_MAGIC: &str = "monochromatic-git-policy-cli push skipped 1";

/// The field names of a record's header lines.
pub(crate) const BRANCH_FIELD: &str = "branch: ";

pub(crate) const COMMIT_FIELD: &str = "commit: ";

/// Said when Git gave no words at all.
pub(crate) const NO_REASON: &str = "Git gave no reason.";

/// The directory that holds the records of a repository.
pub fn notice_directory(git_dir: &Path) -> PathBuf {
    return path_for_reading(git_dir, NOTICE_DIRECTORY);
}

/// The part of every record name that belongs to one branch.
pub(super) fn branch_prefix(branch_ref: &str) -> String {
    let key: String = encode_branch_key(branch_ref);
    let shown: String = if key.len() > LONGEST_KEY {
        format!("{}~{}", &key[..LONGEST_KEY], fnv_hex(key.as_str()))
    } else {
        key
    };
    return format!("push{NAME_SEPARATOR}{shown}{NAME_SEPARATOR}");
}

/// The file name of the record of one failed push.
pub fn record_name(
    branch_ref: &str,
    commit: &str,
) -> String {
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

/// What a record says besides its branch and commit.
pub(super) enum RecordKind<'a> {
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
pub(super) fn earlier_spelling(rest: &str) -> String {
    return format!("{}{rest}", DATA_FOLDER.earlier);
}

/// The last part of a path's name,
/// or an empty text for a path that has none.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// basename(path) ?? ''
/// ```
pub(super) fn file_name_of(path: &Path) -> String {
    let Some(name) = path.file_name() else {
        return String::new();
    };
    return name.to_string_lossy().into_owned();
}
