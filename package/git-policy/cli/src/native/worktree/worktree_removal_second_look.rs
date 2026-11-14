//! What:
//!  The proof that a removal may go,
//!  made again immediately before the destruction.
//! Why:
//!  The proof a landed sweep makes is made on the merge's side,
//! and the destruction is made later,
//!  by the commit service,
//! for as long as the handed-over record waits there.
//!  Everything the proof looked at can change in between:
//!  a worktree registered inside the candidate,
//!  a lock,
//!  an unfinished merge or rebase,
//!  a process that started working in it,
//!  a push failure recorded in its own Git directory.
//!  So the proof belongs where the destruction is,
//!  and this is that proof:
//!  one function,
//!  called by the sweep before it decides what becomes of a candidate,
//!  and by the service's own removal immediately before Git deletes the worktree.
//!  It answers the reason the worktree stays,
//!  or nothing when every condition still holds.
//!  Nothing here decides what stands in the way of the removal itself:
//!  a modified or an untracked file is Git's own refusal,
//!  in Git's own words,
//!  and the destroy of a subvolume has the proofs of `worktree_remove.rs`.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const why = whyTheWorktreeStays({ commonDir, procRoot }, worktree, admin); // string | undefined
//! ```

use crate::merge_isolation_run_types::IN_PROGRESS_MARKERS;
use crate::worktree_landed_registrations::{Registration, registrations_of};
use crate::worktree_process_inside::{ProcessWorking, process_working_inside};
use crate::worktree_removal_rescue::snapshot_one;
use std::path::Path;
use std::time::{Duration, SystemTime};

/// The start of every name Git gives a file an unfinished bisect keeps in a worktree's Git directory.
/// Why:
///  `IN_PROGRESS_MARKERS` answers the same question for a merge the wrapper runs itself,
///  so its list is reused rather than written again,
///  and a bisect needs the question answered too, because it never touches `HEAD`:
///  the checkout stays on its branch and every file stays clean,
///  and `git worktree remove` deletes the worktree and Git's own bisect records without a word.
///  `git bisect start` writes `BISECT_START`,
/// then `BISECT_NAMES`,
/// and opens `BISECT_LOG` only after both
///  (measured with Git 2.56 under `strace`),
///  so a process killed between those writes leaves a bisect whose marks are only the files written before the kill.
///  A proof that knew only the log let the removal take the worktree with the rest:
///  a fixture whose bisect was killed at the first open of `BISECT_LOG` had `BISECT_START` and `BISECT_NAMES`,
///  no log,
///  a clean checkout on its branch,
///  and the wrapped merge removed it and every file with it
///  (a cross-vendor audit of the landed sweep).
///  Every file a bisect keeps in that directory is named under this prefix,
///  and `git bisect reset` removes every one of them
///  (a fixture had none left after the reset),
///  so the prefix is what a removal can ask for without falling behind a Git that adds another file.
const BISECT_MARK_PREFIX: &str = "BISECT_";

/// What:
///  Where one reading of the proof looks,
/// and what it compares the worktree's ignored files with.
/// Why:
///   The sweep and the service read the same facts from different processes:
///  the repository is the common Git directory both name,
///  the process table is the machine's for a service and a fixture's for a control,
///  and the worktree the ignored files are compared with is the one the proof was made from,
///  which a service reads from the registration because no command of the merge is there any more.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type SecondLook = { commonDir: string; procRoot: string };
/// ```
pub(crate) struct SecondLook<'a> {
    /// The canonical common Git directory,
    /// which owns every registration.
    pub(crate) common_dir: &'a Path,
    /// The process table to read.
    pub(crate) proc_root: &'a Path,
}

