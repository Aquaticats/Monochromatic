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

use super::repo_read_layout::dot_git_target;
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

/// How old a temporary file must be before a listing treats it as the remains of a killed write and removes it. A write takes
///  milliseconds; the grace covers a writer stalled on a slow disk or a loaded host.
pub const TEMPORARY_GRACE: std::time::Duration = std::time::Duration::from_secs(10 * 60);

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
    /// The worktree's own Git directory, which names the instance that serves the batch; an intent written before it was
    ///  recorded reads as the Git directory of its repository's worktree.
    pub git_dir: PathBuf,
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
        "seq={}\nrepository={}\ncommon={}\nref={}\nold={}\nnew={}\nids={}\ncommits={}\npaths={}\nderived={}\ncleanup={}\nindex={}\ngitdir={}\n",
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
        intent.git_dir.display(),
    );
}

/// What:
///  The worktree's own Git directory, read from the worktree's `.git` entry: the entry itself when it is a directory, the
///  directory a `gitdir:` file names for a linked worktree, and `.git` under the worktree when it cannot be read.
/// Why:
///   An intent written before it recorded its Git directory has only the repository to name the worktree by; the worktree's
///  instance is the hash of this directory, so the recovery that finds the intent must resolve it the way a commit does.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// gitDirOfWorktree('/work/repo') // '/work/repo/.git', or '/work/main/.git/worktrees/repo' for a linked worktree
/// ```
fn git_dir_of_worktree(repository: &Path) -> PathBuf {
    return match dot_git_target(repository) {
        Some(Some((git_dir, _))) => git_dir,
        Some(None) | None => repository.join(".git"),
    };
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
    let index: PathBuf = value("index").map_or_else(|| { return PathBuf::from(value("repository").unwrap_or("")).join(".git").join("index"); }, PathBuf::from);
    let repository: PathBuf = PathBuf::from(value("repository")?);
    // An intent written before the Git directory was recorded names its worktree by the repository it was made for, never
    //  by the directory its index sits in, which `GIT_INDEX_FILE` can put anywhere.
    let worktree_git_dir: PathBuf = value("gitdir").map_or_else(|| { return git_dir_of_worktree(repository.as_path()); }, PathBuf::from);
    return Some(Intent {
        seq: value("seq")?.parse().ok()?,
        repository,
        common_dir: PathBuf::from(value("common")?),
        reference: value("ref")?.to_string(),
        old: value("old")?.to_string(),
        new: value("new")?.to_string(),
        ids: list("ids")?,
        commits: list("commits")?,
        paths: list("paths")?,
        // Intents written before these fields existed name neither.
        derived: value("derived") == Some("1"),
        index,
        git_dir: worktree_git_dir,
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

/// How many finished batches a journal keeps. A request is answered as already landed only when its own client sends it again
///  after a broken connection: at most `RECONNECTS` times, each after a pause of milliseconds and a wait for a service to
///  start of at most `START_WAIT` (`service_client.rs`), so the resend of a landed request arrives within seconds of the
///  landing. A service lands a batch in the time of a few Git processes, so this many batches reach back past any such resend.
pub const DONE_RETAINED: usize = 1024;

/// What:
///  Removes the files of the oldest finished batches, keeping the newest `retained`, and gives how many it removed.
/// Why:
///   A finished batch's file is only memory of the request identities it landed. Nothing else removed them, so a journal grew by
///  a file per batch for as long as the worktree lived. Batch numbers increase with time, so the oldest are the smallest.
///  Only files named as a batch number with the finished extension are touched, and a file that cannot be removed is left.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await pruneDone(dir, DONE_RETAINED) // 3: removed the three oldest
/// ```
pub fn prune_done(dir: &Path, retained: usize) -> usize {
    let Ok(entries) = fs::read_dir(dir) else {
        return 0;
    };
    let mut finished: Vec<(u64, PathBuf)> = entries
        .flatten()
        .filter_map(|entry| {
            let path: PathBuf = entry.path();
            if path.extension()? != DONE_EXTENSION || !entry.file_type().ok()?.is_file() {
                return None;
            }
            let seq: u64 = path.file_stem()?.to_str()?.parse().ok()?;
            return Some((seq, path));
        })
        .collect();
    finished.sort_by_key(|(seq, _)| { return std::cmp::Reverse(*seq); });
    return finished.into_iter().skip(retained).filter(|(_, path)| { return fs::remove_file(path).is_ok(); }).count();
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
