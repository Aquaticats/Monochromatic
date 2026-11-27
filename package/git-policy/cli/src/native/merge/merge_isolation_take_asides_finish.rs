//! What:
//!  What comes of the files one take moved aside:
//!  the putting back of the ones no write reached,
//!  the proof that nothing can write into one any more,
//!  the removal of the private names that hold the merge's old content,
//!  and the restore of what Git did not write after a checkout failed.
//! Why:
//!   A command that stops between the take and the write leaves the person's file
//! under a private name beside its path,
//! and a file written after the take was checked is kept rather than removed,
//!  so nothing of the person's is lost either way.
//!   A removal waits for a proof that no program can still write into the file:
//! a program that kept its descriptor across the rename writes into the file with
//!  no name at all otherwise.
//!  These are a file of their own so the take itself stays inside the code-line limit.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! for (const entry of taken) await rm(entry.aside); const left = await putBackAwaiting(taken);
//! ```

/// The one job of the blocking pool that makes the step's file calls,
/// so a disk that makes them wait holds no worker of the runtime.
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
/// Scanning every readable process for a descriptor on the file about to lose its name.
use crate::index_lock_holders::{
    HolderEvidence,
    scan_platform_holders_bounded,
};
use crate::lazy_runtime::run_on_runtime;
/// The identity of a file the take read after the rename.
use crate::merge_isolation_changes::FileIdentity;
/// Where a command's recovery runs Git,
/// and the directory every command starts in.
use crate::merge_isolation_recovery_place::RecoveryPlace;
/// The bytes a path holds as Git reads them,
/// the proof that no program can write into a file whose last name is about to go,
/// and the lease that proof rests on.
use crate::merge_isolation_take_asides_proof::{
    ProofNothingCanWrite,
    are_the_file_form_of_awaiting,
    the_proof_that_nothing_can_write_into,
};
/// The rename that never replaces.
use crate::private_storage::rename_noreplace;
use crate::recovery_error::{
    RecoveryError,
    io_failure,
};
/// The same rename as a job of the blocking pool,
/// and what it came to.
use crate::rename_that_never_replaces::{
    TheAnswerOfARename,
    rename_noreplace_on_the_blocking_pool,
};
use crate::transaction_git::GitContext;
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Put back every file that was taken aside and not yet written,
/// and name the ones that cannot go back.
/// Why:
///   A merge that finds one path changed after it judged it writes none of them,
/// so the ones already taken must
///  return before the command ends.
/// A file that cannot go back is never deleted:
/// its name is told to the person
///  instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const putBack = async (taken) => { const left = []; for (const entry of taken) { const moved = await renameNoReplaceOnThePool(entry.aside, entry.location); if (moved.kind !== 'renamed') left.push(...); } return left; };
/// ```
pub async fn put_back_awaiting(taken: &[TakenOnDisk]) -> Vec<String> {
    let mut left: Vec<String> = Vec::new();
    // The renames are file calls that can wait on a disk, so one job of the blocking pool makes them all and the names that
    //  could not go back are the job's answer.
    let names: Vec<(PathBuf, PathBuf)> = taken
        .iter()
        .map(|entry| return (entry.aside.clone(), entry.location.clone()))
        .collect();
    let moved: Result<Vec<(usize, std::io::Error)>, crate::blocking_job::BlockingJobError> =
        run_blocking_job_within_the_budget(move || {
            let mut failed: Vec<(usize, std::io::Error)> = Vec::new();
            for (index, (aside, location)) in names.iter().enumerate() {
                if let Err(error) = rename_noreplace(aside.as_path(), location.as_path()) {
                    failed.push((index, error));
                }
            }
            return failed;
        })
        .await;
    let failed: Vec<(usize, std::io::Error)> = match moved {
        Ok(found) => found,
        // No rename ran, or they may still run: every file is named rather than promised back.
        Err(refused) => {
            for entry in taken {
                left.push(format!(
                    "{} (its bytes are at {}, which was not removed: {refused})",
                    entry.location.display(),
                    entry.aside.display()
                ));
            }
            return left;
        }
    };
    for (index, error) in failed {
        let entry: &TakenOnDisk = &taken[index];
        left.push(format!(
            "{} (its bytes are at {}, which was not removed: {error})",
            entry.location.display(),
            entry.aside.display()
        ));
    }
    return left;
}

