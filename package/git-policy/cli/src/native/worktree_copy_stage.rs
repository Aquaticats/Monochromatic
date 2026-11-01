//! What:
//!  The private, validated copy of one source's ignored state,
//!  staged in a mode-`0700` directory beside a destination worktree.
//! Why:
//!  The stage sits on the destination's filesystem, so a file clone stays local and the install is a rename or a link.
//!  The source is copied entry by entry and compared again at the end,
//!  so a source that changed during the copy never reaches the destination.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const snapshot = await stageIgnoredSnapshot({ sourceRoot, destinationRoot, registeredRoots, gitPath });
//! ```

use super::private_storage::create_private_directory;
use super::random_id::random_uuid;
use super::transaction_git::GitContext;
use super::worktree_copy_compare::assert_final_source_equivalence;
use super::worktree_copy_error::{WorktreeCopyError, raw_failure, wrap_raw};
use super::worktree_copy_manifest::{
    EntryKind, WorktreeCopyEntry, apply_entry_modes, collect_entry_manifest,
};
use super::worktree_copy_selection::{CopySelection, STAGE_PREFIX, filesystem_path, select_roots};
use crate::json_record::quote;
use std::path::{Path, PathBuf};

/// What:
///  The name of the payload directory inside a stage container.
/// Why:
///   The payload mirrors the source's repository paths, so the container can hold other private state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(stageContainer, 'payload')
/// ```
const PAYLOAD_NAME: &str = "payload";

/// What:
///  The validated private snapshot of one source, ready to install.
/// Why:
///   Installation needs the entries it will create, the roots they came from and the stage to copy from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type StagedWorktreeSnapshot = { entries; selectedRoots; sourceRoot; stageContainer; stageRoot };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StagedSnapshot {
    /// Parent-first manifest that was staged and verified.
    pub entries: Vec<WorktreeCopyEntry>,
    /// Git-selected ignored roots that the manifest walked.
    pub selected_roots: Vec<String>,
    /// Canonical source worktree.
    pub source_root: PathBuf,
    /// Private container holding the payload; removed after installation.
    pub stage_container: PathBuf,
    /// Payload root mirroring the source's repository paths.
    pub stage_root: PathBuf,
}

/// What:
///  Creates one private directory with mode `0700`, parents included.
/// Why:
///   Staged content is never readable by other users, even before installation.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { recursive: true, mode: 0o700 });
/// ```
#[cfg(unix)]
fn create_private_tree(path: &Path) -> Result<(), WorktreeCopyError> {
    use std::os::unix::fs::DirBuilderExt;
    return std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(path)
        .map_err(|error| return raw_failure(&error));
}

/// What:
///  Creates one directory tree on a platform without Unix modes.
/// Why:
///   Keeps the staging code identical on every platform; no mode is claimed here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await mkdir(path, { recursive: true, mode: 0o700 });
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn create_private_tree(path: &Path) -> Result<(), WorktreeCopyError> {
    return std::fs::create_dir_all(path).map_err(|error| raw_failure(&error));
}

/// What:
///  Asks the filesystem to clone the source's content into a new destination file, when it can.
/// Why:
///   A clone shares storage on copy-on-write filesystems; any refusal means the caller must copy bytes instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyFile(source, destination, constants.COPYFILE_EXCL | constants.COPYFILE_FICLONE)
/// ```
#[cfg(target_os = "linux")]
fn try_clone(source: &std::fs::File, destination: &std::fs::File) -> bool {
    use std::os::unix::io::AsRawFd;
    // SAFETY: both descriptors are open for the duration of the call, and FICLONE takes the source descriptor as its argument.
    let result: libc::c_int =
        unsafe { libc::ioctl(destination.as_raw_fd(), libc::FICLONE, source.as_raw_fd()) };
    return result == 0;
}

/// What:
///  Declines cloning on platforms where this lane has no clone request, so every copy takes the byte path.
/// Why:
///   The incumbent's fallback is a full copy, which is always correct.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// copyFile(source, destination, constants.COPYFILE_EXCL | constants.COPYFILE_FICLONE)
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(target_os = "linux"))]
fn try_clone(_source: &std::fs::File, _destination: &std::fs::File) -> bool {
    return false;
}

/// What:
///  Creates a destination file exclusively and fills it with the source's bytes,
///  cloning when the filesystem allows.
/// Why:
///   Exclusive creation never overwrites an existing staged file.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyFile(sourcePath, stagePath, COPYFILE_EXCL | COPYFILE_FICLONE);
/// ```
pub fn copy_file_exclusive(source: &Path, destination: &Path) -> Result<(), WorktreeCopyError> {
    return copy_file_exclusive_with(source, destination, try_clone);
}

