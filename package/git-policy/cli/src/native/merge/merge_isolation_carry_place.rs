//! What:
//!  Put one planned worktree write in place,
//!  or leave the path alone because what stands there is not the entry that was judged:
//!  the private name the new bytes take first,
//!  the take of the judged entry under a name of its own,
//!  the check of what the take caught,
//!  the placing of the new entry,
//!  and putting back an entry that turned out not to be the one that was judged.
//! Why:
//!  A judgement and a write are separate steps,
//!  so a person who saves the file between them had their bytes replaced with no
//!  notice;
//! one rename takes the entry atomically,
//!  so the bytes that are checked are exactly the bytes that make way for the write
//!  (`doc/planning/cli-git-merge-and-repair-route-findings-on-main.md`,
//! finding 4,
//!  which is how the second pass's install already writes a merged file).
//!  The identity is the cheap check and the bytes are the authority:
//! a program that keeps its descriptor
//!  across the rename can write the same length into the file and put the modification time back,
//! which leaves every
//!  part of the identity as it was.
//!  A kill between the take and the placing leaves the judged entry under a private name that names its
//!  path (`merge_isolation_take_asides.rs`),
//! which the next recovery puts back;
//!  the new bytes under this module's own private name are made again by the
//!  next command,
//! so a kill that leaves one loses nothing.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const placed = await placeFile({ context, root, path, judged, write }); // renameNoReplace take, check, place or put back
//! ```

use crate::merge_isolation_carry_writes::{CarryFailure, FileWrite, IndexWrite};
use crate::merge_isolation_changes::{FileIdentity, WorktreeState, file_identity, worktree_path};
use crate::merge_isolation_take_aside_names::aside_name;
use crate::merge_isolation_take_asides::{are_the_file_form_of_awaiting, may_remove_the_aside_awaiting};
use crate::rename_that_never_replaces::rename_noreplace;
use crate::transaction_git::GitContext;
use std::path::{Path, PathBuf};

/// The mode a symbolic link is recorded with.
const SYMBOLIC_LINK_MODE: &str = "120000";

/// What:
///  What stood at a worktree path when the write was judged.
/// Why:
///   The take compares both parts of it:
/// the identity of the entry the file system hands out,
///  and the object its bytes make,
/// so an entry written after the judgement is never replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type JudgedFile = { identity?: FileIdentity; content?: string };
/// ```
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct JudgedFile {
    /// The identity of the entry that stood there,
    /// or nothing when nothing did.
    pub(crate) identity: Option<FileIdentity>,
    /// The object the bytes of that entry make,
    /// or nothing when nothing stood there.
    pub(crate) content: Option<String>,
}

/// What one planned write came to.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Placed = { kind: 'written' } | { kind: 'nothing' } | { kind: 'changed'; kept?: string };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Placed {
    /// The path holds what the write asked for.
    Written,
    /// There was nothing for the write to do.
    Nothing,
    /// The path holds bytes the write did not judge,
    /// so nothing was written to it;
    /// `kept` names the entry that was taken aside and could not go back.
    Changed {
        /// Where an entry of the person's waits,
        /// when the path was taken by another write before it could go back.
        kept: Option<PathBuf>,
    },
}

/// What:
///  What stood at a worktree path when the step was judged,
/// for the placement that refuses to replace anything else.
/// Why:
///   The state names the object the bytes make and the identity names the entry itself,
/// and a write needs both to
///  prove that what it is about to replace is still the thing it judged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const judgedOf = (state, identity) => ({ identity, content: state.kind === 'content' ? state.object : undefined });
/// ```
pub(crate) fn judged_of(state: &WorktreeState, identity: Option<FileIdentity>) -> JudgedFile {
    return match state {
        WorktreeState::Content { object, .. } => JudgedFile { identity, content: Some(object.clone()) },
        WorktreeState::Missing | WorktreeState::Other => JudgedFile { identity, content: None },
    };
}

