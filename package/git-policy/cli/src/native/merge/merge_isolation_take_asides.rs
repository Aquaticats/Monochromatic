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
//! In TS you'd write (pseudocode):
//! ```ts
//! const taken = takeAside(location, judged); ... finishAsides(taken, stderr);
//! ```

use crate::index_lock_holders::{HolderEvidence, scan_platform_holders_bounded};
use crate::merge_isolation_changes::{Change, FileIdentity, file_identity, worktree_path};
use crate::recovery_error::{RecoveryError, io_failure};
use crate::rename_that_never_replaces::rename_noreplace;
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
/// which the private name of a take carries so a stranded one can be found again.
fn name_text(location: &Path) -> String {
    return location.file_name().map(name_of).unwrap_or_default();
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
pub(crate) fn aside_name(location: &Path) -> PathBuf {
    let count: u64 = ASIDE_COUNT.fetch_add(1, Ordering::Relaxed);
    let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    return location.with_file_name(format!("{ASIDE_PREFIX}-{}-{}-{count}-{nanos:x}", name_text(location), std::process::id()));
}

/// Whether one byte is one of the hex digits a ran number is written in.
fn is_hex_digit(byte: u8) -> bool {
    return byte.is_ascii_hexdigit();
}

/// Whether one name is a private name a take left for `location`:
/// the prefix of that path's file name,
/// then `<pid>-<count>-<nanos>`.
fn is_aside_of(name: &str, location: &Path) -> bool {
    let Some(rest) = name.strip_prefix(format!("{ASIDE_PREFIX}-{}-", name_text(location)).as_str()) else {
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
pub fn put_back_stranded_takes(root: &Path, changes: &[Change], out: &mut dyn Write) {
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


/// What a take of one worktree path found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Take = { kind: 'nothing' } | { kind: 'aside'; aside: string } | { kind: 'changed' };
/// ```
pub(crate) enum Take {
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
pub(crate) fn take_aside(location: &Path, judged: Option<FileIdentity>) -> Result<Take, RecoveryError> {
    let aside: PathBuf = aside_name(location);
    match rename_noreplace(location, aside.as_path()) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            // The path was empty when it was judged; a file that is there now came after the judgement and is not the
            //  merge's to replace.
            return Ok(if judged.is_none() { Take::Nothing } else { Take::Changed });
        }
        Err(error) => return Err(io_failure("setting aside", location, &error)),
    }
    let taken: Option<FileIdentity> = file_identity(aside.as_path());
    if judged.is_some() && taken == judged {
        return match taken {
            Some(identity) => Ok(Take::Aside { aside, taken: identity }),
            // `taken == judged` with nothing read is no match at all.
            None => Ok(Take::Changed),
        };
    }
    // It is not the file that was judged: put it back whole, and say nothing was replaced.
    return match rename_noreplace(aside.as_path(), location) {
        Ok(()) => Ok(Take::Changed),
        Err(error) => Err(RecoveryError(format!(
            "{} changed after this command judged it and it could not be put back ({error}); it is at {} and nothing was \
             replaced by the merge.",
            location.display(),
            aside.display()
        ))),
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
pub(crate) fn put_back(taken: &[TakenOnDisk]) -> Vec<String> {
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
    /// Where the file that was judged waits.
    pub aside: PathBuf,
    /// The identity the take checked,
    /// so a write into the aside after that check is seen.
    pub taken: FileIdentity,
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
pub fn finish_asides(taken: &[TakenOnDisk], out: &mut dyn Write) -> Result<(), RecoveryError> {
    for entry in taken {
        // `may_remove` proves that nothing can write into the file any more;
        // a file it cannot prove that of is kept where it is,
        //  because its private name is then one of that write's only names.
        if !may_remove(entry) {
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
///  so removing its private name loses nothing.
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
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function mayRemove(entry): boolean; // identity unchanged && no readable process has it open && the scan finished
/// ```
fn may_remove(entry: &TakenOnDisk) -> bool {
    if file_identity(entry.aside.as_path()) != Some(entry.taken) {
        return false;
    }
    let scanned: HolderEvidence = scan_platform_holders_bounded(entry.aside.as_path(), entry.taken.device, entry.taken.inode);
    if !scanned.holders.is_empty() {
        return false;
    }
    for reason in &scanned.partial {
        if !reason.starts_with(UNREADABLE_PROCESS_REASON) {
            return false;
        }
    }
    return true;
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
pub(crate) fn restore_what_git_did_not_write(taken: &[TakenOnDisk], out: &mut dyn Write) -> Result<(), RecoveryError> {
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
        written.push(TakenOnDisk { location: entry.location.clone(), aside: entry.aside.clone(), taken: entry.taken });
    }
    return finish_asides(written.as_slice(), out);
}


/// What:
///  Remove the directories a deleted path leaves empty up to the worktree top.
/// Why:
///   Git removes a deleted file's emptied directories too,
/// and the take has already removed the file itself;
/// a
///  directory that still holds anything stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// for (const directory of parents(file)) { try { await rmdir(directory); } catch { break; } }
/// ```
pub fn remove_empty_parents(root: &Path, file: &Path) {
    let mut directory: Option<&Path> = file.parent();
    while let Some(current) = directory {
        if current == root || !current.starts_with(root) || std::fs::remove_dir(current).is_err() {
            break;
        }
        directory = current.parent();
    }
}