/// What:
///  Why this worktree may not be removed now,
///  or nothing when everything the proof asks for still holds.
/// Why:
///   A removal that deletes what Git ignores has to be sure that nothing there is the person's,
///  and every answer here is a decision a person can read.
///  The checks run cheapest first,
///  and the two that make Git or the walk of a tree work are the last:
///  the registrations are a directory listing,
///  the lock and the marks are file looks,
///  and the process table is one walk of it.
///  The snapshot comes last but one,
///  before the removal may run,
///  because the records it copies are the only thing left of a push failure once the worktree's own Git
///  directory is deleted.
///  A worktree whose records could not all be copied stays,
///  and the reason names the Git directory they stay in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const why = whyTheWorktreeStays(look, worktree, admin);
/// ```
pub(crate) fn why_the_worktree_stays(look: &SecondLook<'_>, worktree: &Path, admin: &Path) -> Option<String> {
    // A worktree a command added inside this one is in the registrations only once that command has finished, so the
    // list any earlier reading answered with proves nothing about it: a removal of the outer one takes the nested
    // worktree's changed files and its own record with it (a cross-vendor review of the landed sweep, round 208).
    let registered: Vec<Registration> = match registrations_of(look.common_dir) {
        Ok(found) => found,
        Err(reason) => return Some(format!("nothing proves no other worktree lies inside this one, so nothing is removed: {reason}")),
    };
    let inside: Vec<&Registration> = worktrees_inside(worktree, registered.as_slice());
    if !inside.is_empty() {
        return Some(nested_worktree_reason(inside.as_slice()));
    }
    if admin.join("locked").exists() {
        return Some(String::from("it is locked, and a lock is a promise to keep it"));
    }
    for marker in IN_PROGRESS_MARKERS {
        if admin.join(marker).exists() {
            return Some(format!("a {marker} stands in it, so an operation there is unfinished"));
        }
    }
    if let Some(mark) = bisect_mark_in(admin) {
        return Some(format!("a bisect is unfinished there, and its {mark} is still in the worktree"));
    }
    // How long the worktree has existed, which is what tells a process that hides its working folder whether it could be
    // inside at all: a process older than the directory cannot have changed into it.
    match process_working_inside(worktree, admin, look.proc_root, age_of(worktree)) {
        ProcessWorking::Nobody => {}
        ProcessWorking::Someone(working) => return Some(working.why()),
        // A scan that did not finish proves nothing about the rest of the table, and a removal on it would take the
        //  folder a process it never reached is working in.
        ProcessWorking::Unfinished(why) => return Some(format!("{why}, so nothing is removed")),
    }
    // The unshown push failures and the unfinished pushes of this worktree live in its own Git directory, which the
    // removal deletes, so they are copied into a removal snapshot before anything is deleted. A record that cannot be
    // copied keeps the worktree, because the copy is the only thing left of it after the removal.
    let unsaved: Vec<String> = snapshot_one(look.common_dir, admin);
    if !unsaved.is_empty() {
        return Some(format!(
            "the unshown push failures and the unfinished pushes its own Git directory holds could not all be copied into a removal snapshot, so nothing is removed and they stay in {}: {}",
            admin.display(),
            unsaved.join(" ")
        ));
    }
    return None;
}

/// What:
///  The other worktrees this repository registers whose checkout lies inside `worktree`,
///  which a removal of it would take with them.
/// Why:
///  A worktree can be added from inside another one,
/// and this repository's own agents do it:
///  the nested checkout is a directory of the outer worktree,
///  usually under an ignored name.
///  Removing the outer one took the nested worktree's files,
///  a modified tracked file and an untracked file among them,
///  and left its registration naming a path that is gone,
///  so `git worktree prune` could then delete the nested worktree's own record,
///  its reflog and the commits held in it
///  (a cross-vendor audit of `574063773d`,
///  finding F2).
///  The paths compared are the canonical ones Git's own registration names.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// worktreesInside(worktree, registered) // the registrations whose worktree starts with worktree
/// ```
pub(crate) fn worktrees_inside<'a>(worktree: &Path, registered: &'a [Registration]) -> Vec<&'a Registration> {
    let mut inside: Vec<&Registration> = Vec::new();
    for other in registered {
        if other.worktree != worktree && other.worktree.starts_with(worktree) {
            inside.push(other);
        }
    }
    return inside;
}

