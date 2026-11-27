//! What:
//!  The publication of one staged regular file at its destination name in one step:
//!  its bytes are written to a file the filesystem has given no name,
//!  the file takes the entry's exact mode,
//!  and one link gives it the destination name only once it holds every byte.
//! Why:
//!  A file created under its final name and filled afterwards exists,
//! for as long as the copy runs,
//!  with a prefix of the bytes and the mode the process's umask chose,
//! so a kill in that window leaves a file at that name that is not what the source holds,
//!  a credential at a mode wider than its source,
//! and a resumed copy refuses the destination and erases its own record.
//!  A file with no name cannot leave either:
//!  the kernel gives its blocks back when the last descriptor closes,
//! so a kill leaves the destination as it was,
//! and the resumed copy installs the entry.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const unnamed = await openUnnamed(directory, mode); // O_TMPFILE, or a private name renamed into place
//! await writeBytes(stagePath, unnamed);
//! await unnamed.chmod(mode);
//! await linkUnnamed(unnamed, destinationPath); // fails when anything is there
//! ```

use crate::worktree_copy_error::{
    WorktreeCopyError,
    failure_with_cause,
    raw_failure,
    raw_failure_of,
};
use crate::worktree_copy_publish_file_giving_it_its_name::{
    link_unnamed,
    move_temporary_into_place,
    remove_present,
};
use crate::worktree_copy_publish_file_making_the_file::{
    UNNAMED_UNSUPPORTED_CODES,
    create_named,
    is_unsupported,
    last_name,
    open_directory,
    open_unnamed,
};
/// The stable name a fallback's temporary is filled under,
/// re-exported so the one caller that names it keeps this module's path.
pub use crate::worktree_copy_publish_file_temporary_name::temporary_name_of;
use std::ffi::{
    OsStr,
    OsString,
};
use std::io::Write;
use std::path::{
    Path,
    PathBuf,
};

/// What:
///  Publish `stage_path`'s bytes at `destination_path` with `mode`,
/// through a file that never holds a name until it is whole.
/// Why:
///   Neither the bytes nor the mode are ever readable at the destination's name before both are complete,
/// so a
///  credential is never at a wider mode than its source,
/// and a kill leaves nothing at that name for a resumed copy to refuse.
///  `temporary` names the file the fallback fills where the filesystem has no unnamed files;
/// it is this copy's own name for this entry.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function publishCopyOfFile({ stagePath, destinationPath, mode, temporary }): Promise<void>;
/// ```
pub fn publish_copy_of_file(
    stage_path: &Path,
    destination_path: &Path,
    mode: u32,
    temporary: &OsStr,
) -> Result<(), WorktreeCopyError> {
    return publish_copy_of_file_with(stage_path, destination_path, mode, temporary, open_unnamed);
}

/// What:
///  `publish_copy_of_file` with the request that makes the file being filled,
/// so a control can make the filesystem refuse an unnamed one and see both paths on every machine.
/// Why:
///   Whether a filesystem has unnamed files is its choice,
/// and the fallback must be exercised where the
///  preferred path exists too.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// async function publishCopyOfFileWith({ stagePath, destinationPath, mode, temporary, unnamed }): Promise<void>;
/// ```
pub fn publish_copy_of_file_with(
    stage_path: &Path,
    destination_path: &Path,
    mode: u32,
    temporary: &OsStr,
    unnamed: fn(&std::fs::File, u32) -> std::io::Result<std::fs::File>,
) -> Result<(), WorktreeCopyError> {
    let parent: PathBuf = match destination_path.parent() {
        Some(found) => found.to_path_buf(),
        None => {
            return Err(failure_with_cause(
                format!(
                    "monochromatic-git-policy-cli: {} names no directory to install into.",
                    crate::json_record::quote(destination_path.to_string_lossy().as_ref())
                )
                .as_str(),
                None,
            ));
        }
    };
    let directory: std::fs::File = open_directory(parent.as_path())?;
    let destination_name: OsString = last_name(destination_path)?.to_os_string();
    // `match` on the unnamed file: where the filesystem has none, the entry is filled under this copy's own name.
    let (mut file, named): (std::fs::File, bool) = match unnamed(&directory, mode) {
        Ok(held) => (held, false),
        Err(error) if is_unsupported(&error, UNNAMED_UNSUPPORTED_CODES) => {
            remove_present(&directory, temporary).map_err(raw_failure_of)?;
            (
                create_named(&directory, temporary, mode).map_err(raw_failure_of)?,
                true,
            )
        }
        Err(error) => return Err(raw_failure(&error)),
    };
    let written: Result<(), WorktreeCopyError> = fill_and_publish(Filling {
        stage_path,
        file: &mut file,
        mode,
        directory: &directory,
        destination_path,
        destination_name: destination_name.as_os_str(),
        temporary,
        named,
    });
    if let Err(error) = written {
        // A file that was not published is not the destination's, so the name it may hold goes with it.
        if named {
            let _removed: std::io::Result<()> = remove_present(&directory, temporary);
        }
        return Err(error);
    }
    return Ok(());
}

