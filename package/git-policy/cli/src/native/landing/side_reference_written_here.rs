//! What:
//!  The reference that keeps a batch's commits reachable until its branch moves,
//!  `refs/worktree/cli-git-service/<batch>`,
//!  written by the landing itself
//!  when its write is provably the write Git would make,
//!  and left to Git for every other repository.
//! Why:
//!   The name is the wrapper's own:
//!  a full name of digits below a directory the wrapper makes,
//!  never packed,
//!  never symbolic,
//!  and never given a reflog,
//!  because the write this replaces passes `core.logAllRefUpdates=false`.
//!  So the write is a directory,
//! a lock file,
//! forty hexadecimal digits and a rename,
//!  and asking Git for it costs a process start the person waits on
//!  for a step whose whole work is those four calls
//!  (measured:
//!  the service's `answered` line names `objects_ms` 12,831 ms with the write of that reference
//!  as `git update-ref` 12,659 ms of it,
//!  `doc/planning/cli-git-commit-latency-under-many-agents.md`).
//!  Nothing a Git start adds here is a guarantee:
//!  the object name is one the landing has just proved present,
//!  and the reference is what the branch write removes in its own transaction.
//!  Every setting whose write Git would make differently answers `LeftToGit`,
//!  and Git then writes the reference exactly as it did before,
//!  with its own words for a refusal.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const plan = thePlanOf(snapshot); // 'left-to-git' | { here: 'none' | 'pid' }
//! if (plan !== 'left-to-git' && writeSideReference(gitDir, name, tip, plan.here) === 'left-to-git') await keepOnTheScratchReference(git, repo, name, tip);
//! ```

use crate::config_snapshot::{ConfigSnapshot, Lookup};
use std::io::Write;
use std::path::{Path, PathBuf};

/// What the landing decided about a batch's side reference before it writes anything.
/// Why:
///  A caller tells its own write from a decline,
///  and a decline is not a failure:
///  Git writes the reference and answers for it as it always has.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SideReferenceWritePlan = 'left-to-git' | { here: LockMark };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SideReferenceWritePlan {
    /// Git writes the reference.
    LeftToGit,
    /// This landing writes it,
    /// with this mark beside its lock.
    Here(LockMark),
}

/// What:
///  How the repository wants a reference lock marked while it stands.
/// Why:
///  Git writes a second file beside a lock it takes when the repository sets `core.lockfilePid`,
///  naming the process that holds it,
///  and the wrapper's own judgement of a standing lock reads that file
///  (`stale_ref_locks.rs`).
///  A reference left locked by a landing that died must look exactly like one Git left,
///  so the mark is written as Git writes it,
/// or the write is left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type LockMark = 'none' | 'pid';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LockMark {
    /// No mark beside the lock,
    /// which is Git's default.
    None,
    /// `<batch>~pid.lock`,
    /// holding `pid <this process>`.
    Pid,
}

/// What the landing answered when it was asked to write a batch's side reference.
/// Why:
///  A write left to Git is not an error,
///  so the caller tells the two answers apart and otherwise keeps its own path unchanged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SideReferenceWrite = 'written' | 'left-to-git';
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SideReferenceWrite {
    /// The reference names the commit,
    /// written here.
    Written,
    /// This landing cannot prove its write is Git's,
    /// so Git must write it.
    LeftToGit,
}

/// The end of the name of the file that marks a standing reference lock with its holder.
const PID_MARK_END: &str = "~pid.lock";

/// The directory every batch's reference is made in,
/// below one worktree's own Git directory.
const DIRECTORY: &str = "cli-git-service";

/// The prefix of a batch's side reference.
const PREFIX: &str = "refs/worktree/cli-git-service/";

/// What:
///  Whether this landing may write the batch's side reference itself,
///  and how it must mark the lock while it stands.
/// Why:
///   Two kinds of setting make Git's write one this writer cannot reproduce:
///  a reference store that is flushed (`core.fsync` naming the reference,
/// which flushes the file and its
///  directory),
///  and a repository whose files are shared with a group or everyone,
///  whose reference files Git widens to the shared mode.
///  A value this module cannot read is a decline too,
///  and so is any storage that is not plain files,
///  which is the answer for a reftable repository and for a configuration read with the caller's own global options.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const plan = (snapshot) => !snapshot?.provesFilesRefStorage() || snapshot.fsyncNamesReference() || snapshot.hasSharedRepository() ? 'left-to-git' : { here: lockMarkOf(snapshot) };
/// ```
pub(crate) fn the_plan_of(snapshot: Option<&ConfigSnapshot>) -> SideReferenceWritePlan {
    let Some(found) = snapshot else { return SideReferenceWritePlan::LeftToGit };
    if !found.proves_files_ref_storage() {
        return SideReferenceWritePlan::LeftToGit;
    }
    let Some(fsync) = found.fsync_components() else { return SideReferenceWritePlan::LeftToGit };
    if fsync.names_reference() {
        return SideReferenceWritePlan::LeftToGit;
    }
    // Any value at all is a repository whose files are shared: the modes it names are ones this writer does not set.
    if !matches!(found.bool_value("core.sharedrepository"), Lookup::Unset) {
        return SideReferenceWritePlan::LeftToGit;
    }
    return match found.bool_value("core.lockfilepid") {
        Lookup::Unset | Lookup::Found(false) => SideReferenceWritePlan::Here(LockMark::None),
        Lookup::Found(true) => SideReferenceWritePlan::Here(LockMark::Pid),
        Lookup::Unhandled => SideReferenceWritePlan::LeftToGit,
    };
}

