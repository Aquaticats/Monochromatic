//! What:
//!  The durable intent of a worktree-creating command: written before real Git runs, removed once the copy requests for
//!  what Git created are on disk.
//! Why:
//!  A wrapper killed after Git created a worktree and before it wrote the request would leave a worktree nobody remembers:
//!  the next command's before-set already holds it. The intent keeps the before-set, the source and the selection, so the
//!  service can find the worktrees the command created and write their requests itself. Writing the requests twice (the
//!  wrapper and the service) is harmless: a registration's request, token and marker are the same for both.
//!  An intent that is old is dropped without being acted on a last time only because nothing can still depend on it: the
//!  age decides when to stop looking, never who may act.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeIntent(commonDir, intent); await runGit(); await writeRequests(); await removeIntent(commonDir, intent);
//! ```

use super::json_record::quote;
use super::private_storage::create_private_directory_all;
use super::worktree_copy_error::{WorktreeCopyError, raw_failure};
use super::worktree_copy_journal::{
    ensure_journal_root, journal_root, remove_file_if_present, write_atomic_unsynced,
};
use super::worktree_copy_request::{decode_selection, encode_selection};
use super::worktree_copy_selection::{CopySelection, js_order};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// The directory of intents under the journal root.
pub const INTENTS_NAME: &str = "intents";

/// The suffix of an intent file.
const INTENT_SUFFIX: &str = ".json";

/// How long the service keeps looking at an intent whose wrapper never removed it: one hour after it was written.
///  After that no Git command that the wrapper started can still be running to completion.
pub const INTENT_KEPT_MILLISECONDS: u128 = 60 * 60 * 1000;

/// What:
///  What a creating command decided before Git ran.
/// Why:
///   The service can find the worktrees the command created from these facts alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Intent = { id; sourceRoot; selection; beforeAdminIds; addedPath };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Intent {
    /// Time-ordered unique name, the same shape as a request's.
    pub id: String,
    /// The canonical root of the worktree the ignored state comes from.
    pub source_root: PathBuf,
    /// Which of the source's ignored paths are copied.
    pub selection: CopySelection,
    /// The administrative names before Git ran.
    pub before_admin_ids: BTreeSet<String>,
    /// The path the command line asked `git worktree add` to create, as written; nothing when it could not be read.
    pub added_path: Option<PathBuf>,
    /// Whether real Git has returned: the worktrees it created exist, so the service may write their requests and drop the
    ///  intent at once. Before that, Git may still be running in a wrapper that is alive, and the intent is kept.
    pub git_returned: bool,
}

impl Intent {
    /// The intent's text: sorted keys, one line.
    pub fn encode(&self) -> String {
        let before: Vec<String> = self.before_admin_ids.iter().map(|name| return quote(name.as_str())).collect();
        let added: String = self
            .added_path
            .as_ref()
            .map_or_else(|| return String::from("null"), |path| return quote(path.to_string_lossy().as_ref()));
        return format!(
            "{{\"addedPath\":{added},\"beforeAdminIds\":[{}],\"gitReturned\":{},\"id\":{},\"selection\":{},\"sourceRoot\":{},\"version\":1}}\n",
            before.join(","),
            self.git_returned,
            quote(self.id.as_str()),
            encode_selection(&self.selection),
            quote(self.source_root.to_string_lossy().as_ref())
        );
    }

    /// Reads an intent, or says why the text is not one this release wrote.
    pub fn decode(text: &str) -> Result<Intent, String> {
        let value: serde_json::Value = serde_json::from_str(text).map_err(|error| return format!("not JSON ({error})"))?;
        if value.get("version").and_then(serde_json::Value::as_u64) != Some(1) {
            return Err(String::from("unsupported version"));
        }
        let text_field = |name: &str| -> Result<String, String> {
            return value
                .get(name)
                .and_then(serde_json::Value::as_str)
                .map(String::from)
                .ok_or_else(|| return format!("{name} is missing or not text"));
        };
        let mut before: BTreeSet<String> = BTreeSet::new();
        for item in value.get("beforeAdminIds").and_then(serde_json::Value::as_array).ok_or("beforeAdminIds is missing")? {
            before.insert(item.as_str().ok_or("a name is not text")?.to_string());
        }
        let added_path: Option<PathBuf> = match value.get("addedPath") {
            Some(serde_json::Value::Null) | None => None,
            Some(serde_json::Value::String(path)) => Some(PathBuf::from(path)),
            Some(_) => return Err(String::from("addedPath is not text")),
        };
        let intent: Intent = Intent {
            id: text_field("id")?,
            source_root: PathBuf::from(text_field("sourceRoot")?),
            selection: decode_selection(value.get("selection").ok_or("selection is missing")?)?,
            before_admin_ids: before,
            added_path,
            git_returned: value.get("gitReturned").and_then(serde_json::Value::as_bool).ok_or("gitReturned is missing")?,
        };
        let plain: bool = !intent.id.is_empty() && intent.id.bytes().all(|byte| return byte.is_ascii_alphanumeric() || byte == b'-');
        if !plain || !intent.source_root.is_absolute() {
            return Err(String::from("a name or a path is not acceptable"));
        }
        return Ok(intent);
    }

