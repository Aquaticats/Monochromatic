//! What:
//!  The take of a worktree file a merge must replace or delete:
//! its private name,
//! the check that the file taken is the file that was judged,
//! putting back what a
//!  stopped command left,
//! and finishing the private names once the merge's content is in place.
//! Why:
//!  A judgement and a write were separate steps,
//! so a file saved between them was replaced with no notice;
//! one rename
//!  takes the file atomically,
//! so the bytes that are checked are the bytes that make way for the merge
//!  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
//! finding 4).
//!  A command that stops between the rename and the write leaves the file under a private name that names its
//!  path,
//! which the next command puts back,
//! and a file written after the take checked it is kept rather than removed.
//!  The steps live in a file of their own so the file that installs the merge stays readable.
//!
//!  Every step that reads Git is two faces of one body:
//! the awaited one a task of the runtime calls,
//! and the synchronous one a caller outside it keeps,
//!  which enters the runtime once.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const taken = await takeAside(location, judged); ... await finishAsides(taken, stderr);
//! ```

use crate::index_lock_holders::{HolderEvidence, scan_platform_holders_bounded};
/// The proof that no program can write into a file whose last name is about to go.
pub(crate) use crate::merge_isolation_take_asides_proof::ProofNothingCanWrite;
/// The proof,
///  asked the way a removal asks it.
use crate::merge_isolation_take_asides_proof::the_proof_that_nothing_can_write_into;
use crate::lazy_runtime::run_on_runtime;
use crate::merge_isolation_carry_parts::Capture;
use crate::merge_isolation_changes::{Change, FileIdentity, file_identity, the_objects_a_link_hashes_to_awaiting, worktree_path};
use crate::merge_isolation_recovery_steps::{Judged, RecoveryPlace};
/// The private name a take leaves.
use crate::merge_isolation_take_aside_names::aside_name;
/// The copies a merge kept,
/// and the one decision every command makes about each.
use crate::merge_isolation_kept_copies_decisions::settle_the_asides_of_a_merge_awaiting;
use crate::recovery_error::{RecoveryError, io_failure};
use crate::rename_that_never_replaces::rename_noreplace;
use crate::transaction_git::{GitContext, GitOutput, GitRequest};
use crate::transaction_git_awaiting::{run_git_awaiting, run_git_checked_awaiting};
use std::io::Write;
use std::path::{Path, PathBuf};

/// What:
///  Put back every file a killed or stopped take left under its private name,
///  remove the copies whose bytes are somewhere the person can reach them,
///  and say which ones stay beside a path that holds something else.
/// Why:
///   Nothing of the person's may be lost,
/// and a file a person cannot find is close to lost:
/// the path is free,
///  so the file goes back and the command says so;
/// the path holds something else,
/// so both stay,
/// the
///  private name is said,
/// and the copy is recorded so a later command ends it
///  (`merge_isolation_kept_copies.rs`).
/// This runs before the paths are judged,
/// because a path whose file waits beside it is empty
///  and would be read as someone else's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const putBack = await settleTheAsidesOfAMerge({ context, root, gitDir, changes, capture, stderr });
/// ```
///  The paths a copy went back to are answered,
/// because the file that arrives there is the one the merge judged:
///  its
///  identity is unchanged by the rename and its change time is not,
/// and a caller that judges those paths against a reading
///  the command that moved the checkout made has to read them again.
pub async fn put_back_stranded_takes_awaiting<W: Write + ?Sized>(
    context: &GitContext,
    root: &Path,
    git_dir: &Path,
    changes: &[Change],
    capture: Option<&Capture>,
    transaction_id: &str,
    out: &mut W,
) -> Vec<PathBuf> {
    return settle_the_asides_of_a_merge_awaiting(context, root, git_dir, changes, capture, transaction_id, out).await;
}