/// What:
///  The batch number of a batch's side reference,
///  which is the one shape this writer takes.
/// Why:
///   A name that is anything else belongs to a person or to another part of the wrapper,
///  and writing it here would be a write Git never made:
///  the wrapper's own refusals (a branch,
/// a tag,
/// a name with a component Git will not take) are Git's to word.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const batch = name.startsWith('refs/worktree/cli-git-service/') && /^[0-9]+$/.test(name.slice(32)) ? name.slice(32) : undefined;
/// ```
fn the_batch_of(name: &str) -> Option<&str> {
    let batch: &str = name.strip_prefix(PREFIX)?;
    if batch.is_empty() || !batch.bytes().all(is_a_digit) {
        return None;
    }
    return Some(batch);
}

/// Whether `byte` is the ASCII digit of a batch number.
fn is_a_digit(byte: u8) -> bool {
    return byte.is_ascii_digit();
}

/// Whether `byte` is a lower-case hexadecimal digit,
/// which is how Git writes an object name.
fn is_lower_hex_digit(byte: u8) -> bool {
    return byte.is_ascii_digit() || matches!(byte, b'a'..=b'f');
}

/// Whether `byte` is anything but the digit zero,
/// so a name of zeros reads as the null name.
fn is_not_the_zero_digit(byte: u8) -> bool {
    return byte != b'0';
}

/// What:
///  Whether `tip` is a whole object name:
///  forty or sixty-four lower-case hexadecimal digits,
///  and not the null name.
/// Why:
///   Git refuses to write a reference at a name that is not an object it can parse,
///  and the length is the repository's object format,
///  so a name of neither length is one this writer cannot vouch for.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const isTip = (text) => (text.length === 40 || text.length === 64) && /^[0-9a-f]+$/.test(text) && !/^0+$/.test(text);
/// ```
fn is_a_whole_object_name(tip: &str) -> bool {
    let whole: bool = matches!(tip.len(), 40 | 64) && tip.bytes().all(is_lower_hex_digit);
    return whole && tip.bytes().any(is_not_the_zero_digit);
}

/// What:
///  A file made fresh at `path`,
///  or nothing when something is already there or the file system refuses.
/// Why:
///   Git takes a reference lock with an exclusive create,
///  so a lock that exists is another writer's,
///  and this landing must not take it,
///  word a refusal of its own,
///  or leave half a file behind.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const exclusive = (path) => { try { return openSync(path, 'wx', 0o666); } catch { return undefined; } };
/// ```
fn made_fresh(path: &Path) -> Option<std::fs::File> {
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o666);
    }
    return options.open(path).ok();
}

/// What:
///  Take the lock of one reference and write the mark the repository asks for beside it.
/// Why:
///   The lock is what proves no other process is writing the same reference,
///  and the mark has to exist before the value is written,
///  as Git writes both,
///  so a landing that dies between them leaves the state Git leaves.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const held = exclusive(`${dir}/${batch}.lock`); if (!held) return undefined; if (mark === 'pid' && !writeMark(`${dir}/${batch}~pid.lock`)) { unlink(lock); return undefined; } return held;
/// ```
fn lock_beside_the_reference(directory: &Path, batch: &str, mark: LockMark) -> Option<std::fs::File> {
    let lock: PathBuf = directory.join(format!("{batch}.lock"));
    let held: std::fs::File = made_fresh(lock.as_path())?;
    if matches!(mark, LockMark::None) {
        return Some(held);
    }
    let marked: PathBuf = directory.join(format!("{batch}{PID_MARK_END}"));
    let written: bool = made_fresh(marked.as_path()).is_some_and(write_the_mark);
    if written {
        return Some(held);
    }
    // The lock is given back the way Git gives back one it cannot keep: nothing of this landing stands when the write is
    //  left to Git.
    drop(held);
    let _given_back = std::fs::remove_file(lock.as_path());
    let _removed = std::fs::remove_file(marked.as_path());
    return None;
}

