//! What:
//!  The checkout an interrupted `git worktree add` never finished,
//! finished by a command run in the worktree it fills.
//! Why:
//!  An add that dies inside its own checkout leaves a worktree Git registered,
//! holding its pointer and a part of the tracked files,
//!  and no index,
//! or with the `index.lock` of the step that publishes one still standing.
//! Nothing else ever finishes it:
//! the next command names it and
//!  hands the person `rm`, `reset --hard` and a second add,
//! which is the wrapper not cleaning up after itself.
//! This pass finishes it,
//! and only where it proves nothing of the person's is there:
//! every entry standing at the destination must hold the very bytes the source holds at that name,
//! so the only files Git's own checkout replaces are the ones the interrupted add made,
//! and a destination holding anything else is named instead of changed.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const end = await finishInterruptedCheckout({ commonDir, destination, source, context });
//! ```

use crate::fs_beneath::{ObjectId, Owned, identity_at, open_root, remove_if_identical};
use crate::index_lock_holders::{HolderEvidence, scan_platform_holders};
use crate::json_record::quote;
use crate::transaction_git::{GitContext, GitOutput, GitRequest, run_git};
use crate::worktree_add_binding::{bind_registration, path_identity};
use crate::worktree_copy_ceiling::CopyCeiling;
use crate::worktree_copy_error::{WorktreeCopyError, failure_with_cause, raw_failure_of};
use crate::worktree_copy_interrupted_add_settlement::{INDEX, INDEX_LOCK, add_is_finished};
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

/// What:
///  How one pass of finishing an interrupted add's checkout ended.
/// Why:
///   A whole checkout is nothing to report;
/// a finished one says what it did;
/// and one nothing proves leaves the person's state alone,
/// with the reason the person reads.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type CheckoutFinish = 'whole' | { finished: string } | { notProved: string };
/// ```
#[derive(Debug, Eq, PartialEq)]
pub enum CheckoutFinish {
    /// The checkout at the destination is whole;
    /// nothing was changed here.
    Whole,
    /// The checkout was incomplete and this pass finished it,
    /// with the line the person reads.
    Finished(String),
    /// The checkout is incomplete and nothing proves the worktree holds only what the interrupted add made.
    NotProvable(String),
}

/// The name the destination's pointer has,
/// which the source never holds the same way.
const POINTER: &str = ".git";

/// What:
///  The names one directory holds,
/// or nothing when it cannot be read.
/// Why:
///   A directory that cannot be listed proves nothing about what it holds,
/// so the walk answers "not proven" rather than "nothing there".
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const names = tryReaddir(directory); // undefined when it cannot be read
/// ```
fn names_of(directory: &Path) -> Option<Vec<OsString>> {
    let listing = std::fs::read_dir(directory).ok()?;
    let mut names: Vec<OsString> = Vec::new();
    for listed in listing {
        names.push(listed.ok()?.file_name());
    }
    return Some(names);
}

/// What:
///  Whether the file at `left` holds the very bytes the file at `right` holds.
/// Why:
///   The interrupted add made its copies from the source's bytes,
/// so a file that holds them is its own,
/// while one holding anything else is somebody's and is never replaced.
/// The size is compared first,
/// so a file of another size costs one lookup and no read.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const same = (left, right) => { const a = statSync(left); const b = statSync(right); return b.isFile() && a.size === b.size && readFileSync(left).equals(readFileSync(right)) };
/// ```
fn holds_the_same_bytes(left: &Path, right: &Path) -> bool {
    let Ok(held) = std::fs::metadata(left) else { return false };
    let Ok(wanted) = std::fs::metadata(right) else { return false };
    if !wanted.is_file() || held.len() != wanted.len() {
        return false;
    }
    return std::fs::read(left).is_ok_and(|mine| { return std::fs::read(right).is_ok_and(|theirs| { return mine == theirs; }); });
}

