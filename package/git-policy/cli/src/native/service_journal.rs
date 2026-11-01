//! What:
//!  The resident service's journal: one file per batch of commits, with the one durable write the batch needs, and the
//!  decision a restart makes about a batch it finds open.
//! Why:
//!  The branch moves only after the intent is on disk, so after any kill the journal and the branch say whether the
//!  batch landed; a restart finishes the one step that may be missing (the index update) or drops the intent, and a
//!  request that landed is remembered by its identity so a retry is answered, not landed twice
//!  (`doc/planning/cli-git-resident-service.md`, "Durability and crash recovery").
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeIntent(dir, intent); // file fsync, rename, directory fsync: the batch's only synchronous write
//! ```

use super::service_protocol::Cleanup;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Extension of an intent whose batch may be unfinished.
pub const OPEN_EXTENSION: &str = "intent";

/// Extension of a finished batch, kept so its request identities are remembered.
pub const DONE_EXTENSION: &str = "done";

/// Extension of a file being written; never read as an intent.
const TEMPORARY_EXTENSION: &str = "tmp";

/// What:
///  What the service decided to do for a batch, written before the branch moves.
/// Why:
///   Enough to finish the batch after a kill: where the branch was, where it goes, which requests, which index paths.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Intent = { seq; repository; commonDir; reference; old; new; ids; paths };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Intent {
    /// Number of the batch, unique and increasing within one journal.
    pub seq: u64,
    /// Worktree root whose index the batch updates.
    pub repository: PathBuf,
    /// Common git directory.
    pub common_dir: PathBuf,
    /// The branch.
    pub reference: String,
    /// The branch before the batch.
    pub old: String,
    /// The branch after the batch.
    pub new: String,
    /// Identities of the requests in the batch.
    pub ids: Vec<String>,
    /// The commit each request became, in the same order as the identities.
    pub commits: Vec<String>,
    /// Paths whose index entries the batch updates.
    pub paths: Vec<String>,
    /// Whether the index entries to update are the paths that differ between `old` and `new`, read from the repository
    ///  when they are needed (a commit Git built has no list of changes to carry, and its names are bytes).
    pub derived: bool,
    /// The conclusion state the landing removes after the branch moved; nothing for any other batch.
    pub cleanup: Option<Cleanup>,
    /// The real index whose lock the landing held, so a restart can remove its own stale lock and finish the index.
    pub index: PathBuf,
}

/// What a restart does with an open intent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Recovery {
    /// The batch never landed; forget the intent.
    Drop,
    /// The batch landed; make sure the index shows it, then remember the identities.
    Finish,
}

/// What:
///  The decision for an open intent from the branch's current value.
/// Why:
///   The branch is the commit point: still at the old value means nothing happened, at the new value or past it
///  (`new_is_ancestor_of_current`) means the batch landed, anywhere else means it cannot land now.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// recoveryFor(intent, current, newIsAncestorOfCurrent)
/// ```
pub fn recovery_for(intent: &Intent, current: &str, new_is_ancestor_of_current: bool) -> Recovery {
    if current == intent.new || (current != intent.old && new_is_ancestor_of_current) {
        return Recovery::Finish;
    }
    return Recovery::Drop;
}

/// The text of an intent: `key=value` lines, tabs between list items.
pub fn intent_text(intent: &Intent) -> String {
    let cleanup: String = intent.cleanup.as_ref().map_or_else(String::new, |found| {
        return format!("{}\t{}\t{}", found.git_dir.display(), found.scratch.display(), u8::from(found.reftable));
    });
    return format!(
        "seq={}\nrepository={}\ncommon={}\nref={}\nold={}\nnew={}\nids={}\ncommits={}\npaths={}\nderived={}\ncleanup={}\nindex={}\n",
        intent.seq,
        intent.repository.display(),
        intent.common_dir.display(),
        intent.reference,
        intent.old,
        intent.new,
        intent.ids.join("\t"),
        intent.commits.join("\t"),
        intent.paths.join("\t"),
        u8::from(intent.derived),
        cleanup,
        intent.index.display(),
    );
}