/// What:
///  The entries an index check accepts,
///  which are the ones already there and the one this step is about to write.
/// Why:
///   A step that was killed after its index records were written finds its own work in the index;
/// reading that
///  as someone else's dropped the layers it had not written yet.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const allowed = (known, plan) => plan.kind === 'entry' ? [...known, [plan.mode, plan.oid]] : plan.kind === 'remove' ? [...known, null] : known;
/// ```
pub(crate) fn explained_entries<'a>(known: &[Option<(&'a str, &'a str)>], plan: &'a IndexWrite) -> Vec<Option<(&'a str, &'a str)>> {
    let mut allowed: Vec<Option<(&str, &str)>> = known.to_vec();
    match plan {
        IndexWrite::Leave => {}
        IndexWrite::Entry(mode, oid) => allowed.push(Some((mode.as_str(), oid.as_str()))),
        IndexWrite::Remove => allowed.push(None),
    }
    return allowed;
}

/// The file name an entry keeps,
/// spelled as text a file system takes on every platform this builds for.
fn the_file_name(found: &std::ffi::OsStr) -> String {
    return found.to_string_lossy().into_owned();
}

/// The private name the new entry is written under before it takes its path.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `${dirname(location)}/.cli-git-carry-${pid}-${count}-${nanos}-${basename(location)}`
/// ```
fn private_name(location: &Path) -> PathBuf {
    static COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let count: u64 = COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nanos: u128 = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos();
    let name: String = location.file_name().map(the_file_name).unwrap_or_default();
    let process: u32 = std::process::id();
    return location.with_file_name(format!(".cli-git-carry-{process}-{count}-{nanos:x}-{name}"));
}

/// What:
///  The mode a written file carries,
/// with the failure the attempt made.
/// Why:
///   A file whose mode could not be set is not the entry the write promised,
/// so the attempt fails and is reported rather than
///  placed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function setMode(file, mode): Promise<void>; // 0o755 for 100755, else 0o644
/// ```
fn set_mode(file: &Path, mode: &str) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let bits: u32 = if mode == "100755" { 0o755 } else { 0o644 };
        return match std::fs::set_permissions(file, std::fs::Permissions::from_mode(bits)) {
            Ok(()) => Ok(()),
            Err(error) => Err(error.to_string()),
        };
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        let _ = (file, mode);
        return Ok(());
    }
}

/// What:
///  Write the new entry under a private name beside its path,
///  so placing it is one rename.
/// Why:
///  A rename in the same directory is one step on whatever file system the worktree is on,
/// so a kill leaves
///  either what was there or the whole new entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writePrivate(location, bytes, mode): string; // throws on a write that failed
/// ```
fn write_private(location: &Path, bytes: &[u8], mode: &str) -> Result<PathBuf, String> {
    if let Some(parent) = location.parent() {
        let _made = std::fs::create_dir_all(parent);
    }
    let private: PathBuf = private_name(location);
    write_entry(private.as_path(), bytes, mode)?;
    return Ok(private);
}

/// What:
///  Write one entry's bytes or its link target under a name beside its path,
///  with the mode it names.
/// Why:
///  The bytes of an entry are written where nothing else writes before they take a path,
/// so a kill leaves either
///  what was there or the whole entry;
/// a copy kept beside a path is written the same way,
/// and both go through
///  one step so a link is made in both cases and never followed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function writeEntry(named, bytes, mode): void; // symlink(target) for 120000, else write + chmod
/// ```
fn write_entry(named: &Path, bytes: &[u8], mode: &str) -> Result<(), String> {
    if mode == SYMBOLIC_LINK_MODE {
        #[cfg(unix)]
        {
            use std::os::unix::ffi::OsStrExt;
            return match std::os::unix::fs::symlink(std::ffi::OsStr::from_bytes(bytes), named) {
                Ok(()) => Ok(()),
                Err(error) => Err(error.to_string()),
            };
        }
        // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
        #[cfg(not(unix))]
        {
            let _ = (named, bytes);
            return Err(String::from("a symbolic link cannot be made on this platform"));
        }
    }
    if let Err(error) = std::fs::write(named, bytes) {
        let _left = std::fs::remove_file(named);
        return Err(error.to_string());
    }
    set_mode(named, mode)?;
    return Ok(());
}

/// What:
///  The object the bytes of one entry make,
/// compared with the object the judgement recorded.
/// Why:
///   The identity alone cannot see a write through a descriptor the file already had,
/// so the bytes of the
///  entry that was taken aside are read and hashed with the path's own attributes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function areTheJudgedBytes(context, root, path, location, judged): Promise<boolean | undefined>;
/// ```
async fn are_the_judged_bytes_awaiting(context: &GitContext, root: &Path, path: &[u8], location: &Path, judged: &JudgedFile) -> Option<bool> {
    let Some(expected) = judged.content.as_deref() else { return Some(false) };
    return are_the_file_form_of_awaiting(context, root, path, location, expected).await;
}