/// One file that was taken aside,
/// and where the merge's content goes.
pub struct TakenOnDisk {
    /// The path the merge writes or deletes.
    pub location: PathBuf,
    /// The repository path of the file,
    /// which names the attributes its bytes are read with.
    pub path: Vec<u8>,
    /// Where the file that was judged waits.
    pub aside: PathBuf,
    /// The identity the take checked,
    /// so a file replaced before the take is seen.
    pub taken: FileIdentity,
    /// The object the file held when it was judged,
    /// which its bytes must still hash to before its private name goes.
    pub expected: String,
}

/// What:
///  Remove the private names of files that were taken aside and are now replaced or deleted.
/// Why:
///   What they hold is the merge's old content,
/// which the object database holds too;
/// a file that could not be
///  removed is named rather than lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of taken) await rm(entry.aside, { force: true });
/// ```
pub fn finish_asides(
    place: &RecoveryPlace,
    root: &Path,
    taken: &[TakenOnDisk],
    out: &mut dyn Write,
) -> Result<(), RecoveryError> {
    return match run_on_runtime(finish_asides_awaiting(place, root, taken, out)) {
        Ok(found) => found,
        Err(refusal) => Err(RecoveryError(format!("{refusal}"))),
    };
}

/// What:
///  The same removal of the private names of taken files,
/// awaited by a caller that is itself a task of the runtime.
/// Why:
///   The synchronous face enters the runtime around the bytes it reads before each removal,
/// which a task cannot do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function finishAsides({ place, root, taken, stderr }): Promise<void>;
/// ```
pub async fn finish_asides_awaiting<W: std::io::Write + ?Sized>(
    place: &RecoveryPlace,
    root: &Path,
    taken: &[TakenOnDisk],
    out: &mut W,
) -> Result<(), RecoveryError> {
    let _ = (place, root);
    for entry in taken {
        // `may_remove_the_aside_awaiting` proves that nothing can write into the file any more,
        // and the proof is held while its last name goes: a file it cannot prove that of is kept where it is,
        //  because its private name is then one of that write's only names.
        let Some(proof) = may_remove_the_aside_awaiting(
            &place.git,
            root,
            entry.path.as_slice(),
            entry.aside.as_path(),
            entry.taken,
            entry.expected.as_str(),
        )
        .await
        else {
            let _noted = out.write_all(
                format!(
                    "monochromatic-git-policy-cli: {} was written or is still held open after this command checked it, so it is kept \
                     and {} holds the merge's content; nothing of the person's was removed.\n",
                    entry.aside.display(),
                    entry.location.display()
                )
                .as_bytes(),
            );
            continue;
        };
        match proof
            .remove_on_the_blocking_pool(entry.aside.as_path())
            .await
        {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io_failure("removing", entry.aside.as_path(), &error)),
        }
    }
    return Ok(());
}

