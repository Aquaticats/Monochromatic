//! What:
//!  The right of one running command to work on one item (a worktree being converted,
//! a checkout being migrated),
//! held
//!  as a numbered claim file that exactly one command creates and that names its owner by process birth.
//! Why:
//!  Two commands that both pass the preflight would both write the journal,
//! share a token on a resume and exchange the
//!  directories back,
//! so one command must own the item for as long as it works.
//! The claim is lock-free:
//! a claim file is
//!  made whole in a private temporary file and published with one exclusive `link` under the next generation number,
//! so
//!  the kernel lets exactly one command create generation `n + 1`;
//! no lock is held,
//! no command ever waits,
//! and a command
//!  that finds a live owner stops with the owner's process id.
//! The newest generation is the claim.
//! A newer generation
//!  replaces an older one only when the older one records a released claim or an owner whose process no longer exists
//!  with that birth (the PID alone is not trusted,
//! because it is reused),
//! so a killed command leaves nothing to clean up
//!  by hand:
//! the next command takes the next generation.
//! The newest generation file is never removed,
//! because removing
//!  it would let a command that read an older generation create the same number again;
//! a released claim is rewritten in
//!  place by renaming a one-line record over it.
//! Older generations are removed by the command that took the newer one.
//!  An owner in another PID namespace cannot be probed;
//! it counts as alive until its claim file is older than
//!  `FOREIGN_OWNER_PATIENCE`,
//! as every owner record of the wrapper does.
//!
//! Files beside this one,
//!  and why the claim is not one file:
//!  `single_owner_claim_record.rs` holds the text of one claim file,
//!  `single_owner_claim_files.rs` holds the numbered files a claim is kept in,
//!  and `single_owner_claim_liveness.rs` decides whether the owner a record names still runs.
//!  `rust/max-lines` splits the claim by role,
//!  and taking the claim,
//!  giving it back,
//!  and answering who owns an item are the role this file keeps.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const outcome = await takeClaim({ directory, prefix }); // { held: claim } or { busy: { pid, path } }
//! // ... work on the item ...
//! await claim.release();
//! ```

/// The PID namespace a planted claim names,
///  which is this process's own.
#[cfg(test)]
use crate::pid_namespace::current_pid_namespace;
/// Every generation of an item's claims,
///  the path of one,
///  and the three steps that publish,
///  read and release one.
use crate::single_owner_claim_files::{
    generations,
    path_of,
    publish_new,
    read_record,
    rewrite_as_released,
};
/// Whether the owner a claim names still runs,
///  and this process's own record.
use crate::single_owner_claim_liveness::{
    live_owner,
    own_record,
};
/// The text a claim this command plants is written as,
///  which only a control plants.
#[cfg(test)]
use crate::single_owner_claim_record::encode;
/// The record a claim file holds,
///  and the decoder the words of one claim file give it back through.
pub use crate::single_owner_claim_record::{
    ClaimRecord,
    decode,
};
/// The refusal a claim fails with,
///  which names the item and what was being done to it.
use crate::subvolume_error::MigrateError;
/// Borrowed file names,
///  which the prefix of a claim series is.
use std::ffi::OsStr;
/// The paths a claim is laid out as.
use std::path::{
    Path,
    PathBuf,
};

/// The most rounds one `take_claim` call makes.
/// Each round after the first follows the progress of another command (it
///  created the generation this one wanted,
/// or a newer one),
/// so more rounds than this mean claimants keep arriving faster
///  than this command can look;
/// it then reports that instead of looping.
const MOST_ROUNDS: u32 = 64;

/// What:
///  A claim this process holds:
/// the right to work on one item.
/// Why:
///   Dropping it releases the claim,
/// so every way out of the command,
/// an early error included,
/// gives the item back.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type OwnerClaim = { path: string; generation: number; release(): Promise<void> };
/// ```
#[derive(Debug)]
pub struct OwnerClaim {
    /// The claim file this process created.
    path: PathBuf,
    /// The directory that holds the claim files of the item.
    directory: PathBuf,
    /// Whether the claim was released already.
    released: bool,
}

/// What:
///  The live owner that stopped a claim.
/// Why:
///   The refusal names the process and the file,
/// so the person can see which command runs and where its claim is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type BusyOwner = { pid: number; path: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BusyOwner {
    /// The owner's process id.
    pub pid: i64,
    /// The claim file that records the owner.
    pub path: PathBuf,
}

/// What:
///  How a claim attempt ended.
/// Why:
///   A busy item is a normal answer (the other command is working),
/// not a failure.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ClaimOutcome = { kind: 'held'; claim: OwnerClaim } | { kind: 'busy'; owner: BusyOwner };
/// ```
#[derive(Debug)]
pub enum ClaimOutcome {
    /// This process owns the item now.
    Held(OwnerClaim),
    /// A live process owns the item.
    Busy(BusyOwner),
}

/// What:
///  Take the claim on the item whose claim files are `<prefix>.<generation>` in `directory`,
/// or report the live owner.
/// Why:
///   See the module description.
/// The call never waits:
/// it reads the newest generation,
/// creates the next one exclusively,
///  confirms that no newer one appeared,
/// and removes the older ones.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const outcome = await takeClaim({ directory, prefix });
/// ```
pub fn take_claim(
    directory: &Path,
    prefix: &OsStr,
) -> Result<ClaimOutcome, MigrateError> {
    return take_claim_seamed(directory, prefix, &mut do_nothing);
}

