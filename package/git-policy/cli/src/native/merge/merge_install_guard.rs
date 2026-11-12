//! What:
//!  The one mark of an install running in a worktree:
//!  the file a merge holds from the instant it is about to change the checkout until its result is whole,
//!  the wait a second merge makes for it,
//!  and the clearing of one a killed merge left.
//! Why:
//!  Two merges in one worktree each clear the paths the merge changes and each carry the person's own work back,
//!  and their steps interleave:
//!  measured,
//! a second merge that starts while the first is part-way leaves an index that lacks two files the merge
//!  commit holds (`F12` of the alpha.12 audit),
//!  or wedges the worktree so every later command exits 2.
//!  A lock is the smallest answer for the one thing that cannot be shared,
//! so it covers one worktree's install and
//!  nothing else,
//!  and it is taken with `O_EXCL`,
//!  which is atomic on every filesystem this program runs on.
//!  Its owner is the same owner record a merge's intent record carries,
//!  so a mark whose merge died is told from one whose merge still runs by the one rule recovery already applies
//!  (`transaction_owner.rs`);
//!  a dead owner's mark is removed by the next command,
//!  and nothing is left for a person to remove.
//!  A command that arrives while the mark is held waits at most `MERGE_LOCK_CEILING`
//!  and says what it waits for and who holds it (`AGENTS.md`,
//! `SLD`:
//! nothing waits forever).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const guard = await takeInstallGuard({ gitDir, owner }); // O_EXCL of <gitDir>/<merge folder>/.installing
//! ```

/// Diagnostics for the log.
use crate::diagnostic_log::debug;
/// The intent record's directory,
/// which holds the mark beside the records it belongs to.
use crate::merge_isolation_journal::intent_directory;
/// Exclusive,
/// synced,
/// private files and directories.
use crate::private_storage::{create_private_directory, write_exclusive_synced};
/// The fail-closed recovery failure.
use crate::recovery_error::{RecoveryError, io_failure};
/// The owner of one merge,
/// its record text,
/// and the liveness rule that decides whether a later command may touch its work.
use crate::transaction_owner::{
    OwnerLiveness, OwnerLivenessError, TransactionOwner, classify_transaction_owner_record, encode_transaction_owner, parse_transaction_owner,
};
/// The inotify watch on the directory,
/// so a release ends the wait at once.
use crate::service_index_watch::{POLL_PAUSE, Watch};
/// The notice cadence of every wait.
use crate::wait_progress::{Cadence, Ticker};
/// The longest a wait lasts.
use crate::wait_ceilings::MERGE_LOCK_CEILING;
/// Writing the notices.
use std::io::Write;
/// `Path` and `PathBuf` are borrowed and owned filesystem paths.
use std::path::{Path, PathBuf};
/// Time spans and the monotonic clock.
use std::time::{Duration, Instant};

/// The name of the file inside the intent directory that says a merge is installing here.
/// It does not end in `.json`,
/// so no reader of records takes it for one.
pub const INSTALL_GUARD_FILE_NAME: &str = ".installing";

/// How many times a take tries after it found a mark whose owner is gone and removed it.
/// A live merge writes one mark and removes it once,
/// so the walk is only ever reached by several dead marks in a row.
const TAKE_ATTEMPTS: usize = 8;

/// What:
///  The path of a worktree's install mark.
/// Why:
///   Every reader and writer names it the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(intentDirectory(gitDir), '.installing')
/// ```
pub fn install_guard_path(git_dir: &Path) -> PathBuf {
    return intent_directory(git_dir).join(INSTALL_GUARD_FILE_NAME);
}

/// What:
///  The mark of the install this command holds,
/// which goes when the command's install ends,
/// however it ends.
/// Why:
///   A drop runs on every return and on an unwound panic,
/// so a failure of the install does not leave the worktree's mark behind;
///  a process killed outright is the one case drop cannot cover,
/// and its owner is provably gone then (`owner_liveness`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// class InstallGuard { dispose() { await rm(this.path, { force: true }) } }
/// ```
#[derive(Debug)]
pub struct InstallGuard {
    /// The file this command created.
    path: PathBuf,
}

