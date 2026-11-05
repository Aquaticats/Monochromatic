//! What:
//!  The exact JSON encoding of one destination's linked-worktree copy journal, and the strict reading and validation of every
//!  field recovery will act on.
//! Why:
//!  A crash can happen after any step, so the journal must say which phase the copy reached and which paths it may have
//!  touched, and recovery must never act on a journal it cannot prove is its own.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const text = encodeJournal(journal);
//! const journal = parseJournal(text, path, journalRoot); // throws when any field is not what the encoder writes
//! ```

use crate::json_record::quote;
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause};
use crate::worktree_copy_install_entry::InstalledPath;
use crate::worktree_copy_selection::{STAGE_PREFIX, assert_safe_repository_path};
use std::path::{Path, PathBuf};

/// What:
///  The parent directory name of the journals, beneath the common Git directory.
/// Why:
///   Every linked worktree of one repository shares the same journal location.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const JOURNAL_PARENT_NAME = 'cli-git-worktree-copy';
/// ```
pub const JOURNAL_PARENT_NAME: &str = "cli-git-worktree-copy";

/// What:
///  The schema directory of the journals.
/// Why:
///   A later schema gets its own directory, so an older wrapper never reads a newer record.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const JOURNAL_VERSION_NAME = 'v1';
/// ```
pub const JOURNAL_VERSION_NAME: &str = "v1";

/// What:
///  The suffix of a final journal file.
/// Why:
///   Only files with this suffix are journals; anything else in the directory is ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const JOURNAL_SUFFIX = '.json';
/// ```
pub const JOURNAL_SUFFIX: &str = ".json";

/// What:
///  The suffix of an unpublished temporary journal write.
/// Why:
///   An interrupted write leaves only files with this suffix, which recovery removes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const TEMPORARY_SUFFIX = '.tmp';
/// ```
pub const TEMPORARY_SUFFIX: &str = ".tmp";

/// What:
///  The durable phase a copy has reached.
/// Why:
///   Recovery resumes an installation and finishes a completed cleanup differently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeCopyJournalPhase = 'complete' | 'installing' | 'staged';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Phase {
    /// The stage is complete and nothing has been installed.
    Staged,
    /// Installation has begun; the intents and creations in the install log say what it touched.
    Installing,
    /// Installation finished; only cleanup remains.
    Complete,
}

impl Phase {
    /// The phase's name as the incumbent writes it.
    fn as_str(self) -> &'static str {
        // `match` keeps each phase's spelling next to its variant.
        match self {
            Phase::Staged => return "staged",
            Phase::Installing => return "installing",
            Phase::Complete => return "complete",
        }
    }
}

/// What:
///  One durable journal record.
/// Why:
///   Every field is read back by recovery before any filesystem path is used.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WorktreeCopyJournal = { createdEntries; intendedEntries; destinationRoot; phase; selectedRoots; sourceRoot; stageContainer; stageRoot; version: 1 };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Journal {
    /// Paths proven created, with their post-creation identities.
    pub created_entries: Vec<InstalledPath>,
    /// Selected paths claimed before installation.
    pub intended_entries: Vec<String>,
    /// The worktree receiving the copy.
    pub destination_root: String,
    /// The durable phase.
    pub phase: Phase,
    /// The ignored roots the staged payload represents.
    pub selected_roots: Vec<String>,
    /// The source worktree.
    pub source_root: String,
    /// The private container beside the destination.
    pub stage_container: String,
    /// The payload root inside the container.
    pub stage_root: String,
}

/// What:
///  A journal and the path it was read from or written to.
/// Why:
///   Removal and recovery need both the record and the file that must be deleted.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PendingWorktreeCopyJournal = { path: string; record: WorktreeCopyJournal };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PendingJournal {
    /// Absolute journal file path.
    pub path: PathBuf,
    /// The validated record.
    pub journal: Journal,
}

impl PendingJournal {
    /// The owner token the journal's file name carries, or nothing for a name no owner is written into.
    pub fn owner_token(&self) -> Option<String> {
        return owner_token_of(self.path.as_path());
    }
}

/// What:
///  The owner token in a journal file name `<id>.<owner>.json`, if the name has that shape.
/// Why:
///   The owner is part of the name so that taking a journal over is one atomic rename.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ownerTokenOf('0a1b.2c3d.json') === '2c3d';
/// ```
pub fn owner_token_of(path: &Path) -> Option<String> {
    let name: String = path.file_name()?.to_string_lossy().into_owned();
    let stem: &str = name.strip_suffix(JOURNAL_SUFFIX)?;
    let (_identifier, token) = stem.rsplit_once('.')?;
    return Some(token.to_string());
}

