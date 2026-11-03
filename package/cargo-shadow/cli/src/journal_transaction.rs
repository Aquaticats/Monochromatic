//! What:
//! The transaction handle of a run: a journal directory of lockfile and manifest snapshots, kept alive by an exclusive lock
//! that the run holds for as long as it lives.
//! Why:
//! The reviews of 2026-11-03 showed that naming a journal by process id and deciding liveness from `/proc` fails open
//! (a reused id, an unreadable `/proc`, a recorded identity from another namespace).
//! The kernel releases a lock when its holder dies, so "the lock can be taken" is the proof that the owner is gone,
//! and trying for it never waits (owner rule of 2026-11-03: "nothing should ever wait forever or indefinitely").
//! Every journal has its own unique name and is created, never reused; a `ready` record, written last, lists every file with its
//! length, so a recovery can tell a complete journal from a partial one.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // journal-transaction.ts
//! export function begin(root: string, snapshots: LockSnapshot[], expectsManifests: boolean): Transaction
//! export class Transaction { addManifests(edits: ManifestEdit[]): void; finish(): void }
//! ```

use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use rustix::fs::{FlockOperation, flock};

use crate::crates::LockSnapshot;
use crate::journal::{DONE_SUFFIX, INDEX_FILE, JOURNAL_DIR, LOCK_FILE, MANIFEST_INDEX_FILE, OWNER_FILE, READY_FILE, atomic_write, create_directories_synced, sync_directory, write_synced};
use crate::manifest_edit::ManifestEdit;

/// The most journal names tried when one exists already.
/// A bound chosen for the owner's rule of 2026-11-03 ("nothing should ever wait forever or indefinitely"); the names carry a
/// clock, a process id and a counter, so a second try is already unreachable in practice.
const MAX_JOURNAL_NAME_ATTEMPTS: u32 = 100;

/// The counter that keeps two journals of one process apart.
static JOURNAL_COUNTER: AtomicU64 = AtomicU64::new(0);

/// A journal held by its owner.
/// Dropping it without calling `finish` is what a killed run looks like to a recovery: the journal stays, its lock is free.
pub struct Transaction {
    /// The journal directory.
    directory: PathBuf,
    /// The locked file; closing it (by drop or by death) releases the lock.
    _lock: File,
    /// The files already listed for the ready record.
    listed: std::cell::RefCell<Vec<String>>,
}

/// What:
/// `pub fn encode_path(path: &Path) -> String` writes the bytes of a path as hexadecimal.
/// Why:
/// A path is bytes, not necessarily UTF-8; the journal must give back exactly the path it was given.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export const encodePath = (path: Buffer): string => path.toString('hex');
/// ```
pub fn encode_path(path: &Path) -> String {
    let mut text: String = String::new();
    for byte in path.as_os_str().as_bytes() {
        text.push_str(&format!("{byte:02x}"));
    }
    return text;
}

/// What:
/// `fn journal_name() -> String` is a unique, sortable name for a new journal (clock, process id, counter).
/// Why:
/// The name carries no meaning for liveness; it only has to be new.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const journalName = (): string => `${hrtime}-${pid}-${counter++}`;
/// ```
fn journal_name() -> String {
    let nanos: u128 = match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(elapsed) => elapsed.as_nanos(),
        Err(_) => 0,
    };
    return format!("{nanos:040}-{}-{}", std::process::id(), JOURNAL_COUNTER.fetch_add(1, Ordering::Relaxed));
}

/// What:
/// `pub fn try_lock(file: &File) -> io::Result<bool>` tries for the exclusive lock without waiting;
/// `Ok(false)` means someone holds it.
/// Why:
/// Nothing in cargo-shadow waits for a lock: a held lock is an answer (alive, or being recovered), not a reason to block.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function tryLock(file: number): boolean  // flock(LOCK_EX | LOCK_NB)
/// ```
pub fn try_lock(file: &File) -> io::Result<bool> {
    return match flock(file, FlockOperation::NonBlockingLockExclusive) {
        Ok(()) => Ok(true),
        Err(code) if code == rustix::io::Errno::WOULDBLOCK => Ok(false),
        Err(code) => Err(io::Error::from(code)),
    };
}

/// What:
/// `fn write_ready(directory: &Path, listed: &[String], manifests: bool) -> io::Result<()>` writes the ready record:
/// every listed file with its length.
/// Why:
/// A recovery trusts a journal only if every listed file is present with exactly that length.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeReady(directory: string, listed: string[], manifests: boolean): void
/// ```
fn write_ready(directory: &Path, listed: &[String], manifests: bool) -> io::Result<()> {
    let mut text: String = format!("v1\nmanifests {}\n", u8::from(manifests));
    for name in listed {
        let length: u64 = fs::metadata(directory.join(name))?.len();
        text.push_str(&format!("{name}\t{length}\n"));
    }
    text.push_str(&format!("end {}\n", listed.len()));
    atomic_write(&directory.join(READY_FILE), &text)?;
    return sync_directory(directory);
}

