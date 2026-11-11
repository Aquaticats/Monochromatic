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
use crate::lazy_runtime::run_on_runtime;
use crate::merge_isolation_changes::{Change, FileIdentity, file_identity, the_objects_a_link_hashes_to_awaiting, worktree_path};
use crate::merge_isolation_recovery_steps::{Judged, RecoveryPlace};
use crate::recovery_error::{RecoveryError, io_failure};
use crate::rename_that_never_replaces::rename_noreplace;
use crate::transaction_git::{GitContext, GitOutput, GitRequest};
use crate::transaction_git_awaiting::{run_git_awaiting, run_git_checked_awaiting};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// The start of the private name a file is set aside under while the merge's content takes its place.
///  It sits in the file's own directory,
/// so the take is one rename on whatever file system the worktree is on,
/// and it
///  ends in no name Git reads.
pub(crate) const ASIDE_PREFIX: &str = ".cli-git-recovery";

/// Makes two names set aside by one process differ.
static ASIDE_COUNT: AtomicU64 = AtomicU64::new(0);

/// The text of one file name,
/// for the private name of a take.
fn name_of(found: &std::ffi::OsStr) -> String {
    return found.to_string_lossy().into_owned();
}

/// The text of a file's name,
/// which the private name of a take names by a short key.
fn name_text(location: &Path) -> String {
    return location.file_name().map(name_of).unwrap_or_default();
}

/// The base of the FNV-1a hash,
/// in 64 bits.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;

/// The prime of the FNV-1a hash,
/// in 64 bits.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// What:
///  A short,
///  stable key for a file's name,
///  which the private name of a take carries in place of the name itself.
/// Why:
///   A file name may fill the file system's 255-byte component limit,
/// and a private name that embedded it could not exist,
///  so a valid tracked file made a saved merge unfinishable and its record then stopped every command
///  (a cross-vendor review of the second round).
/// The key is the FNV-1a hash of the name's bytes in 64 bits,
/// which every release
///  of this program computes the same way,
/// so a private name one release left behind is found by the next.
/// An upper-case
///  spelling of the same name is a different key,
/// as it is a different file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function nameKey(text: string): string; // FNV-1a 64 of the UTF-8 bytes, 16 hex digits
/// ```
fn name_key(text: &str) -> String {
    let mut hash: u64 = FNV_OFFSET_BASIS;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    return format!("{hash:016x}");
}

/// What:
///  A private name beside `location`,
///  naming the file it holds,
///  that no other process uses.
/// Why:
///   A kill between the rename of a take and the write that follows leaves the file under this name,
/// so the name
///  must say which path it belongs to:
/// the next recovery finds it there and puts it back
///  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
/// finding 4's take,
/// round two).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${dirname}/.cli-git-recovery-${basename}-${pid}-${count}-${nanos}`;
/// ```
pub fn aside_name(location: &Path) -> PathBuf {
    let count: u64 = ASIDE_COUNT.fetch_add(1, Ordering::Relaxed);
    let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    return location.with_file_name(format!("{ASIDE_PREFIX}-{}-{}-{count}-{nanos:x}", name_key(name_text(location).as_str()), std::process::id()));
}