/// What:
///  Creates a destination file exclusively and fills it with the source's bytes,
///  using the clone request the caller supplies.
/// Why:
///   Whether the filesystem clones is the machine's choice, so a test supplies a clone that accepts or refuses
///  and sees each path taken on every machine.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await copyFileWith({ sourcePath, stagePath, clone });
/// ```
fn copy_file_exclusive_with(
    source: &Path,
    destination: &Path,
    clone: fn(&std::fs::File, &std::fs::File) -> bool,
) -> Result<(), WorktreeCopyError> {
    let mut input: std::fs::File =
        std::fs::File::open(source).map_err(|error| return raw_failure(&error))?;
    let mut output: std::fs::File = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| return raw_failure(&error))?;
    // A successful clone already holds every byte, so only a refused clone copies them.
    if !clone(&input, &output) {
        std::io::copy(&mut input, &mut output).map_err(|error| return raw_failure(&error))?;
    }
    return Ok(());
}

/// What:
///  Copies one manifest's entries into the stage, parent first.
/// Why:
///   Each entry is recreated from the source's no-follow view, so no symbolic link is ever followed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function copyManifestEntries({ sourceRoot, stageRoot, entries }): Promise<void>;
/// ```
fn copy_manifest_entries(
    source_root: &Path,
    stage_root: &Path,
    entries: &[WorktreeCopyEntry],
) -> Result<(), WorktreeCopyError> {
    for entry in entries {
        let source_path: PathBuf = filesystem_path(source_root, entry.relative_path.as_str())?;
        let stage_path: PathBuf = filesystem_path(stage_root, entry.relative_path.as_str())?;
        // `match` on the kind: directories are created, files copied, and links recreated with their exact target.
        match entry.kind {
            EntryKind::Directory => {
                create_private_tree(stage_path.as_path())?;
            }
            EntryKind::File => {
                create_private_tree(parent_of(stage_path.as_path()).as_path())?;
                copy_file_exclusive(source_path.as_path(), stage_path.as_path())?;
            }
            EntryKind::Symlink => {
                create_private_tree(parent_of(stage_path.as_path()).as_path())?;
                let target: PathBuf = std::fs::read_link(source_path.as_path())
                    .map_err(|error| return raw_failure(&error))?;
                symlink_at(target.as_path(), stage_path.as_path())?;
            }
        }
    }
    return Ok(());
}

/// What:
///  The parent directory of a native path, which always exists for a validated path under a root.
/// Why:
///   Selected nested files can need parents the selection did not name, so each file creates its own.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// dirname(path)
/// ```
fn parent_of(path: &Path) -> PathBuf {
    return path
        .parent()
        .map_or_else(|| return path.to_path_buf(), Path::to_path_buf);
}

/// What:
///  Creates a symbolic link with the exact target text.
/// Why:
///   The target is never resolved, so a link to a missing or outside path is copied as-is.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await symlink(target, stagePath);
/// ```
#[cfg(unix)]
fn symlink_at(target: &Path, link: &Path) -> Result<(), WorktreeCopyError> {
    return std::os::unix::fs::symlink(target, link).map_err(|error| return raw_failure(&error));
}

/// What:
///  Creates a symbolic link on a platform where this lane does not stage links.
/// Why:
///   Links are a Unix feature in the copy contract; other platforms refuse them with the generic sentence.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await symlink(target, stagePath);
/// ```
// UNTESTED ON WINDOWS: no gate compiles or runs this branch (the crate does not build for Windows; README, Requirements).
#[cfg(not(unix))]
fn symlink_at(_target: &Path, _link: &Path) -> Result<(), WorktreeCopyError> {
    return Err(raw_failure(&std::io::Error::other(
        "symbolic links are not staged on this platform",
    )));
}

/// What:
///  The registered roots strictly nested under the source, and the stage container, as exclusions.
/// Why:
///   A nested worktree or the stage itself must never be walked as ignored state.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// sourceExclusions({ sourceRoot, registeredRoots, stageContainer })
/// ```
fn source_exclusions(
    source_root: &Path,
    registered_roots: &[PathBuf],
    stage_container: &Path,
) -> Vec<PathBuf> {
    let mut exclusions: Vec<PathBuf> = registered_roots.to_vec();
    exclusions.push(stage_container.to_path_buf());
    return nested_under(source_root, exclusions.as_slice());
}

/// What:
///  The roots strictly inside the source root, never the source root itself.
/// Why:
///   A registered worktree nested in the source is another checkout's state, so the source's own walk must skip it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// roots.filter((root) => root !== sourceRoot && isInside(sourceRoot, root))
/// ```
fn nested_under(source_root: &Path, roots: &[PathBuf]) -> Vec<PathBuf> {
    return roots
        .iter()
        .filter(|root| return root.as_path() != source_root && root.starts_with(source_root))
        .cloned()
        .collect();
}

/// What:
///  Whether a repository path names a private stage at any component.
/// Why:
///   A private stage is never selected, even when Git lists it as ignored.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// repositoryPath.split('/').some((component) => component.startsWith(STAGE_PREFIX))
/// ```
fn names_private_stage(repository_path: &str) -> bool {
    return repository_path
        .split('/')
        .any(|component| return component.starts_with(STAGE_PREFIX));
}