/// What:
///  Reads an intent's text.
/// Why:
///   A damaged file is no intent: recovery drops it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// parseIntent(text) // undefined when damaged
/// ```
pub fn parse_intent(text: &str) -> Option<Intent> {
    let value = |key: &str| -> Option<&str> { return text.lines().find_map(|line| { return line.strip_prefix(key)?.strip_prefix('='); }); };
    let list = |key: &str| -> Option<Vec<String>> {
        return Some(value(key)?.split('\t').filter(|item| { return !item.is_empty(); }).map(str::to_string).collect());
    };
    return Some(Intent {
        seq: value("seq")?.parse().ok()?,
        repository: PathBuf::from(value("repository")?),
        common_dir: PathBuf::from(value("common")?),
        reference: value("ref")?.to_string(),
        old: value("old")?.to_string(),
        new: value("new")?.to_string(),
        ids: list("ids")?,
        commits: list("commits")?,
        paths: list("paths")?,
        // Intents written before these fields existed name neither.
        derived: value("derived") == Some("1"),
        index: value("index").map_or_else(|| { return PathBuf::from(value("repository").unwrap_or("")).join(".git").join("index"); }, PathBuf::from),
        cleanup: value("cleanup").and_then(|entry| {
            let mut parts = entry.split('\t');
            let (git_dir, scratch, reftable) = (parts.next()?, parts.next()?, parts.next()?);
            return Some(Cleanup { git_dir: PathBuf::from(git_dir), scratch: PathBuf::from(scratch), reftable: reftable == "1" });
        }),
    });
}

/// The path of a batch's file with an extension.
pub fn batch_path(dir: &Path, seq: u64, extension: &str) -> PathBuf {
    return dir.join(format!("{seq}.{extension}"));
}

/// What:
///  Writes an intent durably: the file is synced, renamed into place, and the directory is synced.
/// Why:
///   This is the only synchronous write of a batch, whatever its size.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeIntent(dir, intent)
/// ```
pub fn write_intent(dir: &Path, intent: &Intent) -> std::io::Result<()> {
    let temporary: PathBuf = batch_path(dir, intent.seq, TEMPORARY_EXTENSION);
    let mut file: File = File::create(&temporary)?;
    file.write_all(intent_text(intent).as_bytes())?;
    file.sync_all()?;
    fs::rename(&temporary, batch_path(dir, intent.seq, OPEN_EXTENSION))?;
    return File::open(dir)?.sync_all();
}

/// Marks a batch finished. Not synced: if the mark is lost a restart redoes an idempotent step.
pub fn mark_done(dir: &Path, seq: u64) -> std::io::Result<()> {
    return fs::rename(batch_path(dir, seq, OPEN_EXTENSION), batch_path(dir, seq, DONE_EXTENSION));
}

/// What:
///  The intents in the journal with an extension, in batch order, and the temporary files removed on the way.
/// Why:
///   Recovery reads the open ones; startup reads the done ones for the request identities.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// listIntents(dir, 'intent')
/// ```
pub fn list_intents(dir: &Path, extension: &str) -> Vec<(PathBuf, Option<Intent>)> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut found: Vec<(PathBuf, Option<Intent>)> = Vec::new();
    for entry in entries.flatten() {
        let path: PathBuf = entry.path();
        match path.extension().and_then(|text| { return text.to_str(); }) {
            Some(TEMPORARY_EXTENSION) => {
                let _removed = fs::remove_file(&path);
            }
            Some(found_extension) if found_extension == extension => {
                let parsed: Option<Intent> = fs::read_to_string(&path).ok().and_then(|text| { return parse_intent(text.as_str()); });
                found.push((path, parsed));
            }
            _ => {}
        }
    }
    found.sort_by_key(|(path, intent)| {
        return intent.as_ref().map_or_else(|| { return path.to_string_lossy().len() as u64; }, |parsed| { return parsed.seq; });
    });
    return found;
}

/// The request identities of every finished batch, each with the commit it became.
pub fn landed_commits(dir: &Path) -> HashMap<String, String> {
    let mut landed: HashMap<String, String> = HashMap::new();
    for (_, intent) in list_intents(dir, DONE_EXTENSION) {
        if let Some(parsed) = intent {
            landed.extend(parsed.ids.into_iter().zip(parsed.commits));
        }
    }
    return landed;
}

/// Journal controls stay out of the release executable.
#[cfg(test)]
#[path = "service_journal_tests.rs"]
mod tests;
