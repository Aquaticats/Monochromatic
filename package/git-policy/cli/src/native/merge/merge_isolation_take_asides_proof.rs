//! What:
//!  The evidence a take or a removal rests on:
//!  the bytes a path holds as Git's object store reads them,
//!  the proof that no program can write into a file whose last name is about to go,
//!  and the last look a kept copy's removal is decided by.
//! Why:
//!  Removing a file's last name loses whatever a program still holding it writes afterwards,
//! and a scan of `/proc` cannot read the descriptors of a program of this account that is not dumpable,
//!  so the kernel's own answer is asked for where it offers leases
//!  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
//! its first finding).
//!  A platform without leases has only the scan,
//! whose answer is the whole of what a link's removal needs.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const proof = theProofThatNothingCanWriteInto(aside, scanned); // a read lease, or the scan's answer
//! ```

/// One file call of a removal handed to the blocking pool,
///  so a disk that makes it wait holds no worker of the runtime.
use crate::blocking_jobs_within_the_budget::run_blocking_job_within_the_budget;
use crate::index_lock_holders::scan_platform_holders_bounded;
/// Every process this platform can read,
///  as one scan of the file's holders found them.
use crate::index_lock_holders::{
    Holder,
    HolderEvidence,
};
/// The scan's reason for a process another account owns,
///  which a platform without leases acts through.
#[cfg(not(target_os = "linux"))]
use crate::index_lock_holders_accounts::OTHER_ACCOUNTS_PROCESS_REASON;
use crate::merge_isolation_changes::FileIdentity;
use crate::merge_isolation_changes::file_identity;
use crate::merge_isolation_changes::the_objects_a_link_hashes_to_awaiting;
use crate::merge_isolation_changes::worktree_path;
use crate::transaction_git_awaiting::run_git_awaiting;
use crate::transaction_git_awaiting::run_git_checked_awaiting;
use crate::transaction_git_request_and_reply_types::GitContext;
use crate::transaction_git_request_and_reply_types::GitOutput;
use crate::transaction_git_request_and_reply_types::GitRequest;
/// `Path` is a borrowed filesystem path of raw OS bytes,
/// and `PathBuf` is the owned one a job of the pool can carry.
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The proof that nothing but this command can write into one file,
///  held for as long as the name of that file is being removed.
/// Why:
///   Removing a file's last name loses whatever a program still holding it writes afterwards,
/// and a scan of `/proc` cannot read the descriptors of a program of this account that is not dumpable,
///  so such a program could hold the file and write into an inode no name reaches
///  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
/// its first finding).
/// A read lease is what the kernel refuses while any program has the file open for writing,
/// and one that is held keeps a later writer's open from reaching the file before the name goes,
///  so the kernel's answer is a proof the scan cannot give.
///  A file system that offers no leases answers nothing,
/// and the file is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type ProofNothingCanWrite = { lease?: Handle }; // dropped to let a later open through
/// ```
pub(crate) enum ProofNothingCanWrite {
    /// The open file whose read lease keeps a writer out until this is dropped.
    #[cfg(target_os = "linux")]
    KernelLease(std::fs::File),
    /// A scan that looked at every process this platform can read and found none holding the file,
    /// which is all a symbolic link's removal needs and all a platform without leases has.
    Scan,
}

impl ProofNothingCanWrite {
    /// What:
    ///  Remove the file this proof is about,
    ///  while the proof holds.
    /// Why:
    ///   The name goes while the lease keeps a writer out,
    /// and the lease is released only after the name is gone,
    ///  so no program can open the file between the proof and the removal.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// remove(aside) { const outcome = rm(aside); close(this.lease); return outcome; }
    /// ```
    pub(crate) fn remove(
        self,
        aside: &Path,
    ) -> std::io::Result<()> {
        let outcome: std::io::Result<()> = std::fs::remove_file(aside);
        #[cfg(target_os = "linux")]
        if let Self::KernelLease(lease) = self {
            // Closing the descriptor releases the lease; the name is already gone.
            drop(lease);
        }
        return outcome;
    }

    /// What:
    ///  Remove the file this proof is about,
    ///  as a job of the blocking pool,
    ///  with the proof held until the job's own end.
    /// Why:
    ///   A removal is a file call that can wait on a disk,
    /// so a step that runs on a task hands it to the pool,
    ///  and the lease travels into the job,
    ///  so it is released only once the name is gone.
    /// A refused thread answers an error,
    /// because the name may still be there.
    ///
    /// In TS you'd write (pseudocode):
    /// ```ts
    /// const outcome = await proof.removeOnTheBlockingPool(aside); // rm(aside) with this.lease held in the job
    /// ```
    pub(crate) async fn remove_on_the_blocking_pool(
        self,
        aside: &Path,
    ) -> std::io::Result<()> {
        let named: PathBuf = aside.to_path_buf();
        return match run_blocking_job_within_the_budget(move || return self.remove(named.as_path()))
            .await
        {
            Ok(outcome) => outcome,
            // The system gave the pool no thread: the name was not removed here, and the queued job may remove it later.
            Err(refused) => Err(std::io::Error::other(refused.to_string())),
        };
    }
}