/// The word and the newline Git puts in the mark beside a lock it holds.
fn pid_mark_text() -> String {
    return format!("pid {}\n", std::process::id());
}

/// What:
///  Write the mark of a standing lock into the file just made for it.
/// Why:
///   Git's mark names the process that holds the lock,
///  and a mark this landing leaves has to name this one,
///  so a lock it died on is read as a lock Git died on.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const writeMark = (file) => file.writeAllSync(`pid ${process.pid}\n`) && true;
/// ```
fn write_the_mark(mut mark: std::fs::File) -> bool {
    return mark.write_all(pid_mark_text().as_bytes()).is_ok();
}

/// What:
///  Give the lock back,
///  with the mark beside it when one was made.
/// Why:
///   A write that is left to Git must leave no lock of its own,
///  or Git's wait for the same reference would end at this landing's file,
///  and a mark with no lock beside it is a holder that is not there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const giveBack = (dir, batch, mark, held) => { held.close(); if (mark === 'pid') unlink(`${batch}~pid.lock`); unlink(`${batch}.lock`); };
/// ```
fn give_the_lock_back(directory: &Path, batch: &str, mark: LockMark, held: std::fs::File) {
    drop(held);
    if matches!(mark, LockMark::Pid) {
        let _removed = std::fs::remove_file(directory.join(format!("{batch}{PID_MARK_END}")).as_path());
    }
    let _given_back = std::fs::remove_file(directory.join(format!("{batch}.lock")).as_path());
}

/// What:
///  Write the batch's side reference at `tip` in `git_dir`,
///  or answer that Git has to.
/// Why:
///   The landing has just proved every object of the commit present,
///  so the last thing between the person's commit and its branch is a reference
///  the wrapper names and owns;
///  writing it here saves the process start the person would wait on,
///  and every shape whose write Git would make differently answers `LeftToGit` instead of writing.
///  The reference is written where a per-worktree reference lives:
///  in the worktree's own Git directory,
///  which is `.git` for the main worktree and `.git/worktrees/<id>` for a linked one
///  (measured on Git 2.56.0:
///  `git update-ref refs/worktree/cli-git-service/<n>` in a linked worktree writes
///  `.git/worktrees/<id>/refs/worktree/cli-git-service/<n>`,
///  and `git rev-parse --git-path` names the same file).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const answer = writeSideReference(gitDir, name, tip, mark); // 'written' | 'left-to-git'
/// ```
pub(crate) fn write_side_reference(git_dir: &Path, name: &str, tip: &str, mark: LockMark) -> SideReferenceWrite {
    let Some(batch) = the_batch_of(name) else { return SideReferenceWrite::LeftToGit };
    if !is_a_whole_object_name(tip) {
        return SideReferenceWrite::LeftToGit;
    }
    let directory: PathBuf = git_dir.join("refs").join("worktree").join(DIRECTORY);
    if std::fs::create_dir_all(directory.as_path()).is_err() {
        return SideReferenceWrite::LeftToGit;
    }
    let reference: PathBuf = directory.join(batch);
    // A name that is not a plain file is a shape this writer cannot replace: a symbolic reference, or a directory Git
    //  would refuse. Git answers both.
    match std::fs::symlink_metadata(reference.as_path()) {
        Ok(found) if !found.is_file() => return SideReferenceWrite::LeftToGit,
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(_) => return SideReferenceWrite::LeftToGit,
    }
    let Some(mut held) = lock_beside_the_reference(directory.as_path(), batch, mark) else { return SideReferenceWrite::LeftToGit };
    let value: String = format!("{tip}\n");
    if held.write_all(value.as_bytes()).is_err() {
        give_the_lock_back(directory.as_path(), batch, mark, held);
        return SideReferenceWrite::LeftToGit;
    }
    drop(held);
    if std::fs::rename(directory.join(format!("{batch}.lock")).as_path(), reference.as_path()).is_err() {
        let _given_back = std::fs::remove_file(directory.join(format!("{batch}.lock")).as_path());
        if matches!(mark, LockMark::Pid) {
            let _removed = std::fs::remove_file(directory.join(format!("{batch}{PID_MARK_END}")).as_path());
        }
        return SideReferenceWrite::LeftToGit;
    }
    // The mark goes the way Git's goes, once the rename is what names the commit.
    if matches!(mark, LockMark::Pid) {
        let _removed = std::fs::remove_file(directory.join(format!("{batch}{PID_MARK_END}")).as_path());
    }
    return SideReferenceWrite::Written;
}

/// Controls for this writer stay out of the release executable.
#[cfg(test)]
#[path = "side_reference_written_here_tests.rs"]
mod tests;
