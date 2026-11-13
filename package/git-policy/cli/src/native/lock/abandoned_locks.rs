//! What:
//!  The pass a command makes over its repository's lock files:
//!  every lock file of the worktree's Git directory,
//!  of the common Git directory and of every linked worktree,
//!  judged by one proof,
//!  with the ones whose holder is proven gone removed.
//! Why:
//!  A killed command can leave any of Git's locks behind:
//!  the index lock,
//!  the lock of `HEAD`,
//!  the shallow file,
//!  a reference or the list of the reftable backend's tables.
//!  The command that meets one of them must not wait a minute for a process that will never let go,
//!  and must not fail for something the wrapper can prove.
//!  So the pass is one walk and one proof,
//! and it names every lock it removed;
//!  a lock it cannot prove keeps the bounded wait and the report it always had.
//!  The walk reads directory entries by their own file type,
//!  so a symbolic link is never followed,
//!  and it skips the object store,
//! whose files are a pack writer's own.
//!  The pass runs from the wrapper's own process before the command reaches Git,
//!  and from the commit service before its landing waits for an index lock,
//!  so the next command of any kind meets a clear repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = clearAbandonedLocks(roots); stderr.write(outcome.notice());
//! ```

/// The proof and the removal of one lock.
use crate::abandoned_lock_proving as proving;
/// The names a lock and its PID file have.
use crate::stopped_git_locks::{LOCK_SUFFIX, PID_LOCK_SUFFIX};
/// The common Git directory that holds the records of the processes started here.
use crate::git_directory_roots::{common_dir_for, git_directory_of};
/// The records themselves:
/// the repository this command works in,
/// and the account of this process.
use crate::lock_holder_records::{record_this_process, remember_repository, the_self_record_is_this_processs};
/// The read of the records,
/// for the sweep that keeps only the ones that explain a lock.
use crate::lock_holder_record_reads::sweep;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};

/// How deep below a Git directory the walk looks,
/// which reaches every reference and every linked worktree's own directory.
const MOST_DEPTH: usize = 8;

/// How many directory entries one walk reads,
/// so a Git directory a program filled cannot hold a command.
const MOST_ENTRIES: usize = 8192;

/// The folders of a Git directory that hold no lock of the worktree's state:
/// the object store,
/// whose `*.lock` files are a pack writer's own temporary names,
/// and the logs,
/// which hold no lock at all.
const SKIPPED_FOLDERS: [&str; 2] = ["objects", "logs"];

/// What:
///  What one pass removed.
/// Why:
///  The caller writes one line per removal,
///  and the records are swept against the numbers that still stand beside a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Outcome = { cleared: Clearance[]; standing: string[] };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Outcome {
    /// Every lock the pass removed.
    pub cleared: Vec<proving::Clearance>,
    /// Every lock file that stood when the pass ended,
    ///  which is what the records of the processes started here are swept against.
    pub standing: Vec<PathBuf>,
    /// One line per standing lock that cannot release itself and that no record explains.
    pub unexplained: Vec<String>,
}

impl Outcome {
    /// What:
    ///  The lines the person reads,
    ///  one per lock that was removed,
    ///  and nothing when none was.
    /// Why:
    ///  A command that removed nothing says nothing,
    ///  so a healthy command prints no line about locks.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// outcome.cleared.map(clearanceLine).join('')
    /// ```
    pub fn notice(&self) -> String {
        let mut lines: String = String::new();
        for clearance in &self.cleared {
            lines.push_str(proving::clearance_line(clearance).as_str());
        }
        for line in &self.unexplained {
            lines.push_str(line.as_str());
        }
        return lines;
    }

    /// Whether any lock was removed.
    pub fn cleared_any(&self) -> bool {
        return !self.cleared.is_empty();
    }
}

/// What:
///  Every lock file below the Git directories `roots`,
///  in path order,
///  leaving out the PID files Git writes beside them.
/// Why:
///  One list decides which files the pass judges,
///  and a control can ask for it without a repository.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function lockCandidates(roots: string[]): string[];
/// ```
pub fn lock_candidates(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut seen: usize = 0;
    for root in roots {
        let mut pending: Vec<(PathBuf, usize)> = vec![(root.clone(), 0)];
        while let Some((directory, depth)) = pending.pop() {
            let Ok(entries) = std::fs::read_dir(directory.as_path()) else {
                continue;
            };
            for entry in entries.flatten() {
                seen += 1;
                if seen > MOST_ENTRIES {
                    break;
                }
                let Ok(kind) = entry.file_type() else {
                    continue;
                };
                if kind.is_dir() {
                    if depth < MOST_DEPTH && !SKIPPED_FOLDERS.contains(&entry.file_name().to_string_lossy().as_ref()) {
                        pending.push((entry.path(), depth + 1));
                    }
                    continue;
                }
                if kind.is_file() && is_a_lock_name(entry.file_name().to_string_lossy().as_ref()) {
                    found.push(entry.path());
                }
            }
        }
    }
    found.sort();
    found.dedup();
    return found;
}

/// What:
///  Whether a file name is a lock file's rather than the PID file Git writes beside one.
/// Why:
///  A PID file is the evidence about a lock and never a lock itself.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// name.endsWith('.lock') && !name.endsWith('~pid.lock')
/// ```
fn is_a_lock_name(name: &str) -> bool {
    return name.ends_with(LOCK_SUFFIX) && !name.ends_with(PID_LOCK_SUFFIX);
}