impl InstallGuard {
    /// The file this command created.
    pub fn path(&self) -> &Path {
        return self.path.as_path();
    }
}

impl Drop for InstallGuard {
    fn drop(&mut self) {
        match std::fs::remove_file(self.path.as_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            // A mark that cannot be removed is left with its dead owner once this process ends, so the next command
            //  clears it; a person is told nothing of a file they never wrote.
            Err(error) => debug("mergeInstallGuard", format!("the install mark {} could not be removed: {error}", self.path.display()).as_str()),
        }
    }
}

/// What:
///  What a take came of.
/// Why:
///   A caller must tell "the install is mine" from "another merge holds it":
/// the first goes on,
/// and the second says who holds it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TakeOutcome = { took: InstallGuard } | { heldBy: number };
/// ```
pub enum TakeOutcome {
    /// This command holds the mark.
    Took(InstallGuard),
    /// Another merge holds it,
    /// and its owner is alive.
    HeldBy(i64),
}

/// What:
///  What a wait for the worktree's mark came of.
/// Why:
///   A wait that ended with the mark gone is not the same as one that ended at its ceiling:
/// the first means another merge installed,
/// so this command's round is computed again,
/// and the second means this merge saves itself for the next command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type WaitOutcome = 'free' | { releasedAfterMs } | { stillHeldAfterMs };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GuardWait {
    /// No merge was installing here.
    Free,
    /// Another merge installed and released the worktree.
    Released(
        /// How long the wait lasted.
        Duration,
    ),
    /// A merge still held the worktree at the ceiling.
    StillHeld(
        /// How long the wait lasted.
        Duration,
    ),
}

/// What:
///  The owner the worktree's mark names,
/// or nothing when no merge is installing here.
/// Why:
///   The reading is the take's and the wait's;
/// both tell a live merge from a dead one by it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function holder(gitDir): Promise<TransactionOwner | undefined>;
/// ```
pub fn install_guard_holder(git_dir: &Path) -> Result<Option<TransactionOwner>, RecoveryError> {
    let path: PathBuf = install_guard_path(git_dir);
    let bytes: Vec<u8> = match std::fs::read(path.as_path()) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io_failure("reading", path.as_path(), &error)),
    };
    return match parse_transaction_owner(bytes.as_slice()) {
        Some(owner) => Ok(Some(owner)),
        None => Err(RecoveryError(format!(
            "The mark of an installing merge {} is malformed. It was left in place; if no merge is running in this worktree, \
             remove it and run the command again.",
            path.display()
        ))),
    };
}

/// What:
///  Whether the owner of the worktree's mark is alive,
/// as recovery decides it for a record.
/// Why:
///   One rule for both:
/// a mark whose owner runs is never touched,
/// and one whose owner is gone is cleared by the next command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// classifyTransactionOwnerRecord(owner, guardPath)
/// ```
fn owner_liveness(owner: &TransactionOwner, git_dir: &Path) -> Result<OwnerLiveness, RecoveryError> {
    let path: PathBuf = install_guard_path(git_dir);
    return match classify_transaction_owner_record(owner, path.as_path()) {
        Ok(found) => Ok(found),
        // A probe this account may not make counts as alive, exactly as recovery reads it: nothing of a merge that
        //  may run is ever touched.
        Err(OwnerLivenessError::Probe(error)) if error.raw_os_error() == Some(libc::EPERM) => Ok(OwnerLiveness::Alive),
        Err(error) => Err(RecoveryError(format!("probing the owner {} of the installing merge {} failed: {error:?}", owner.owner_pid, path.display()))),
    };
}