/// What:
///  Everything one fill needs to end with the file published,
/// or the failure that stopped it.
/// Why:
///   The steps read the same five values,
/// and a control can read them in one place.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Filling = { stagePath; file; mode; directory; destinationPath; destinationName; temporary; named };
/// ```
struct Filling<'a> {
    /// The staged file whose bytes are copied.
    stage_path: &'a Path,
    /// The file being filled.
    file: &'a mut std::fs::File,
    /// The entry's permission bits.
    mode: u32,
    /// The destination's directory,
    /// held open.
    directory: &'a std::fs::File,
    /// The destination the file is published at.
    destination_path: &'a Path,
    /// The destination's one name.
    destination_name: &'a OsStr,
    /// The private name the fallback fills.
    temporary: &'a OsStr,
    /// Whether the file being filled holds a name.
    named: bool,
}

/// What:
///  Fill the file,
/// give it its mode,
/// and publish it,
/// stopping at the first failure.
/// Why:
///   Each step must happen on the same object in this order,
/// and the caller removes a name the fallback made when one of them fails.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function fillAndPublish(filling): void { fillFromStage(...); filling.file.chmod(...); publish(...); }
/// ```
fn fill_and_publish(filling: Filling<'_>) -> Result<(), WorktreeCopyError> {
    fill_from_stage(filling.stage_path, filling.file)?;
    filling
        .file
        .set_permissions(std::os::unix::fs::PermissionsExt::from_mode(filling.mode))
        .map_err(raw_failure_of)?;
    return publish(
        filling.directory,
        filling.file,
        filling.destination_path,
        filling.destination_name,
        filling.temporary,
        filling.named,
    );
}

/// What:
///  Copy every byte of the staged file into the file being filled.
/// Why:
///   The bytes are the staged entry's own,
/// so the destination holds what was compared before the install.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// await pipeline(createReadStream(stagePath), unnamed);
/// ```
fn fill_from_stage(
    stage_path: &Path,
    output: &mut std::fs::File,
) -> Result<(), WorktreeCopyError> {
    let mut input: std::fs::File = std::fs::File::open(stage_path).map_err(raw_failure_of)?;
    std::io::copy(&mut input, output).map_err(raw_failure_of)?;
    output.flush().map_err(raw_failure_of)?;
    return Ok(());
}

/// What:
///  Give the filled file the destination name,
/// by the link an unnamed file takes or the rename a named one takes.
/// Why:
///   Both are one step and neither replaces an entry that is already there.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// named ? renameNoReplace(temporary, destination) : linkUnnamed(unnamed, destination);
/// ```
fn publish(
    directory: &std::fs::File,
    file: &std::fs::File,
    destination_path: &Path,
    destination_name: &OsStr,
    temporary: &OsStr,
    named: bool,
) -> Result<(), WorktreeCopyError> {
    if named {
        move_temporary_into_place(directory, temporary, destination_name)
            .map_err(raw_failure_of)?;
        return Ok(());
    }
    link_unnamed(file, destination_path).map_err(raw_failure_of)?;
    return Ok(());
}

/// Publication controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_copy_publish_file_tests.rs"]
mod tests;
