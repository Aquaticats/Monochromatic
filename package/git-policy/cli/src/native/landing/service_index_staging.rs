//! What:
//!  The staging files of an index lock:
//! their names,
//! their creation with the marker written,
//! and the removal of the ones a killed attempt left.
//! Why:
//!  The marker is written to a file that is new and becomes the lock in one step,
//! so the lock never exists without it.
//! A staging name carries the process and a token no other attempt has,
//! so no attempt opens a file another left,
//! and a later attempt tells from the name alone whether the writer still runs.
//! The module is split from `service_index_placement.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const { path, handle } = await stageMarker(lock, seq, marker)
//! ```

use crate::process_identity::process_birth_identity;
use crate::random_id::random_uuid;
use crate::service_index_marker::{HolderState, ReaderView, current_pid_scope, holder_state_in, marker_of};
use std::io::Write;
use std::path::{Path, PathBuf};

/// What follows the lock's file name in the name of a staging file:
/// the marker is written there before it becomes the lock.
pub const STAGED_INFIX: &str = ".cli-git-marker";

/// What:
///  The name of a staging file of the lock at `lock`:
/// this process with its PID namespace,
/// the batch,
/// and a token that no other attempt has.
/// Why:
///   The process number alone recurs once the process is gone,
/// and a name that recurs can be the name of a hard-link alias a killed holder left,
/// which is the lock's own inode;
/// the token makes every name new.
/// The process number means something only in its PID namespace,
/// and two services in different namespaces can share a Git directory,
/// so the namespace is in the name and a cleaner in another one does not read the number as its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${lock}.cli-git-marker-${process.pid}.${pidNamespace}-${seq}-${token}`
/// ```
pub fn staged_path_for(lock: &Path, seq: u64, token: &str) -> PathBuf {
    let mut name = lock.as_os_str().to_os_string();
    match current_pid_scope() {
        Some(scope) => name.push(format!("{STAGED_INFIX}-{}.{scope}-{seq}-{token}", std::process::id())),
        None => name.push(format!("{STAGED_INFIX}-{}-{seq}-{token}", std::process::id())),
    }
    return PathBuf::from(name);
}

/// What:
///  Creates the staging file of the lock at `lock` for batch `seq`,
/// new and holding `marker`.
/// Why:
///   Opened with exclusive creation and never truncation,
/// so a name that already exists (whatever it is a name of) is an error and no file of another attempt is emptied.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(stagedPath, marker, { flag: 'wx' })
/// ```
///
/// # Errors
///
/// The storage error when the file cannot be created or written;
/// it is removed again when only the write failed.
/// The file is returned open:
/// the holder keeps it for as long as the hold lasts,
/// because it is the lock's own inode.
pub fn stage_marker(lock: &Path, seq: u64, marker: &str) -> std::io::Result<(PathBuf, std::fs::File)> {
    let token: String = random_uuid().map_err(|error| { return std::io::Error::other(error.0); })?;
    let staged: PathBuf = staged_path_for(lock, seq, token.as_str());
    let mut file = std::fs::OpenOptions::new().read(true).write(true).create_new(true).open(&staged)?;
    if let Err(error) = file.write_all(marker.as_bytes()) {
        drop(file);
        let _removed = std::fs::remove_file(&staged);
        return Err(error);
    }
    return Ok((staged, file));
}

/// The process number and the PID namespace a staging file's name carries.
/// A name of an earlier release carries a number and no namespace,
/// and the fixed name of the earliest releases carries neither.
fn writer_in_name(suffix: &[u8]) -> Option<(i64, Option<String>)> {
    let rest: &[u8] = suffix.strip_prefix(b"-")?;
    let digits: Vec<u8> = rest.iter().copied().take_while(u8::is_ascii_digit).collect();
    let pid: i64 = std::str::from_utf8(digits.as_slice()).ok()?.parse::<i64>().ok()?;
    let after: &[u8] = &rest[digits.len()..];
    let scope: Option<String> = after.strip_prefix(b".").map(|tail| {
        let number: Vec<u8> = tail.iter().copied().take_while(u8::is_ascii_digit).collect();
        return String::from_utf8_lossy(number.as_slice()).into_owned();
    });
    return Some((pid, scope.filter(|text| { return !text.is_empty(); })));
}