/// What:
///  Whether the entry at `location` is the one the write judged.
/// Why:
///   An entry that differs is the person's,
///  written after the write looked,
/// so it goes back where it came from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function isTheJudgedEntry(context, root, path, location, judged): Promise<boolean | undefined>;
/// ```
async fn is_the_judged_entry_awaiting(context: &GitContext, root: &Path, path: &[u8], location: &Path, judged: &JudgedFile) -> Option<bool> {
    if judged.identity.is_none() || file_identity(location) != judged.identity {
        return Some(false);
    }
    return are_the_judged_bytes_awaiting(context, root, path, location, judged).await;
}

/// What:
///  Put an entry that turned out not to be the one that was judged back where it was found,
///  and say that the path was left alone.
/// Why:
///   The bytes are the person's,
/// and a path freed by the take is the one place they belong;
/// a path another
///  write already took keeps both entries,
/// and the one that was put aside is named rather than dropped.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const putBack = (location, aside) => renameNoReplace(aside, location) ? { kind: 'changed' } : { kind: 'changed', kept: aside };
/// ```
fn put_back(location: &Path, aside: &Path) -> Result<Placed, CarryFailure> {
    return match rename_noreplace(aside, location) {
        Ok(()) => Ok(Placed::Changed { kept: None }),
        // Another write took the path, so both entries stay and the one put aside is named.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Ok(Placed::Changed { kept: Some(aside.to_path_buf()) }),
        Err(error) => Err(CarryFailure::Failed(format!("{} could not be put back: {error}", aside.display()))),
    };
}

/// What:
///  Whether the entry a take put aside may lose its private name,
///  and the private name that stays when it may not.
/// Why:
///   The bytes the take holds are the merge's own,
///  which the capture keeps,
///  but a program that already had the file open
///  can still write into it after the take checked it,
///  so the private name goes only on the proof nothing can write any
///  more,
///  and a file the proof does not cover is kept and named
///  (`merge_isolation_take_asides.rs`).
///  A take that holds no identity cannot prove anything about the file,
/// so its private name stays.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function dropTheAside({ context, root, path, aside, judged, kept }): Promise<void>; // mayRemove ? unlink : kept.push(aside)
/// ```
async fn drop_the_aside_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    aside: &Path,
    judged: &JudgedFile,
    kept: &mut Vec<PathBuf>,
) -> Result<(), CarryFailure> {
    let (Some(taken), Some(expected)) = (judged.identity, judged.content.as_deref()) else {
        kept.push(aside.to_path_buf());
        return Ok(());
    };
    if !may_remove_the_aside_awaiting(context, root, path, aside, taken, expected).await {
        kept.push(aside.to_path_buf());
        return Ok(());
    }
    return match std::fs::remove_file(aside) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(CarryFailure::Failed(format!("{} could not be removed: {error}", aside.display()))),
    };
}

/// What a take of one worktree entry found.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Taken = { kind: 'aside'; aside: string } | { kind: 'changed'; kept?: string };
/// ```
enum Taken {
    /// The entry that was judged waits at this private name.
    Aside(PathBuf),
    /// The path holds bytes the write did not judge,
    /// and nothing was taken.
    Changed {
        /// Where an entry of the person's waits,
        /// when it could not go back.
        kept: Option<PathBuf>,
    },
}

/// What:
///  Take the entry at `location` aside under a private name,
///  and check that it is the one the write judged.
/// Why:
///   The take is one rename that never replaces,
/// so nothing standing at the path can be destroyed by it;
///  the check then runs on a name nobody else writes.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function takeJudged({ context, root, path, location, judged }): Promise<Taken>;
/// ```
async fn take_judged_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    location: &Path,
    judged: &JudgedFile,
    transaction_id: &str,
) -> Result<Taken, CarryFailure> {
    let aside: PathBuf = aside_name(location, transaction_id);
    match rename_noreplace(location, aside.as_path()) {
        Ok(()) => {}
        // The path was freed between the judgement and the take, which is not this write to make.
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Taken::Changed { kept: None }),
        Err(error) => return Err(CarryFailure::Failed(format!("{} could not be set aside: {error}", location.display()))),
    }
    match is_the_judged_entry_awaiting(context, root, path, aside.as_path(), judged).await {
        Some(true) => return Ok(Taken::Aside(aside)),
        // A read that fails cannot prove the bytes, so the entry goes back rather than be replaced.
        Some(false) | None => {
            return match put_back(location, aside.as_path())? {
                Placed::Changed { kept } => Ok(Taken::Changed { kept }),
                Placed::Written | Placed::Nothing => Ok(Taken::Changed { kept: None }),
            };
        }
    }
}

