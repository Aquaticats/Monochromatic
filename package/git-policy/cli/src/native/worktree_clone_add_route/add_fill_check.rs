//! What:
//!  The check of a filled worktree:
//!  the pointer,
//!  its registration,
//!  the common directory and the index.
//! Why:
//!  "Mark the binding usable only after the fill is verified",
//!  and the verification is the facts every later reader relies on,
//!  so each is read from the file system rather than from the values the fill
//!  itself wrote
//!  (`doc/planning/cli-git-zfs-add-binding-record-design.md`,
//!  A12).
//!  `add_fill.rs` runs the fill in order and would go over the file-size limit
//!  carrying the check,
//!  and the check is the last thing the fill does,
//!  so a reader meets it after
//!  every write it judges.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! checkTheResult(input); // throws unless the worktree is the record's own
//! ```

use super::add_fill_input_and_result::FillInput;
use std::path::PathBuf;

/// What:
///  Check the filled worktree:
///  the pointer,
///  its registration,
///  the common directory,
///  the index and the head.
/// Why:
///  "Mark the binding usable only after the fill is verified",
///  and the verification is the facts every later reader relies on.
pub(super) fn check_the_result(input: &FillInput<'_>) -> Result<(), String> {
    let git: PathBuf = input.clone_root.join(".git");
    let metadata: std::fs::Metadata =
        std::fs::symlink_metadata(git.as_path()).map_err(|error| {
            return format!("the filled worktree's .git cannot be examined ({error})");
        })?;
    if !metadata.is_file() {
        return Err(format!(
            "the filled worktree {} holds a .git that is no pointer file, so the exchange did not land",
            input.clone_root.display()
        ));
    }
    // The pointer is read as a plain file and to a bound, so a name swapped for a named pipe after the look above is
    //  refused at once and the fill is reported as not landed (`add_fill_tests`).
    let pointer: Vec<u8> = crate::plain_file::read_plain_file(
        git.as_path(),
        crate::worktree_convert_inspection::GIT_FILE_LIMIT,
    )
    .map_err(|error| return format!("the filled worktree's .git cannot be read ({error})"))?;
    if pointer != input.pointer {
        return Err(String::from(
            "the filled worktree's .git is not the pointer this add saved, so the fill is not the worktree the record names",
        ));
    }
    let registration: PathBuf =
        crate::worktree_add_binding::pointer_target(input.clone_root, pointer.as_slice())?;
    if registration != input.admin_path {
        return Err(format!(
            "the filled worktree's pointer names {}, and this add's registration is {}",
            registration.display(),
            input.admin_path.display()
        ));
    }
    // The one reader of `commondir` (`foundation/common_directory_of_a_git_directory.rs`) opens the file as a plain one, so a
    //  named pipe there is a fill that cannot be verified rather than a wait.
    let commondir: Vec<u8> =
        crate::common_directory_of_a_git_directory::bytes_named(registration.as_path())
            .map_err(|error| {
                return format!("the registration's commondir cannot be read ({error})");
            })?
            .ok_or_else(|| {
                return String::from(
                    "the registration's commondir is not there, so the fill is not whole",
                );
            })?;
    let resolved: PathBuf = crate::worktree_add_binding::resolve_lexically(
        registration.as_path(),
        commondir
            .strip_suffix(b"\n")
            .unwrap_or(commondir.as_slice()),
    )
    .map_err(|()| {
        return String::from("the registration's commondir leaves the file system root");
    })?;
    if resolved != input.common_dir
        && std::fs::canonicalize(resolved.as_path()).ok()
            != std::fs::canonicalize(input.common_dir).ok()
    {
        return Err(format!(
            "the registration's commondir names {}, and this repository's common directory is {}",
            resolved.display(),
            input.common_dir.display()
        ));
    }
    if !input.admin_path.join("index").is_file() {
        return Err(String::from(
            "the filled worktree's registration holds no index, so the fill is not whole",
        ));
    }
    return Ok(());
}
