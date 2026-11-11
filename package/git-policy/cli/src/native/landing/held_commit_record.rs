//! What:
//!  The record of a commit that a reference lock keeps off its branch:
//!  the branch and its two values,
//!  the commits,
//!  the journal whose intent holds the batch,
//!  the locks that held the write off,
//!  and when it was held.
//! Why:
//!  Git refuses a reference update at once when a lock file stands,
//!  and no proof this repository has ties a reference lock to a holder that has ended,
//!  so the lock is never removed
//!  (`SPEC.md`,
//!  "Reference locks left by a Git that was killed").
//!  The owner's rule is that commits never fail,
//!  and the owner chose "Make it,
//!  hold until lock goes" (2026-11-09):
//!  the commit is written and kept on a side ref,
//!  the command exits `0`,
//!  and the next command that finds the lock gone moves the branch.
//!  This record is what makes that durable:
//!  it is published whole,
//!  either complete or absent,
//!  before the commit is answered,
//!  so a kill at any instruction leaves a record that names the batch or no record at all,
//!  and the commit is never lost and never applied twice.
//!  A command reads these records to say what a held commit waits for and to name the lock,
//!  and the settlement moves the branch from them
//!  (`held_commit_settle.rs`).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeHeld(gitDir, { seq, journal, reference, old, new, commits, index, reflog, locks, created });
//! for (const held of await listHeld(gitDir)) { ... }
//! ```

use crate::path_bytes_in_records::{HEX_SUFFIX, bytes_of_hex, hex_of_bytes, hex_of_path, path_of_hex};
use std::path::{Path, PathBuf};

/// The folder below a worktree's own Git directory that holds the records.
pub const FOLDER: &str = "monochromatic-git-policy-cli/held-commits";

/// The end of a record's name.
pub const HELD_EXTENSION: &str = "held";

/// The end of the name a record is staged under;
/// never read as one.
const TEMPORARY_EXTENSION: &str = "tmp";

/// Separates the items of one field.
const ITEM: &str = "\t";

/// One commit a reference lock keeps off its branch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Held {
    /// The batch number in the journal.
    pub seq: u64,
    /// The folder of the journal that holds the batch's intent.
    pub journal: PathBuf,
    /// The worktree root.
    pub repository: PathBuf,
    /// The common Git directory.
    pub common_dir: PathBuf,
    /// The worktree's own Git directory,
    /// which holds this record.
    pub git_dir: PathBuf,
    /// The branch.
    pub reference: String,
    /// The branch before the batch.
    pub old: String,
    /// The branch after the batch.
    pub new: String,
    /// The commit each request became,
    /// in the order of the batch.
    pub commits: Vec<String>,
    /// The real index whose entries the batch updates.
    pub index: PathBuf,
    /// The message the branch move writes in the reflog,
    /// byte for byte.
    pub reflog: Vec<u8>,
    /// The locks that stood when the branch write did not happen.
    pub locks: Vec<PathBuf>,
    /// When the commit was held,
    /// as ISO 8601.
    pub created: String,
}

/// What:
///  What a landing knows about a batch it is about to hold behind a reference lock.
/// Why:
///  The record is written from the batch's own intent,
///  which is already on disk,
///  and from the locks the judgement left standing,
///  so the record and the intent cannot describe different branch values.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Holding = { journal, repository, commonDir, gitDir, index, reference, seq, intent, locks, reflog };
/// ```
pub struct Holding<'a> {
    /// The folder of the journal that holds the batch's intent.
    pub journal: &'a Path,
    /// The worktree root.
    pub repository: &'a Path,
    /// The common Git directory.
    pub common_dir: &'a Path,
    /// The worktree's own Git directory,
    /// which is given the record.
    pub git_dir: &'a Path,
    /// The real index whose entries the batch updates.
    pub index: &'a Path,
    /// The branch.
    pub reference: &'a str,
    /// The batch number.
    pub seq: u64,
    /// The batch's intent,
    /// as the landing wrote it before the branch write.
    pub intent: &'a crate::service_journal::Intent,
    /// The locks that stood when the branch write did not happen.
    pub locks: Vec<PathBuf>,
    /// The message the branch move writes in the reflog,
    /// byte for byte.
    pub reflog: &'a [u8],
}