/// What:
///  The first entry standing in the destination that the source does not hold with the very same bytes,
/// or nothing when every entry is the interrupted add's own copy.
/// Why:
///   Git's own checkout writes every tracked file and replaces one that stands there,
/// so finishing it is safe only while everything standing there came from the source.
/// A directory is no file of its own,
/// and that checkout leaves an empty one exactly as it is,
/// so directories are walked and never named.
/// The pointer Git wrote at `.git` is the worktree itself,
/// and the source's own `.git` is never the same file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const foreign = firstForeignEntry(destination, source, ''); // undefined when nothing is foreign
/// ```
fn first_foreign_entry(destination: &Path, source: &Path, relative: &Path) -> Option<PathBuf> {
    let names: Vec<OsString> = names_of(destination)?;
    for name in names {
        let here: PathBuf = destination.join(name.as_os_str());
        let named: PathBuf = relative.join(name.as_os_str());
        if relative.as_os_str().is_empty() && name.as_os_str() == OsStr::new(POINTER) {
            continue;
        }
        let Ok(held) = std::fs::symlink_metadata(here.as_path()) else { return Some(named) };
        if held.is_dir() {
            if let Some(foreign) = first_foreign_entry(here.as_path(), source, named.as_path()) {
                return Some(foreign);
            }
            continue;
        }
        let there: PathBuf = source.join(named.as_path());
        if held.file_type().is_symlink() {
            let same: bool = std::fs::read_link(here.as_path()).is_ok_and(|mine| { return std::fs::read_link(there.as_path()).is_ok_and(|theirs| { return mine == theirs; }); });
            if !same {
                return Some(named);
            }
            continue;
        }
        if !held.is_file() || !holds_the_same_bytes(here.as_path(), there.as_path()) {
            return Some(named);
        }
    }
    return None;
}

/// What:
///  Whether the destination can be listed and every entry it holds came from the source.
/// Why:
///   This is the whole proof the finish rests on,
/// and a destination that cannot be listed is not proven.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const provable = namesOf(destination) !== undefined && firstForeignEntry(destination, source, '') === undefined;
/// ```
fn destination_holds_only_the_sources_bytes(destination: &Path, source: &Path) -> bool {
    return names_of(destination).is_some() && first_foreign_entry(destination, source, Path::new("")).is_none();
}

/// What:
///  Whether no process of this machine holds the lock open,
/// which is the evidence a lock whose writer died rests on.
/// Why:
///   A scan that could not see everything proves nothing,
/// so a partial scan,
/// an unresolvable descriptor and a live holder all answer no.
/// A lock that stands where the published index stands is not asked this:
/// that file is the index itself,
/// and taking the lock's name away takes no data.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unheld = (lock) => { const e = scanPlatformHolders(lock); return e.holders.length === 0 && e.partial.length === 0 && !e.unresolved() };
/// ```
fn nothing_holds_the_lock_open(lock: &Path, locked: ObjectId) -> bool {
    let evidence: HolderEvidence = scan_platform_holders(lock, locked.device, locked.inode);
    return evidence.holders.is_empty() && evidence.partial.is_empty() && !evidence.unresolved();
}