/// What:
///  The private names one write leaves beside a path,
///  and what ends each of them.
/// Why:
///  A write either lands on the path or keeps what it would have written beside it,
/// and the second is
///  what a path its caller cannot prove holds:
/// a path a save may stand at is left as it is,
/// and the bytes the
///  merge carries stay beside it,
/// recorded with the copies of a merge and named,
/// so they are not a file no
///  command ends.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SetAsides = { heldOpen: Path[]; unproven: Path[]; keepThePlannedBytes: boolean };
/// ```
pub(crate) struct SetAsides<'a> {
    /// The names a program could still write into,
    /// which stay and are named.
    pub(crate) held_open: &'a mut Vec<PathBuf>,
    /// The names holding bytes this command did not write at the path,
    /// which stay beside it and are named.
    pub(crate) unproven: &'a mut Vec<PathBuf>,
    /// Whether the bytes planned for this path stay beside it instead of taking it.
    pub(crate) keep_the_planned_bytes: bool,
}

/// What:
///  Remove the entry at `location`
///  if it is the one the write judged.
/// Why:
///   A removal is a write of "nothing" and is judged the same way:
/// an entry that appeared after the judgement is the
///  person's and is left alone.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function removeJudged({ context, root, path, location, judged }): Promise<Placed>;
/// ```
async fn remove_judged_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    location: &Path,
    judged: &JudgedFile,
    transaction_id: &str,
    kept: &mut Vec<PathBuf>,
) -> Result<Placed, CarryFailure> {
    if judged.content.is_none() {
        // Nothing stood there when the write looked, so an entry that is there now came after it.
        return match std::fs::symlink_metadata(location) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(Placed::Nothing),
            _ => Ok(Placed::Changed { kept: None }),
        };
    }
    return match take_judged_awaiting(context, root, path, location, judged, transaction_id).await? {
        Taken::Aside(aside) => {
            // The entry that was judged is the one this write removes, and its bytes are in the merge's capture.
            drop_the_aside_awaiting(context, root, path, aside.as_path(), judged, kept).await?;
            Ok(Placed::Written)
        }
        Taken::Changed { kept: stranded } => Ok(Placed::Changed { kept: stranded }),
    };
}