    /// The milliseconds since the epoch the intent's identity starts with, if it has them.
    pub fn written_at(&self) -> Option<u128> {
        return u128::from_str_radix(self.id.split('-').next()?, 16).ok();
    }
}

/// The directory of intents.
pub fn intents_directory(common_dir: &Path) -> PathBuf {
    return journal_root(common_dir).join(INTENTS_NAME);
}

/// What:
///  Writes an intent before real Git runs.
/// Why:
///   Written whole (private temporary, rename), so a reader sees it complete or not at all, and a process that is killed at
///  any instruction leaves it. It is not synchronized: a synchronization stands between the person's command and Git, on
///  a disk where it can take seconds, and the intent is a recovery hint for killed processes, which a power failure that
///  loses it takes the whole machine with.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeIntent(commonDir, intent)
/// ```
pub fn write_intent(common_dir: &Path, intent: &Intent) -> Result<(), WorktreeCopyError> {
    ensure_journal_root(common_dir)?;
    let directory: PathBuf = intents_directory(common_dir);
    create_private_directory_all(directory.as_path()).map_err(|error| return raw_failure(&error))?;
    return write_atomic_unsynced(
        directory.join(format!("{}{INTENT_SUFFIX}", intent.id)).as_path(),
        intent.encode().as_bytes(),
        "intent",
    );
}

/// What:
///  Records that real Git returned, replacing the intent whole but without synchronizing it.
/// Why:
///   A power failure that loses this rewrite leaves the earlier intent, which the service handles as one whose Git may
///  still run: it reads it again and drops it when it is an hour old. A killed process leaves either whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await markReturned(commonDir, intent)
/// ```
pub fn mark_git_returned(common_dir: &Path, intent: &Intent) -> Result<(), WorktreeCopyError> {
    let returned: Intent = Intent { git_returned: true, ..intent.clone() };
    return write_atomic_unsynced(
        intents_directory(common_dir).join(format!("{}{INTENT_SUFFIX}", returned.id)).as_path(),
        returned.encode().as_bytes(),
        "intent",
    );
}

/// Removes an intent; an intent that is gone is removed. The removal is not synchronized: one that a power failure brings
///  back is read once more by the service, which finds every request already written.
pub fn remove_intent(common_dir: &Path, intent: &Intent) -> Result<(), WorktreeCopyError> {
    return remove_file_if_present(intents_directory(common_dir).join(format!("{}{INTENT_SUFFIX}", intent.id)).as_path());
}

/// Whether any intent stands.
pub fn intents_exist(common_dir: &Path) -> Result<bool, WorktreeCopyError> {
    // `match` treats an absent directory as having no intents, and every other failure as fatal.
    return match std::fs::read_dir(intents_directory(common_dir)) {
        Ok(mut listing) => Ok(listing.any(|listed| {
            return listed.is_ok_and(|entry| return entry.file_name().to_string_lossy().ends_with(INTENT_SUFFIX));
        })),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(raw_failure(&error)),
    };
}

/// What:
///  Reads every intent in the order it was written; one that cannot be read is skipped and named in the second part.
/// Why:
///   An unreadable intent must not stop the others, and is reported rather than dropped silently.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { intents, unreadable } = await readIntents(commonDir)
/// ```
pub fn read_intents(common_dir: &Path) -> Result<(Vec<Intent>, Vec<String>), WorktreeCopyError> {
    let directory: PathBuf = intents_directory(common_dir);
    // `match` treats an absent directory as having no intents, and every other failure as fatal.
    let listing = match std::fs::read_dir(directory.as_path()) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok((Vec::new(), Vec::new())),
        Err(error) => return Err(raw_failure(&error)),
    };
    let mut names: Vec<String> = Vec::new();
    for listed in listing {
        let name: String = listed.map_err(|error| return raw_failure(&error))?.file_name().to_string_lossy().into_owned();
        if name.ends_with(INTENT_SUFFIX) {
            names.push(name);
        }
    }
    names.sort_by(|left, right| return js_order(left, right));
    let mut intents: Vec<Intent> = Vec::new();
    let mut unreadable: Vec<String> = Vec::new();
    for name in names {
        let text: String = match std::fs::read_to_string(directory.join(name.as_str())) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(raw_failure(&error)),
        };
        match Intent::decode(text.as_str()) {
            Ok(intent) if format!("{}{INTENT_SUFFIX}", intent.id) == name => intents.push(intent),
            Ok(_) => unreadable.push(format!("{name}: its name is not its identity")),
            Err(reason) => unreadable.push(format!("{name}: {reason}")),
        }
    }
    return Ok((intents, unreadable));
}

#[cfg(test)]
#[path = "worktree_copy_intent_tests.rs"]
mod tests;