/// What:
///  Names a fresh stage container beside a destination worktree, creating nothing.
/// Why:
///   The copy records the name durably before it creates the container, so an interrupted copy can be cleaned up.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// join(dirname(destinationRoot), `${STAGE_PREFIX}${randomUUID()}`)
/// ```
pub fn new_stage_container(destination_root: &Path) -> Result<PathBuf, WorktreeCopyError> {
    let destination_parent: &Path = destination_root.parent().unwrap_or(destination_root);
    let unique: String = random_uuid().map_err(|error| return raw_failure(&error.0))?;
    return Ok(destination_parent.join(format!("{STAGE_PREFIX}{unique}")));
}

/// What:
///  Stages every selected ignored entry of one source into a named stage container.
/// Why:
///   The stage is removed on any failure, so a failed copy leaves nothing behind for a later invocation to replay.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function stageIgnoredSnapshot({ sourceRoot, stageContainer, registeredRoots, gitPath }): Promise<StagedWorktreeSnapshot>;
/// ```
pub fn stage_ignored_snapshot(
    context: &GitContext,
    source_root: &Path,
    stage_container: &Path,
    registered_roots: &[PathBuf],
    selection: &CopySelection,
) -> Result<StagedSnapshot, WorktreeCopyError> {
    let nested: Vec<PathBuf> = nested_under(source_root, registered_roots);
    let git_selected: Vec<String> = select_roots(context, source_root, nested.as_slice(), selection)?;
    create_private_directory(stage_container).map_err(|error| return raw_failure(&error))?;
    let stage_root: PathBuf = stage_container.join(PAYLOAD_NAME);
    create_private_tree(stage_root.as_path())?;
    let staged: Result<StagedSnapshot, WorktreeCopyError> = stage_into(
        source_root,
        registered_roots,
        stage_container,
        stage_root.as_path(),
        git_selected,
    );
    // Any failure removes the whole container, so no partial stage survives the attempt.
    match staged {
        Ok(snapshot) => return Ok(snapshot),
        Err(error) => {
            // A failed removal replaces the copy failure, as the incumbent's cleanup throws its own error there.
            std::fs::remove_dir_all(stage_container)
                .map_err(|removal| return raw_failure(&removal))?;
            return Err(words_for_source(error, source_root));
        }
    }
}

/// What:
///  Words a raw failure of the staging steps with the source it was copying.
/// Why:
///   The user learns which worktree's ignored state could not be staged.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// new WorktreeCopyError(`cli-git: could not stage ignored state from ${JSON.stringify(sourceRoot)}.`, error)
/// ```
fn words_for_source(error: WorktreeCopyError, source_root: &Path) -> WorktreeCopyError {
    let message: String = format!(
        "cli-git: could not stage ignored state from {}.",
        quote(source_root.to_string_lossy().as_ref())
    );
    return wrap_raw(error, message.as_str());
}

/// What:
///  The copy steps that run once the private stage exists.
/// Why:
///   Separating them lets one cleanup cover every failure after the container is created.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// try { entries = await collectEntryManifest(...); await copyManifestEntries(...); await applyEntryModes(...); await assertFinalSourceEquivalence(...); } catch { await rm(stageContainer, ...); }
/// ```
fn stage_into(
    source_root: &Path,
    registered_roots: &[PathBuf],
    stage_container: &Path,
    stage_root: &Path,
    git_selected: Vec<String>,
) -> Result<StagedSnapshot, WorktreeCopyError> {
    let excluded_source_roots: Vec<PathBuf> =
        source_exclusions(source_root, registered_roots, stage_container);
    let selected_roots: Vec<String> = git_selected
        .into_iter()
        .filter(|repository_path| return !names_private_stage(repository_path.as_str()))
        .filter(|repository_path| {
            let source_path: PathBuf = match filesystem_path(source_root, repository_path.as_str())
            {
                Ok(path) => path,
                Err(_) => return false,
            };
            return !excluded_source_roots
                .iter()
                .any(|excluded| return source_path.starts_with(excluded.as_path()));
        })
        .collect();
    let entries: Vec<WorktreeCopyEntry> = collect_entry_manifest(
        source_root,
        selected_roots.as_slice(),
        excluded_source_roots.as_slice(),
    )?;
    copy_manifest_entries(source_root, stage_root, entries.as_slice())?;
    apply_entry_modes(stage_root, entries.as_slice())?;
    assert_final_source_equivalence(
        source_root,
        stage_root,
        selected_roots.as_slice(),
        excluded_source_roots.as_slice(),
        entries.as_slice(),
    )?;
    return Ok(StagedSnapshot {
        entries,
        selected_roots,
        source_root: source_root.to_path_buf(),
        stage_container: stage_container.to_path_buf(),
        stage_root: stage_root.to_path_buf(),
    });
}

/// Stage controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_stage_tests.rs"]
mod tests;