/// What:
///  The JSON text of one journal, with the incumbent's key order and a final line feed.
/// Why:
///   The incumbent reads these bytes, so the order and spelling must match its `JSON.stringify` output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${JSON.stringify(serializedRecord)}\n`
/// ```
pub fn encode_journal(journal: &Journal) -> String {
    let created: Vec<String> = journal
        .created_entries
        .iter()
        .map(encode_installed_entry)
        .collect();
    return format!(
        "{{\"createdEntries\":[{}],\"intendedEntries\":{},\"destinationRoot\":{},\"phase\":{},\"selectedRoots\":{},\"sourceRoot\":{},\"stageContainer\":{},\"stageRoot\":{},\"version\":1}}\n",
        created.join(","),
        string_array(journal.intended_entries.as_slice()),
        quote(journal.destination_root.as_str()),
        quote(journal.phase.as_str()),
        string_array(journal.selected_roots.as_slice()),
        quote(journal.source_root.as_str()),
        quote(journal.stage_container.as_str()),
        quote(journal.stage_root.as_str()),
    );
}

/// What:
///  One created entry as its JSON object, with the incumbent's key order.
/// Why:
///   The install log and the journal share this spelling, so both wrappers read either.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify({ device, inode, relativePath, selected })
/// ```
pub fn encode_installed_entry(entry: &InstalledPath) -> String {
    return format!(
        "{{\"device\":\"{}\",\"inode\":\"{}\",\"relativePath\":{},\"selected\":{}}}",
        entry.device,
        entry.inode,
        quote(entry.relative_path.as_str()),
        entry.selected
    );
}

/// What:
///  A JSON array of strings, each quoted as `JSON.stringify` quotes it.
/// Why:
///   Repository paths are the only arrays a journal holds.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// JSON.stringify(items)
/// ```
pub fn string_array(items: &[String]) -> String {
    let quoted: Vec<String> = items.iter().map(String::as_str).map(quote).collect();
    return format!("[{}]", quoted.join(","));
}

/// What:
///  Whether a path component is a plain name: not empty, not `.` and not `..`.
/// Why:
///   A journal path that is not in canonical form could name a different file than the one validated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// component !== '' && component !== '.' && component !== '..'
/// ```
fn is_plain_component(component: &str) -> bool {
    return !component.is_empty() && component != "." && component != "..";
}

