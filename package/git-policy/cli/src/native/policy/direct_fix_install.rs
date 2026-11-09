//! What:
//!  Install the corrections a converged `git monochromatic-git-policy-cli fix` settled on into the
//!       worktree,
//!  all of them or none that this command still owns,
//!  and never touch the index.
//! Why:
//!  Every corrected file is written into a journal inside the Git directory and then swapped with
//!  the file,
//!  so the bytes the swap displaced stay readable;
//!  they are compared with the bytes the fix read,
//!  and the very same swap puts them back when they differ.
//!  The real index must hold the same bytes
//!  afterwards.
//!  A file this command no longer owns - someone wrote to it after its swap - is never
//!  touched again:
//!  the journal stays and the person is told where those bytes are.
//!
//! What a kill leaves,
//! and who finishes it:
//!  a journal directory inside the Git directory,
//!  holding the corrected bytes (or,
//! after the swap,
//! the
//!  bytes the swap displaced) and a record of what the file was;
//!  the next `fix` finds it,
//!  and
//!  when the process that wrote it is proven gone it either removes it (the file was never touched)
//!  or swaps the displaced bytes back (the correction never verified its index),
//!  and anything it cannot
//!  prove safe it leaves and names (`direct_fix_install_journal.rs`).
//!  Nothing of a fix lives in the
//!  worktree,
//! so no `gitignore` rule can expose it and no `git add` can stage it.
//!
//! What window remains:
//!  a write through a descriptor opened before a swap lands in the displaced file,
//!  and a write to the
//!  path after the swap replaces the corrected bytes (the newest bytes win,
//! as the person asked).
//!  A
//!  descriptor write that arrives before the last reading of the displaced file is kept and named;
//!  one
//!  that arrives after that reading and before the journal is removed is lost,
//! which is the window
//!  every atomic replacement has (`mv` included),
//! and it is microseconds wide.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await installDirectFix({ scope, changedPaths, finalCandidates, originals });
//! ```

/// The journal a correction is installed through,
///  and the sweep that finishes what a kill interrupted.
use crate::direct_fix_install_journal::{content_of, open_journal, remove};
/// The judgement of what a name holds.
use crate::direct_fix_install_judging::{displaced_is_unchanged, holds_exactly, state_of, SiteState};
/// The sentences the install reports.
use crate::direct_fix_install_messages::{
    could_not_remove, install_failure, journal_refusal, kept_bytes_notice, kept_notice, unchanged_failure,
};
/// The two file-system primitives,
///  and what the file being replaced had.
use crate::direct_fix_install_writes::{Kept, exchange, write_new};
/// The mode a candidate carries,
///  which the installed wrapper's fix records beside each correction.
use crate::candidate_object::CandidateMode;
/// The conversion of Git pathname bytes to a native path.
use crate::git_metadata::path_from_git_bytes;
/// The proof that this process can be named as the journal's writer.
use crate::worktree_copy_owner::current_copy_owner;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;

/// What:
///  One file to correct:
///  its pathname,
///  mode,
///  the bytes the fix read and the bytes to
///       write.
///  `#[derive(...)]` generates cloning,
///  debug printing and `==`.
/// Why:
///   The read bytes prove nobody changed the file while the fix ran.
///  A replacement keeps the
///       destination's permission bits and owner (see `Kept`);
///  what it does not carry is everything else a
///       file can hold:
///  its extended attributes and access-control list,
///  its inode number and birth time,
///  and its
///       share of a hard link,
///  whose other name keeps the bytes it had.
///  A symbolic link at the path is refused rather
///       than replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallChange = { path: Uint8Array; mode: CandidateMode; original: Uint8Array; replacement: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstallChange {
    /// The pathname,
    ///  relative to the repository's top level,
    ///  as Git's raw bytes.
    pub path: Vec<u8>,
    /// The file's mode;
    ///  executable files stay executable.
    pub mode: CandidateMode,
    /// The bytes the fix read;
    ///  the worktree file must still hold them.
    pub original: Rc<[u8]>,
    /// The corrected bytes.
    pub replacement: Rc<[u8]>,
}

/// What:
///  One correction,
///  ready to swap:
///  the file,
///  the journal that holds its bytes,
///  and what the file
///       was when the fix read it.
/// Why:
///   The swap and every later judgement need only these,
///  and nothing here changes the worktree until the swap.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Prepared = { destination: string; journal: string; original: Uint8Array; replacement: Uint8Array; kept: Kept };
/// ```
#[derive(Debug)]
pub struct Prepared {
    /// The worktree file.
    destination: PathBuf,
    /// The journal directory that holds the corrected bytes,
    ///  and then the bytes the swap displaced.
    pub(crate) journal: PathBuf,
    /// The bytes the fix read.
    pub(crate) original: Rc<[u8]>,
    /// The corrected bytes.
    pub(crate) replacement: Rc<[u8]>,
    /// The permission bits and owner the file had.
    pub(crate) kept: Kept,
}

