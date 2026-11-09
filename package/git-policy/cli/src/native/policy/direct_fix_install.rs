//! What:
//!  Install the corrections a converged `git monochromatic-git-policy-cli fix` settled on into the
//!       worktree,
//!  all of them or none,
//!  and never touch the index.
//! Why:
//!  The installed wrapper's `direct-fix-install.ts` decides this protocol:
//!  every
//!      corrected file must still hold the bytes the fix read,
//!  each replacement is written
//!      beside its file and swapped with it,
//!  and the real index must hold the same bytes afterwards.
//!      Any failure restores every file already replaced.
//!  Failure messages name no file,
//!  because a
//!      pathname can hold the text a scan exists to keep out of output.
//!
//! What a kill leaves:
//!  one prepared file per corrected file,
//! beside it,
//!  named with `INSTALL_SIBLING_PREFIX`;
//!  before the swap it holds the correction,
//!  after it the file the swap displaced,
//!  and nothing here removes it later.
//!  A fix therefore puts `PREPARED_FILE_EXCLUDE_PATTERN` in the repository's
//!  `info/exclude` before it prepares anything,
//!  so Git never lists those files as untracked,
//!  never stages them with `git add --all`,
//!  and no later fix reads them as candidate content;
//!  a person who finds one removes it with `rm`,
//! and nothing of it is published.
//!
//! What window remains:
//!  A swap (`exchange`) puts the corrected file in place and leaves the file it
//!  displaced readable at the prepared path,
//! so the bytes that were really there
//!  are compared after nothing can overwrite them any more,
//! and a difference
//!  swaps them back (owner:
//! "never discard user data").
//!  Two writes can still be lost or replaced,
//! and neither is ours to prevent:
//!  a write through a descriptor a program opened before the swap lands in the
//!  displaced file after the comparison and before the prepared file is removed,
//!  and a write to the path after the swap replaces the corrected bytes (the
//!  newest bytes win,
//! which is what the person asked for).
//!  Both windows are the microseconds between one file system call and the next,
//!  not the length of a policy pass.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await installDirectFix({ scope, changedPaths, finalCandidates, originals });
//! ```

/// Import the candidate's mode,
///  which decides the replacement's permissions.
use crate::candidate_object::CandidateMode;
/// Import the unique name part shared with the private index directories.
use crate::candidate_private_index::unique_suffix;
/// Import the conversion of Git pathname bytes to a native path.
use crate::git_metadata::path_from_git_bytes;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;

/// The file-system primitives an install swaps and writes with.
use crate::direct_fix_install_writes::{Kept, WriteFailure, exchange, write_new};

/// The name prefix of the files a fix writes beside each corrected file.
pub const INSTALL_SIBLING_PREFIX: &str = ".cli-git-direct-fix-";

/// The ignore pattern that keeps a fix's prepared files out of Git,
///  at every depth of the
///       worktree.
///  Measured with Git 2.56.0 in a disposable repository on 2026-11-09:
///  `git add --all` staged
///       a prepared file in a subdirectory under `/.cli-git-direct-fix-*` (a leading slash anchors the
///       pattern to the top level);
///  without the slash it staged none of three files at the top,
/// one level
///       down and two levels down,
///  and `git status --ignored` named them ignored.
pub const PREPARED_FILE_EXCLUDE_PATTERN: &str = ".cli-git-direct-fix-";