/// What:
///  Why a take stopped,
///  and the file it left under its private name when it could not put it back.
/// Why:
///   The rename happens before the bytes are read,
/// so a read that fails has already changed the worktree;
/// the
///  caller must add that file to its rollback and say the merge began when the file cannot go back
///  (a cross-vendor review of the fourth round,
/// the brief's item 3).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type TakeFailure = { error: RecoveryError; stranded?: Path };
/// ```
pub struct TakeFailure {
    /// What stopped the take.
    pub error: RecoveryError,
    /// The file that stayed at its private name,
    /// when it could not be put back.
    pub stranded: Option<PathBuf>,
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
pub(crate) async fn raw_bytes_of_awaiting(git: &GitContext, root: &Path, location: &Path) -> Option<String> {
    let kind: std::fs::FileType = std::fs::symlink_metadata(location).ok()?.file_type();
    if kind.is_symlink() {
        return the_objects_a_link_hashes_to_awaiting(git, root, location).await.ok();
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
pub(crate) async fn the_file_form_of_awaiting(git: &GitContext, root: &Path, path: &[u8], expected: &str) -> Option<String> {
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
pub(crate) async fn are_the_file_form_of_awaiting(git: &GitContext, root: &Path, path: &[u8], location: &Path, expected: &str) -> Option<bool> {
    let taken: String = raw_bytes_of_awaiting(git, root, location).await?;
    let held: String = the_file_form_of_awaiting(git, root, path, expected).await?;
    return Some(taken == held);
}

/// What a take of one worktree path found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Take = { kind: 'nothing' } | { kind: 'aside'; aside: string } | { kind: 'changed' };
/// ```
pub enum Take {
    /// Nothing stands there,
    /// as it was judged.
    Nothing,
    /// The file that was judged waits at this private name,
    /// with the identity the take read after the rename.
    Aside {
        /// Where the file that was judged waits.
        aside: PathBuf,
        /// What the file system says of it there.
        taken: FileIdentity,
        /// The object the file held when it was judged.
        expected: String,
    },
    /// Something else stands there now,
    /// or the judged file was written after it was judged;
    /// it is not the merge's to replace,
    /// and it is back where it was found.
    Changed,
}

/// What:
///  Take the file at `location` aside by one rename that never replaces,
/// and check that what was taken is the file that was judged.
/// Why:
///   A judgement and the write of a merged file were separate steps,
/// so a file saved between them was
///  overwritten with no notice
/// (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 4).
/// One `rename` takes
///  the file atomically,
/// so the bytes that are checked are exactly the bytes that make way for the merge;
/// a
///  file that differs from the judgement is put back and never replaced.
/// The check is the file's identity,
/// which is
///  what `stale_ref_lock_retire.rs` compares for a lock.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function takeAside(location: string, judged?: FileIdentity): Take; // renameNoReplace; identityOf(aside) === judged ? aside : putBack
/// ```
pub async fn take_aside_awaiting(
    place: &RecoveryPlace,
    root: &Path,
    path: &[u8],
    location: &Path,
    judged: &Judged,
    transaction_id: &str,
) -> Result<Take, TakeFailure> {
    let aside: PathBuf = aside_name(location, transaction_id);
    match rename_noreplace(location, aside.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // The path was empty when it was judged; a file that is there now came after the judgement and is not the
            //  merge's to replace.
            return Ok(if judged.content.is_some() { Take::Changed } else { Take::Nothing });
        }
        Err(error) => return Err(TakeFailure { error: io_failure("setting aside", location, &error), stranded: None }),
    }
    let taken: Option<FileIdentity> = file_identity(aside.as_path());
    let Some(expected) = judged.content.as_deref() else {
        // Nothing stood there when it was judged, so a file that is there now is someone else's.
        return put_back_and_report(location, aside);
    };
    // The identity is the cheap check and the bytes are the authority:
    // a program that keeps its descriptor across the rename can write the same
    //  length into the file and put the modification time back, which leaves every part of the identity as it was.
    if judged.identity.is_none() || taken != judged.identity {
        return put_back_and_report(location, aside);
    }
    match are_the_file_form_of_awaiting(&place.git, root, path, aside.as_path(), expected).await {
        Some(true) => {}
        Some(false) => return put_back_and_report(location, aside),
        // A read that fails cannot prove the bytes, so the file goes back rather than be replaced.
        None => {
            return Err(put_back_after_a_failed_read(
                location,
                aside,
                RecoveryError(String::from("the bytes of the file the merge replaces could not be read")),
            ));
        }
    }
    return match taken {
        Some(identity) => Ok(Take::Aside { aside, taken: identity, expected: String::from(expected) }),
        None => put_back_and_report(location, aside),
    };
}

/// What:
///  Put back a file that turned out not to be the one that was judged,
///  and say what the take made of it.
/// Why:
///  A file that changed after the judgement belongs to someone else,
/// so it goes back where it was found and the
///  merge stops for it rather than replacing it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// putBack(location, aside): Take; // renameNoReplace(aside, location) ? 'changed' : throw
/// ```
fn put_back_and_report(location: &Path, aside: PathBuf) -> Result<Take, TakeFailure> {
    return match rename_noreplace(aside.as_path(), location) {
        Ok(()) => Ok(Take::Changed),
        Err(error) => Err(TakeFailure {
            error: RecoveryError(format!(
                "{} changed after this command judged it and it could not be put back ({error}); it is at {} and nothing was \
                 replaced by the merge.",
                location.display(),
                aside.display()
            )),
            stranded: Some(aside),
        }),
    };
}

/// What:
///  Put back a file the take moved aside when the read of its bytes failed,
///  and say whether it went back.
/// Why:
///   A read that fails after the rename still changed the worktree,
/// so the caller's rollback must know about this
///  file,
/// and a file that cannot go back is named and counted as installed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// putBackAfterAFailedRead(location, aside, error): TakeFailure; // renameNoReplace(aside, location) ? { error } : { error, stranded: aside }
/// ```
fn put_back_after_a_failed_read(location: &Path, aside: PathBuf, error: RecoveryError) -> TakeFailure {
    return match rename_noreplace(aside.as_path(), location) {
        Ok(()) => TakeFailure { error, stranded: None },
        Err(back) => TakeFailure {
            error: RecoveryError(format!(
                "{} could not be read for the merge ({}) and could not be put back either ({back}); it is at {} and nothing was \
                 replaced by the merge.",
                location.display(),
                error.0,
                aside.display()
            )),
            stranded: Some(aside),
        },
    };
}

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
/// function putBack(taken: { aside: string; location: string }[]): string[]; // names of files left at their private name
/// ```
pub fn put_back(taken: &[TakenOnDisk]) -> Vec<String> {
    let mut left: Vec<String> = Vec::new();
    for entry in taken {
        match rename_noreplace(entry.aside.as_path(), entry.location.as_path()) {
            Ok(()) => {}
            Err(error) => left.push(format!("{} (its bytes are at {}, which was not removed: {error})", entry.location.display(), entry.aside.display())),
        }
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
pub fn finish_asides(place: &RecoveryPlace, root: &Path, taken: &[TakenOnDisk], out: &mut dyn Write) -> Result<(), RecoveryError> {
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
pub async fn finish_asides_awaiting<W: std::io::Write + ?Sized>(place: &RecoveryPlace, root: &Path, taken: &[TakenOnDisk], out: &mut W) -> Result<(), RecoveryError> {
    let _ = (place, root);
    for entry in taken {
        // `may_remove_the_aside_awaiting` proves that nothing can write into the file any more,
        // and the proof is held while its last name goes: a file it cannot prove that of is kept where it is,
        //  because its private name is then one of that write's only names.
        let Some(proof) = may_remove_the_aside_awaiting(&place.git, root, entry.path.as_slice(), entry.aside.as_path(), entry.taken, entry.expected.as_str()).await
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
        match proof.remove(entry.aside.as_path()) {
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
pub(crate) async fn may_remove_the_aside_awaiting(git: &GitContext, root: &Path, path: &[u8], aside: &Path, taken: FileIdentity, expected: &str) -> Option<ProofNothingCanWrite> {
    // The scan comes first and names a holder it can see, then a control may write as a program that kept its descriptor would,
    //  and then the proof is taken: a writer that still holds the file keeps the kernel from granting a lease to it, a writer
    //  that let go has already written so the bytes then show it, and the identity is read because a clean filter can hide
    //  changed bytes: an appended line that the filter drops leaves the object as it was and the size changed.
    let scanned: HolderEvidence = scan_platform_holders_bounded(aside, taken.device, taken.inode);
    if !scanned.holders.is_empty() {
        return None;
    }
    crate::service_fault::reach("aside-scanned");
    let proof: ProofNothingCanWrite = the_proof_that_nothing_can_write_into(aside, &scanned)?;
    if are_the_file_form_of_awaiting(git, root, path, aside, expected).await != Some(true) {
        return None;
    }
    return Some(proof);
}

/// What one last look at a copy and the file at its path came to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CopyRemoval = 'proven' | 'held-open' | 'changed';
/// ```
pub(crate) enum CopyRemoval {
    /// Nothing can write into the copy and the file at its path still holds the bytes that were compared,
    /// with the proof held until the copy's name goes.
    Proven(ProofNothingCanWrite),
    /// A program this command can read still holds the copy open,
    /// or nothing could prove that none does.
    HeldOpen,
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
pub(crate) async fn may_remove_the_copy_awaiting(git: &GitContext, root: &Path, aside: &Path, the_file: &Path, held_by_the_file: FileIdentity) -> CopyRemoval {
    let Some(taken): Option<FileIdentity> = file_identity(aside) else { return CopyRemoval::Changed };
    let Some(held): Option<String> = raw_bytes_of_awaiting(git, root, aside).await else { return CopyRemoval::Changed };
    let scanned: HolderEvidence = scan_platform_holders_bounded(aside, taken.device, taken.inode);
    if !scanned.holders.is_empty() {
        return CopyRemoval::HeldOpen;
    }
    // A control reaches here to write as a program that kept its descriptor would, after the scan and before the last look.
    crate::service_fault::reach("copy-scanned");
    // The proof is taken before the last looks, so what they read cannot change before the name goes.
    let Some(proof) = the_proof_that_nothing_can_write_into(aside, &scanned) else {
        return CopyRemoval::HeldOpen;
    };
    if file_identity(aside) != Some(taken) {
        return CopyRemoval::Changed;
    }
    if file_identity(the_file) != Some(held_by_the_file) {
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
pub async fn restore_what_git_did_not_write_awaiting<W: std::io::Write + ?Sized>(place: &RecoveryPlace, root: &Path, taken: &[TakenOnDisk], out: &mut W) -> Result<(), RecoveryError> {
    let mut written: Vec<TakenOnDisk> = Vec::new();
    for entry in taken {
        if std::fs::symlink_metadata(entry.location.as_path()).is_err() {
            match rename_noreplace(entry.aside.as_path(), entry.location.as_path()) {
                Ok(()) => {}
                Err(error) => {
                    return Err(RecoveryError(format!(
                        "{} could not be put back after the merge's files could not be written ({error}); it is at {} and nothing was \
                         removed.",
                        entry.location.display(),
                        entry.aside.display()
                    )));
                }
            }
            continue;
        }
        written.push(TakenOnDisk {
            location: entry.location.clone(),
            path: entry.path.clone(),
            aside: entry.aside.clone(),
            taken: entry.taken,
            expected: entry.expected.clone(),
        });
    }
    return finish_asides_awaiting(place, root, written.as_slice(), out).await;
}
