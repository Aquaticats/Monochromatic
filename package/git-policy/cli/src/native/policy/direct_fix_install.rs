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
use crate::direct_fix_install_journal::{
    JournalFailure, content_of, open_journal, recover_interrupted_installs, remove,
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

/// The start of every installation failure message.
const INSTALL_FAILED: &str = "monochromatic-git-policy-cli fix could not install its corrections";

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
    journal: PathBuf,
    /// The bytes the fix read.
    original: Rc<[u8]>,
    /// The corrected bytes.
    replacement: Rc<[u8]>,
    /// The permission bits and owner the file had.
    kept: Kept,
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
///  An installation failure message:
///  the shared start,
///  then what happened.
/// Why:
///   Every failure reads the same way and names no candidate file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const installFailure = (detail: string) => `monochromatic-git-policy-cli fix could not install its corrections: ${detail}`;
/// ```
fn install_failure(detail: &str) -> String {
    return format!("{INSTALL_FAILED}: {detail}");
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
///  The situation of one prepared site,
///  read from a name.
/// Why:
///   A name can be gone,
///  no longer be a regular file,
///  or hold other bytes than the ones a caller expects;
///  each
///       answer is different for the caller.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type State = 'absent' | 'not-a-file' | { bytes: Uint8Array; kept: Kept };
/// ```
enum SiteState {
    /// Nothing is at the name.
    Absent,
    /// Something is there,
    ///  and it is not a regular file.
    NotAFile,
    /// A regular file,
    ///  with its bytes and what it carries.
    File {
        /// The bytes it holds.
        bytes: Vec<u8>,
        /// Its permission bits and owner.
        kept: Kept,
    },
}

/// What:
///  Read what is at `path` now,
///  without following a symbolic link.
///  `&str` names the step that failed,
///  for the person's message.
/// Why:
///   Every judgement of a site is about the file itself,
///  never a link that leads elsewhere.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const state = lstatAndRead(path); // 'absent' | 'not-a-file' | { bytes, kept }
/// ```
fn state_of(path: &Path, step: &str) -> Result<SiteState, String> {
    let described: std::fs::Metadata = match std::fs::symlink_metadata(path) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(SiteState::Absent),
        Err(error) => return Err(format!("{step} failed ({error})")),
    };
    if !described.file_type().is_file() {
        return Ok(SiteState::NotAFile);
    }
    let kept: Kept = Kept::of(&described);
    let bytes: Vec<u8> = match std::fs::read(path) {
        Ok(read) => read,
        Err(error) => return Err(format!("{step} failed ({error})")),
    };
    return Ok(SiteState::File { bytes, kept });
}

/// What:
///  Whether the bytes a swap displaced are still the bytes the fix read,
///  and carry the mode
///       and owner the file had.
///  `Ok(false)` is "the file changed while the fix ran".
/// Why:
///   The comparison is made after the swap,
///  when the displaced file can no longer be overwritten by
///       the swap itself;
///  a changed permission or owner is the same kind of change as changed bytes,
///  and
///       installing over it would hand the file a mode the person did not choose.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const unchanged = (site) => { const s = stateOf(contentOf(site)); return s.kind === 'file' && equal(s.bytes, site.original) && s.kept.mode === site.kept.mode && s.kept.uid === site.kept.uid && s.kept.gid === site.kept.gid; };
/// ```
fn displaced_is_unchanged(site: &Prepared) -> Result<bool, String> {
    let content: PathBuf = content_of(site.journal.as_path());
    let found: SiteState = state_of(content.as_path(), "reading what a corrected file displaced")?;
    let SiteState::File { bytes, kept } = found else {
        return Ok(false);
    };
    if kept.mode != site.kept.mode || kept.uid != site.kept.uid || kept.gid != site.kept.gid {
        return Ok(false);
    }
    return Ok(bytes.as_slice() == &*site.original);
}

/// What:
///  The sentence a journal that stays is reported with.
/// Why:
///   The person must be able to find the bytes,
///  and the message must not name the candidate's pathname,
///       which can hold the text a scan exists to keep out of output.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// `kept ${contentOf(journal)} ...`;
/// ```
fn kept_notice(site: &Prepared) -> String {
    return format!(
        "monochromatic-git-policy-cli kept {}, which holds the bytes of a file that was written while the fix ran; nothing there was removed, and the file itself holds what the other writer put there.",
        content_of(site.journal.as_path()).display()
    );
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
        Ok(true) => remove(site.journal.as_path()),
        Ok(false) => notices.push(kept_notice(site)),
        Err(detail) => {
            notices.push(format!("{detail}; {}", kept_notice(site)));
        }
    }
}

/// What:
///  Put back what a site displaced,
///  but only while the site still holds this fix's bytes.
///  `Ok(false)` is
///       "someone wrote to the file,
/// so nothing here is ours to undo any more".
/// Why:
///   A blind swap back would move a newer save out of the path and leave it in a journal this
///       command then removes;
///  comparing first means a newer save stays exactly where the person put it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// if (!holds(destination, site.replacement)) return false; exchange(contentOf(site), site.destination); removeJournal(site.journal); return true;
/// ```
fn undo_if_still_ours(site: &Prepared) -> Result<bool, String> {
    let ours: bool = match std::fs::read(site.destination.as_path()) {
        Ok(bytes) => bytes.as_slice() == &*site.replacement,
        Err(_gone) => false,
    };
    if !ours {
        return Ok(false);
    }
    exchange(
        content_of(site.journal.as_path()).as_path(),
        site.destination.as_path(),
    )
    .map_err(|error| { return format!("restoring a corrected file failed ({error})"); })?;
    remove(site.journal.as_path());
    return Ok(true);
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
        .map_err(|detail| { return install_failure(&format!("{detail}, so no file was changed.")); })?;
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
    .map_err(|failure| { return journal_refusal(&failure); })?;
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
///  The failure sentence of a journal that could not be made.
/// Why:
///   Both refusals are complete answers for the person,
///  and one of them is not an accident of this
///       machine but a property of the repository's layout.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const refusal = (failure) => installFailure(`${failure}, so no file was changed.`);
/// ```
fn journal_refusal(failure: &JournalFailure) -> String {
    return install_failure(format!("{failure}, so no file was changed.").as_str());
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
        match undo_if_still_ours(site) {
            Ok(true) => {}
            Ok(false) => notices.push(kept_notice(site)),
            Err(detail) => notices.push(format!("{detail}; {}", kept_notice(site))),
        }
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
    real_index: &Path,
    changes: &[InstallChange],
) -> Result<InstallOutcome, String> {
    let Some(git_dir) = real_index.parent() else {
        return Err(install_failure(
            "the index has no directory, so no journal can be kept beside it and no file was changed.",
        ));
    };
    let mut notices: Vec<String> = Vec::new();
    recover_interrupted_installs(git_dir, &mut notices);
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