/// What:
///  The proof that nothing can write into a file that was taken aside any more,
///  so removing its private name loses nothing,
///  held while that name goes,
///  and whether its bytes are
///  the branch's own form rather than bytes a clean filter hides.
/// Why:
///   Every step that removes a private name asks this same question:
///  the install that places the merge's own entries
///  (`merge_isolation_recovery_install.rs`)
///  and the carry that replaces or deletes a path
///  (`merge_isolation_carry_place.rs`),
///  so a write that arrives through an old descriptor is kept by both.
///   A program that already has the file open keeps that descriptor across the rename,
/// and a write through it lands
///  in the file that waits aside;
/// comparing the identity a moment before the unlink cannot close that,
/// because the
///  write can arrive in the instant after the comparison
/// (a cross-vendor review of the second round).
/// Three things are
///  asked instead,
/// and all must hold:
/// the file's identity is the one the take checked,
/// the kernel grants a read lease on the file,
///  which it refuses while any program has it open for writing,
/// and its bytes are the judged ones.
///  A file system without leases,
/// a lease the kernel refuses,
/// and a scan that saw a holder or proved nothing
/// all keep the file and its name
///  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
/// its first finding,
/// where a non-dumpable program of this account wrote and fsynced into an unlinked inode).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mayRemoveTheAside(git, root, path, aside, taken, expected): ProofNothingCanWrite | undefined; // identity unchanged && the kernel's lease granted && the bytes are the judged ones
/// ```
pub(crate) async fn may_remove_the_aside_awaiting(
    git: &GitContext,
    root: &Path,
    path: &[u8],
    aside: &Path,
    taken: FileIdentity,
    expected: &str,
) -> Option<ProofNothingCanWrite> {
    // The scan comes first and names a holder it can see, then a control may write as a program that kept its descriptor would,
    //  and then the proof is taken: a writer that still holds the file keeps the kernel from granting a lease to it, a writer
    //  that let go has already written so the bytes then show it, and the identity is read because a clean filter can hide
    //  changed bytes: an appended line that the filter drops leaves the object as it was and the size changed.
    let scanned: HolderEvidence = scan_platform_holders_bounded(aside, taken.device, taken.inode);
    if !scanned.holders.is_empty() {
        return None;
    }
    crate::service_fault::reach("aside-scanned");
    // The proof is a read lease the kernel grants on an opened file, and opening a user's file can wait on a disk, so a job of the
    //  blocking pool asks for it.
    let named: PathBuf = aside.to_path_buf();
    let asked = run_blocking_job_within_the_budget(move || {
        return the_proof_that_nothing_can_write_into(named.as_path(), &scanned);
    })
    .await;
    let proof: ProofNothingCanWrite = match asked {
        Ok(Some(proof)) => proof,
        // A proof the pool never took, and a file system that offers no lease, both keep the name where it is.
        Ok(None) | Err(_) => return None,
    };
    if are_the_file_form_of_awaiting(git, root, path, aside, expected).await != Some(true) {
        return None;
    }
    return Some(proof);
}

/// What:
///  Put back the files Git did not write,
///  and finish the ones it did,
///  after the checkout failed.
/// Why:
///  A checkout that stops partway leaves some paths written and others untouched,
/// and the untouched ones must have
///  their old content back:
/// deleting the taken files would leave the person's file missing and the next command
///  refusing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const entry of taken) if (!exists(entry.location)) renameNoReplace(entry.aside, entry.location); else keepOrDrop(entry);
/// ```
pub async fn restore_what_git_did_not_write_awaiting<W: std::io::Write + ?Sized>(
    place: &RecoveryPlace,
    root: &Path,
    taken: &[TakenOnDisk],
    out: &mut W,
) -> Result<(), RecoveryError> {
    let mut written: Vec<TakenOnDisk> = Vec::new();
    for entry in taken {
        // The look at the path is a file call that can wait on a disk, so a job of the blocking pool makes it.
        let goes_to: PathBuf = entry.location.clone();
        let was_free: Result<bool, crate::blocking_job::BlockingJobError> =
            run_blocking_job_within_the_budget(move || {
                return std::fs::symlink_metadata(goes_to.as_path()).is_err();
            })
            .await;
        let free: bool = match was_free {
            Ok(found) => found,
            // A look the pool never made proves nothing about the path, so the file is not moved onto one that may be taken.
            Err(refused) => {
                return Err(RecoveryError(format!(
                    "{} could not be read after the merge's files could not be written: {refused}",
                    entry.location.display()
                )));
            }
        };
        if !free {
            written.push(TakenOnDisk {
                location: entry.location.clone(),
                path: entry.path.clone(),
                aside: entry.aside.clone(),
                taken: entry.taken,
                expected: entry.expected.clone(),
            });
            continue;
        }
        // Putting the file back is one rename, so a job of the blocking pool makes it.
        match rename_noreplace_on_the_blocking_pool(entry.aside.as_path(), entry.location.as_path())
            .await
        {
            TheAnswerOfARename::Renamed => {}
            TheAnswerOfARename::NameTaken(error) | TheAnswerOfARename::Failed(error) => {
                return Err(RecoveryError(format!(
                    "{} could not be put back after the merge's files could not be written ({error}); it is at {} and nothing was \
                     removed.",
                    entry.location.display(),
                    entry.aside.display()
                )));
            }
            TheAnswerOfARename::NoThreadToRunIt(refused) => {
                return Err(RecoveryError(format!(
                    "{} could not be put back after the merge's files could not be written ({refused}); it is at {} and nothing \
                     was removed.",
                    entry.location.display(),
                    entry.aside.display()
                )));
            }
        }
    }
    return finish_asides_awaiting(place, root, written.as_slice(), out).await;
}
