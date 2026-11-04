//! What:
//!  Which files below a Git directory are reference locks, and which files go with one.
//! Why:
//!  Only the locks of references are judged here. `index.lock`, `config.lock`, `shallow.lock`, object and pack locks and
//!  every `~pid.lock` file belong to other protocols or to the lock they accompany. The walk lists directory entries by
//!  their own file type, so a symbolic link is never followed and a link named like a lock is never a candidate.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const candidates = refLockCandidates(roots); // packed-refs.lock, HEAD.lock, refs/**/*.lock
//! ```

use crate::stopped_git_locks::{LOCK_SUFFIX, PID_LOCK_SUFFIX};
use std::path::{Path, PathBuf};

/// The staging file Git writes beside `packed-refs.lock` while it holds that lock (`refs/packed-backend.c:1398`).
const PACKED_STAGING: &str = "packed-refs.new";

/// The name of the lock of the packed references.
const PACKED_LOCK: &str = "packed-refs.lock";

/// A reference lock and the Git directory it was found below.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    /// The Git directory (or common Git directory) whose walk found it.
    pub root: PathBuf,
    /// The lock file, or for an orphan staging file the staging file itself.
    pub lock: PathBuf,
    /// Whether this is `packed-refs.new` found with no `packed-refs.lock` beside it.
    pub orphan_staging: bool,
}

/// Whether a directory entry is a regular file whose name makes it a reference lock.
fn is_reference_lock(path: &Path, is_file: bool) -> bool {
    let Some(name) = path.file_name().and_then(|name| { return name.to_str(); }) else {
        return false;
    };
    return is_file && name.ends_with(LOCK_SUFFIX) && !name.ends_with(PID_LOCK_SUFFIX);
}

/// Every reference lock below `roots`, in path order: `packed-refs.lock`, `HEAD.lock` and every lock under `refs`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function refLockCandidates(roots: string[]): Candidate[];
/// ```
pub fn ref_lock_candidates(roots: &[PathBuf]) -> Vec<Candidate> {
    let mut found: Vec<Candidate> = Vec::new();
    for root in roots {
        for name in [PACKED_LOCK, "HEAD.lock"] {
            let path: PathBuf = root.join(name);
            if std::fs::symlink_metadata(path.as_path()).is_ok_and(|metadata| { return metadata.is_file(); }) {
                found.push(Candidate { root: root.clone(), lock: path, orphan_staging: false });
            }
        }
        let staging: PathBuf = root.join(PACKED_STAGING);
        if std::fs::symlink_metadata(staging.as_path()).is_ok_and(|metadata| { return metadata.is_file(); })
            && std::fs::symlink_metadata(root.join(PACKED_LOCK)).is_err_and(|error| { return error.kind() == std::io::ErrorKind::NotFound; })
        {
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
    found.sort_by(|left, right| { return left.lock.cmp(&right.lock); });
    found.dedup_by(|left, right| { return left.lock == right.lock; });
    return found;
}

/// The pid file Git writes beside a lock: the lock's name without `.lock`, then `~pid.lock`.
pub fn pid_file_beside(lock: &Path) -> PathBuf {
    let name: String = lock.file_name().map(|name| { return name.to_string_lossy().into_owned(); }).unwrap_or_default();
    let stem: &str = name.strip_suffix(LOCK_SUFFIX).unwrap_or(name.as_str());
    return lock.with_file_name(format!("{stem}{PID_LOCK_SUFFIX}"));
}

/// The files that go with a lock and exist now: its pid file and, for `packed-refs.lock`, the staging file.
pub fn companions_of(lock: &Path) -> Vec<PathBuf> {
    let mut companions: Vec<PathBuf> = vec![pid_file_beside(lock)];
    if lock.file_name().is_some_and(|name| { return name == PACKED_LOCK; }) {
        companions.push(lock.with_file_name(PACKED_STAGING));
    }
    companions.retain(|path| { return std::fs::symlink_metadata(path).is_ok_and(|metadata| { return metadata.is_file(); }); });
    return companions;
}