/// What:
///  Stop the kernel's lease-break notice from ending this command.
/// Why:
///   A program that opens a file this command holds a lease on makes the kernel send `SIGIO`,
/// whose default action is to terminate,
///  and this command asks for no asynchronous notification of its own:
/// the lease is taken to learn whether a writer holds the file and is released a step later.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// process.once('SIGIO', () => {}); // the kernel's break notice says nothing this command waits for
/// ```
#[cfg(target_os = "linux")]
fn ignore_the_lease_break_notice() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(ignore_the_lease_break_notice_once);
}

/// What:
///  Ignore the kernel's lease-break notice,
///  which is the one step of `ignore_the_lease_break_notice`.
/// Why:
///  A once-guard takes a function rather than a closure that captures nothing;
/// the signal's disposition is the process's,
///  so one call is all it needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const ignoreOnce = () => { const previous = ignoreSignal('SIGIO'); };
/// ```
#[cfg(target_os = "linux")]
fn ignore_the_lease_break_notice_once() {
    // SAFETY: `signal` with `SIG_IGN` for a signal this process never asks to receive, and the call is made once.
    let _previous = unsafe { libc::signal(libc::SIGIO, libc::SIG_IGN) };
}

/// What:
///  The proof that nothing can write into one aside,
///  from the kernel where it offers leases and from the scan where it does not.
/// Why:
///   The kernel's answer covers a writer this command cannot inspect,
/// and a platform without leases has only the scan:
/// a file whose scan found a holder,
///  or was cut at its bound,
/// or could not list the processes,
/// has no proof and is kept.
///  A symbolic link is not opened for writing by anyone,
/// so the scan's answer is the whole of what a link's removal needs.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const theProofThatNothingCanWriteInto = (aside, scanned) => lstat(aside).isSymbolicLink() ? (scanProved(scanned) ? { kind: 'scan' } : undefined) : leaseOrUndefined(aside);
/// ```
pub(crate) fn the_proof_that_nothing_can_write_into(
    aside: &Path,
    scanned: &HolderEvidence,
) -> Option<ProofNothingCanWrite> {
    #[cfg(target_os = "linux")]
    {
        let _ = scanned;
        if matches!(std::fs::symlink_metadata(aside), Ok(found) if !found.file_type().is_file()) {
            return Some(ProofNothingCanWrite::Scan);
        }
        // A read lease, because the person may have made the file unwritable: it is refused while any program has the file open
        //  for writing, which is the only open that can change it, and it keeps a later writer out while the name goes.
        ignore_the_lease_break_notice();
        use std::os::fd::AsRawFd;
        let lease: std::fs::File = std::fs::File::open(aside).ok()?;
        // SAFETY: the descriptor is open for the call, and this command holds the only claim on it.
        let refused: libc::c_int =
            unsafe { libc::fcntl(lease.as_raw_fd(), libc::F_SETLEASE, libc::F_RDLCK) };
        if refused != 0 {
            return None;
        }
        return Some(ProofNothingCanWrite::KernelLease(lease));
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(target_os = "linux"))]
    {
        let _ = aside;
        // A process another account owns cannot write into a file of this account's, and the scan of a platform without leases
        //  cannot see its descriptors: those reasons are the limit a removal acts through.
        let unruled_out: bool = scanned
            .partial
            .iter()
            .any(|reason| !reason.starts_with(OTHER_ACCOUNTS_PROCESS_REASON));
        if scanned.holders.is_empty() && !unruled_out && !scanned.unresolved() {
            return Some(ProofNothingCanWrite::Scan);
        }
        return None;
    }
}