/// What:
///  Judge every lock of the Git directories `roots`,
///  remove the ones whose holder is proven gone,
///  and say what was removed.
/// Why:
///  Every command that may meet a lock calls this once,
///  so the proof,
/// the sequence and the words are one rule everywhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clearAbandonedLocks(roots: string[]): Outcome;
/// ```
pub fn clear_abandoned_locks(roots: &[PathBuf]) -> Outcome {
    let mut outcome: Outcome = Outcome::default();
    let candidates: Vec<PathBuf> = lock_candidates(roots);
    let Some(first) = roots.first() else {
        return outcome;
    };
    let common_dir: PathBuf = common_dir_for(first.as_path());
    for lock in &candidates {
        match proving::clear_one(lock.as_path(), common_dir.as_path()) {
            Some(clearance) => outcome.cleared.push(clearance),
            None => {
                // A lock whose holder looks gone and whose number no record explains is the one a person must hear about:
                //  it cannot release itself, and nothing here may remove it.
                if let Some(line) = proving::unexplained_dead_holder_line(lock.as_path(), common_dir.as_path()) {
                    outcome.unexplained.push(line);
                }
                outcome.standing.push(lock.clone());
            }
        }
    }
    // The records are swept against the locks that still stand, so a record that can still explain a lock stays and the
    //  litter of finished commands goes.
    sweep(common_dir.as_path(), standing_lock_pids(outcome.standing.as_slice()).as_slice());
    return outcome;
}

/// What:
///  Write the pass's lines to `out` and return how many locks were removed.
/// Why:
///   The commands that meet a lock write what they did to the person's standard error,
///  and a stream whose reader has gone is not a reason to stop the command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clearAbandonedLocksTelling(roots, stderr): number;
/// ```
pub fn clear_abandoned_locks_telling(roots: &[PathBuf], out: &mut dyn std::io::Write) -> usize {
    let outcome: Outcome = clear_abandoned_locks(roots);
    let notice: String = outcome.notice();
    if !notice.is_empty() {
        let written: std::io::Result<()> = out.write_all(notice.as_bytes());
        if written.is_ok() {
            let _flushed = out.flush();
        }
    }
    return outcome.cleared.len();
}

/// What:
///  The numbers that stand beside the locks of one repository now.
/// Why:
///  The sweep keeps every record that can still explain a lock,
///  and removes the records whose commands are over and whose numbers stand beside nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pids = standingLockPids(locks);
/// ```
fn standing_lock_pids(locks: &[PathBuf]) -> Vec<i64> {
    let mut pids: Vec<i64> = Vec::new();
    for lock in locks {
        if let Some(pid) = crate::stopped_git_locks::pid_named_by(proving_pid_file(lock).as_path()) {
            pids.push(i64::from(pid));
        }
    }
    return pids;
}

/// The PID file Git writes beside one lock.
fn proving_pid_file(lock: &Path) -> PathBuf {
    return crate::stale_ref_lock_walk::pid_file_beside(lock);
}

/// What:
///  The roots a lock pass needs for the worktree whose Git directory is `git_dir`.
/// Why:
///  A caller that knows only its own Git directory still searches the common one,
///  where a linked worktree's locks and the shared references live.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function rootsOfGitDirectory(gitDir: string): string[]; // [gitDir, commonDir] when they differ
/// ```
pub fn roots_of_git_directory(git_dir: &Path) -> Vec<PathBuf> {
    let common: PathBuf = common_dir_for(git_dir);
    let mut roots: Vec<PathBuf> = vec![git_dir.to_path_buf()];
    if common != git_dir {
        roots.push(common);
    }
    return roots;
}

/// What:
///  Meet the locks of the repository the working directory is in,
///  before this command reaches Git:
///  remember the repository,
///  note this process in its record,
///  and clear the locks whose holder is proven gone.
/// Why:
///  This is the one place a client command passes,
///  so every command of any kind meets a clear repository,
///  and a `git status` after a kill is enough to repair what the killed command left.
///  The Git directory is found by reading a few small files,
/// never by starting Git,
///  and a directory outside any repository costs one failed look.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// meetTheLocks(env) // GIT_DIR, else the working directory's repository
/// ```
pub fn meet_the_locks_of_the_working_directory(environment: &[(std::ffi::OsString, std::ffi::OsString)]) {
    let working: PathBuf = std::env::current_dir().unwrap_or_default();
    let mut named: Option<PathBuf> = None;
    for (name, value) in environment.iter().rev() {
        if name == std::ffi::OsStr::new("GIT_DIR") {
            named = Some(working.join(value));
            break;
        }
    }
    let found: Option<PathBuf> = match named {
        Some(directory) => Some(directory),
        None => git_directory_of(working.as_path()),
    };
    let Some(git_dir) = found else {
        return;
    };
    let common: PathBuf = common_dir_for(git_dir.as_path());
    remember_repository(common.as_path());
    // One record per process: a command that runs the pass twice (a client, then a forward that meets another repository)
    //  adds no second line and costs one atomic read.
    if !the_self_record_is_this_processs() {
        return;
    }
    record_this_process(common.as_path());
    clear_abandoned_locks_telling(roots_of_git_directory(git_dir.as_path()).as_slice(), &mut std::io::stderr());
}
