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
//!  in the repository the command works on rather than the one its working directory belongs to,
//!  and from the commit service before its landing waits for an index lock,
//!  so the next command of any kind meets a clear repository.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = clearAbandonedLocks(roots); stderr.write(outcome.notice());
//! ```

/// The proof and the removal of one lock.
use crate::abandoned_lock_proving as proving;
/// The words for what a clearance or a standing lock is.
use crate::abandoned_lock_words::{clearance_line, standing_lock_line};
/// The names a lock and its PID file have.
use crate::stopped_git_locks::{LOCK_SUFFIX, PID_LOCK_SUFFIX};
/// The common Git directory that holds the records of the processes started here.
use crate::git_directory_roots::common_dir_for;
/// The records themselves:
/// the repository this command works in,
/// which every Git it starts is noted in.
use crate::lock_holder_records::remember_repository;
/// The records themselves, read before the walk and removed only on what the walk found after that reading.
use crate::lock_holder_record_reads::{Record, every_record, remove_records_no_one_needs, remove_this_process_record_if_no_lock_needs_it};
/// The repository a command works on, resolved once by the one resolver every reader of the invocation uses.
use crate::git_directory_roots::CommandRepository;
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

/// The folder of the common Git directory that holds each linked worktree's own administrative directory.
const LINKED_WORKTREES_FOLDER: &str = "worktrees";

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
            lines.push_str(clearance_line(clearance).as_str());
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
    return walked(roots).locks;
}

/// What:
///  The lock files one walk found,
///  and whether the walk saw every file of the Git directories it searched.
/// Why:
///  A lock the walk did not visit can still need the record that explains it,
///  and the sweep removes records nothing needs.
///  A walk that stopped at its depth bound or at its entry bound has not seen every lock,
///  and no sweep may then decide what is litter:
///  a lock below the bound had its record taken away by exactly that mistake
///  (a review of `675771e759`,
/// finding 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Walked = { locks: string[]; complete: boolean };
/// ```
pub struct Walked {
    /// Every lock file the walk found,
    ///  in path order.
    pub locks: Vec<PathBuf>,
    /// Whether every directory the walk reached was searched to its end.
    pub complete: bool,
}