/// What:
///  Write one entry's bytes at its path,
///  or leave the path alone when it holds something the write did not judge.
/// Why:
///   The take is one rename that never replaces,
/// so the bytes that are checked are the bytes that make way for the
///  new entry.
/// A kill between the take and the placing leaves the judged entry under a private name beside the path,
///  which the next command puts back.
///  The entry the take moved aside is removed once the new entry stands there,
/// because its bytes are the merge's own,
///  which the capture keeps;
/// the one exception is a take nothing proves:
/// the file may be a save the person made after a killed
///  merge,
/// so it is kept and named instead of removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function placeBytes({ context, root, path, judged, write, setAsides }): Promise<Placed>;
/// ```
async fn place_bytes_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    judged: &JudgedFile,
    write: &FileWrite,
    transaction_id: &str,
    set_asides: &mut SetAsides<'_>,
) -> Result<Placed, CarryFailure> {
    let FileWrite::Bytes(bytes, mode) = write else {
        return Err(CarryFailure::Failed(format!("{} was planned as bytes and is not", crate::path_bytes::path_message_text(path))));
    };
    let location: PathBuf = worktree_path(root, path);
    if set_asides.keep_the_planned_bytes {
        // Nothing proves the entry standing at the path is the one Git's own update wrote, and its bytes are exactly the merge's
        //  own, which is also what a save made after a killed merge holds. The merge finishes and the file stays as it is: the
        //  bytes this carry would have written are kept beside it under a name that names the merge, and the caller records and
        //  names them, so the person's bytes exist whichever of the two the file was.
        let aside: PathBuf = aside_name(location.as_path(), transaction_id);
        return match write_entry(aside.as_path(), bytes, mode) {
            Ok(()) => {
                set_asides.unproven.push(aside);
                Ok(Placed::Nothing)
            }
            Err(error) => Err(CarryFailure::Failed(format!("{} could not be kept beside {}, which was left as it is: {error}", location.display(), location.display()))),
        };
    }
    let private: PathBuf = match write_private(location.as_path(), bytes, mode) {
        Ok(made) => made,
        Err(error) => return Err(CarryFailure::Failed(format!("{} could not be written: {error}", location.display()))),
    };
    if judged.content.is_none() {
        // Nothing stood there when the write looked, so the path is made and never replaced.
        return match rename_noreplace(private.as_path(), location.as_path()) {
            Ok(()) => Ok(Placed::Written),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let _left = std::fs::remove_file(private.as_path());
                Ok(Placed::Changed { kept: None })
            }
            Err(error) => Err(CarryFailure::Failed(format!("{} could not be made: {error}", location.display()))),
        };
    }
    let aside: PathBuf = match take_judged_awaiting(context, root, path, location.as_path(), judged, transaction_id).await? {
        Taken::Aside(waits) => waits,
        Taken::Changed { kept: stranded } => {
            let _left = std::fs::remove_file(private.as_path());
            return Ok(Placed::Changed { kept: stranded });
        }
    };
    let placed: Placed = match rename_noreplace(private.as_path(), location.as_path()) {
        Ok(()) => Placed::Written,
        // Another write took the path in the gap; the entry put aside is kept and named, and the bytes this write
        //  would have placed are made again by the next command.
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => Placed::Changed { kept: Some(aside.clone()) },
        // The entry put aside stays where it is, so the message names it rather than leaving a person to find it.
        Err(error) => {
            return Err(CarryFailure::Failed(format!(
                "{} could not be written: {error}; the file it held before is at {}",
                location.display(),
                aside.display()
            )));
        }
    };
    if matches!(placed, Placed::Written) {
        // What the take holds is the entry the write replaces, whose bytes the merge's capture keeps.
        drop_the_aside_awaiting(context, root, path, aside.as_path(), judged, set_asides.held_open).await?;
    } else {
        let _left = std::fs::remove_file(private.as_path());
    }
    return Ok(placed);
}

/// What:
///  Write one planned worktree write,
/// which is bytes,
///  a removal,
/// or nothing at all.
/// Why:
///   Every write of a carry,
/// a clear and a putting back goes through this one step,
/// so an entry changed
///  after it was judged is never replaced by any of them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function placeFile({ context, root, path, judged, write, setAsides }): Promise<Placed>;
/// ```
pub(crate) async fn place_file_awaiting(
    context: &GitContext,
    root: &Path,
    path: &[u8],
    judged: &JudgedFile,
    write: &FileWrite,
    transaction_id: &str,
    set_asides: &mut SetAsides<'_>,
) -> Result<Placed, CarryFailure> {
    // The comparisons run from the top of the worktree, where a relative directory option of the invocation does not reach:
    //  a `-C repo` applied to a request that already runs in the worktree would name the worktree twice, and every read of
    //  `--path=<path>` would fail, which reads as a file that nobody judged.
    let rooted: GitContext = GitContext { global_prefix: Vec::new(), ..context.clone() };
    let location: PathBuf = worktree_path(root, path);
    return match write {
        FileWrite::Leave => Ok(Placed::Nothing),
        FileWrite::Remove => remove_judged_awaiting(&rooted, root, path, location.as_path(), judged, transaction_id, set_asides.held_open).await,
        FileWrite::Bytes(_, _) => place_bytes_awaiting(&rooted, root, path, judged, write, transaction_id, set_asides).await,
    };
}

/// What:
///  Whether the entry at one path is still the one a write judged,
///  read without taking anything.
/// Why:
///   A step judges every path before it writes the first byte of its index records,
/// so a path written
///  before that point stops the whole step with nothing written;
/// the take then covers the window that follows.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function isStillTheJudgedEntry(root, path, judged): boolean; // identityOf(location) === judged.identity
/// ```
pub(crate) fn is_still_the_judged_entry(root: &Path, path: &[u8], judged: &JudgedFile) -> bool {
    return file_identity(worktree_path(root, path).as_path()) == judged.identity;
}

/// The controls of the placement stay out of the release executable.
#[cfg(test)]
#[path = "merge_isolation_carry_place_tests.rs"]
mod tests;
