//! What:
//!  Which registration of the invoking repository a worktree directory belongs to,
//! and whether Git would pick that same
//!  worktree for the name the person typed.
//! Why:
//!  Git removes only a worktree that its own list for the repository it runs in contains,
//! found first by the last
//!  components of every registered path and only then by the resolved path (`find_worktree` in Git's `worktree.c`).
//! The
//!  `.git` file of a worktree and the `gitdir` file of an administrative directory that name each other prove only that
//!  the pair is consistent,
//! not that the repository the command runs in registered it,
//! and a directory that exists under
//!  the typed name proves nothing about the worktree Git would remove for that name.
//! Anything but a registration of the
//!  invoking repository that Git would also pick is left to Git,
//! which words its own answer.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const registration = registrationOf({ commonDir, target, operand }); // throws the reason Git must decide
//! ```

use crate::worktree_path_failure_message::path_failure;
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

/// What:
///  The registration of one linked worktree in the repository the command runs in.
/// Why:
///   The administrative directory is where the registration lives;
/// its name is the worktree's id.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Registration = { id: string; admin: string; worktrees: string; gitdirText: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Registration {
    /// The name of the administrative directory below `worktrees`.
    pub id: OsString,
    /// The canonical administrative directory,
    /// `<common>/worktrees/<id>`.
    pub admin: PathBuf,
    /// The canonical `<common>/worktrees` directory.
    pub worktrees: PathBuf,
    /// The bytes of the `gitdir` file as they were proven to name the worktree,
    /// to compare again before the removal.
    pub gitdir_text: Vec<u8>,
}

/// The line of a file without its final newline.
fn without_newline(text: &[u8]) -> &[u8] {
    return text.strip_suffix(b"\n").unwrap_or(text);
}

/// What:
///  Whether Git's match by last components finds `candidate` for `operand`:
/// the operand is non-empty,
/// ends the path,
/// and
///  starts at a directory boundary.
/// ASCII case is ignored,
/// which also covers `core.ignorecase`,
/// so a match is never missed.
/// Why:
///   Git's `find_worktree_by_suffix`;
/// a looser match only sends more removals to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function endsWithName(candidate: string, operand: string): boolean;
/// ```
fn ends_with_name(candidate: &[u8], operand: &[u8]) -> bool {
    if operand.is_empty() || operand.len() > candidate.len() {
        return false;
    }
    let start: usize = candidate.len() - operand.len();
    let at_boundary: bool = start == 0 || candidate[start - 1] == b'/';
    return at_boundary && candidate[start..].eq_ignore_ascii_case(operand);
}

/// What:
///  Every worktree path the repository lists:
/// the main one,
/// and each `gitdir` of `worktrees/*`,
/// as Git reads them.
/// Why:
///   The list Git matches the typed name against.
/// A `gitdir` file Git skips (unreadable or empty) is skipped;
/// a
///  `worktrees` directory that cannot be listed leaves the list unknown,
/// which is an error.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function registeredPaths(commonDir: string): string[];
/// ```
fn registered_paths(common_dir: &Path, worktrees: &Path) -> Result<Vec<Vec<u8>>, String> {
    let mut paths: Vec<Vec<u8>> = Vec::new();
    let main: &[u8] = common_dir.as_os_str().as_bytes();
    paths.push(main.strip_suffix(b"/.git").unwrap_or(main).to_vec());
    let listing = match std::fs::read_dir(worktrees) {
        Ok(listing) => listing,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(paths),
        Err(error) => return Err(format!("the registrations in {} cannot be listed ({error})", worktrees.display())),
    };
    for found in listing {
        let entry: std::fs::DirEntry = match found {
            Ok(listed) => listed,
            Err(error) => return Err(format!("the registrations in {} cannot be listed ({error})", worktrees.display())),
        };
        let Ok(text) = std::fs::read(entry.path().join("gitdir")) else { continue };
        let trimmed: &[u8] = text.trim_ascii_end();
        if trimmed.is_empty() {
            continue;
        }
        let path: &[u8] = trimmed.strip_suffix(b"/.git").unwrap_or(trimmed);
        if path.starts_with(b"/") {
            paths.push(path.to_vec());
        } else {
            // A relative record is relative to the administrative directory.
            let joined: PathBuf = entry.path().join(OsStr::from_bytes(path));
            let resolved: PathBuf = std::fs::canonicalize(joined.as_path()).unwrap_or(joined);
            paths.push(resolved.into_os_string().into_vec());
        }
    }
    return Ok(paths);
}