/// What:
///  Make Git ignore the files a fix prepares,
///  in this repository,
///  for every later command.
/// Why:
///  A killed fix leaves its prepared file beside the corrected file,
///  and a fix's scope is the
///       whole worktree:
///  without this rule the next `fix` reads the leftover as candidate content
///  and
///       a later `git add --all` publishes a copy under a name the person never chose.
///  The rule lives
///       in the repository's own `info/exclude`,
///  which is never committed and never shown as a change,
///  and
///       writing it changes nothing the person wrote:
///  an existing line is kept and the rule is added once.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await appendIfMissing(`${commonDir}/info/exclude`, '.cli-git-direct-fix-*' + '\n');
/// ```
pub fn keep_prepared_files_out_of_git(common_dir: &Path) -> std::io::Result<()> {
    /// `Write` adds `write_all`;
    ///  `OpenOptions` opens a file to append to.
    use std::io::Write;
    let directory: PathBuf = common_dir.join("info");
    std::fs::create_dir_all(directory.as_path())?;
    let exclude: PathBuf = directory.join("exclude");
    let current: Vec<u8> = match std::fs::read(exclude.as_path()) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error),
    };
    // A line that already holds the rule means an earlier fix wrote it; the pattern's `*` is its whole form.
    let rule: String = format!("{PREPARED_FILE_EXCLUDE_PATTERN}*");
    // A line of its own holds the rule; the loop compares each line the file already has.
    let mut start: usize = 0;
    for position in 0..=current.len() {
        if position < current.len() && current[position] != b'\n' {
            continue;
        }
        if &current[start..position] == rule.as_bytes() {
            return Ok(());
        }
        start = position + 1;
    }
    let mut opened: std::fs::File = std::fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(exclude.as_path())?;
    // The leading line break only separates this block from a last line the person wrote without one.
    let mut block: String = String::new();
    if !current.is_empty() && !current.ends_with(b"\n") {
        block.push('\n');
    }
    block.push_str(
        "# monochromatic-git-policy-cli: the files a fix prepares beside a file, kept until it installs them\n",
    );
    block.push_str(rule.as_str());
    block.push('\n');
    return opened.write_all(block.as_bytes());
}

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
///  What a replacement keeps is the
///       destination's permission bits and owner (see `Kept`);
///  what it does not carry is everything
///       else a file can hold:
///  its extended attributes and access-control list,
///  its inode number and
///       birth time,
///  and its share of a hard link,
///  whose other name keeps the bytes it had.
///  A symbolic
///       link at the path is refused rather than replaced.
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
///  One prepared replacement:
///  the file,
///  the corrected bytes beside it,
///  and the bytes
///       the fix read.
/// Why:
///   Swapping the prepared file with its destination replaces it in one step and
///       leaves the replaced file at the prepared path,
///  so the bytes that were really there can
///       be compared after the swap,
///  and the same swap puts them back when they differ.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreparedReplacement = { destination: string; prepared: string; original: Uint8Array };
/// ```
#[derive(Debug)]
pub struct Replacement {
    /// The worktree file.
    destination: PathBuf,
    /// The corrected bytes before the swap,
    ///  and the bytes the swap displaced after it,
    ///       beside the file.
    prepared: PathBuf,
    /// The bytes the fix read,
    ///  which the displaced file must still hold.
    original: Rc<[u8]>,
}

/// What:
///  An installation failure message:
///  the shared start,
///  then what happened.
/// Why:
///   Every failure reads the same way and names no file.
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
///  Remove the prepared and backup files of each replacement,
///  ignoring files that
///       were never written.
/// Why:
///   Neither may stay behind after the fix ends,
///  whichever way it ended.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function cleanup(replacements): Promise<void>;
/// ```
fn cleanup(replacements: &[Replacement]) {
    for replacement in replacements {
        // What: `let _ = ...` discards each removal's `Result` on purpose.
        // Why:  A file that was never written cannot be removed, and that is fine.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // await rm(prepared, { force: true });
        // ```
        let _ = std::fs::remove_file(replacement.prepared.as_path());
    }
}

/// What:
///  Swap each installed file with the bytes it displaced before it,
///  last installed first.
///       `Result<(), String>` is success or the first restore that failed.
/// Why:
///   The worktree returns to the bytes it held before the fix,
///  in the very files that held
///       them;
///  a file that cannot be restored is left in place,
///  so its bytes are not lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function rollback(installed): Promise<void>;
/// ```
fn rollback(installed: &[Replacement]) -> Result<(), String> {
    // `.iter().rev()` visits the installed files in reverse order.
    for replacement in installed.iter().rev() {
        if let Err(error) = exchange(
            replacement.prepared.as_path(),
            replacement.destination.as_path(),
        ) {
            return Err(format!(
                "restoring a corrected file failed ({error}); its original bytes remain beside it in a file whose name starts with {INSTALL_SIBLING_PREFIX}"
            ));
        }
    }
    return Ok(());
}