/// What:
///  Records a batch as held and answers the record.
/// Why:
///   The commit is answered `0` after this returns,
///  so the record is the only thing that makes it durable;
///  the intent it names stays where it is,
///  and the side reference holding the commit is left in place
///  (`service_scratch_ref_marker_of_a_batch.rs` keeps it while the intent exists).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = { ...holding, new: intent.new, old: intent.old, commits: intent.commits, created: nowIso() }; await write(held)
/// ```
///
/// # Errors
///
/// The operating system's answer when the record cannot be published.
pub fn hold(holding: Holding<'_>) -> std::io::Result<Held> {
    let held: Held = Held {
        seq: holding.seq,
        journal: holding.journal.to_path_buf(),
        repository: holding.repository.to_path_buf(),
        common_dir: holding.common_dir.to_path_buf(),
        git_dir: holding.git_dir.to_path_buf(),
        reference: holding.reference.to_string(),
        old: holding.intent.old.clone(),
        new: holding.intent.new.clone(),
        commits: holding.intent.commits.clone(),
        index: holding.index.to_path_buf(),
        reflog: holding.reflog.to_vec(),
        locks: holding.locks,
        created: crate::iso_time::now_iso(),
    };
    write(holding.git_dir, &held)?;
    return Ok(held);
}

/// What:
///  The folder of a worktree's held records.
/// Why:
///   The records are the worktree's own,
///  like the push notices and the work queue beside them,
///  and the folder is made when the first record is written.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(gitDir, 'monochromatic-git-policy-cli/held-commits')
/// ```
pub fn directory_of(git_dir: &Path) -> PathBuf {
    return git_dir.join(FOLDER);
}

/// The path of the record of batch `seq`.
pub fn path_of(git_dir: &Path, seq: u64) -> PathBuf {
    return directory_of(git_dir).join(format!("{seq}.{HELD_EXTENSION}"));
}

/// One field of the record that holds a path,
/// its bytes in hexadecimal.
fn field_of_path(key: &str, path: &Path) -> String {
    return format!("{key}{HEX_SUFFIX}={}", hex_of_path(path));
}

/// The items of a list field,
/// each in hexadecimal,
/// separated by a tab.
fn items_of_paths(paths: &[PathBuf]) -> String {
    let mut words: Vec<String> = Vec::new();
    for path in paths {
        words.push(hex_of_path(path.as_path()));
    }
    return words.join(ITEM);
}

/// The one text form of a record.
fn text(held: &Held) -> String {
    let lines: Vec<String> = vec![
        format!("seq={}", held.seq),
        format!("ref={}", held.reference),
        format!("old={}", held.old),
        format!("new={}", held.new),
        format!("commits={}", held.commits.join(ITEM)),
        format!("reflogHex={}", hex_of_bytes(held.reflog.as_slice())),
        format!("locksHex={}", items_of_paths(held.locks.as_slice())),
        format!("created={}", held.created),
        field_of_path("journal", held.journal.as_path()),
        field_of_path("repository", held.repository.as_path()),
        field_of_path("common", held.common_dir.as_path()),
        field_of_path("gitdir", held.git_dir.as_path()),
        field_of_path("index", held.index.as_path()),
    ];
    return format!("{}\n", lines.join("\n"));
}

/// The value of a field of a record's text.
fn value_of<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    for line in text.lines() {
        if let Some(found) = line.strip_prefix(key).and_then(without_the_key) {
            return Some(found);
        }
    }
    return None;
}

/// The rest of a record line after the `=` that ends its key.
/// Why:
///  `value_of` reads one field of the text,
///  and the separator belongs to the spelling of a record and to no other step.
fn without_the_key(rest: &str) -> Option<&str> {
    return rest.strip_prefix('=');
}

/// The words of a list field,
/// empty for a field that names nothing.
fn items(text: &str, key: &str) -> Vec<String> {
    return match value_of(text, key) {
        Some(found) if !found.is_empty() => found.split(ITEM).map(str::to_string).collect(),
        _ => Vec::new(),
    };
}

/// The paths of a list field,
/// nothing when one of its items is not a path's bytes.
fn paths(text: &str, key: &str) -> Option<Vec<PathBuf>> {
    let mut found: Vec<PathBuf> = Vec::new();
    for item in items(text, key) {
        found.push(path_of_hex(item.as_str())?);
    }
    return Some(found);
}