// What: A control's step between one site's swap and the next, so a control can act as another writer at
//       exactly the moment the review's sequences name.
// Why:  A write that arrives between two file-system calls cannot be provoked without a step there, and a control
//       that slept instead would be a race, not a control. Nothing outside the tests sets it.
//
// In TS you'd write (pseudocode):
// ```ts
// // if (test) afterSwap.push((destination) => { ... });
// ```
#[cfg(test)]
thread_local! {
    // The control's step, if one is running on this thread.
    pub(crate) static AFTER_SWAP: std::cell::RefCell<Option<Box<dyn FnMut(&Path)>>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Run the control's step after a site's swap,
///  when a control is running.
/// Why:
///   The production path has no step here,
///  and the two file-system calls stay adjacent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // afterSwap.forEach((step) => step(destination));
/// ```
fn after_swap(destination: &Path) {
    #[cfg(test)]
    {
        AFTER_SWAP.with(|hook| {
            let mut borrowed = hook.borrow_mut();
            if let Some(step) = borrowed.as_mut() {
                step(destination);
            }
        });
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(test))]
    {
        let _ = destination;
    }
}

// What: A control's step between a rollback's reading of the destination and its swap, which is the window the
//       round-three review names.
// Why:  A save that lands there cannot be provoked without a step in it; nothing outside the tests sets it.
//
// In TS you'd write (pseudocode):
// ```ts
// // if (test) beforeRollbackSwap.push((destination) => { ... });
// ```
#[cfg(test)]
thread_local! {
    // The control's step, if one is running on this thread.
    pub(crate) static BEFORE_ROLLBACK_SWAP: std::cell::RefCell<Option<Box<dyn FnMut(&Path)>>> = const { std::cell::RefCell::new(None) };
}

/// What:
///  Run the control's step between a rollback's reading of the destination and its swap.
/// Why:
///   The production path has no step here,
///  and the two file-system calls stay adjacent.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// // beforeRollbackSwap.forEach((step) => step(destination));
/// ```
fn before_rollback_swap(destination: &Path) {
    #[cfg(test)]
    {
        BEFORE_ROLLBACK_SWAP.with(|hook| {
            let mut borrowed = hook.borrow_mut();
            if let Some(step) = borrowed.as_mut() {
                step(destination);
            }
        });
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(test))]
    {
        let _ = destination;
    }
}

/// What:
///  What an install leaves for the person to look at.
///  `Vec<String>` is one sentence per site,
///  empty when every journal was removed.
/// Why:
///   A site someone wrote to after its swap keeps its journal,
///  and the person must be told where
///       those bytes are without the message naming a candidate's pathname.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type InstallOutcome = { notices: string[] };
/// ```
#[derive(Debug, Default, Eq, PartialEq)]
pub struct InstallOutcome {
    /// One sentence per journal that stays.
    pub notices: Vec<String>,
}