/// What:
///  Take the worktree's mark for this command,
/// waiting for nothing.
/// Why:
///  The install's own entry point takes the mark where it decides to write,
/// so a command that arrives while another merge installs never begins a second install;
/// it reads what the mark says and answers from it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const taken = await takeInstallGuard({ gitDir, owner, wait: false });
/// ```
pub fn take_install_guard(git_dir: &Path, owner: &TransactionOwner) -> Result<TakeOutcome, RecoveryError> {
    let directory: PathBuf = intent_directory(git_dir);
    if let Err(error) = create_private_directory(directory.as_path())
        && error.kind() != std::io::ErrorKind::AlreadyExists
    {
        return Err(io_failure("creating", directory.as_path(), &error));
    }
    let path: PathBuf = install_guard_path(git_dir);
    for _attempt in 0..TAKE_ATTEMPTS {
        match write_exclusive_synced(path.as_path(), encode_transaction_owner(owner).as_bytes()) {
            Ok(()) => return Ok(TakeOutcome::Took(InstallGuard { path })),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(io_failure("writing", path.as_path(), &error)),
        }
        let Some(holder) = install_guard_holder(git_dir)? else {
            // The holder released it between the two looks, so the next attempt takes it.
            continue;
        };
        match owner_liveness(&holder, git_dir)? {
            OwnerLiveness::Alive => return Ok(TakeOutcome::HeldBy(holder.owner_pid)),
            // A mark a killed merge left holds nothing: the merge that wrote it is found by its own record, and
            //  recovery settles that; the mark is only the name of an install that nothing runs any more.
            OwnerLiveness::Dead => clear_install_guard(git_dir)?,
        }
    }
    return Err(RecoveryError(format!("the mark of an installing merge {} could not be taken: another merge's mark stayed there through {TAKE_ATTEMPTS} looks", path.display())));
}

/// What:
///  Whether a mark stands whose owner is gone.
/// Why:
///  A merge killed between taking the mark and writing its record leaves a mark no record names,
///  so the walk over records would never look at it and every command would refuse forever.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dead = await installGuardOwnerGone(gitDir);
/// ```
pub fn a_dead_install_guard(git_dir: &Path) -> Result<bool, RecoveryError> {
    let Some(holder) = install_guard_holder(git_dir)? else {
        return Ok(false);
    };
    return Ok(owner_liveness(&holder, git_dir)? == OwnerLiveness::Dead);
}

/// What:
///  Remove the worktree's mark.
/// Why:
///  A dead merge's mark is the one thing a later command may remove,
///  and it does so on the same proof recovery uses;
///  a successful removal is not synced,
/// because a mark that a power loss brings back is cleared again by the next command.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await rm(installGuardPath(gitDir), { force: true });
/// ```
pub fn clear_install_guard(git_dir: &Path) -> Result<(), RecoveryError> {
    let path: PathBuf = install_guard_path(git_dir);
    return match std::fs::remove_file(path.as_path()) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io_failure("removing", path.as_path(), &error)),
    };
}

/// What:
///  Wait,
/// bounded,
/// for the merge that is installing in this worktree,
/// saying what is waited for and who holds it.
/// Why:
///   A second merge cannot install beside the first without leaving a state neither can finish,
///   so it waits for the first,
///   and the wait is bounded because a merge can stop for good:
///   at the ceiling this command says so and goes on to save its own merge on a side ref,
///   which is the answer the package already gives a merge that cannot install now.
///  The wait is one look when no merge is installing,
/// which is every command that is not contending.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await waitForTheInstallingMerge({ gitDir, stderr }); // 'free' | 'released-after-ms' | 'still-held-after-ms'
/// ```
pub fn wait_for_the_installing_merge<W: Write + ?Sized>(git_dir: &Path, out: &mut W) -> Result<GuardWait, RecoveryError> {
    return wait_within(git_dir, MERGE_LOCK_CEILING, Cadence::from_environment(), out);
}