/// What:
/// `pub fn begin(root, snapshots, expects_manifests) -> io::Result<Transaction>` creates a new journal directory, locks it,
/// writes the snapshots and, when no manifests are expected, the ready record.
/// Why:
/// Nothing may be mutated without a complete recovery record.
/// The directory is created, never reused (an existing name is an error and a new name is tried),
/// so an earlier journal's snapshots are never truncated.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function begin(root: string, snapshots: LockSnapshot[], expectsManifests: boolean): Transaction
/// ```
pub fn begin(root: &Path, snapshots: &[LockSnapshot], expects_manifests: bool) -> io::Result<Transaction> {
    let journals: PathBuf = root.join(JOURNAL_DIR);
    create_directories_synced(&journals)?;
    let mut directory: Option<PathBuf> = None;
    for _attempt in 0..MAX_JOURNAL_NAME_ATTEMPTS {
        let candidate: PathBuf = journals.join(journal_name());
        match fs::create_dir(&candidate) {
            Ok(()) => {
                directory = Some(candidate);
                break;
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    let Some(directory) = directory else {
        return Err(io::Error::other(format!("{} could not be given a new journal name; run the command again", journals.display())));
    };
    let lock: File = OpenOptions::new().write(true).create_new(true).open(directory.join(LOCK_FILE))?;
    if !try_lock(&lock)? {
        return Err(io::Error::other("a journal that was just created is locked by someone else"));
    }
    write_synced(&directory.join(OWNER_FILE), &format!("v2\n{}\n", std::process::id()))?;
    let mut index: String = String::from("v2\n");
    let mut listed: Vec<String> = Vec::new();
    for (position, snapshot) in snapshots.iter().enumerate() {
        let name: String = format!("{position}.lock");
        write_synced(&directory.join(&name), &snapshot.source)?;
        listed.push(name);
        index.push_str(&snapshot.crate_dir);
        index.push('\n');
    }
    index.push_str(&format!("end {}\n", snapshots.len()));
    atomic_write(&directory.join(INDEX_FILE), &index)?;
    listed.insert(0, INDEX_FILE.to_string());
    if !expects_manifests {
        write_ready(&directory, &listed, false)?;
    }
    sync_directory(&directory)?;
    sync_directory(&journals)?;
    return Ok(Transaction { directory, _lock: lock, listed: std::cell::RefCell::new(listed) });
}

impl Transaction {
    /// What:
    /// `pub fn add_manifests(&self, edits) -> io::Result<()>` journals the manifests a `--breaking` run may rewrite and writes the
    /// ready record.
    /// Why:
    /// Their original text is the only copy once a run rewrites them, so it is synced before the run starts.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// addManifests(edits: ManifestEdit[]): void
    /// ```
    pub fn add_manifests(&self, edits: &[ManifestEdit]) -> io::Result<()> {
        let mut index: String = String::from("v1\n");
        let mut listed: Vec<String> = self.listed.borrow().clone();
        for (position, edit) in edits.iter().enumerate() {
            if edit.crate_dir.contains('\n') || edit.crate_dir.contains('\t') {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, format!("the crate root {:?} contains a line feed or tab, which the journal cannot list", edit.crate_dir)));
            }
            let name: String = format!("m{position}.toml");
            write_synced(&self.directory.join(&name), &edit.before)?;
            listed.push(name);
            index.push_str(&format!("{}\t{}\n", edit.crate_dir, encode_path(&edit.path)));
        }
        index.push_str(&format!("end {}\n", edits.len()));
        atomic_write(&self.directory.join(MANIFEST_INDEX_FILE), &index)?;
        listed.push(MANIFEST_INDEX_FILE.to_string());
        write_ready(&self.directory, &listed, true)?;
        *self.listed.borrow_mut() = listed;
        return sync_directory(&self.directory);
    }

    /// What:
    /// `pub fn finish(self) -> io::Result<()>` settles the transaction: the journal is renamed to its `.done` name (the durable
    /// completion marker), then removed.
    /// Why:
    /// A removal that stops partway leaves a `.done` directory, which the next recovery finishes, and never a journal that looks
    /// incomplete.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// finish(): void
    /// ```
    pub fn finish(self) -> io::Result<()> {
        return mark_done_and_remove(&self.directory);
    }
}

/// What:
/// `pub fn mark_done_and_remove(directory: &Path) -> io::Result<()>` renames a journal to its `.done` name, syncs the parent,
/// removes the directory and syncs the parent again.
/// Why:
/// The rename is the completion marker; failures of the removal are reported, not ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// export function markDoneAndRemove(directory: string): void
/// ```
pub fn mark_done_and_remove(directory: &Path) -> io::Result<()> {
    let parent: PathBuf = match directory.parent() {
        Some(found) => found.to_path_buf(),
        None => PathBuf::from("."),
    };
    let name: String = directory.file_name().map(|found| { return found.to_string_lossy().to_string(); }).unwrap_or_default();
    let done: PathBuf = if name.ends_with(DONE_SUFFIX) { directory.to_path_buf() } else { parent.join(format!("{name}{DONE_SUFFIX}")) };
    if done != directory {
        fs::rename(directory, &done)?;
        sync_directory(&parent)?;
    }
    fs::remove_dir_all(&done)?;
    return sync_directory(&parent);
}
