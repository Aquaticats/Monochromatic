//! What:
//!  What a carry writes,
//! and the vocabulary of one step's writes:
//!  the index records of a batch,
//!  one file's bytes through a private name and a rename,
//!  a symbolic link,
//!  and the bounded wait for `index.lock` that begins the batch.
//! Why:
//!   Every write of one step lands as one step of the file system or of the index,
//!  so a kill leaves either what was there or the whole result;
//!  the judgements that decide a step are `merge_isolation_carry_apply.rs`,
//!  which is at its code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! await writeSteps({ context, root, gitDir, steps, out }); // one update-index call, then one file at a time
//! ```

use crate::merge_isolation_carry_reads::run_bytes_awaiting;
use crate::merge_isolation_changes::worktree_path;
use crate::transaction_git::{GitContext, GitOutput};
use std::ffi::OsString;
use std::path::{Path, PathBuf};

/// The mode an `update-index --index-info` record gives a path it removes.
const REMOVAL_MODE: &str = "0";

/// The mode of a symbolic link.
const SYMBOLIC_LINK_MODE: &str = "120000";

/// What one layer of one path writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type IndexWrite = { kind: 'leave' } | { kind: 'entry'; mode: string; oid: string } | { kind: 'remove' };
/// ```
pub(crate) enum IndexWrite {
    /// The layer already holds it.
    Leave,
    /// The entry to write.
    Entry(String, String),
    /// Nothing stands at the path.
    Remove,
}

/// What the worktree of one path writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type FileWrite = { kind: 'leave' } | { kind: 'bytes'; bytes: Uint8Array; mode: string } | { kind: 'remove' };
/// ```
pub(crate) enum FileWrite {
    /// The layer already holds it.
    Leave,
    /// The file form and the mode to write.
    Bytes(Vec<u8>, String),
    /// Nothing stands at the path.
    Remove,
}

/// One path's writes and what the report says of it.
pub(crate) struct Step {
    /// The path the step writes,
    /// from the top of the worktree.
    pub(crate) path: Vec<u8>,
    /// What its index entry becomes.
    pub(crate) index: IndexWrite,
    /// What its worktree file becomes.
    pub(crate) worktree: FileWrite,
}

/// Which wait for the index lock a step makes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockWait = 'before-the-apply' | 'while-finishing';
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum LockWait {
    /// The merge's own apply is about to run,
    /// so the wait's words are the merge's own and a dead owner's lock is cleared.
    BeforeTheApply,
    /// A merge that is being finished,
    /// where nothing is written when the lock stays held and the next command tries again.
    WhileFinishing,
}

/// Why a carry or a clear stopped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CarryFailure = { kind: 'index-held'; detail: string } | { kind: 'failed'; detail: string } | { kind: 'foreign'; path: Uint8Array };
/// ```
#[derive(Debug)]
pub enum CarryFailure {
    /// Another program held `index.lock` until the wait's bound,
    /// so nothing was written.
    IndexHeld(String),
    /// A read or a write failed;
    /// the capture still holds the bytes.
    Failed(String),
    /// One path holds bytes neither the capture nor the change explains.
    Foreign(Vec<u8>),
}

/// What:
///  The index records that write these entries and remove these paths.
/// Why:
///  One `update-index` call writes every entry,
/// so the index moves between two states in one step and a kill cannot
///  leave half of it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function indexRecords(steps: Step[], width: number): Uint8Array;
/// ```
fn index_records(steps: &[Step], width: usize) -> Vec<u8> {
    let mut records: Vec<u8> = Vec::new();
    for step in steps {
        match &step.index {
            IndexWrite::Leave => continue,
            IndexWrite::Entry(mode, oid) => records.extend_from_slice(format!("{mode} {oid}\t").as_bytes()),
            IndexWrite::Remove => records.extend_from_slice(format!("{REMOVAL_MODE} {}\t", "0".repeat(width)).as_bytes()),
        }
        records.extend_from_slice(step.path.as_slice());
        records.push(0);
    }
    return records;
}

/// What:
///  Write one path's file under a private name in its own directory and rename it into place,
/// or make the link,
/// or remove what stands there.
/// Why:
///   A rename in the same directory is one step on whatever file system the worktree is on,
/// so a kill leaves either
///  the file that was there or the whole new one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeFile(root, path, write: FileWrite): Promise<void>;
/// ```
fn write_file(root: &Path, path: &[u8], write: &FileWrite) -> std::io::Result<()> {
    let location: PathBuf = worktree_path(root, path);
    match write {
        FileWrite::Leave => return Ok(()),
        FileWrite::Remove => {
            return match std::fs::remove_file(location.as_path()) {
                Ok(()) => Ok(()),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
                Err(error) => Err(error),
            };
        }
        FileWrite::Bytes(bytes, mode) => {
            if let Some(parent) = location.parent()
                && let Err(error) = std::fs::create_dir_all(parent)
            {
                return Err(error);
            }
            if mode == SYMBOLIC_LINK_MODE {
                return make_link(location.as_path(), bytes);
            }
            let aside: PathBuf = private_name(location.as_path());
            std::fs::write(aside.as_path(), bytes)?;
            set_mode(aside.as_path(), mode)?;
            return std::fs::rename(aside.as_path(), location.as_path());
        }
    }
}

