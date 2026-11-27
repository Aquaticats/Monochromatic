//! What:
//!  The `.git` entry a directory names,
//! and the `HEAD` text a plain layout must hold.
//! Why:
//!  Walking up a worktree stops at the first entry a plain layout has,
//! and a layout this reader answers for is one whose `HEAD` names a branch or a full object name.
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const target: [string, boolean] | null | undefined = dotGitTarget(worktree); // undefined: ask Git
//! ```

use crate::hooks_path_values::path_answer;
use crate::repo_read_layout_disk_and_directory_facts::{
    is_hex_byte,
    small_file_bytes,
};
use crate::repo_read_object_format::ObjectFormat;
use std::path::{
    Path,
    PathBuf,
};

/// Whether the `HEAD` text is one of the two spellings Git accepts that this reader knows.
pub(crate) fn head_is_valid(
    text: &str,
    format: ObjectFormat,
) -> bool {
    let Some(line) = text.strip_suffix('\n') else {
        return false;
    };
    if let Some(name) = line.strip_prefix("ref: ") {
        return name.starts_with("refs/") && !name.contains('\n');
    }
    return line.len() == format.hex_length() && line.bytes().all(is_hex_byte);
}

/// What:
///  The Git directory a `.git` entry names:
/// the entry itself when it is a directory,
///  the `gitdir:` line of a plain file,
/// `None` for a link or anything else.
/// Why:
///   The line is read as the bytes the file holds and the path it names is built from them,
///  because a Git directory whose own path is not UTF-8 is still the one this worktree uses,
/// and declining it sent every read of that worktree to Git.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const named = contents.subarray('gitdir: '.length).trimEnd('\r\n'); // Buffer, never a string
/// ```
pub fn dot_git_target(worktree: &Path) -> Option<Option<(PathBuf, bool)>> {
    let dot_git: PathBuf = worktree.join(".git");
    let metadata = match std::fs::symlink_metadata(&dot_git) {
        Ok(found) => found,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Some(None),
        Err(_) => return None,
    };
    if metadata.is_dir() {
        return Some(Some((dot_git, false)));
    }
    if !metadata.is_file() {
        return None;
    }
    let contents: Vec<u8> = small_file_bytes(dot_git.as_path())?;
    // Git strips every trailing carriage return and line feed (`read_gitfile_gently` in `setup.c`).
    let mut target: &[u8] = contents.strip_prefix(b"gitdir: ")?;
    while matches!(target.last(), Some(b'\r' | b'\n')) {
        target = &target[..target.len() - 1];
    }
    if target.is_empty() || target.contains(&b'\n') {
        return None;
    }
    return Some(Some((worktree.join(path_answer(target)), true)));
}