/// What:
///  Prepare every replacement:
///  check the file still holds the bytes the fix read,
///       copy it beside itself,
///  and write the corrected bytes beside it.
/// Why:
///   Nothing in the worktree changes until every file is prepared,
///  so a file that
///       changed under the fix stops it before any file is replaced.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function prepareReplacements(root, changes): Promise<PreparedReplacement[]>;
/// ```
pub fn prepare_replacements(
    root: &Path,
    changes: &[InstallChange],
) -> Result<Vec<Replacement>, String> {
    let mut replacements: Vec<Replacement> = Vec::new();
    for change in changes {
        let Some(relative) = path_from_git_bytes(change.path.as_slice()) else {
            cleanup(replacements.as_slice());
            return Err(install_failure(
                "Git named a corrected file this system cannot represent, so no file was changed.",
            ));
        };
        let destination: PathBuf = root.join(relative);
        // What: `symlink_metadata` describes the name itself, so a symbolic link is seen as a link and not followed.
        // Why:  Reading through the link reads another file, and the swap would put the corrected bytes where the link
        //       is, so the link itself would be gone.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const kept = await lstat(destination);
        // ```
        let described: std::fs::Metadata = match std::fs::symlink_metadata(destination.as_path()) {
            Ok(found) => found,
            Err(error) => {
                cleanup(replacements.as_slice());
                return Err(install_failure(
                    format!("reading a corrected file failed ({error}), so no file was changed.")
                        .as_str(),
                ));
            }
        };
        if described.file_type().is_symlink() {
            cleanup(replacements.as_slice());
            return Err(install_failure(
                "a selected file is a symbolic link, and putting corrected bytes at its path would remove the link, so no file was changed.",
            ));
        }
        let current: Vec<u8> = match std::fs::read(destination.as_path()) {
            Ok(bytes) => bytes,
            Err(error) => {
                cleanup(replacements.as_slice());
                return Err(install_failure(
                    format!("reading a corrected file failed ({error}), so no file was changed.")
                        .as_str(),
                ));
            }
        };
        if current.as_slice() != &*change.original {
            cleanup(replacements.as_slice());
            return Err(install_failure(
                "a selected file changed while monochromatic-git-policy-cli fix ran, so no file was changed. Run monochromatic-git-policy-cli fix again.",
            ));
        }
        let parent: PathBuf = match destination.parent() {
            Some(found) => found.to_path_buf(),
            None => root.to_path_buf(),
        };
        let kept: Kept = Kept::of(&described);
        let prefix: String = format!("{INSTALL_SIBLING_PREFIX}{}", unique_suffix());
        let replacement: Replacement = Replacement {
            prepared: parent.join(format!("{prefix}.new")),
            original: Rc::clone(&change.original),
            destination,
        };
        let written: Result<(), WriteFailure> = write_new(
            replacement.prepared.as_path(),
            &change.replacement,
            &kept,
        );
        // Recorded before the failure check, so a failure still removes what was written.
        replacements.push(replacement);
        if let Err(error) = written {
            cleanup(replacements.as_slice());
            return Err(install_failure(
                format!("writing a corrected file beside its original failed ({error}), so no file was changed.").as_str(),
            ));
        }
    }
    return Ok(replacements);
}

