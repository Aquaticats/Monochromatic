//! What:
//!  The worktree a conversion might change,
//! read from its `.git` file and from the registration that file names,
//! without starting Git:
//! its canonical root,
//! its administrative and common directories,
//! and the exact bytes of its `.git` file.
//! Why:
//!  A conversion swaps a whole directory,
//! so it starts from the worktree behind the path the person typed,
//! and from the proof that the registration names that very path back
//! rather than a main checkout,
//! a submodule,
//! or a stale record.
//! Every refusal is decided from these facts
//! (`worktree_convert_inspection.rs`),
//! so they are read once and handed on.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const candidate = inspect(path); // throws the reason when the path is not a registered linked worktree
//! ```

/// Import the bounded read of one plain file,
/// which is how the `.git` file and the registration are read.
use crate::plain_file::{
    read_plain_file,
    read_plain_text,
};
/// Import the two message shapes of a failed step on a path.
use crate::worktree_path_failure_message::{
    path_failure,
    subject_failure,
};
use std::os::unix::ffi::OsStrExt;
use std::path::{
    Path,
    PathBuf,
};
/// The most bytes of a `.git` pointer or a registration file that are read.
pub const GIT_FILE_LIMIT: u64 = 1 << 16;

/// What:
///  The facts about a worktree path that every mode needs.
/// Why:
///   They are read once,
/// from the `.git` file and the administrative directory,
/// without starting Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// type Candidate = { path: string; admin: string; common: string; gitFile: Uint8Array };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    /// The canonical worktree root.
    pub path: PathBuf,
    /// The worktree's administrative directory.
    pub admin: PathBuf,
    /// The common Git directory.
    pub common: PathBuf,
    /// The exact bytes of the worktree's `.git` file.
    pub git_file: Vec<u8>,
}

/// What:
///  Read the registration of a worktree path:
/// its `.git` file,
/// its administrative directory,
/// and the back-pointer.
/// Why:
///   Only a linked worktree whose administrative directory names this very path back is converted;
/// anything else (the main
///  checkout,
/// a submodule,
/// a stale registration) is refused with the reason.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// function inspect(path: string): Candidate; // throws the reason
/// ```
pub fn inspect(path: &Path) -> Result<Candidate, String> {
    let real: PathBuf = path_failure(std::fs::canonicalize(path), path, "cannot be resolved")?;
    let git_path: PathBuf = real.join(".git");
    let Ok(metadata) = std::fs::symlink_metadata(&git_path) else {
        return Err(format!(
            "{} has no .git entry, so it is not a worktree",
            real.display()
        ));
    };
    if metadata.is_dir() {
        return Err(format!(
            "{} holds a .git directory: it is a main checkout or a repository, not a linked worktree",
            real.display()
        ));
    }
    let git_file: Vec<u8> = path_failure(
        read_plain_file(&git_path, GIT_FILE_LIMIT),
        git_path.as_path(),
        "is unreadable",
    )?;
    let Ok(text) = std::str::from_utf8(git_file.as_slice()) else {
        return Err(format!("{} is not text", git_path.display()));
    };
    let Some(named) = text.trim().strip_prefix("gitdir:") else {
        return Err(format!("{} has no gitdir line", git_path.display()));
    };
    let admin: PathBuf = match std::fs::canonicalize(real.join(named.trim())) {
        Ok(resolved) => resolved,
        Err(error) => {
            return Err(format!(
                "the administrative directory {} cannot be resolved ({error})",
                named.trim()
            ));
        }
    };
    let recorded: Vec<u8> = path_failure(
        read_plain_file(&admin.join("gitdir"), GIT_FILE_LIMIT),
        admin.as_path(),
        "has no gitdir file",
    )?;
    // The registration may spell this worktree through a symbolic link or relative to itself; the question is which file it names.
    let recorded_path: PathBuf = recorded_path(admin.as_path(), recorded.trim_ascii());
    if !crate::real_git_candidate::same_file(recorded_path.as_path(), git_path.as_path()) {
        return Err(format!(
            "{} does not name {} back (it names {}), so the registration is stale or points elsewhere",
            admin.join("gitdir").display(),
            git_path.display(),
            recorded_path.display()
        ));
    }
    let common_text: String = path_failure(
        read_plain_text(&admin.join("commondir")),
        admin.as_path(),
        "has no commondir",
    )?;
    let common: PathBuf = subject_failure(
        std::fs::canonicalize(admin.join(common_text.trim())),
        &"the common directory",
        "cannot be resolved",
    )?;
    return Ok(Candidate {
        path: real,
        admin,
        common,
        git_file,
    });
}

/// What:
///  The path a registration's `gitdir` file names,
/// absolute whether the record is absolute or written relative to the registration.
/// Why:
///   Git writes the record relative to the registration's directory under `--relative-paths` and `worktree.useRelativePaths`;
/// both spellings name the same file,
/// so the record is resolved before it is compared.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = isAbsolute(recorded) ? recorded : join(admin, recorded);
/// ```
fn recorded_path(
    admin: &Path,
    recorded: &[u8],
) -> PathBuf {
    let named: &Path = Path::new(std::ffi::OsStr::from_bytes(recorded));
    return if named.is_absolute() {
        named.to_path_buf()
    } else {
        admin.join(named)
    };
}