/// What:
///  The object the raw bytes of a file hash to,
///  with no filter applied.
/// Why:
///   An aside exists to keep a person's bytes,
/// and a clean filter can hide bytes from the object Git records for the
///  path,
/// so the bytes themselves are what is compared.
/// A symbolic link's object is the text of its target,
/// and
///  `git hash-object <path>` opens the file the link points at instead,
/// so a link is hashed from its target text and
///  never followed.
///   The link was hashed by following it,
/// so a saved merge that changes one compared its target's bytes with the
///  link's own object,
/// put the link back and refused,
/// and every retry met the same mismatch
///  (a cross-vendor review of the ninth round,
/// its sixth finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// rawBytesOf(place, location): string | undefined; // link ? hash-object --stdin < readlink : regular file ? hash-object --no-filters -- <location> : undefined
/// ```
pub(crate) async fn raw_bytes_of_awaiting(
    git: &GitContext,
    root: &Path,
    location: &Path,
) -> Option<String> {
    // The look at the entry is a file call that can wait on a disk, so a job of the blocking pool makes it.
    let looked_at: PathBuf = location.to_path_buf();
    let kind: std::fs::FileType = match run_blocking_job_within_the_budget(move || {
        return std::fs::symlink_metadata(looked_at.as_path());
    })
    .await
    {
        Ok(Ok(found)) => found.file_type(),
        // An entry that is not there, and a look the pool never made, both leave the bytes unknown.
        Ok(Err(_)) | Err(_) => return None,
    };
    if kind.is_symlink() {
        return the_objects_a_link_hashes_to_awaiting(git, root, location)
            .await
            .ok();
    }
    // A pipe, a socket, a device or a directory has no bytes a comparison can read, and `hash-object` on a pipe waits for a writer
    //  that may never come: a copy beside a path whose entry is one of those is a copy the reading cannot prove, so the answer is
    //  "the bytes are not known" rather than a wait with no end (`SPEC.md`, "nothing should ever wait forever or indefinitely").
    if !kind.is_file() {
        return None;
    }
    let mut request: GitRequest = GitRequest::new(root, &["hash-object", "--no-filters", "--"]);
    request.arguments.push(location.as_os_str().to_os_string());
    let output: GitOutput = run_git_awaiting(git, &request).await.ok()?;
    if !output.succeeded() {
        return None;
    }
    return Some(String::from_utf8_lossy(&output.stdout).trim().to_owned());
}

/// What:
///  The object one stored object's file form hashes to:
///  the path's filters applied to the object,
///  then the raw bytes of that
///  hashed with no filter.
/// Why:
///   That is what the worktree would hold had nobody touched the file,
/// so a file whose raw bytes hash to something else carries bytes of the
///  person's that the filter hides.
///  The second pass compares one of the branch's objects this way,
///  and the carry compares a captured or merged object the same way.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// theFileFormOf(place, root, path, expected): string | undefined; // cat-file --filters --path=<path> <expected> | hash-object --no-filters --stdin
/// ```
pub(crate) async fn the_file_form_of_awaiting(
    git: &GitContext,
    root: &Path,
    path: &[u8],
    expected: &str,
) -> Option<String> {
    let mut named: std::ffi::OsString = std::ffi::OsString::from("--path=");
    named.push(worktree_path(Path::new(""), path).into_os_string());
    let mut cat: GitRequest = GitRequest::new(root, &["cat-file", "--filters"]);
    cat.arguments.push(named);
    cat.arguments.push(std::ffi::OsString::from(expected));
    let filtered: GitOutput = run_git_awaiting(git, &cat).await.ok()?;
    if !filtered.succeeded() {
        return None;
    }
    let mut hash: GitRequest = GitRequest::new(root, &["hash-object", "--no-filters", "--stdin"]);
    hash.input = Some(filtered.stdout);
    let output: GitOutput = run_git_checked_awaiting(git, &hash).await.ok()?;
    return Some(String::from_utf8_lossy(&output.stdout).trim().to_owned());
}

/// What:
///  Whether the bytes at `location` are the file form of `expected`,
///  or nothing when either read fails.
/// Why:
///   One comparison serves the take and the removal,
/// so a file either side accepts is the same file,
///  and the carry
///  compares the file it takes with the object its own judgement recorded.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// areTheFileFormOf(git, root, path, location, expected): boolean | undefined;
/// ```
pub(crate) async fn are_the_file_form_of_awaiting(
    git: &GitContext,
    root: &Path,
    path: &[u8],
    location: &Path,
    expected: &str,
) -> Option<bool> {
    let taken: String = raw_bytes_of_awaiting(git, root, location).await?;
    let held: String = the_file_form_of_awaiting(git, root, path, expected).await?;
    return Some(taken == held);
}

/// What one last look at a copy and the file at its path came to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopyRemoval = 'proven' | 'held-by' | 'unproven' | 'changed';
/// ```
pub(crate) enum CopyRemoval {
    /// Nothing can write into the copy and the file at its path still holds the bytes that were compared,
    /// with the proof held until the copy's name goes.
    Proven(ProofNothingCanWrite),
    /// The programs a scan of the process table found holding the copy open,
    /// each with the process number and command it was read by.
    HeldBy(Vec<Holder>),
    /// No program the scan could read holds the copy open,
    /// and no lease on it could be taken,
    /// so nothing proves that none can write into it.
    Unproven,
    /// The copy or the file at its path is no longer the entry the comparison read.
    Changed,
}