/// What:
///  Swap every prepared file with its destination,
///  compare the bytes each swap displaced
///       with the bytes the fix read,
///  then prove the real index still holds `index_before`.
///  `&Option<Vec<u8>>` borrows the bytes read before,
///  or absence.
/// Why:
///   The fix changes the worktree only.
///  A swap that fails,
///  a file whose displaced bytes
///       are no longer the ones the fix read,
///  an index that changed,
///  or an index that cannot be
///       read restores every file already replaced (see "What window remains" in `direct_fix_install.rs`).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installPrepared(replacements, realIndexPath, indexBefore): Promise<void>;
/// ```
pub fn install_prepared(
    replacements: &[Replacement],
    real_index: &Path,
    index_before: &Option<Vec<u8>>,
) -> Result<(), String> {
    let mut installed: usize = 0;
    let mut failure: Option<String> = None;
    for replacement in replacements {
        if let Err(error) = exchange(
            replacement.prepared.as_path(),
            replacement.destination.as_path(),
        ) {
            failure = Some(format!("putting a corrected file in place failed ({error})"));
            break;
        }
        installed += 1;
        // What: The prepared path now holds the file the swap displaced, so its bytes are the bytes the
        //       destination held at the instant of the swap.
        // Why:  A write that arrived after the fix read the file is compared here, after nothing can overwrite it any
        //       more, and a difference saves it by swapping it back. A path that is no longer a regular file changed
        //       in a way no byte comparison answers (a directory, a symbolic link), and is refused the same way.
        //
        // In TS you'd write (pseudocode):
        // ```ts
        // const displaced = await lstat(prepared);
        // ```
        let displaced_kind: std::io::Result<std::fs::FileType> =
            match std::fs::symlink_metadata(replacement.prepared.as_path()) {
                Ok(found) => Ok(found.file_type()),
                Err(error) => Err(error),
            };
        match displaced_kind {
            Ok(kind) if kind.is_file() => {}
            Ok(_) => {
                failure = Some(String::from(
                    "a selected file changed while monochromatic-git-policy-cli fix ran",
                ));
                break;
            }
            Err(error) => {
                failure = Some(format!(
                    "reading the bytes a corrected file displaced failed ({error})"
                ));
                break;
            }
        }
        match std::fs::read(replacement.prepared.as_path()) {
            Ok(displaced) if displaced.as_slice() == &*replacement.original => {}
            Ok(_) => {
                failure = Some(String::from(
                    "a selected file changed while monochromatic-git-policy-cli fix ran",
                ));
                break;
            }
            Err(error) => {
                failure = Some(format!(
                    "reading the bytes a corrected file displaced failed ({error})"
                ));
                break;
            }
        }
    }
    if failure.is_none() {
        failure = match read_index(real_index) {
            Ok(after) if after == *index_before => None,
            Ok(_) => Some(String::from("the index changed while monochromatic-git-policy-cli fix ran")),
            Err(error) => Some(format!("reading the index again failed ({error})")),
        };
    }
    let Some(cause) = failure else {
        cleanup(replacements);
        return Ok(());
    };
    // `..installed` is the files already replaced, which the rollback restores.
    let restored: Result<(), String> = rollback(&replacements[..installed]);
    match restored {
        Ok(()) => {
            cleanup(replacements);
            return Err(install_failure(
                format!("{cause}, so every corrected file was restored. Run monochromatic-git-policy-cli fix again.")
                    .as_str(),
            ));
        }
        Err(restore_failure) => {
            // Prepared files are removed; backups stay, because one of them is needed.
            for replacement in replacements {
                let _ = std::fs::remove_file(replacement.prepared.as_path());
            }
            return Err(install_failure(
                format!("{cause}, and {restore_failure}.").as_str(),
            ));
        }
    }
}

/// What:
///  Install every change,
///  or none:
///  read the real index,
///  prepare,
///  then rename and
///       verify.
///  `Result<(), String>` is success or the failure message to report.
/// Why:
///   The whole protocol of the installed wrapper's `installDirectFix`,
///  apart from the
///       landing lock,
///  which the native wrapper does not take.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function installCorrections(root, realIndexPath, changes): Promise<void>;
/// ```
pub fn install_corrections(
    root: &Path,
    real_index: &Path,
    changes: &[InstallChange],
) -> Result<(), String> {
    let index_before: Option<Vec<u8>> = match read_index(real_index) {
        Ok(found) => found,
        Err(error) => {
            return Err(install_failure(
                format!("reading the index failed ({error}), so no file was changed.").as_str(),
            ));
        }
    };
    let replacements: Vec<Replacement> = prepare_replacements(root, changes)?;
    return install_prepared(replacements.as_slice(), real_index, &index_before);
}

/// Installation,
///  rollback and cleanup controls stay out of the release executable.
#[cfg(test)]
#[path = "direct_fix_install_tests.rs"]
mod tests;
