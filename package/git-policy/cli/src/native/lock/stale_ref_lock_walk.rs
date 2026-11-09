//! What:
//!  Which files below a Git directory are reference locks,
//! and which files go with one.
//! Why:
//!  Only the locks of references are judged here.
//! `index.lock`,
//! `config.lock`,
//! `shallow.lock`,
//! object and pack locks and
//!  every `~pid.lock` file belong to other protocols or to the lock they accompany.
//! The walk lists directory entries by
//!  their own file type,
//! so a symbolic link is never followed and a link named like a lock is never a candidate.
//! A repository keeps its references in one of two backends,
//! and each locks its own file:
//! the files backend locks `packed-refs.lock`,
//! `HEAD.lock` and `refs/**/*.lock`,
//! and the reftable backend locks `reftable/tables.list.lock`,
//! with no files-backend lock beside it (audit of 2026-11-09,
//! finding 1).
//! Both are read here,
//! whatever the repository's own backend is,
//! because a repository that switched backends may hold either.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const candidates = refLockCandidates(roots); // packed-refs.lock, HEAD.lock, refs/**/*.lock, reftable/tables.list.lock
//! ```

use crate::stopped_git_locks::{LOCK_SUFFIX, PID_LOCK_SUFFIX};
use std::path::{Path, PathBuf};

/// The staging file Git writes beside `packed-refs.lock` while it holds that lock (`refs/packed-backend.c:1398`).
const PACKED_STAGING: &str = "packed-refs.new";

/// The name of the lock of the packed references.
const PACKED_LOCK: &str = "packed-refs.lock";

/// The lock of the reftable backend's list of tables,
/// below the Git directory,
/// which every writer of a reference takes:
/// `reftable_new_stack` names the list `reftable/tables.list` and
/// `reftable_stack_init_addition` holds its lock while it writes (`reftable/stack.c`).
/// The list is written into the lock and renamed,
/// so the lock is empty for most of the hold,
/// and Git writes `tables.list~pid.lock` beside it under `core.lockfilePid` exactly as it does for the files backend.
const REFTABLE_TABLES_LOCK: &str = "reftable/tables.list.lock";

/// The files a Git directory locks for its references,
/// whichever backend it keeps them in.
const REFERENCE_LOCKS: [&str; 3] = [PACKED_LOCK, "HEAD.lock", REFTABLE_TABLES_LOCK];

/// A reference lock and the Git directory it was found below.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    /// The Git directory (or common Git directory) whose walk found it.
    pub root: PathBuf,
    /// The lock file,
    /// or for an orphan staging file the staging file itself.
    pub lock: PathBuf,
    /// Whether this is `packed-refs.new` found with no `packed-refs.lock` beside it.
    pub orphan_staging: bool,
}

/// Whether `path` names a regular file itself (a symbolic link does not count);
/// a path that cannot be looked at is not one.
fn is_regular_file(path: &Path) -> bool {
    return match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata.is_file(),
        Err(_) => false,
    };
}

/// Whether nothing exists at `path`:
/// looking at it fails with "not found" (any other failure is not an answer).
fn is_missing(path: &Path) -> bool {
    return match std::fs::symlink_metadata(path) {
        Ok(_) => false,
        Err(error) => error.kind() == std::io::ErrorKind::NotFound,
    };
}

/// Order two candidates by lock path.
fn by_lock(left: &Candidate, right: &Candidate) -> std::cmp::Ordering {
    return left.lock.cmp(&right.lock);
}

/// Whether two neighbours in a sorted list name the same lock.
fn same_lock(left: &mut Candidate, right: &mut Candidate) -> bool {
    return left.lock == right.lock;
}

/// Whether a directory entry is a regular file whose name makes it a reference lock.
fn is_reference_lock(path: &Path, is_file: bool) -> bool {
    let Some(name) = path.file_name().and_then(std::ffi::OsStr::to_str) else {
        return false;
    };
    return is_file && name.ends_with(LOCK_SUFFIX) && !name.ends_with(PID_LOCK_SUFFIX);
}

/// Every reference lock below `roots`,
/// in path order:
/// `packed-refs.lock`,
/// `HEAD.lock`,
/// the reftable backend's `reftable/tables.list.lock` and every lock under `refs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refLockCandidates(roots: string[]): Candidate[];
/// ```
pub fn ref_lock_candidates(roots: &[PathBuf]) -> Vec<Candidate> {
    let mut found: Vec<Candidate> = Vec::new();
    for root in roots {
        for name in REFERENCE_LOCKS {
            let path: PathBuf = root.join(name);
            if is_regular_file(path.as_path()) {
                found.push(Candidate { root: root.clone(), lock: path, orphan_staging: false });
            }
        }
        let staging: PathBuf = root.join(PACKED_STAGING);
        if is_regular_file(staging.as_path()) && is_missing(root.join(PACKED_LOCK).as_path()) {
            found.push(Candidate { root: root.clone(), lock: staging, orphan_staging: true });
        }
        let mut pending: Vec<PathBuf> = vec![root.join("refs")];
        while let Some(directory) = pending.pop() {
            let Ok(entries) = std::fs::read_dir(directory.as_path()) else {
                continue;
            };
            for entry in entries.flatten() {
                let Ok(kind) = entry.file_type() else {
                    continue;
                };
                if kind.is_dir() {
                    pending.push(entry.path());
                } else if is_reference_lock(entry.path().as_path(), kind.is_file()) {
                    found.push(Candidate { root: root.clone(), lock: entry.path(), orphan_staging: false });
                }
            }
        }
    }
    found.sort_by(by_lock);
    found.dedup_by(same_lock);
    return found;
}

/// The pid file Git writes beside a lock:
/// the lock's name without `.lock`,
/// then `~pid.lock`.
pub fn pid_file_beside(lock: &Path) -> PathBuf {
    let name: String = match lock.file_name() {
        Some(file_name) => file_name.to_string_lossy().into_owned(),
        None => String::new(),
    };
    let stem: &str = name.strip_suffix(LOCK_SUFFIX).unwrap_or(name.as_str());
    return lock.with_file_name(format!("{stem}{PID_LOCK_SUFFIX}"));
}

/// The files that go with a lock and exist now:
/// its pid file and,
/// for `packed-refs.lock`,
/// the staging file.
pub fn companions_of(lock: &Path) -> Vec<PathBuf> {
    let mut companions: Vec<PathBuf> = vec![pid_file_beside(lock)];
    if lock.file_name() == Some(std::ffi::OsStr::new(PACKED_LOCK)) {
        companions.push(lock.with_file_name(PACKED_STAGING));
    }
    let mut existing: Vec<PathBuf> = Vec::new();
    for path in companions {
        if is_regular_file(path.as_path()) {
            existing.push(path);
        }
    }
    return existing;
}