/// What:
///  Takes away the lock a killed index install left,
/// or says why nothing was changed.
/// Why:
///   Git's own checkout cannot start while `index.lock` stands,
/// and that lock is exactly what held the recovery back.
/// A lock that is the published index's own file goes on its identity alone;
/// one that stands alone goes only when the add that made it is proven dead
/// and no process holds it open,
/// so a live writer's lock is never taken away.
/// The removal acts on the identity read first,
/// so a lock another command takes between the reading and the removal is kept.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearTheLock(admin, ownerIsDead); // throws the reason nothing was removed
/// ```
fn clear_the_lock(admin: &Path, owner_is_dead: bool) -> Result<(), WorktreeCopyError> {
    let directory = open_root(admin).map_err(raw_failure_of)?;
    let lock: &OsStr = OsStr::new(INDEX_LOCK);
    let Ok(locked) = identity_at(&directory, lock) else {
        // Nothing at the lock's name is the answer this pass wants.
        return Ok(());
    };
    let published: bool = identity_at(&directory, OsStr::new(INDEX)).is_ok_and(|index| { return index == locked; });
    if !published && !owner_is_dead {
        return Err(failure_with_cause(
            format!(
                "monochromatic-git-policy-cli: the lock {} of an interrupted `git worktree add` stands and nothing proves the add that made it is gone, so nothing was changed.",
                quote(admin.join(INDEX_LOCK).to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        ));
    }
    if !published && !nothing_holds_the_lock_open(admin.join(INDEX_LOCK).as_path(), locked) {
        return Err(failure_with_cause(
            format!(
                "monochromatic-git-policy-cli: the lock {} of an interrupted `git worktree add` stands and a process may hold it open, so nothing was changed.",
                quote(admin.join(INDEX_LOCK).to_string_lossy().as_ref())
            )
            .as_str(),
            None,
        ));
    }
    return match remove_if_identical(&directory, lock, locked).map_err(raw_failure_of)? {
        Owned::Removed | Owned::Absent => Ok(()),
        Owned::Kept(reason) => Err(failure_with_cause(
            format!("monochromatic-git-policy-cli: the lock of an interrupted `git worktree add` was not removed ({reason}).").as_str(),
            None,
        )),
    };
}

/// What:
///  Runs Git's own checkout in the worktree Git registered,
/// the way the add's own fallback does.
/// Why:
///   `reset --hard` writes the index and every tracked file,
/// which is the step the interrupted add never finished;
/// the administrative directory and the worktree are named,
/// so it acts on this worktree whatever the caller's directory is,
/// and the run inherits the wrapper's own Git switches
/// (hooks off and no `core.fsmonitor`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const out = await runGit({ cwd: destination, args: ['reset', '--hard', '--no-recurse-submodules'], env: { GIT_DIR: admin, GIT_WORK_TREE: destination } });
/// ```
fn check_out_like_git(context: &GitContext, destination: &Path, admin: &Path) -> Result<GitOutput, WorktreeCopyError> {
    let request: GitRequest = GitRequest {
        cwd: destination.to_path_buf(),
        arguments: vec![OsString::from("reset"), OsString::from("--hard"), OsString::from("--no-recurse-submodules")],
        without_prefix: true,
        environment: vec![(OsString::from("GIT_DIR"), admin.as_os_str().to_os_string()), (OsString::from("GIT_WORK_TREE"), destination.as_os_str().to_os_string())],
        ..GitRequest::default()
    };
    return run_git(context, &request).map_err(raw_failure_of);
}

/// What:
///  Finish the checkout an interrupted add left incomplete at `destination`,
/// or say why nothing was changed there.
/// Why:
///   Only the worktree's own command may do this:
/// the work is that worktree's,
/// and a command elsewhere gains nothing from it.
/// The destination must be a registration of this repository whose pointer names its own
///  administrative directory,
/// the source must still stand,
/// and everything at the destination must be the interrupted add's own copy of it.
/// Then the lock a killed index install left goes,
/// Git's own checkout completes the worktree,
/// and the copy of the source's ignored state that comes after it finds the worktree whole.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const end = await finishInterruptedCheckout({ commonDir, destination, source, context, ownerIsDead, ceiling });
/// ```
pub fn finish_interrupted_checkout(
    common_dir: &Path,
    destination: &Path,
    source: &Path,
    context: &GitContext,
    owner_is_dead: bool,
    ceiling: &CopyCeiling,
) -> Result<CheckoutFinish, WorktreeCopyError> {
    let Ok(bound) = bind_registration(common_dir, destination) else {
        return Ok(CheckoutFinish::NotProvable(String::from("it is not a registration of this repository that its pointer names")));
    };
    if bound.binding.destination != destination || path_identity(destination) != Some(bound.binding.destination_id) {
        return Ok(CheckoutFinish::NotProvable(String::from("the directory there is not the one Git made for the add")));
    }
    let admin: PathBuf = bound.binding.admin.clone();
    if add_is_finished(destination, admin.as_path()) {
        return Ok(CheckoutFinish::Whole);
    }
    if !source.is_dir() {
        return Ok(CheckoutFinish::NotProvable(format!("the source {} stands no more", quote(source.to_string_lossy().as_ref()))));
    }
    if !destination_holds_only_the_sources_bytes(destination, source) {
        let foreign: String = first_foreign_entry(destination, source, Path::new(""))
            .map(|found| { return String::from_utf8_lossy(found.as_os_str().as_encoded_bytes()).into_owned(); })
            .unwrap_or_default();
        return Ok(CheckoutFinish::NotProvable(format!("the entry {foreign} there holds something the source does not")));
    }
    if ceiling.reached() {
        return Ok(CheckoutFinish::NotProvable(ceiling.stopped_sentence()));
    }
    clear_the_lock(admin.as_path(), owner_is_dead)?;
    let checked: GitOutput = check_out_like_git(context, destination, admin.as_path())?;
    if !checked.succeeded() {
        return Ok(CheckoutFinish::NotProvable(format!("Git's own checkout of it ended with {}", String::from_utf8_lossy(checked.stderr.as_slice()).trim())));
    }
    if !add_is_finished(destination, admin.as_path()) {
        return Ok(CheckoutFinish::NotProvable(String::from("Git's own checkout of it ended and the worktree still holds no index")));
    }
    return Ok(CheckoutFinish::Finished(format!(
        "monochromatic-git-policy-cli: finished the checkout an interrupted `git worktree add` left incomplete in {}: every entry it had already made held the source's bytes, and Git's own checkout completed the rest.",
        quote(destination.to_string_lossy().as_ref())
    )));
}

/// Checkout-finishing controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_interrupted_checkout_tests.rs"]
mod tests;
