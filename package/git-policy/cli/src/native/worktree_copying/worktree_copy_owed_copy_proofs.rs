//! What:
//!  What must still hold before the pass that finishes an owed copy touches the disk:
//!  the destination is still the registration the interrupted add made and its checkout is whole,
//!  the source is still a worktree of this repository,
//!  and anything standing at the stage's name is provably this copy's own.
//! Why:
//!  A copy that comes back to records a killed add left must never write into a worktree the person has since
//!  made into something else,
//! must never stage from a directory that is no longer a worktree,
//!  and must never remove a stage another program put at that name.
//! Each answer here is one of those three questions,
//! and the pass reads them all before it copies.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const binding = wholeRegistration(commonDir, destination); // undefined when nothing may be filled
//! const registered = sourceIsRegistered(roots, source);
//! const cleared = clearTheOwnStage(container, destination); // false when the container is not the copy's own
//! ```

use crate::worktree_add_binding::{
    Binding,
    bind_registration,
};
use crate::worktree_copy_error::WorktreeCopyError;
use crate::worktree_copy_interrupted_add_settlement::{
    add_is_finished,
    registration_directory,
};
use crate::worktree_copy_journal::{
    Role,
    assert_private_path,
};
use crate::worktree_copy_journal_encoding_and_validation::TEMPORARY_SUFFIX;
use crate::worktree_copy_journal_removal::remove_stage;
use crate::worktree_copy_stage_binding::stage_binding;
use crate::worktree_copy_stage_claim_reaping::SCRATCH_SUFFIX;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  The registration of a destination,
/// when its checkout is whole and it is still a worktree of this repository.
/// Why:
///   The copy fills a worktree;
/// a destination that is gone,
/// whose checkout is not whole,
/// or whose pointer no longer names a registration here has nothing this pass may write into,
/// and the
///  settlement names it instead.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const binding = wholeRegistration(commonDir, destination); // undefined when nothing may be filled
/// ```
pub(crate) fn whole_registration(
    common_dir: &Path,
    destination: &Path,
) -> Option<Binding> {
    let bound: Binding = bind_registration(common_dir, destination).ok()?.binding;
    // `match` on the pointer: the registration it names now is the one whose index the proof reads, and a destination whose
    // pointer cannot be read is proven against the registration this command bound.
    let admin: PathBuf = match registration_directory(destination) {
        Some(found) => found,
        None => bound.admin.clone(),
    };
    if !add_is_finished(destination, admin.as_path()) {
        return None;
    }
    return Some(bound);
}

/// What:
///  Whether the source a record names is a worktree of this repository now.
/// Why:
///   Staging reads the source's ignored files,
/// so a source that was removed,
/// moved or replaced is another worktree and never one this copy may take state from.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const registered = roots.some((root) => canonical(root) === canonical(source));
/// ```
pub(crate) fn source_is_registered(
    roots: &[PathBuf],
    source: &Path,
) -> bool {
    let wanted: PathBuf = match std::fs::canonicalize(source) {
        Ok(canonical) => canonical,
        Err(_gone) => return false,
    };
    // Git writes the roots as it sees them, so both sides are compared in their canonical form: a symbolic link above a
    // worktree must not hide a source the repository still registers.
    for root in roots {
        let Ok(found) = std::fs::canonicalize(root) else {
            continue;
        };
        if found.as_path() == wanted.as_path() {
            return true;
        }
    }
    return false;
}

/// What:
///  Removes the stage this copy itself left,
/// or says that nothing proves it is the copy's.
/// Why:
///   A stage killed while it was being filled holds part of the source's ignored state,
/// and the copy cannot fill that name again without removing it first;
/// only a container that records the registration this copy was made for proves the stage is the
///  copy's own,
/// so a directory another process put at that name is never removed.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// clearTheOwnStage(container, destination); // false when the container is not the copy's own
/// ```
pub(crate) fn clear_the_own_stage(
    container: &Path,
    destination: &Path,
) -> Result<bool, WorktreeCopyError> {
    let present: bool = match std::fs::symlink_metadata(container) {
        Ok(_) => true,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(true),
        Err(error) => return Err(crate::worktree_copy_error::raw_failure(&error)),
    };
    if !present {
        return Ok(true);
    }
    // A copy writes its binding into the container before its first payload entry, so a container that holds nothing but
    // the parts of that record is a copy killed while recording it; nothing of the source's state is there to lose, and
    // only the claim's random identifier names that container. A container that holds nothing at all is the step before
    // that write, and it holds nothing to lose either.
    let own: bool = match stage_binding(container)? {
        Some(binding) => binding.destination == destination,
        None => holds_nothing_a_copy_wrote(container),
    };
    if !own {
        return Ok(false);
    }
    assert_private_path(container, Role::PrivateStage)?;
    remove_stage(container)?;
    return Ok(true);
}

/// What:
///  Whether a container holds nothing this wrapper cannot account for:
/// no entry at all,
/// or nothing but the stage binding a copy writes before its first payload entry
/// and a part of the write that publishes it.
/// Why:
///   A copy creates its container and records which registration it fills as the step after it,
/// so a kill in between leaves an empty
///  container at a name only this wrapper's own private claim chose,
/// and nothing stands in it that can be lost:
/// the add it serves names the
///  same container,
/// so the empty folder is the wrapper's own step and not the person's work.
/// The write of that record publishes it by a rename,
/// so a kill at the next instruction leaves either the record's own temporary or the
///  file that temporary was being written in;
/// such a container holds no ignored file of the source,
/// while one that holds a payload,
///  a snapshot or anything else holds data unless its binding names this destination,
/// and is never removed.
/// Only the record's own names count,
/// so an ignored file of the source that merely begins with them keeps the container.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = readdir(container);
/// const bare = named.every((name) => namesTheBindingWrite(name));
/// ```
fn holds_nothing_a_copy_wrote(container: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(container) else {
        return false;
    };
    for entry in entries.flatten() {
        let name: String = entry.file_name().to_string_lossy().into_owned();
        if !names_the_binding_write(name.as_str()) {
            return false;
        }
    }
    return true;
}

/// What:
///  Whether one entry of a stage container is the binding record,
/// the temporary that publishes it,
/// or the file that temporary is written in.
/// Why:
///   Those three names are the only ones a kill between the container's creation and its first payload entry leaves,
/// so anything else is the source's ignored state or a directory another program put there,
/// and it keeps the container.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const written = (name) => name === RECORD || /^\.?RECORD\..*\.(tmp|partial)$/.test(name);
/// ```
fn names_the_binding_write(name: &str) -> bool {
    // The file a write is made in is hidden by a leading dot (`private_storage::staging_beside`), the temporary it is
    // published as is not.
    let bare: &str = name.strip_prefix('.').unwrap_or(name);
    let Some(rest): Option<&str> =
        bare.strip_prefix(crate::worktree_copy_stage_binding::STAGE_BINDING_NAME)
    else {
        return false;
    };
    if rest.is_empty() {
        return true;
    }
    return rest.starts_with('.')
        && (rest.ends_with(TEMPORARY_SUFFIX) || rest.ends_with(SCRATCH_SUFFIX));
}