/// What:
///  Whether removing a copy the file at its path already holds loses nothing,
/// because nothing can write into the copy any more,
///  and the file at the path still holds what was compared with it.
/// Why:
///   A copy whose bytes the person's file holds again is a second file of the same bytes,
/// and the copy beside the path is the one to go;
///  but the copy may be a file a program still has open,
/// because a take moved it and a descriptor outlives the rename,
///  and a write through that descriptor lands in the file that would be unlinked and then has no name at all
/// (`SPEC.md`,
/// the removal the aside's own end acts on).
///  So the same proof is asked before any copy is unlinked:
/// the identity the copy had when the bytes were compared is still its identity,
///  that identity
/// is the one the scan looked for,
/// no process this command can read holds it open,
/// and its bytes still hash to the
///  object the comparison read.
///  The file at the path is asked again beside all that,
/// because it is the only other name those bytes have:
///  a save or a chmod that arrived after the comparison would leave the copy the last name of the older bytes or the older
///  permissions,
/// so the identity the comparison read must still be the file's,
///  and its bytes must still be the copy's own
/// (a Kimi K3.1 Pro review of `1a98764d1f`,
/// its second finding).
/// An entry that is not a regular file or a symbolic link is never removed here:
/// a pipe has no
///  bytes a reading can compare.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function mayRemoveTheCopy(git, root, aside, theFile, heldByTheFile): Promise<CopyRemoval>; // regular entry && no readable process holds it && the aside's identity and bytes are unchanged && the file at the path is still the entry the comparison read
/// ```
pub(crate) async fn may_remove_the_copy_awaiting(
    git: &GitContext,
    root: &Path,
    aside: &Path,
    the_file: &Path,
    held_by_the_file: FileIdentity,
) -> CopyRemoval {
    // The identity of the copy is a look at a user's file, which can wait on a disk, so a job of the blocking pool makes it.
    let looked_at: PathBuf = aside.to_path_buf();
    let taken_before: Option<FileIdentity> =
        match run_blocking_job_within_the_budget(move || return file_identity(looked_at.as_path()))
            .await
        {
            Ok(found) => found,
            // A look the pool never made proves nothing, so the copy stays.
            Err(_refused) => return CopyRemoval::Changed,
        };
    let Some(taken) = taken_before else {
        return CopyRemoval::Changed;
    };
    let Some(held): Option<String> = raw_bytes_of_awaiting(git, root, aside).await else {
        return CopyRemoval::Changed;
    };
    let scanned: HolderEvidence = scan_platform_holders_bounded(aside, taken.device, taken.inode);
    if !scanned.holders.is_empty() {
        return CopyRemoval::HeldBy(scanned.holders);
    }
    // A control reaches here to write as a program that kept its descriptor would, after the scan and before the last look.
    crate::service_fault::reach("copy-scanned");
    // The proof is taken before the last looks, so what they read cannot change before the name goes. It is a read lease the
    //  kernel grants on an opened file, so a job of the blocking pool asks for it.
    let named: PathBuf = aside.to_path_buf();
    let asked = run_blocking_job_within_the_budget(move || {
        return the_proof_that_nothing_can_write_into(named.as_path(), &scanned);
    })
    .await;
    let proof = match asked {
        Ok(Some(proof)) => proof,
        // A proof the pool never took, and a file system that offers no lease, both keep the copy where it is.
        Ok(None) | Err(_) => return CopyRemoval::Unproven,
    };
    // The two identities are looks at user's files, so one job of the blocking pool makes both.
    let (copy_at, path_at): (PathBuf, PathBuf) = (aside.to_path_buf(), the_file.to_path_buf());
    let compared = run_blocking_job_within_the_budget(move || {
        return (
            file_identity(copy_at.as_path()),
            file_identity(path_at.as_path()),
        );
    })
    .await;
    let (the_copy, the_path): (Option<FileIdentity>, Option<FileIdentity>) = match compared {
        Ok(answers) => answers,
        // A look the pool never made proves nothing about either entry, so the copy stays.
        Err(_refused) => return CopyRemoval::Changed,
    };
    if the_copy != Some(taken) {
        return CopyRemoval::Changed;
    }
    if the_path != Some(held_by_the_file) {
        return CopyRemoval::Changed;
    }
    if raw_bytes_of_awaiting(git, root, aside).await.as_deref() != Some(held.as_str()) {
        return CopyRemoval::Changed;
    }
    if raw_bytes_of_awaiting(git, root, the_file).await.as_deref() != Some(held.as_str()) {
        return CopyRemoval::Changed;
    }
    return CopyRemoval::Proven(proof);
}