/// What:
///  The real index's bytes,
///  or nothing when it does not exist.
///       `std::io::Result<Option<Vec<u8>>>` is "bytes,
///  absent,
///  or a read failure".
/// Why:
///   A repository without an index file has an empty index,
///  and must still have none
///       after the fix.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function readIndex(path: string): Promise<Uint8Array | typeof INDEX_ABSENT>;
/// ```
pub fn read_index(path: &Path) -> std::io::Result<Option<Vec<u8>>> {
    match std::fs::read(path) {
        Ok(bytes) => return Ok(Some(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error),
    }
}

/// What:
///  The token that names this process in a journal,
///  or nothing where the platform cannot report it.
/// Why:
///   The sweep removes a journal only on a proof about its writer,
///  and a writer nothing can name is never
///       taken for dead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const token = currentCopyOwner()?.token();
/// ```
fn writer_token() -> Option<String> {
    return match current_copy_owner() {
        Ok(owner) => Some(owner.token()),
        Err(_unreported) => None,
    };
}

/// What:
///  Keep or remove the journal of a site whose swap succeeded and whose displaced bytes were the
///       bytes the fix read.
/// Why:
///   On success the displaced bytes are the file's own earlier bytes,
///  so removing them loses nothing
///       the fix must keep;
///  but a write that arrived through an already-open descriptor lands in that file,
///  so
///       it is read once more and kept and named when it holds anything else.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (await displacedIsUnchanged(site)) removeJournal(site.journal); else notices.push(keptNotice(site));
/// ```
fn finish_site(site: &Prepared, notices: &mut Vec<String>) {
    match displaced_is_unchanged(site) {
        Ok(true) => {
            if !remove(site.journal.as_path()) {
                notices.push(could_not_remove(site));
            }
        }
        Ok(false) => notices.push(kept_notice(site)),
        Err(detail) => {
            notices.push(format!("{detail}; {}", kept_bytes_notice(site)));
        }
    }
}

/// What:
///  Put back what a site displaced,
///  then judge the bytes the swap took out of the file:
///  the journal is removed
///       only when they are this fix's replacement,
///  and otherwise it stays and is named.
/// Why:
///  Reading the destination before the swap has a window:
///  a save that lands in it is moved into the
///       journal by the swap and then removed with it.
///  What comes out of the swap is the bytes that were really
///       there,
///  at a name this command owns,
///  so the comparison is about the swap and not about a reading
///       beside it.
///  A swap that fails leaves the journal for the person.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// exchange(contentOf(site.journal), site.destination); if (holdsExactly(contentOf(site.journal), site.replacement, site.kept)) removeOrName(site.journal); else notices.push(keptNotice(site));
/// ```
fn roll_back_site(site: &Prepared, notices: &mut Vec<String>) {
    let content: PathBuf = content_of(site.journal.as_path());
    // What: A control's step stands here, between the last reading of the destination and the swap, which is the
    //       window this rollback removes.
    // Why:  The swap is what takes the bytes out, so nothing this function decides reads the destination first.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // beforeRollbackSwap(site.destination); exchange(content, site.destination);
    // ```
    before_rollback_swap(site.destination.as_path());
    if let Err(error) = exchange(content.as_path(), site.destination.as_path()) {
        notices.push(format!(
            "restoring a corrected file failed ({error}); {}",
            kept_bytes_notice(site)
        ));
        return;
    }
    match holds_exactly(
        content.as_path(),
        &site.replacement,
        &site.kept,
        "reading what a rollback displaced",
    ) {
        Ok(true) => {
            if !remove(site.journal.as_path()) {
                notices.push(could_not_remove(site));
            }
        }
        Ok(false) => notices.push(kept_notice(site)),
        Err(detail) => notices.push(format!("{detail}; {}", kept_bytes_notice(site))),
    }
}

/// What:
///  Prepare every replacement:
///  check the file still holds the bytes the fix read,
///       record what the
///       file was,
///  and write the corrected bytes into the journal inside the Git directory.
/// Why:
///   Nothing in the worktree changes until every file is prepared,
///  and every change is recorded before
///       it is made,
///  so a file that changed under the fix stops it before any file is replaced and a
///       kill leaves a record of what to do.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareReplacements(root, gitDir, changes): Promise<Prepared[]>;
/// ```
pub fn prepare_replacements(
    root: &Path,
    git_dir: &Path,
    changes: &[InstallChange],
) -> Result<Vec<Prepared>, String> {
    let mut prepared: Vec<Prepared> = Vec::new();
    for change in changes {
        match prepare_one(root, git_dir, change) {
            Ok(site) => prepared.push(site),
            Err(message) => {
                for earlier in &prepared {
                    remove(earlier.journal.as_path());
                }
                return Err(message);
            }
        }
    }
    return Ok(prepared);
}

/// What:
///  `prepare_replacements` for one change.
/// Why:
///   The failure path of the whole preparation is one place,
///  so each refusal here just says what happened.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareOne(root, gitDir, change): Promise<Prepared>;
/// ```
fn prepare_one(root: &Path, git_dir: &Path, change: &InstallChange) -> Result<Prepared, String> {
    let Some(relative) = path_from_git_bytes(change.path.as_slice()) else {
        return Err(install_failure(
            "Git named a corrected file this system cannot represent, so no file was changed.",
        ));
    };
    let destination: PathBuf = root.join(relative);
    let found: SiteState = state_of(destination.as_path(), "reading a corrected file")
        .map_err(unchanged_failure)?;
    let SiteState::File { bytes, kept } = found else {
        if let SiteState::Absent = found {
            return Err(install_failure(
                "a selected file is gone, and putting corrected bytes at its path would create a file the fix never read, so no file was changed.",
            ));
        }
        // `symlink_metadata` describes the name itself, so a symbolic link is seen as a link and not followed; reading
        // through it would read another file, and the swap would put bytes where the link is.
        return Err(install_failure(
            "a selected file is not a regular file (a symbolic link, a directory or a device), and putting corrected bytes at its path would remove what is there, so no file was changed.",
        ));
    };
    if bytes.as_slice() != &*change.original {
        return Err(install_failure(
            "a selected file changed while monochromatic-git-policy-cli fix ran, so no file was changed. Run monochromatic-git-policy-cli fix again.",
        ));
    }
    let journal: PathBuf = open_journal(
        git_dir,
        destination.as_path(),
        &kept,
        &change.replacement,
        writer_token().as_deref(),
    )
    .map_err(journal_refusal)?;
    let content: PathBuf = content_of(journal.as_path());
    match write_new(content.as_path(), &change.replacement, &kept) {
        Ok(()) => {}
        Err(failure) => {
            remove(journal.as_path());
            return Err(install_failure(
                format!("writing a corrected file into its journal failed ({failure}), so no file was changed.")
                    .as_str(),
            ));
        }
    }
    return Ok(Prepared {
        destination,
        journal,
        original: Rc::clone(&change.original),
        replacement: Rc::clone(&change.replacement),
        kept,
    });
}

/// What:
///  Swap every prepared file with its destination,
///  judge what each swap displaced,
///  then prove
///       the real index still holds `index_before`.
///  `&Option<Vec<u8>>` borrows the bytes read before,
///  or absence;
///       `&mut Vec<String>` collects what stays for the person.
/// Why:
///   The fix changes the worktree only.
///  A swap that fails,
///  a file that changed under the fix,
///  an
///       index that changed,
///  or an index that cannot be read restores every file this command still
///       owns,
///  keeps whatever it does not,
///  and says so.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installPrepared(prepared, realIndexPath, indexBefore, notices): Promise<void>;
/// ```
pub fn install_prepared(
    prepared: &[Prepared],
    real_index: &Path,
    index_before: &Option<Vec<u8>>,
    notices: &mut Vec<String>,
) -> Result<(), String> {
    let mut swapped: usize = 0;
    let mut failure: Option<String> = None;
    for site in prepared {
        if let Err(error) = exchange(
            content_of(site.journal.as_path()).as_path(),
            site.destination.as_path(),
        ) {
            failure = Some(format!("putting a corrected file in place failed ({error})"));
            break;
        }
        swapped += 1;
        match displaced_is_unchanged(site) {
            Ok(true) => {}
            Ok(false) => {
                failure = Some(String::from(
                    "a selected file changed while monochromatic-git-policy-cli fix ran",
                ));
                break;
            }
            Err(detail) => {
                failure = Some(detail);
                break;
            }
        }
        after_swap(site.destination.as_path());
    }
    if failure.is_none() {
        failure = match read_index(real_index) {
            Ok(after) if after == *index_before => None,
            Ok(_) => Some(String::from(
                "the index changed while monochromatic-git-policy-cli fix ran",
            )),
            Err(error) => Some(format!("reading the index again failed ({error})")),
        };
    }
    let Some(cause) = failure else {
        for site in prepared {
            finish_site(site, notices);
        }
        return Ok(());
    };
    // What: Undo the sites this command still owns, last swapped first, and leave the rest untouched.
    // Why:  A site someone wrote to after its swap holds bytes that are newer than anything this command wrote, so
    //       swapping it back would move that write into a journal this command then removes.
    //
    // In TS you'd write (pseudocode):
    // ```ts
    // for (const site of swappedSites.reverse()) if (!await undoIfStillOurs(site)) notices.push(keptNotice(site));
    // ```
    for site in prepared[..swapped].iter().rev() {
        roll_back_site(site, notices);
    }
    // A site that was never swapped holds only the correction, so removing its journal loses nothing of the person's.
    for site in &prepared[swapped..] {
        remove(site.journal.as_path());
    }
    let kept: String = if notices.is_empty() {
        String::from("so every corrected file was restored")
    } else {
        format!(
            "so the corrected files this command still owned were restored, and what it does not own stays: {}",
            notices.join(" ")
        )
    };
    return Err(install_failure(
        format!("{cause}, {kept}. Run monochromatic-git-policy-cli fix again.").as_str(),
    ));
}

/// What:
///  Install every change,
///  or none that this command still owns:
///  finish what a killed fix left,
///  read the
///       real index,
///  prepare,
///  then swap and verify.
///  `Result<InstallOutcome, String>` is what stays
///       for the person,
///  or the failure message to report.
/// Why:
///   The whole protocol of the installed wrapper's `installDirectFix`,
///  apart from the
///       landing lock,
///  which the native wrapper does not take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installCorrections(root, realIndexPath, changes): Promise<InstallOutcome>;
/// ```
pub fn install_corrections(
    root: &Path,
    git_dir: &Path,
    real_index: &Path,
    changes: &[InstallChange],
) -> Result<InstallOutcome, String> {
    let mut notices: Vec<String> = Vec::new();
    let index_before: Option<Vec<u8>> = match read_index(real_index) {
        Ok(found) => found,
        Err(error) => {
            return Err(install_failure(
                format!("reading the index failed ({error}), so no file was changed.").as_str(),
            ));
        }
    };
    let prepared: Vec<Prepared> = prepare_replacements(root, git_dir, changes)?;
    install_prepared(prepared.as_slice(), real_index, &index_before, &mut notices)?;
    return Ok(InstallOutcome { notices });
}

/// Installation,
///  journal and recovery controls stay out of the release executable.
#[cfg(test)]
#[path = "direct_fix_install_tests.rs"]
mod tests;