/// What:
///  Replace whatever stands at a path with a symbolic link to `target`.
/// Why:
///   A link cannot be renamed into place as a file can,
/// so the path is freed and the link made;
/// a kill between the
///  two leaves the path absent and the capture still holds its target text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function makeLink(location, target: Uint8Array): Promise<void>;
/// ```
fn make_link(location: &Path, target: &[u8]) -> std::io::Result<()> {
    match std::fs::remove_file(location) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    #[cfg(unix)]
    {
        use std::os::unix::ffi::OsStrExt;
        return std::os::unix::fs::symlink(std::ffi::OsStr::from_bytes(target), location);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        return std::os::unix::fs::symlink(String::from_utf8_lossy(target).into_owned(), location);
    }
}

/// The file name a private name keeps,
/// spelled as text a file system takes on every platform this builds for.
fn the_file_name(found: &std::ffi::OsStr) -> String {
    return found.to_string_lossy().into_owned();
}

/// The private name a file is written under before it takes its path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${dirname(location)}/.cli-git-carry-${pid}-${count}-${nanos}-${basename(location)}`
/// ```
fn private_name(location: &Path) -> PathBuf {
    static COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let count: u64 = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    let name: String = location.file_name().map(the_file_name).unwrap_or_default();
    let process: u32 = std::process::id();
    return location.with_file_name(format!(".cli-git-carry-{process}-{}-{count}-{nanos:x}-{name}", 0u8));
}

/// The mode a written file carries.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function setMode(file, mode: string): Promise<void>; // 0o755 for 100755, else 0o644
/// ```
fn set_mode(file: &Path, mode: &str) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let bits: u32 = if mode == "100755" { 0o755 } else { 0o644 };
        return std::fs::set_permissions(file, std::fs::Permissions::from_mode(bits));
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = (file, mode);
        return Ok(());
    }
}

/// The width of the object name these steps write,
/// from the first step that writes one.
/// Why:
///   A removal record repeats an all-zero object name of the same width,
///  and only an entry's own name knows what that width is.
fn entry_width(step: &Step) -> Option<usize> {
    return match &step.index {
        IndexWrite::Entry(_, oid) => Some(oid.len()),
        IndexWrite::Remove | IndexWrite::Leave => None,
    };
}

/// What:
///  Write every planned entry and file,
/// after a bounded wait for `index.lock`.
/// Why:
///   The index is the one thing two commands cannot write at once,
/// so the wait is named and bounded,
/// and a lock
///  that outlasts it leaves everything as it was.
///  Both Git starts are awaited,
///  because a start enters the runtime and is refused from inside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function writeStepsAwaiting({ context, root, gitDir, steps, out }): Promise<void>;
/// ```
pub(crate) async fn write_steps_awaiting<W: std::io::Write + ?Sized>(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    steps: &[Step],
    wait: LockWait,
    out: &mut W,
) -> Result<(), CarryFailure> {
    let waited = match wait {
        LockWait::BeforeTheApply => crate::index_holder_wait::wait_for_index_release_before_a_merge_that_clears_a_dead_lock(git_dir.join("index").as_path(), out),
        LockWait::WhileFinishing => crate::index_holder_wait::wait_for_index_release_while_finishing_a_merge(git_dir.join("index").as_path(), out),
    };
    if !matches!(
        waited,
        crate::index_holder_wait::IndexWait::NotHeld | crate::index_holder_wait::IndexWait::Released(_)
    ) {
        return Err(CarryFailure::IndexHeld(git_dir.join("index.lock").display().to_string()));
    }
    let width: usize = steps.iter().find_map(entry_width).unwrap_or(40);
    let records: Vec<u8> = index_records(steps, width);
    if !records.is_empty() {
        let arguments: Vec<OsString> = vec![OsString::from("update-index"), OsString::from("-z"), OsString::from("--index-info")];
        let written: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), Some(records)).await.map_err(CarryFailure::Failed)?;
        if !written.succeeded() {
            let detail: String = written.error_text();
            if detail.contains("index.lock") {
                return Err(CarryFailure::IndexHeld(detail));
            }
            return Err(CarryFailure::Failed(detail));
        }
    }
    let mut wrote_a_file: bool = false;
    for step in steps {
        if !matches!(step.worktree, FileWrite::Leave) {
            wrote_a_file = true;
        }
        if let Err(error) = write_file(root, step.path.as_slice(), &step.worktree) {
            let path: String = String::from_utf8_lossy(step.path.as_slice()).into_owned();
            let _ = out.write_all(
                format!(
                    "monochromatic-git-policy-cli: {path} could not be written ({error}); the bytes this merge carries are kept, and the next \
                     git command in this worktree finishes carrying them.\n"
                )
                .as_bytes(),
            );
            return Err(CarryFailure::Failed(format!("{path} could not be written: {error}")));
        }
    }
    // A file this step wrote carries a new modification time and size, and Git compares those with the index's record of the
    //  file before it updates a path (`read-tree` refuses an entry that is not up to date). The refresh re-reads the record for
    //  every entry whose content still hashes to what the index says, so the next step reads the checkout as it is.
    if wrote_a_file {
        let arguments: Vec<OsString> = vec![OsString::from("update-index"), OsString::from("-q"), OsString::from("--refresh")];
        let refreshed: GitOutput = run_bytes_awaiting(context, root, arguments.as_slice(), None).await.map_err(CarryFailure::Failed)?;
        if !refreshed.succeeded() && refreshed.code != Some(1) {
            return Err(CarryFailure::Failed(refreshed.error_text()));
        }
    }
    return Ok(());
}