/// The call `take_claim` makes between the listing and the creation:
/// none.
fn do_nothing() {}

/// `take_claim` with a call between the listing of the generations and the creation of the next one,
/// where a control may let
///  another command act.
pub fn take_claim_seamed(
    directory: &Path,
    prefix: &OsStr,
    after_listing: &mut dyn FnMut(),
) -> Result<ClaimOutcome, MigrateError> {
    let record: ClaimRecord = own_record()?;
    for _round in 0..MOST_ROUNDS {
        let existing: Vec<u64> = generations(directory, prefix)?;
        after_listing();
        let target: u64 = match existing.last() {
            None => 1,
            Some(newest) => {
                let newest_path: PathBuf = path_of(directory, prefix, *newest);
                let Some(found) = read_record(newest_path.as_path())? else {
                    continue;
                };
                if let Some(pid) = live_owner(&found, newest_path.as_path())? {
                    return Ok(ClaimOutcome::Busy(BusyOwner {
                        pid,
                        path: newest_path,
                    }));
                }
                newest + 1
            }
        };
        let path: PathBuf = path_of(directory, prefix, target);
        match publish_new(directory, path.as_path(), &record) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(MigrateError::io("create the claim", path.as_path(), &error)),
        }
        // A command that read an older generation may have published one below the newest after the others were removed;
        // a newer generation than this one means this claim lost.
        if let Some(newest) = generations(directory, prefix)?.last()
            && *newest > target
        {
            let _withdrawn: std::io::Result<()> = std::fs::remove_file(&path);
            continue;
        }
        for older in &existing {
            if *older < target {
                let _removed: std::io::Result<()> =
                    std::fs::remove_file(path_of(directory, prefix, *older));
            }
        }
        return Ok(ClaimOutcome::Held(OwnerClaim {
            path,
            directory: directory.to_path_buf(),
            released: false,
        }));
    }
    return Err(MigrateError::Journal(format!(
        "other commands kept taking the claim in {} while this one looked; nothing was changed, run the command again",
        directory.display()
    )));
}

/// What:
///  The live owner of the item whose claim files are `<prefix>.<generation>` in `directory`,
/// if any;
/// nothing is created.
/// Why:
///   A command that works on a different item inside the same checkout (a migration of the checkout,
/// a conversion of a worktree
///  in it) must not start while the other runs,
/// and reading the claim is enough to know.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const owner = await liveOwnerOf({ directory, prefix }); // undefined when nobody owns the item
/// ```
pub fn live_owner_of(
    directory: &Path,
    prefix: &OsStr,
) -> Result<Option<BusyOwner>, MigrateError> {
    if !directory.is_dir() {
        return Ok(None);
    }
    let existing: Vec<u64> = generations(directory, prefix)?;
    let Some(newest) = existing.last() else {
        return Ok(None);
    };
    let path: PathBuf = path_of(directory, prefix, *newest);
    let Some(found) = read_record(path.as_path())? else {
        return Ok(None);
    };
    return match live_owner(&found, path.as_path())? {
        Some(pid) => Ok(Some(BusyOwner {
            pid,
            path,
        })),
        None => Ok(None),
    };
}

impl OwnerClaim {
    /// Give the item back:
    /// the claim file is rewritten,
    /// atomically,
    /// as a released claim,
    /// so the next command takes the next
    ///  generation without judging a process.
    pub fn release(&mut self) -> Result<(), MigrateError> {
        if self.released {
            return Ok(());
        }
        let written: std::io::Result<()> =
            rewrite_as_released(self.directory.as_path(), self.path.as_path());
        self.released = true;
        return match written {
            Ok(()) => Ok(()),
            Err(error) => Err(MigrateError::io(
                "release the claim",
                self.path.as_path(),
                &error,
            )),
        };
    }
}

impl Drop for OwnerClaim {
    fn drop(&mut self) {
        // A release that failed leaves a claim that names this process; it stops counting as alive when the process ends.
        let _released: Result<(), MigrateError> = self.release();
    }
}

/// Plant a claim file as another process would have left it.
#[cfg(test)]
pub(crate) fn plant_claim_for_tests(
    directory: &Path,
    prefix: &OsStr,
    generation: u64,
    pid: i64,
    birth: &str,
) {
    let record: ClaimRecord = ClaimRecord::Owned {
        pid,
        birth: birth.to_string(),
        namespace: current_pid_namespace(),
    };
    std::fs::write(path_of(directory, prefix, generation), encode(&record)).expect("planted claim");
}

/// A process id that names no process:
/// the first number the kernel never hands out.
#[cfg(test)]
pub(crate) fn unused_pid_for_tests() -> i64 {
    let text: String = std::fs::read_to_string("/proc/sys/kernel/pid_max").expect("pid_max");
    return text.trim().parse().expect("a number");
}

/// The claim controls stay out of the release executable.
#[cfg(test)]
#[path = "single_owner_claim_tests.rs"]
mod tests;