/// What:
///  The worktree path Git picks for `operand` by the last components of the registered paths,
///  or nothing when it picks none or more than one.
/// Why:
///   Git matches the typed name against the end of every registered worktree path at a directory boundary before it
///  resolves a real path (`find_worktree_by_suffix` in `worktree.c`),
///  so a person may name a worktree by its last component and the wrapper serves the same removal.
/// A name two registered paths end in is Git's ambiguous case,
/// which falls back to the resolved path,
///  and no path here resolves to a directory for it.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function pathGitPicksForOperand(commonDir: string, operand: string): string | undefined;
/// ```
pub fn path_git_picks_for_operand(common_dir: &Path, operand: &OsStr) -> Option<PathBuf> {
    let worktrees: PathBuf = common_dir.join("worktrees");
    let registered: Vec<Vec<u8>> = registered_paths(common_dir, worktrees.as_path()).ok()?;
    let mut picked: Option<&Vec<u8>> = None;
    for path in &registered {
        if !ends_with_name(path.as_slice(), operand.as_bytes()) {
            continue;
        }
        // Git's suffix match answers a unique worktree or none; two matches send it to the resolved path instead.
        if picked.is_some() {
            return None;
        }
        picked = Some(path);
    }
    let only: &Vec<u8> = picked?;
    let resolved: PathBuf = std::fs::canonicalize(Path::new(OsStr::from_bytes(only.as_slice()))).ok()?;
    return resolved.is_dir().then_some(resolved);
}

/// What:
///  Whether a path of Git's worktree list names the same directory as `target`.
/// Why:
///   Git's list may spell a worktree through a symbolic link or by another name of the same directory,
/// and two spellings
///  of one directory are one worktree,
/// so the name Git would pick is compared by the file it names and not by its text.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function sameWorktree(registered: string, target: string): boolean;
/// ```
fn same_worktree(registered: &[u8], target: &Path) -> bool {
    return crate::real_git_candidate::same_file(Path::new(OsStr::from_bytes(registered)), target);
}

/// What:
///  The registration of `target` in the repository whose common Git directory is `common_dir`,
/// when it is a linked
///  worktree of that repository that Git would also pick for `operand`.
/// Why:
///   See the module description.
/// The error is the reason,
/// for the log;
/// the removal is then left to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function registrationOf(args: { commonDir: string; target: string; operand: string }): Registration; // throws the reason
/// ```
pub fn registration_of(common_dir: &Path, target: &Path, operand: &OsStr) -> Result<Registration, String> {
    let pointer_path: PathBuf = target.join(".git");
    let pointer: Vec<u8> = match std::fs::read(pointer_path.as_path()) {
        Ok(bytes) => bytes,
        Err(error) => return Err(format!("{} cannot be read as a file ({error}), so it is not a linked worktree", pointer_path.display())),
    };
    let line: &[u8] = without_newline(pointer.as_slice());
    let Some(named) = line.strip_prefix(b"gitdir: ") else {
        return Err(format!("{} does not start with `gitdir: `", pointer_path.display()));
    };
    let admin: PathBuf = match std::fs::canonicalize(target.join(OsStr::from_bytes(named))) {
        Ok(resolved) => resolved,
        Err(error) => return Err(format!("the administrative directory {} that {} names cannot be resolved ({error})", String::from_utf8_lossy(named), pointer_path.display())),
    };
    let worktrees: PathBuf = match std::fs::canonicalize(common_dir.join("worktrees")) {
        Ok(resolved) => resolved,
        Err(error) => return Err(format!("{} cannot be resolved ({error}), so this repository has no linked worktree", common_dir.join("worktrees").display())),
    };
    if admin.parent() != Some(worktrees.as_path()) {
        return Err(format!(
            "the administrative directory {} of {} is not one of the registrations in {}, the repository this command runs in",
            admin.display(),
            target.display(),
            worktrees.display()
        ));
    }
    let Some(admin_name) = admin.file_name() else {
        return Err(format!("{} has no name", admin.display()));
    };
    let id: OsString = admin_name.to_os_string();
    let gitdir_text: Vec<u8> = path_failure(std::fs::read(admin.join("gitdir")), admin.join("gitdir").as_path(), "cannot be read")?;
    let recorded: PathBuf = PathBuf::from(OsStr::from_bytes(without_newline(gitdir_text.as_slice())));
    // The registration may spell this worktree through a symbolic link or relative to itself; the question is which file it names.
    let recorded_pointer: PathBuf = if recorded.is_absolute() { recorded.clone() } else { admin.join(recorded.as_path()) };
    if !crate::real_git_candidate::same_file(recorded_pointer.as_path(), pointer_path.as_path()) {
        return Err(format!("{} records {}, not {}", admin.join("gitdir").display(), recorded_pointer.display(), pointer_path.display()));
    }
    let registered: Vec<Vec<u8>> = registered_paths(common_dir, worktrees.as_path())?;
    for path in registered {
        if ends_with_name(path.as_slice(), operand.as_bytes()) && !same_worktree(path.as_slice(), target) {
            return Err(format!(
                "Git matches the name {} to the registered worktree {} by its last components, not to {}",
                operand.to_string_lossy(),
                String::from_utf8_lossy(path.as_slice()),
                target.display()
            ));
        }
    }
    return Ok(Registration { id, admin, worktrees, gitdir_text });
}

/// Registry controls stay out of the release executable.
#[cfg(test)]
#[path = "worktree_remove_registry_tests.rs"]
mod tests;