/// Whether one byte is one of the hex digits a ran number is written in.
fn is_hex_digit(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// Whether one name is a private name a take left for `location`:
/// the prefix of that path's file name,
/// then `<pid>-<count>-<nanos>`.
fn is_aside_of(name: &str, location: &Path) -> bool {
    let Some(rest) = name.strip_prefix(format!("{ASIDE_PREFIX}-{}-", name_key(name_text(location).as_str())).as_str()) else {
        return false;
    };
    let mut parts = rest.split('-');
    let (Some(_pid), Some(_count), Some(nanos), None) = (parts.next(), parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    return !nanos.is_empty() && nanos.bytes().all(is_hex_digit);
}

/// What:
///  The private names in `location`'s directory that a take of that path left.
/// Why:
///   A recovery that stopped for any reason can leave one,
/// and the next recovery must find it before it judges the path,
///  because the path is empty while the file waits beside it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// (await readdir(dirname(location))).filter((name) => isAsideOf(name, location));
/// ```
fn asides_of(location: &Path) -> Vec<PathBuf> {
    let Some(directory) = location.parent() else {
        return Vec::new();
    };
    let Ok(listing) = std::fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in listing.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if is_aside_of(name.as_str(), location) {
            found.push(entry.path());
        }
    }
    found.sort();
    return found;
}

/// What:
///  Put back every file a killed or stopped take left under its private name,
///  and say which ones stayed beside a path that holds something else.
/// Why:
///   Nothing of the person's may be lost,
/// and a file a person cannot find is close to lost:
/// the path is free,
///  so the file goes back and the command says so;
/// the path holds something else,
/// so both stay and the
///  private name is said.
/// This runs before the paths are judged,
/// because a path whose file waits beside it is empty
///  and would be read as someone else's.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const location of locations) for (const aside of asidesOf(location)) if (free(location)) { renameNoReplace(aside, location); say(`put back ${location}`); } else say(`${aside} holds ...`);
/// ```
pub fn put_back_stranded_takes<W: Write + ?Sized>(root: &Path, changes: &[Change], out: &mut W) {
    for change in changes {
        let location: PathBuf = worktree_path(root, change.path.as_slice());
        for aside in asides_of(location.as_path()) {
            match rename_noreplace(aside.as_path(), location.as_path()) {
                Ok(()) => {
                    let _noted = out.write_all(
                        format!(
                            "monochromatic-git-policy-cli: put back {}, which a command that stopped while finishing a merge had set aside; \
                             its bytes are the ones it had.\n",
                            location.display()
                        )
                        .as_bytes(),
                    );
                }
                Err(_) => {
                    let _noted = out.write_all(
                        format!(
                            "monochromatic-git-policy-cli: {} holds a file that a command that stopped while finishing a merge had set aside \
                             for {}, which now holds something else; nothing was removed.\n",
                            aside.display(),
                            location.display()
                        )
                        .as_bytes(),
                    );
                }
            }
        }
    }
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
/// rawBytesOf(place, location): string | undefined; // link ? hash-object --stdin < readlink : hash-object --no-filters -- <location>
/// ```
async fn raw_bytes_of_awaiting(git: &GitContext, root: &Path, location: &Path) -> Option<String> {
    if std::fs::symlink_metadata(location).ok()?.file_type().is_symlink() {
        return the_objects_a_link_hashes_to_awaiting(git, root, location).await.ok();
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
async fn the_file_form_of_awaiting(git: &GitContext, root: &Path, path: &[u8], expected: &str) -> Option<String> {
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
) -> Result<Take, TakeFailure> {
    let aside: PathBuf = aside_name(location);
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
        // `may_remove` proves that nothing can write into the file any more;
        // a file it cannot prove that of is kept where it is,
        //  because its private name is then one of that write's only names.
        if !may_remove_awaiting(&place.git, root, entry.path.as_slice(), entry.aside.as_path(), entry.taken, entry.expected.as_str()).await {
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
        }
        match std::fs::remove_file(entry.aside.as_path()) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(io_failure("removing", entry.aside.as_path(), &error)),
        }
    }
    return Ok(());
}

/// The beginning of a scan's reason for a process whose descriptors were refused,
/// as opposed to a scan cut at its bound or unable to list the processes.
const UNREADABLE_PROCESS_REASON: &str = "PID ";

/// What:
///  Whether nothing can write into a file that was taken aside any more,
///  so removing its private name loses nothing,
///  and whether its bytes are
///  the branch's own form rather than bytes a clean filter hides.
/// Why:
///   A program that already has the file open keeps that descriptor across the rename,
/// and a write through it lands
///  in the file that waits aside;
/// comparing the identity a moment before the unlink cannot close that,
/// because the
///  write can arrive in the instant after the comparison
/// (a cross-vendor review of the second round).
/// Two things are
///  asked instead,
/// and both must hold:
/// the file's identity is the one the take checked,
/// and a scan of every process
///  this command can read finds none with that file open.
/// That is the proof the commit service's lock removal acts on,
/// with
///  the same limit:
/// a process in another user namespace,
/// or one that cannot be read,
/// is not seen.
///  A scan that was cut at its bound,
/// or could not list the processes,
/// proves nothing and the file is kept.
///  Every take of a merge's paths asks this before it removes a private name,
///  whether the file was replaced or deleted,
///  so a carry and the second pass lose nothing a program can still write.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mayRemove(git, root, path, aside, taken, expected): boolean; // identity unchanged && no readable process has it open && the scan finished && the bytes still match
/// ```
pub(crate) async fn may_remove_awaiting(git: &GitContext, root: &Path, path: &[u8], aside: &Path, taken: FileIdentity, expected: &str) -> bool {
    // The scan comes first, and the two looks come after it, as the last steps before the unlink: a writer that still holds
    //  the file is seen by the scan, a writer that let go has already written so the bytes and the identity then show it, and
    //  the identity is read because a clean filter can hide changed bytes: an appended line that the filter drops leaves the
    //  object as it was and the size changed.
    let scanned: HolderEvidence = scan_platform_holders_bounded(aside, taken.device, taken.inode);
    if !scanned.holders.is_empty() {
        return false;
    }
    for reason in &scanned.partial {
        if !reason.starts_with(UNREADABLE_PROCESS_REASON) {
            return false;
        }
    }
    // A control reaches here to write as a program that kept its descriptor would, after the scan and before the last look.
    crate::service_fault::reach("aside-scanned");
    return are_the_file_form_of_awaiting(git, root, path, aside, expected).await == Some(true);
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