/// What:
///  Whether a byte is an ASCII decimal digit.
/// Why:
///   Device and inode numbers are digits only.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// /[0-9]/.test(char)
/// ```
fn is_digit_byte(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// What:
///  A JSON value as text, when it is a string.
/// Why:
///   Every path in a journal must be a string before it is checked as a path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// typeof value === 'string' ? value : undefined
/// ```
fn text_of(item: &serde_json::Value) -> Option<String> {
    return item.as_str().map(str::to_string);
}

/// What:
///  Whether a path is absolute and already in lexical canonical form.
/// Why:
///   A journal path that is not canonical could name a different file than the one validated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isAbsolute(value) && resolve(value) === value
/// ```
pub fn is_canonical_absolute(value: &str) -> bool {
    if !value.starts_with('/') {
        return false;
    }
    if value == "/" {
        return true;
    }
    return value[1..]
        .split('/')
        .all(is_plain_component);
}

/// What:
///  Whether a text is a canonical decimal identity: digits only, no leading zero.
/// Why:
///   Device and inode numbers are compared as text, so only one spelling of each may exist.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isDecimalIdentity(value: unknown): value is string;
/// ```
pub fn decimal_identity(value: &serde_json::Value) -> Option<u64> {
    let text: &str = value.as_str()?;
    if text.is_empty() || (text.len() > 1 && text.starts_with('0')) {
        return None;
    }
    if !text.bytes().all(is_digit_byte) {
        return None;
    }
    return text.parse::<u64>().ok();
}

/// What:
///  One created-entry object, read strictly.
/// Why:
///   An identity with any wrong type is not an identity cli-git wrote.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isInstalledEntry(value: unknown): value is InstalledWorktreePath;
/// ```
pub fn installed_entry(value: &serde_json::Value) -> Option<InstalledPath> {
    let object = value.as_object()?;
    return Some(InstalledPath {
        device: decimal_identity(object.get("device")?)?,
        inode: decimal_identity(object.get("inode")?)?,
        relative_path: object.get("relativePath")?.as_str()?.to_string(),
        selected: object.get("selected")?.as_bool()?,
    });
}

/// What:
///  An array of strings, read strictly.
/// Why:
///   A non-string path would reach the filesystem without the validation a string gets.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// isStringArray(value)
/// ```
fn string_list(value: Option<&serde_json::Value>) -> Option<Vec<String>> {
    let array = value?.as_array()?;
    return array.iter().map(text_of).collect();
}

/// What:
///  The record's fields read strictly, before any path is checked.
/// Why:
///   The shape check refuses every record that a later check would have to guess about.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!(isRecord(value) && value.version === 1 && ...)) throw new WorktreeCopyError('... is corrupt ...');
/// ```
fn read_shape(value: &serde_json::Value) -> Option<Journal> {
    let object = value.as_object()?;
    if object.get("version")?.as_f64()? != 1.0 {
        return None;
    }
    let phase: Phase = match object.get("phase")?.as_str()? {
        "staged" => Phase::Staged,
        "installing" => Phase::Installing,
        "complete" => Phase::Complete,
        _ => return None,
    };
    let created_entries: Vec<InstalledPath> = object
        .get("createdEntries")?
        .as_array()?
        .iter()
        .map(installed_entry)
        .collect::<Option<Vec<InstalledPath>>>()?;
    return Some(Journal {
        created_entries,
        intended_entries: string_list(object.get("intendedEntries"))?,
        destination_root: object.get("destinationRoot")?.as_str()?.to_string(),
        phase,
        selected_roots: string_list(object.get("selectedRoots"))?,
        source_root: object.get("sourceRoot")?.as_str()?.to_string(),
        stage_container: object.get("stageContainer")?.as_str()?.to_string(),
        stage_root: object.get("stageRoot")?.as_str()?.to_string(),
    });
}

/// What:
///  Parses and validates one journal's text, refusing every shape recovery cannot trust.
/// Why:
///   Recovery turns journal paths into deletions, so every relation between the paths is checked first.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// validateJournalValue({ value: JSON.parse(text), path, journalRoot });
/// ```
pub fn parse_journal(
    text: &str,
    path: &Path,
    journal_root: &Path,
) -> Result<Journal, WorktreeCopyError> {
    let value: serde_json::Value = match serde_json::from_str(text) {
        Ok(parsed) => parsed,
        Err(error) => {
            let message: String = format!(
                "cli-git: could not read worktree-copy journal {}.",
                quote(path.to_string_lossy().as_ref())
            );
            return Err(failure_with_cause(message.as_str(), Some(&error)));
        }
    };
    let Some(journal) = read_shape(&value) else {
        return Err(corrupt(path));
    };
    if path.parent() != Some(journal_root) {
        return Err(failure_with_cause(
            format!(
                "cli-git: worktree-copy journal escaped private root: {}.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        ));
    }
    for (field, field_value) in [
        ("destinationRoot", journal.destination_root.as_str()),
        ("sourceRoot", journal.source_root.as_str()),
        ("stageContainer", journal.stage_container.as_str()),
        ("stageRoot", journal.stage_root.as_str()),
    ] {
        if !is_canonical_absolute(field_value) {
            return Err(failure_with_cause(
                format!(
                    "cli-git: worktree-copy journal has unsafe {field}: {}.",
                    quote(field_value)
                )
                .as_str(),
                None,
            ));
        }
    }
    let stage_named: bool = match Path::new(journal.stage_container.as_str()).file_name() {
        Some(name) => name.to_string_lossy().starts_with(STAGE_PREFIX),
        None => false,
    };
    let stage_relation: bool = Path::new(journal.stage_container.as_str()).parent()
        == Path::new(journal.destination_root.as_str()).parent()
        && stage_named
        && journal.stage_root == format!("{}/payload", journal.stage_container);
    if !stage_relation {
        return Err(failure_with_cause(
            format!(
                "cli-git: worktree-copy journal has unsafe private stage relation: {}.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        ));
    }
    for selected in &journal.selected_roots {
        assert_safe_repository_path(selected)?;
    }
    for intended in &journal.intended_entries {
        assert_safe_repository_path(intended)?;
    }
    for created in &journal.created_entries {
        assert_safe_repository_path(created.relative_path.as_str())?;
    }
    let mut created_paths: Vec<&str> = Vec::new();
    for entry in &journal.created_entries {
        created_paths.push(entry.relative_path.as_str());
    }
    let duplicated: bool = has_duplicates(journal.selected_roots.as_slice())
        || has_duplicates(journal.intended_entries.as_slice())
        || has_duplicates(created_paths.as_slice());
    if duplicated {
        return Err(failure_with_cause(
            format!(
                "cli-git: worktree-copy journal contains duplicate repository paths: {}.",
                quote(path.to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        ));
    }
    return Ok(journal);
}

/// What:
///  Whether a list holds the same entry twice.
/// Why:
///   Duplicate claims would make rollback ownership ambiguous.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new Set(items).size !== items.length
/// ```
fn has_duplicates<T: AsRef<str>>(items: &[T]) -> bool {
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    // The loop stops at the first repeated item, as `any` did.
    for item in items {
        if !seen.insert(item.as_ref()) {
            return true;
        }
    }
    return false;
}

/// What:
///  The corruption failure naming one journal.
/// Why:
///   The user learns which file to inspect; the file itself is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new WorktreeCopyError(`cli-git: worktree-copy journal is corrupt: ${JSON.stringify(path)}.`)
/// ```
fn corrupt(path: &Path) -> WorktreeCopyError {
    return failure_with_cause(
        format!(
            "cli-git: worktree-copy journal is corrupt: {}.",
            quote(path.to_string_lossy().as_ref())
        )
        .as_str(),
        None,
    );
}