/// What:
///  The paths of the worktrees found inside,
/// as the person reads them,
///  a few named and the rest counted.
/// Why:
///  A person decides about their own worktrees,
///  so the ones that keep this one are named,
///  and a list longer than a few would bury the sentence it sits in.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// namesOf(inside) // '`a` and 2 more'
/// ```
fn names_of(inside: &[&Registration]) -> String {
    const SHOWN: usize = 2;
    let mut parts: Vec<String> = Vec::new();
    for other in inside.iter().take(SHOWN) {
        parts.push(format!("`{}`", other.worktree.display()));
    }
    let rest: usize = inside.len().saturating_sub(parts.len());
    if rest > 0 {
        parts.push(format!("and {rest} more"));
    }
    return parts.join(", ");
}

/// What:
///  Why a worktree that holds another registered worktree stays.
/// Why:
///  The two readings that keep it,
///  the one over the registrations a sweep read at its start and the one over the registrations read as the removal
///  runs,
///  answer a person in the same words,
///  so the sentence is written once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// nestedWorktreeReason(inside) // the sentence a kept parent prints
/// ```
pub(crate) fn nested_worktree_reason(inside: &[&Registration]) -> String {
    return format!(
        "another worktree this repository registers is inside it ({}), and a removal of this one would take that worktree's files and its own record with it",
        names_of(inside)
    );
}

/// What:
///  The name of a file an unfinished bisect left in `admin`,
///  or nothing when no bisect is in progress there.
/// Why:
///   The file is Git's own record that a bisect is unfinished,
///  and its name is what tells a person where Git kept the bisect,
///  while a question about `BISECT_LOG` alone answers nothing when the kill came before that file was opened.
///  The smallest name is taken,
/// not the one the filesystem listed first,
///  so the reason reads the same every time.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// bisectMarkIn(admin) // 'BISECT_NAMES'
/// ```
fn bisect_mark_in(admin: &Path) -> Option<String> {
    let mut marks: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(admin).ok()?.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if name.starts_with(BISECT_MARK_PREFIX) {
            marks.push(name);
        }
    }
    marks.sort();
    return marks.into_iter().next();
}

/// What:
///  How long a directory has existed,
///  as the filesystem records when it was made.
/// Why:
///   A process that hides its working folder can only be working inside a directory that existed when it started,
///  so the worktree's age is what separates the services that start at login from a process this worktree's own work began.
///  A filesystem that records no birth time says nothing,
///  and the hidden process then keeps the worktree.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ageOf(directory) // Duration | undefined
/// ```
#[cfg(target_os = "linux")]
pub(crate) fn age_of(directory: &Path) -> Option<Duration> {
    use std::os::unix::ffi::OsStrExt as _;
    let name: std::ffi::CString = std::ffi::CString::new(directory.as_os_str().as_bytes()).ok()?;
    // SAFETY: an all-zero `statx` is a valid value for the call to fill.
    let mut found: libc::statx = unsafe { std::mem::zeroed() };
    // SAFETY: `name` outlives the call, `found` is a live `statx`, and `AT_FDCWD` with an absolute path names the directory.
    let called: libc::c_int = unsafe { libc::statx(libc::AT_FDCWD, name.as_ptr(), 0, libc::STATX_BTIME, &mut found) };
    if called != 0 || found.stx_mask & libc::STATX_BTIME == 0 {
        return None;
    }
    let seconds: u64 = u64::try_from(found.stx_btime.tv_sec).ok()?;
    let made: SystemTime = SystemTime::UNIX_EPOCH + Duration::new(seconds, found.stx_btime.tv_nsec);
    return SystemTime::now().duration_since(made).ok();
}

// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
/// No birth time to read on a platform without `statx`.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// ageOf(directory) // undefined
/// ```
#[cfg(not(target_os = "linux"))]
pub(crate) fn age_of(_directory: &Path) -> Option<Duration> {
    return None;
}

/// Controls of the proof made again stay out of the release executable.
#[cfg(test)]
#[path = "worktree_removal_second_look_tests.rs"]
mod tests;