/// What:
///  Search the Git directories `roots` for lock files,
///  and say whether the search was whole.
/// Why:
///  A directory at the depth bound,
///  one whose entries ran past the entry bound,
///  and one that could not be read are three ways to miss a lock,
///  and each leaves `complete` false.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function walked(roots: string[]): Walked;
/// ```
pub fn walked(roots: &[PathBuf]) -> Walked {
    let mut found: Vec<PathBuf> = Vec::new();
    let mut complete: bool = true;
    let mut seen: usize = 0;
    for root in roots {
        let mut pending: Vec<(PathBuf, usize)> = vec![(root.clone(), 0)];
        while let Some((directory, depth)) = pending.pop() {
            let Ok(entries) = std::fs::read_dir(directory.as_path()) else {
                // A directory this account may not read may hold a lock.
                complete = false;
                continue;
            };
            for entry in entries.flatten() {
                seen += 1;
                if seen > MOST_ENTRIES {
                    complete = false;
                    break;
                }
                let Ok(kind) = entry.file_type() else {
                    complete = false;
                    continue;
                };
                if kind.is_dir() {
                    if SKIPPED_FOLDERS.contains(&entry.file_name().to_string_lossy().as_ref()) {
                        continue;
                    }
                    if depth < MOST_DEPTH {
                        pending.push((entry.path(), depth + 1));
                    } else {
                        // Deeper than this walk goes: what it holds is a lock no sweep may call litter.
                        complete = false;
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
    return Walked { locks: found, complete };
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
///  Whether a lock file stands where the command that made this pass will meet it.
/// Why:
///  A command meets the locks of the Git directory of the worktree it runs in and of the shared common directory;
///  a linked worktree's own administrative directory is that worktree's state,
///  which its own commands meet and no other worktree's command waits for.
///  So a lock in another worktree is kept or cleared by the same proof everywhere,
///  and is named only to the commands that run there:
///  an add printed a standing line about another add's `index.lock` in the gate on `0.6.1-alpha.34`,
///  where its own worktree's command had nothing to do with that file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// standsInTheWay('/r/.git/worktrees/held/index.lock', '/r/.git', '/r/.git'); // false
/// ```
fn stands_in_the_way(lock: &Path, git_dir: &Path, common_dir: &Path) -> bool {
    if !lock.starts_with(common_dir.join(LINKED_WORKTREES_FOLDER).as_path()) {
        return true;
    }
    // A linked worktree's own administrative directory is its own state: this command meets it only when it runs there.
    return git_dir.as_os_str() != common_dir.as_os_str() && lock.starts_with(git_dir);
}

/// What:
///  Judge every lock of the Git directories `roots`,
///  remove the ones whose holder is proven gone,
///  remove the records nothing can need any more,
///  and say what was removed.
/// Why:
///  Every command that may meet a lock calls this once,
///  so the proof,
/// the sequence and the words are one rule everywhere.
///  The records are read before the walk,
///  and removed on what that later walk found:
///  a lock another command's Git makes between the two is then seen,
///  and the record that explains it stays.
///  A record read after the walk would be removed on a sight of the locks older than its own reading
///  (a review of `2af2ecb879`, finding 2).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function clearAbandonedLocks(roots: string[]): Outcome; // read the records, walk, clear, remove litter
/// ```
pub fn clear_abandoned_locks(roots: &[PathBuf]) -> Outcome {
    let mut outcome: Outcome = Outcome::default();
    let Some(first) = roots.first() else {
        return outcome;
    };
    let common_dir: PathBuf = common_dir_for(first.as_path());
    // What: The records are read first, so the walk below is the sight of the locks a removal is decided on.
    // Why:  A record removed on a walk older than its reading loses the proof of a lock made in between.
    let records: Vec<Record> = every_record(common_dir.as_path());
    #[cfg(test)]
    crate::abandoned_lock_pass_fault_hooks_test_support::after_the_records_are_read();
    let searched: Walked = walked(roots);
    let candidates: Vec<PathBuf> = searched.locks;
    for lock in &candidates {
        match proving::clear_one(lock.as_path(), common_dir.as_path()) {
            Some(clearance) => outcome.cleared.push(clearance),
            None => {
                // A lock whose holder looks gone and whose number no record explains is the one a person must hear about:
                //  it cannot release itself, and nothing here may remove it.
                //  Words stay where the person is: a lock of another linked worktree's own state is that worktree's,
                //  and a command of this one neither waits for it nor reports it.
                if stands_in_the_way(lock.as_path(), first.as_path(), common_dir.as_path())
                    && let Some(line) = standing_lock_line(lock.as_path(), common_dir.as_path())
                {
                    outcome.unexplained.push(line);
                }
                outcome.standing.push(lock.clone());
            }
        }
    }
    // The records are removed against the locks that still stand, so a record that can still explain a lock stays and the
    //  litter of finished commands goes.
    //  A walk that stopped at its own bounds has not seen every lock, so a record that no standing lock of it names may
    //  still explain one below the bound: nothing is removed then.
    if searched.complete {
        remove_records_no_one_needs(common_dir.as_path(), records.as_slice(), pids_named_by_standing_locks(outcome.standing.as_slice()).as_slice());
    }
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
///  Remove the record this command wrote once the command that met its locks has ended,
///  when no lock of the repository names any process that record names,
///  and the folder with it once it holds no record.
/// Why:
///  A record written for a Git this command started explains a lock that Git may have left;
///  once the command returns and no lock of the repository names any process it recorded,
///  the record explains nothing and goes with the folder that held it,
///  so a command that changed nothing else leaves the Git directory as it found it.
///  This process is the record's only writer and it will append nothing more,
///  which is the one thing no other command can know,
///  so it may remove its own record while every other command must prove this one gone first.
///  A process whose only record is the one written just before it became Git never reaches this:
///  its replacement is the process, and the next command's pass removes the record.
///  The walk is made here, after the record was read,
///  and a walk that stopped at its own bounds removes nothing.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// removeMyRecordAfterTheCommand(roots) // nothing when this process wrote no record, or the walk was not whole
/// ```
pub fn sweep_the_records_the_command_left(roots: &[PathBuf]) {
    let Some(first) = roots.first() else {
        return;
    };
    let common: PathBuf = common_dir_for(first.as_path());
    // A command that started no Git wrote no record, and the walk below would have nothing to decide.
    if !crate::lock_holder_records::this_process_has_a_record(common.as_path()) {
        return;
    }
    let searched: Walked = walked(roots);
    if !searched.complete {
        return;
    }
    remove_this_process_record_if_no_lock_needs_it(common.as_path(), pids_named_by_standing_locks(searched.locks.as_slice()).as_slice());
}

/// What:
///  The numbers the locks of one repository name,
///  as their PID files record them,
///  read from a walk the caller made.
/// Why:
///  A record that names one of these numbers can still explain a lock,
///  so neither the writer nor any other command may remove it.
///  A caller that removes records takes this from the walk it made after reading them,
///  so a lock made in between is in the list.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pids = pidsNamedByStandingLocks(locks);
/// ```
pub fn pids_named_by_standing_locks(locks: &[PathBuf]) -> Vec<i64> {
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
///  Meet the locks of the repository the command works on,
///  before this command reaches Git:
///  remember the repository,
///  and clear the locks whose holder is proven gone.
/// Why:
///  This is the one place a client command passes,
///  so every command of any kind meets a clear repository,
///  and a `git status` after a kill is enough to repair what the killed command left.
///  The repository is the one the command works on,
///  resolved once before this call by the resolver every other reader uses
///  (`git_directory_roots::repository_a_command_works_on`),
///  never the one its working directory happens to belong to:
///  a command that names another repository leaves no record and clears no lock in this one.
///  The Git directory is found by reading a few small files,
/// never by starting Git,
///  and a directory outside any repository costs one failed look.
///  The roots it met come back so the command can remove the record it leaves behind when it ends
///  (`sweep_the_records_the_command_left`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// meetTheLocks(repository) // the -C chain, --git-dir or GIT_DIR, and GIT_COMMON_DIR, resolved once
/// ```
pub fn meet_the_locks_of_the_repository_the_command_works_on(repository: &CommandRepository) -> Vec<PathBuf> {
    let Some(common) = repository.common_dir.as_ref() else {
        return Vec::new();
    };
    remember_repository(common.as_path());
    let roots: Vec<PathBuf> = repository.roots();
    clear_abandoned_locks_telling(roots.as_slice(), &mut std::io::stderr());
    return roots;
}

/// Controls of the answer that decides which locks are named stay out of the release executable.
#[cfg(test)]
#[path = "abandoned_locks_tests.rs"]
mod tests;
