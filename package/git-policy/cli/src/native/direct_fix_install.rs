//! What:
//!  Install the corrections a converged `git cli-git fix` settled on into the
//!       worktree,
//!  all of them or none,
//!  and never touch the index.
//! Why:
//!  The installed wrapper's `direct-fix-install.ts` decides this protocol:
//!  every
//!      corrected file must still hold the bytes the fix read,
//!  each replacement is written
//!      beside its file and renamed over it,
//!  a copy of the old file is kept until the end,
//!      and the real index must hold the same bytes afterwards.
//!  Any failure restores every
//!      file already replaced.
//!  Failure messages name no file,
//!  because a pathname can hold
//!      the text a scan exists to keep out of output.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! // await installDirectFix({ scope, changedPaths, finalCandidates, originals });
//! ```

/// Import the candidate's mode,
///  which decides the replacement's permissions.
use super::candidate_object::CandidateMode;
/// Import the unique name part shared with the private index directories.
use super::candidate_private_index::unique_suffix;
/// Import the conversion of Git pathname bytes to a native path.
use super::git_metadata::path_from_git_bytes;
/// `Path`/`PathBuf` are borrowed/owned filesystem paths.
use std::path::{Path, PathBuf};
/// `Rc<T>` is a shared,
///  read-only handle.
use std::rc::Rc;

/// The name prefix of the files a fix writes beside each corrected file.
pub const INSTALL_SIBLING_PREFIX: &str = ".cli-git-direct-fix-";

/// The start of every installation failure message.
const INSTALL_FAILED: &str = "cli-git fix could not install its corrections";

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
///  the new bytes beside it,
///  and a copy of the
///       old file beside it.
/// Why:
///   Renaming the new file over the old one replaces it in one step,
///  and the copy
///       restores it if a later step fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type PreparedReplacement = { destination: string; prepared: string; backup: string };
/// ```
#[derive(Debug)]
pub struct Replacement {
    /// The worktree file.
    destination: PathBuf,
    /// The corrected bytes,
    ///  written beside it.
    prepared: PathBuf,
    /// A copy of the file as it was,
    ///  beside it.
    backup: PathBuf,
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
/// const installFailure = (detail: string) => `cli-git fix could not install its corrections: ${detail}`;
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
        let _ = std::fs::remove_file(replacement.backup.as_path());
    }
}

/// What:
///  Rename each installed file's backup over it,
///  last installed first.
///       `Result<(), String>` is success or the first restore that failed.
/// Why:
///   The worktree returns to the bytes it held before the fix.
///  A backup that cannot
///       be restored is left in place,
///  so its bytes are not lost.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function rollback(installed): Promise<void>;
/// ```
fn rollback(installed: &[Replacement]) -> Result<(), String> {
    // `.iter().rev()` visits the installed files in reverse order.
    for replacement in installed.iter().rev() {
        if let Err(error) = std::fs::rename(
            replacement.backup.as_path(),
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
///  Write `bytes` to a new file at `path` with permissions for `mode`.
/// Why:
///   The file must not exist yet,
///  so no other file is overwritten;
///  executable
///       files get `0755` and others `0644`,
///  before the umask,
///  as the installed wrapper
///       writes them.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await writeFile(path, bytes, { mode: mode === 'executable' ? 0o755 : 0o644, flag: 'wx' });
/// ```
fn write_new(path: &Path, bytes: &[u8], mode: CandidateMode) -> std::io::Result<()> {
    /// `Write` adds `write_all` to the open file.
    use std::io::Write;
    let mut options: std::fs::OpenOptions = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        /// `OpenOptionsExt` adds `mode`,
        ///  the permissions a new file is created with.
        use std::os::unix::fs::OpenOptionsExt;
        let permissions: u32 = if mode == CandidateMode::Executable {
            0o755
        } else {
            0o644
        };
        options.mode(permissions);
    }
    // UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
    #[cfg(not(unix))]
    {
        // Only Unix records an executable bit; the mode decides nothing elsewhere.
        let _ = mode;
    }
    return options.open(path)?.write_all(bytes);
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
                "a selected file changed while cli-git fix ran, so no file was changed. Run cli-git fix again.",
            ));
        }
        let parent: PathBuf = match destination.parent() {
            Some(found) => found.to_path_buf(),
            None => root.to_path_buf(),
        };
        let prefix: String = format!("{INSTALL_SIBLING_PREFIX}{}", unique_suffix());
        let replacement: Replacement = Replacement {
            prepared: parent.join(format!("{prefix}.new")),
            backup: parent.join(format!("{prefix}.old")),
            destination,
        };
        let written: std::io::Result<()> = match std::fs::copy(
            replacement.destination.as_path(),
            replacement.backup.as_path(),
        ) {
            Ok(_) => write_new(
                replacement.prepared.as_path(),
                &change.replacement,
                change.mode,
            ),
            Err(error) => Err(error),
        };
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
///  Rename every prepared file over its destination,
///  then prove the real index still
///       holds `index_before`.
///  `&Option<Vec<u8>>` borrows the bytes read before,
///  or absence.
/// Why:
///   The fix changes the worktree only.
///  A rename that fails,
///  an index that changed,
///       or an index that cannot be read restores every file already replaced.
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
        if let Err(error) = std::fs::rename(
            replacement.prepared.as_path(),
            replacement.destination.as_path(),
        ) {
            failure = Some(format!("replacing a corrected file failed ({error})"));
            break;
        }
        installed += 1;
    }
    if failure.is_none() {
        failure = match read_index(real_index) {
            Ok(after) if after == *index_before => None,
            Ok(_) => Some(String::from("the index changed while cli-git fix ran")),
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
                format!("{cause}, so every corrected file was restored. Run cli-git fix again.")
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