/// What:
///  The wait for `ceiling`,
/// over the mark's file,
/// with the notices of `cadence`.
/// Why:
///   A control reaches the ceiling in a moment with a short one,
/// and production names the shipping ceiling.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// waitWithin(gitDir, ceiling, cadence, stderr)
/// ```
pub fn wait_within<W: Write + ?Sized>(git_dir: &Path, ceiling: Duration, cadence: Cadence, out: &mut W) -> Result<GuardWait, RecoveryError> {
    let path: PathBuf = install_guard_path(git_dir);
    let Some(first) = install_guard_holder(git_dir)? else {
        return Ok(GuardWait::Free);
    };
    // The watch is set before the first look at the holder's state, so a release between them is an event and not a
    //  sleep to the next pause.
    let mut watch: Option<Watch> = path.parent().and_then(Watch::on);
    let file_name: std::ffi::OsString = path.file_name().map(std::ffi::OsStr::to_os_string).unwrap_or_default();
    let mut holder: TransactionOwner = first;
    let began: Instant = Instant::now();
    let mut ticker: Ticker = Ticker::announced_at_start(cadence);
    let mut announced: bool = false;
    loop {
        // The holder is read again only from the second look on: the first look is the one that found it above, and a
        //  read of a live merge every pause would cost a process-table probe for nothing.
        if announced {
            match install_guard_holder(git_dir)? {
                Some(found) => holder = found,
                None => return Ok(GuardWait::Released(began.elapsed())),
            }
        }
        let elapsed: Duration = began.elapsed();
        let due: bool = !announced || ticker.due(elapsed) || elapsed >= ceiling;
        if due {
            let line: String = match elapsed >= ceiling {
                true => ceiling_line(path.as_path(), holder.owner_pid, elapsed),
                false => notice_line(path.as_path(), holder.owner_pid, elapsed, ceiling, !announced),
            };
            announced = true;
            // A closed terminal is not a reason to stop the command the person asked for.
            let _noted = out.write_all(line.as_bytes());
            let _flushed = out.flush();
        }
        if elapsed >= ceiling {
            return Ok(GuardWait::StillHeld(elapsed));
        }
        let pause: Duration = ticker.next_in(elapsed).min(ceiling.saturating_sub(elapsed)).max(Duration::from_millis(1));
        match watch.as_ref() {
            Some(active) => {
                if !active.wait(pause, file_name.as_os_str()) {
                    drop(watch.take());
                    watch = path.parent().and_then(Watch::on);
                }
            }
            None => std::thread::sleep(pause.min(POLL_PAUSE)),
        }
    }
}

/// What:
///  The line that says what this command waits for,
/// who holds it,
/// and how the wait ends.
/// Why:
///   The owner's rule for a bounded wait is one line naming the blocker,
///  and this wait is short in every ordinary case,
///  so the line says the merge it waits for is another command's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: still waiting for the merge installing here (process ${pid}) after ${s} seconds ...`
/// ```
fn notice_line(path: &Path, holder_pid: i64, waited: Duration, ceiling: Duration, first: bool) -> String {
    let spelled: String = match first {
        true => format!("is installing its result in this worktree (process {holder_pid}, {})", path.display()),
        false => format!("is still installing its result in this worktree (process {holder_pid}, {})", path.display()),
    };
    return format!(
        "monochromatic-git-policy-cli: still waiting for the merge that {spelled} for {} seconds; one merge installs in a worktree at a time, so this merge has changed nothing yet. It waits at most {} seconds and then saves itself on a ref of its own for the next command in this worktree to finish.\n",
        waited.as_secs(),
        ceiling.as_secs()
    );
}

/// What:
///  The line a wait at its ceiling ends with.
/// Why:
///   It says what the command does next,
///  so a merge the person typed learns where its result went.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `monochromatic-git-policy-cli: the merge installing here (process ${pid}, ${path}) was still running after ${s} seconds, so this merge stops waiting ...`
/// ```
fn ceiling_line(path: &Path, holder_pid: i64, waited: Duration) -> String {
    return format!(
        "monochromatic-git-policy-cli: the merge installing here (process {holder_pid}, {}) was still running {} seconds after this merge began to wait for it, so this merge stops waiting and saves itself on a ref of its own for the next command in this worktree to finish; nothing was changed and nothing of yours is lost.\n",
        path.display(),
        waited.as_secs()
    );
}

/// The mark's controls stay out of the release executable.
#[cfg(test)]
#[path = "merge_install_guard_tests.rs"]
mod tests;