/// What a record's text says about one held batch,
/// nothing when it is damaged.
fn parse(text: &str) -> Option<Held> {
    return Some(Held {
        seq: value_of(text, "seq")?.parse::<u64>().ok()?,
        reference: value_of(text, "ref")?.to_string(),
        old: value_of(text, "old")?.to_string(),
        new: value_of(text, "new")?.to_string(),
        commits: items(text, "commits"),
        reflog: bytes_of_hex(value_of(text, "reflogHex")?)?,
        locks: paths(text, "locksHex")?,
        created: value_of(text, "created")?.to_string(),
        journal: path_of_hex(value_of(text, "journalHex")?)?,
        repository: path_of_hex(value_of(text, "repositoryHex")?)?,
        common_dir: path_of_hex(value_of(text, "commonHex")?)?,
        git_dir: path_of_hex(value_of(text, "gitdirHex")?)?,
        index: path_of_hex(value_of(text, "indexHex")?)?,
    });
}

/// What:
///  Publishes the record of a held commit,
///  synced,
///  in one rename.
/// Why:
///   The command answers `0` after this returns,
///  so a power loss must not take the record and leave the person believing a commit was made
///  that neither the branch nor the kept refs can show.
///  The record is staged under a name nothing reads and published whole,
///  so a kill leaves the record it was writing or none,
///  never half of one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writePrivateFile(staged); await rename(staged, record); await syncDirectory(folder);
/// ```
///
/// # Errors
///
/// The operating system's answer when the folder or the record cannot be written.
pub fn write(git_dir: &Path, held: &Held) -> std::io::Result<()> {
    let folder: PathBuf = directory_of(git_dir);
    crate::private_storage::create_private_directory_all(folder.as_path())?;
    let staged: PathBuf = folder.join(format!("{}.{TEMPORARY_EXTENSION}", held.seq));
    let _stale = std::fs::remove_file(staged.as_path());
    crate::private_storage::write_private_file(staged.as_path(), text(held).as_bytes())?;
    let record: PathBuf = path_of(git_dir, held.seq);
    std::fs::rename(staged.as_path(), record.as_path())?;
    return crate::private_storage::sync_directory(folder.as_path());
}

/// What:
///  Every readable record of a worktree,
///  oldest batch number first,
///  with the lines of the ones that cannot be read.
/// Why:
///   A record this build cannot read names a batch whose commit is kept on the side ref the settlement names,
///  so it is said and never removed:
///  removing it would leave a commit nothing points at
///  (the owner's rule "never discard user data").
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of readdir(folder)) { name.endsWith('.held') ? read : nothing }
/// ```
pub fn list(git_dir: &Path) -> (Vec<Held>, Vec<PathBuf>) {
    let folder: PathBuf = directory_of(git_dir);
    let Ok(listing) = std::fs::read_dir(folder.as_path()) else {
        return (Vec::new(), Vec::new());
    };
    let mut found: Vec<Held> = Vec::new();
    let mut damaged: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let path: PathBuf = entry.path();
        if path.extension().and_then(std::ffi::OsStr::to_str) != Some(HELD_EXTENSION) {
            continue;
        }
        match std::fs::read_to_string(path.as_path()).ok().as_deref().and_then(parse) {
            Some(held) => found.push(held),
            None => damaged.push(path),
        }
    }
    found.sort_by_key(number_of);
    return (found, damaged);
}

/// The batch number of a record,
/// for the listing that puts them in the order they were made.
fn number_of(held: &Held) -> u64 {
    return held.seq;
}

/// What:
///  Forgets one record,
///  the settlement's last step for a batch that landed.
/// Why:
///  The branch move and the index update are already on disk by then,
///  so the record has nothing left to say and a stale one would keep a command
///  telling the person about a commit that is on the branch.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(join(folder, `${seq}.held`), { force: true })
/// ```
pub fn forget(git_dir: &Path, seq: u64) {
    let _removed = std::fs::remove_file(path_of(git_dir, seq));
}

/// Controls stay out of the release executable.
#[cfg(test)]
#[path = "held_commit_record_tests.rs"]
mod tests;
