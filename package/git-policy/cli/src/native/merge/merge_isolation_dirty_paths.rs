//! What:
//!  The paths a merge that writes or deletes them would discard.
//! Why:
//!  `git status` lists what changed in its own reading of a worktree:
//!  it compares the executable bit alone,
//!  and it never lists a file an ignore rule covers,
//! so a mode the person set and an ignored untracked file are work
//!  it does not show,
//! and a merge that deletes or writes over either takes the only copy
//!  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
//! its second finding).
//!
//! In TS you'd write (pseudocode):
//! ```ts
//! const paths = [...statusPaths, ...ignoredUntrackedAmong, ...pathsWithAModeAPersonSet];
//! ```

/// Names and sets of unsaved work.
use crate::auto_push_dirty_paths::status_paths;
/// The failure a command hands back to native Git.
use crate::merge_isolation_run_types::Bail;
/// The mode Git records for what stands at a worktree path.
use crate::merge_isolation_carry::mode_of;
/// The permission bits a person set and the bits a checkout writes,
///  which is how their own mode is told from Git's.
#[cfg(unix)]
use crate::merge_isolation_carry_permissions::{carries_a_mode_a_person_set, the_permission_bits_of};
/// The filesystem path of one repository path.
use crate::merge_isolation_changes::worktree_path;
/// The ignored untracked files of the person's that stand at the paths a merge writes,
/// which `git status` never lists.
use crate::merge_isolation_ignored_paths::ignored_untracked_among;
/// One run of Git under the command's own configuration.
use crate::merge_isolation_git_reads::Session;
/// What a Git start wrote.
use crate::transaction_git::GitOutput;
/// `Path` is a borrowed filesystem path.
use std::path::Path;

/// What:
///  Every path whose worktree or index differs from the branch,
/// untracked files included,
/// and the ignored untracked
///  files of the person's that stand at the paths the merge touches.
/// Why:
///   These are exactly the paths a merge that writes them would discard,
/// and the untracked ones are why native
///  Git refuses a merge that adds a path a file already stands at.
///  `git status` never lists a file an ignore rule covers,
/// so those paths are read from Git's own list of the untracked files the
///  rules cover (`merge_isolation_ignored_paths.rs`);
/// the merge writes over the only copy of their bytes as it would over
///  any other untracked file's.
///  `git status` compares the executable bit alone,
/// so a `chmod 0600` on a tracked `0644` file is work it never lists either;
///  those paths are read from the worktree itself,
/// because a merge that deletes one takes the file and the mode they set with it
///  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
/// its second finding).
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const dirty = statusPaths(await git(['--no-optional-locks', 'status', '--porcelain=v1', '-z', '--untracked-files=all'])).concat(ignoredUntrackedAmong(context, root, touched)).concat(pathsWithAModeAPersonSet(root, touched));
/// ```
pub(crate) fn dirty_paths(session: &Session, root: &Path, touched: &[Vec<u8>]) -> Result<Vec<Vec<u8>>, Bail> {
    let status: GitOutput = session.run(&[
        "--no-optional-locks",
        "status",
        "--porcelain=v1",
        "-z",
        "--untracked-files=all",
    ])?;
    if !status.succeeded() {
        return Err(Bail::from(format!("git status failed: {}", status.error_text())));
    }
    let mut paths: Vec<Vec<u8>> = status_paths(status.stdout.as_slice());
    for path in ignored_untracked_among(&session.context, root, touched).map_err(Bail::from)? {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    for path in paths_with_a_mode_a_person_set(root, touched) {
        if !paths.contains(&path) {
            paths.push(path);
        }
    }
    return Ok(paths);
}

/// What:
///  The paths the merge touches whose worktree file carries permission bits Git's own checkout would not have written.
/// Why:
///  A mode the person set without touching the bytes is theirs as the bytes are,
/// and `git status` compares the executable bit alone,
/// so nothing else would carry it;
///  a merge that deletes such a path takes their file and the mode with it
///  (a Kimi K3.1 Pro audit of `0.6.1-alpha.12`,
/// its second finding).
///  A file that is not a regular file has no mode of a person's,
///  and a link's own bits are the platform's,
/// so only regular files are read here.
///
/// In TS you'd write (pseudocode):
/// ```ts
/// const pathsWithAModeAPersonSet = (root, touched) => touched.filter((path) => { const s = lstatSync(join(root, path)); return s.isFile() && carriesAModeAPersonSet(modeOf(s), thePermissionBitsOf(s)); });
/// ```
#[cfg(unix)]
fn paths_with_a_mode_a_person_set(root: &Path, touched: &[Vec<u8>]) -> Vec<Vec<u8>> {
    let mut found: Vec<Vec<u8>> = Vec::new();
    for path in touched {
        let Ok(metadata) = std::fs::symlink_metadata(worktree_path(root, path.as_slice())) else { continue };
        if !metadata.file_type().is_file() {
            continue;
        }
        if carries_a_mode_a_person_set(mode_of(&metadata), Some(the_permission_bits_of(&metadata))) {
            found.push(path.clone());
        }
    }
    return found;
}

/// The same on a platform whose permissions are not these bits:
///  a mode Git does not record cannot be set there.
#[cfg(not(unix))]
fn paths_with_a_mode_a_person_set(_root: &Path, _touched: &[Vec<u8>]) -> Vec<Vec<u8>> {
    return Vec::new();
}