/// What:
///  Whether a staging file is a killed writer's.
/// Why:
///   A whole marker decides alone:
/// it names its writer by birth identity,
/// boot and PID namespace,
/// and the reader's verdict (gone,
/// running,
/// unknown) is the file's,
/// so a writer that runs,
/// or that this reader cannot compare with,
/// keeps its file whatever its name says.
/// Without a whole marker (empty,
/// half written,
/// or the fixed name of the earliest releases) the name decides,
/// and only when it carries the PID namespace of this reader and this reader's `/proc` numbers it as the system does:
/// a number of another namespace,
/// or of an earlier release's name that says none,
/// is not looked up here,
/// and a `/proc` that cannot be read is no evidence of death.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// markerOf(path)?.holder ? holderState(holder) === 'gone' : writerInNameIsGone(name)
/// ```
fn staging_is_dead(path: &Path, suffix: &[u8]) -> bool {
    return staging_is_dead_in(path, suffix, &ReaderView::current(), &process_birth_identity);
}

/// `staging_is_dead` for a reader with the view `view`,
/// asking `ask` about processes,
/// which a control gives a `/proc` that disagrees with the reader's PID.
pub fn staging_is_dead_in(
    path: &Path,
    suffix: &[u8],
    view: &ReaderView,
    ask: &dyn Fn(i64) -> Result<Option<String>, crate::process_identity::ProcessIdentityError>,
) -> bool {
    if suffix.is_empty() {
        // The fixed name an earlier release used: no attempt of this or the last release writes it.
        return true;
    }
    if let Some(marked) = marker_of(path) {
        return holder_state_in(marked.holder.as_ref(), view, ask) == HolderState::Gone;
    }
    let Some((pid, Some(scope))) = writer_in_name(suffix) else { return false };
    if !view.procfs_agrees || view.scope != Some(scope.as_str()) {
        return false;
    }
    return matches!(ask(pid), Ok(None));
}

/// What:
///  Removes the staging files of the lock at `lock` that a killed attempt left:
/// the fixed name an earlier release used,
/// and any that `staging_is_dead` says are a killed writer's.
/// Why:
///   A kill before the rename leaves the staging file behind.
/// An empty or half written file is the file of a writer that is still at it unless its name says otherwise,
/// and removing it fails that writer's placement.
/// A live writer's file is its own and stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const name of await readdir(dir)) if (name.startsWith(prefix) && stagingIsDead(name)) await rm(name)
/// ```
pub fn remove_dead_staging(lock: &Path) {
    use std::os::unix::ffi::OsStrExt;
    let (Some(directory), Some(file)) = (lock.parent(), lock.file_name()) else { return };
    let mut prefix: Vec<u8> = file.as_bytes().to_vec();
    prefix.extend_from_slice(STAGED_INFIX.as_bytes());
    let Ok(entries) = std::fs::read_dir(directory) else { return };
    for entry in entries.flatten() {
        let name = entry.file_name();
        let Some(suffix) = name.as_bytes().strip_prefix(prefix.as_slice()) else { continue };
        if staging_is_dead(entry.path().as_path(), suffix) {
            let _removed = std::fs::remove_file(entry.path());
        }
    }
}

/// What:
///  `stage_marker`,
/// with the staged file flushed to storage by `sync` before it is returned.
/// Why:
///   After a power loss the lock's directory entry can outlast the data of the file it names;
/// a marker flushed before the file becomes the lock leaves either no lock or a complete one.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const { path, handle } = await stageMarker(lock, seq, marker); await handle.sync()
/// ```
///
/// # Errors
///
/// As `stage_marker`,
/// and the flush failure;
/// the staging file is removed again then.
pub fn stage_marker_synced(sync: &dyn Fn(&std::fs::File) -> std::io::Result<()>, lock: &Path, seq: u64, marker: &str) -> std::io::Result<(PathBuf, std::fs::File)> {
    let (staged, file) = stage_marker(lock, seq, marker)?;
    if let Err(error) = sync(&file) {
        drop(file);
        let _removed = std::fs::remove_file(&staged);
        return Err(error);
    }
    return Ok((staged, file));
}

/// Sets the modification time of the staged file to now.
/// Why:
///   The wrapped commands judge an unreadable or holderless hold by the age of the lock file,
/// and the file is the staged one,
/// which was written before a wait of any length;
/// the time of the placement is the time the hold began.
pub fn refresh_age(file: &std::fs::File) {
    let _refreshed = file.set_modified(std::time::SystemTime::now());
}
